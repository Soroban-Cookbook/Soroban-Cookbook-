//! # Multi-Token Balance Manager
//!
//! Demonstrates a registry for multiple SEP-41-compatible token contracts,
//! batched balance reads, and batched token transfers.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token::TokenClient, Address,
    Env, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetadataStandard {
    Sep41,
    StellarAsset,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub standard: MetadataStandard,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenBalance {
    pub token: Address,
    pub balance: i128,
    pub metadata: TokenMetadata,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferRequest {
    pub token: Address,
    pub to: Address,
    pub amount: i128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BalanceManagerError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotRegistered = 3,
    InvalidAmount = 4,
    EmptyBatch = 5,
    NotAuthorized = 6,
}

const EVENT_REGISTERED: Symbol = symbol_short!("register");
const EVENT_REMOVED: Symbol = symbol_short!("remove");
const EVENT_BATCH: Symbol = symbol_short!("batch");

#[contract]
pub struct MultiTokenBalanceManager;

#[contractimpl]
impl MultiTokenBalanceManager {
    /// Set the registry administrator once.
    pub fn initialize(env: Env, admin: Address) -> Result<(), BalanceManagerError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(BalanceManagerError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Register or update metadata for a token contract.
    pub fn register_token(
        env: Env,
        admin: Address,
        token: Address,
        metadata: TokenMetadata,
    ) -> Result<(), BalanceManagerError> {
        require_admin(&env, &admin)?;

        env.storage()
            .persistent()
            .set(&DataKey::Token(token.clone()), &metadata);
        env.events()
            .publish((EVENT_REGISTERED, token), metadata.symbol);
        Ok(())
    }

    /// Remove a token from the registry.
    pub fn unregister_token(
        env: Env,
        admin: Address,
        token: Address,
    ) -> Result<(), BalanceManagerError> {
        require_admin(&env, &admin)?;

        env.storage()
            .persistent()
            .remove(&DataKey::Token(token.clone()));
        env.events().publish((EVENT_REMOVED, token), ());
        Ok(())
    }

    /// Read metadata for a registered token.
    pub fn metadata(env: Env, token: Address) -> Result<TokenMetadata, BalanceManagerError> {
        read_metadata(&env, &token)
    }

    /// Return balances for one user across several registered token contracts.
    pub fn batch_balance(
        env: Env,
        user: Address,
        tokens: Vec<Address>,
    ) -> Result<Vec<TokenBalance>, BalanceManagerError> {
        require_non_empty(&tokens)?;

        let mut balances = Vec::new(&env);
        for token in tokens.iter() {
            let metadata = read_metadata(&env, &token)?;
            let balance = TokenClient::new(&env, &token).balance(&user);
            balances.push_back(TokenBalance {
                token,
                balance,
                metadata,
            });
        }

        Ok(balances)
    }

    /// Transfer several registered tokens from one user in one call.
    pub fn batch_transfer(
        env: Env,
        from: Address,
        transfers: Vec<TransferRequest>,
    ) -> Result<(), BalanceManagerError> {
        require_non_empty(&transfers)?;
        from.require_auth();

        for transfer in transfers.iter() {
            if transfer.amount <= 0 {
                return Err(BalanceManagerError::InvalidAmount);
            }
            read_metadata(&env, &transfer.token)?;
            TokenClient::new(&env, &transfer.token).transfer(&from, &transfer.to, &transfer.amount);
        }

        env.events().publish((EVENT_BATCH, from), transfers.len());
        Ok(())
    }
}

fn require_admin(env: &Env, admin: &Address) -> Result<(), BalanceManagerError> {
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(BalanceManagerError::NotInitialized)?;

    if stored_admin != *admin {
        return Err(BalanceManagerError::NotAuthorized);
    }

    admin.require_auth();
    Ok(())
}

fn read_metadata(env: &Env, token: &Address) -> Result<TokenMetadata, BalanceManagerError> {
    env.storage()
        .persistent()
        .get(&DataKey::Token(token.clone()))
        .ok_or(BalanceManagerError::NotRegistered)
}

fn require_non_empty<T>(items: &Vec<T>) -> Result<(), BalanceManagerError> {
    if items.len() == 0 {
        return Err(BalanceManagerError::EmptyBatch);
    }
    Ok(())
}

#[cfg(test)]
mod test;

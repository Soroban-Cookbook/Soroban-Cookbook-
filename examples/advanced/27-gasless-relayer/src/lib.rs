#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, xdr::ToXdr, Address, BytesN,
    Env, Symbol,
};

const CONTRACT_NS: Symbol = symbol_short!("relayer");
const ACTION_EXEC: Symbol = symbol_short!("exec");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RelayerError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidNonce = 3,
    NonceAlreadyUsed = 4,
    InvalidSignature = 5,
    InvalidRelayer = 6,
    InvalidRecipient = 7,
    InvalidAmount = 8,
    InsufficientBalance = 9,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetaTx {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
    pub nonce: u64,
    pub deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelayRecord {
    pub relayer: Address,
    pub used_at: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Nonce(Address),
    UsedNonce(Address, u64),
    Relay(Address),
    TrustedRelayer(Address),
    Balance(Address),
    SignerPubkey(Address),
    Initialized,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelayEventData {
    pub action: Symbol,
    pub from: Address,
    pub to: Address,
    pub amount: i128,
    pub nonce: u64,
    pub timestamp: u64,
}

#[contract]
pub struct GaslessRelayerContract;

#[contractimpl]
impl GaslessRelayerContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), RelayerError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(RelayerError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::TrustedRelayer(admin.clone()), &true);
        Ok(())
    }

    pub fn add_trusted_relayer(env: Env, relayer: Address) -> Result<(), RelayerError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::TrustedRelayer(relayer), &true);
        Ok(())
    }

    pub fn remove_trusted_relayer(env: Env, relayer: Address) -> Result<(), RelayerError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::TrustedRelayer(relayer), &false);
        Ok(())
    }

    pub fn register_signer(
        env: Env,
        owner: Address,
        pubkey: BytesN<32>,
    ) -> Result<(), RelayerError> {
        owner.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::SignerPubkey(owner), &pubkey);
        Ok(())
    }

    pub fn fund(env: Env, owner: Address, amount: i128) -> Result<(), RelayerError> {
        owner.require_auth();
        if amount <= 0 {
            return Err(RelayerError::InvalidAmount);
        }

        let current: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Balance(owner.clone()))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::Balance(owner), &(current + amount));
        Ok(())
    }

    pub fn relay_transfer(
        env: Env,
        relayer: Address,
        tx: MetaTx,
        signature: BytesN<64>,
    ) -> Result<(), RelayerError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(RelayerError::NotInitialized);
        }

        if !Self::is_trusted_relayer(&env, &relayer) {
            return Err(RelayerError::InvalidRelayer);
        }

        if tx.amount <= 0 {
            return Err(RelayerError::InvalidAmount);
        }
        if tx.deadline < env.ledger().timestamp() {
            return Err(RelayerError::InvalidNonce);
        }

        let nonce_key = DataKey::Nonce(tx.from.clone());
        let current_nonce: u64 = env.storage().instance().get(&nonce_key).unwrap_or(0);
        if tx.nonce != current_nonce + 1 {
            return Err(RelayerError::InvalidNonce);
        }

        let used_key = DataKey::UsedNonce(tx.from.clone(), tx.nonce);
        if env.storage().instance().has(&used_key) {
            return Err(RelayerError::NonceAlreadyUsed);
        }

        let pubkey: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::SignerPubkey(tx.from.clone()))
            .ok_or(RelayerError::InvalidSignature)?;

        let message = tx.clone().to_xdr(&env);
        let message_hash = env.crypto().sha256(&message).to_bytes();
        let message_bytes: soroban_sdk::Bytes = message_hash.into();
        env.crypto()
            .ed25519_verify(&pubkey, &message_bytes, &signature);

        let balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Balance(tx.from.clone()))
            .unwrap_or(0);
        if balance < tx.amount {
            return Err(RelayerError::InsufficientBalance);
        }

        let recipient_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Balance(tx.to.clone()))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::Balance(tx.from.clone()), &(balance - tx.amount));
        env.storage().instance().set(
            &DataKey::Balance(tx.to.clone()),
            &(recipient_balance + tx.amount),
        );
        env.storage().instance().set(&nonce_key, &(tx.nonce));
        env.storage().instance().set(&used_key, &true);
        env.storage()
            .instance()
            .set(&DataKey::Relay(relayer.clone()), &true);

        #[allow(deprecated)]
        env.events().publish(
            (
                CONTRACT_NS,
                ACTION_EXEC,
                tx.from.clone(),
                tx.to.clone(),
                tx.nonce,
            ),
            RelayEventData {
                action: ACTION_EXEC,
                from: tx.from.clone(),
                to: tx.to.clone(),
                amount: tx.amount,
                nonce: tx.nonce,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    pub fn next_nonce(env: Env, owner: Address) -> u64 {
        let key = DataKey::Nonce(owner);
        env.storage().instance().get(&key).unwrap_or(0) + 1
    }

    fn is_trusted_relayer(env: &Env, relayer: &Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::TrustedRelayer(relayer.clone()))
            .unwrap_or(false)
    }
}

mod test;

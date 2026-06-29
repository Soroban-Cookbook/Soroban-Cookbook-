#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Name,
    Symbol,
    TotalSupply,
    Owner(u32),
    Balance(Address),
    TokenByIndex(u32),
    TokenIndex(u32),
    OwnerTokenIndex(u32),
    OwnedToken(Address, u32),
    Approved(u32),
    ApproveAll(Address, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum NftError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    TokenNotFound = 3,
    TokenAlreadyExists = 4,
    NotOwner = 5,
    NotApproved = 6,
    NotAdmin = 7,
}

#[contract]
pub struct BasicNftContract;

#[contractimpl]
impl BasicNftContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
    ) -> Result<(), NftError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(NftError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::TotalSupply, &0u32);

        env.events().publish(
            (symbol_short!("init"), symbol_short!("nft")),
            (name, symbol),
        );

        Ok(())
    }

    pub fn name(env: Env) -> Result<String, NftError> {
        env.storage()
            .instance()
            .get(&DataKey::Name)
            .ok_or(NftError::NotInitialized)
    }

    pub fn symbol(env: Env) -> Result<String, NftError> {
        env.storage()
            .instance()
            .get(&DataKey::Symbol)
            .ok_or(NftError::NotInitialized)
    }

    pub fn total_supply(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    pub fn owner_of(env: Env, token_id: u32) -> Result<Address, NftError> {
        env.storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .ok_or(NftError::TokenNotFound)
    }

    pub fn balance_of(env: Env, owner: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner))
            .unwrap_or(0)
    }

    pub fn token_by_index(env: Env, index: u32) -> Result<u32, NftError> {
        let supply = Self::total_supply(env.clone());
        if index >= supply {
            return Err(NftError::TokenNotFound);
        }
        env.storage()
            .persistent()
            .get(&DataKey::TokenByIndex(index))
            .ok_or(NftError::TokenNotFound)
    }

    pub fn tokens_of_owner(env: Env, owner: Address) -> Vec<u32> {
        let balance = Self::balance_of(env.clone(), owner.clone());
        let mut result = Vec::new(&env);
        let mut index = 0u32;
        while index < balance {
            let token_id: u32 = env
                .storage()
                .persistent()
                .get(&DataKey::OwnedToken(owner.clone(), index))
                .unwrap();
            result.push_back(token_id);
            index += 1;
        }
        result
    }

    pub fn approve(
        env: Env,
        owner: Address,
        approved: Address,
        token_id: u32,
    ) -> Result<(), NftError> {
        owner.require_auth();

        let current_owner = Self::owner_of(env.clone(), token_id)?;
        if current_owner != owner {
            return Err(NftError::NotOwner);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Approved(token_id), &approved);
        env.events().publish(
            (symbol_short!("approve"), symbol_short!("nft")),
            (owner, approved, token_id),
        );

        Ok(())
    }

    pub fn set_approval_for_all(
        env: Env,
        owner: Address,
        operator: Address,
        approved: bool,
    ) -> Result<(), NftError> {
        owner.require_auth();
        env.storage().persistent().set(
            &DataKey::ApproveAll(owner.clone(), operator.clone()),
            &approved,
        );
        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "set_approval_for_all"),
                symbol_short!("nft"),
            ),
            (owner, operator, approved),
        );
        Ok(())
    }

    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::ApproveAll(owner, operator))
            .unwrap_or(false)
    }

    pub fn get_approved(env: Env, token_id: u32) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Approved(token_id))
    }

    pub fn transfer(env: Env, from: Address, to: Address, token_id: u32) -> Result<(), NftError> {
        from.require_auth();
        Self::transfer_from_impl(env, from.clone(), from, to, token_id)
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        token_id: u32,
    ) -> Result<(), NftError> {
        spender.require_auth();
        Self::check_approved(env.clone(), spender.clone(), from.clone(), token_id)?;
        Self::transfer_from_impl(env, spender, from, to, token_id)
    }

    pub fn mint(env: Env, admin: Address, to: Address, token_id: u32) -> Result<(), NftError> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(NftError::NotInitialized)?;
        if stored_admin != admin {
            return Err(NftError::NotAdmin);
        }

        if env.storage().persistent().has(&DataKey::Owner(token_id)) {
            return Err(NftError::TokenAlreadyExists);
        }

        let supply = Self::total_supply(env.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Owner(token_id), &to);
        env.storage()
            .persistent()
            .set(&DataKey::TokenByIndex(supply), &token_id);
        env.storage()
            .persistent()
            .set(&DataKey::TokenIndex(token_id), &supply);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply + 1));
        Self::add_token_to_owner(&env, to.clone(), token_id);
        env.events().publish(
            (symbol_short!("mint"), symbol_short!("nft")),
            (to, token_id),
        );

        Ok(())
    }

    fn transfer_from_impl(
        env: Env,
        _spender: Address,
        from: Address,
        to: Address,
        token_id: u32,
    ) -> Result<(), NftError> {
        let owner = Self::owner_of(env.clone(), token_id)?;
        if owner != from {
            return Err(NftError::NotOwner);
        }

        if from == to {
            return Ok(());
        }

        Self::remove_token_from_owner(&env, from.clone(), token_id);
        Self::add_token_to_owner(&env, to.clone(), token_id);
        env.storage()
            .persistent()
            .set(&DataKey::Owner(token_id), &to);
        env.storage()
            .persistent()
            .remove(&DataKey::Approved(token_id));
        env.events().publish(
            (symbol_short!("transfer"), symbol_short!("nft")),
            (from, to, token_id),
        );
        Ok(())
    }

    fn check_approved(
        env: Env,
        spender: Address,
        owner: Address,
        token_id: u32,
    ) -> Result<(), NftError> {
        if spender == owner {
            return Ok(());
        }

        if let Some(approved) = Self::get_approved(env.clone(), token_id) {
            if approved == spender {
                return Ok(());
            }
        }

        if Self::is_approved_for_all(env, owner.clone(), spender.clone()) {
            return Ok(());
        }

        Err(NftError::NotApproved)
    }

    fn add_token_to_owner(env: &Env, owner: Address, token_id: u32) {
        let balance = Self::balance_of(env.clone(), owner.clone());
        env.storage()
            .persistent()
            .set(&DataKey::OwnedToken(owner.clone(), balance), &token_id);
        env.storage()
            .persistent()
            .set(&DataKey::OwnerTokenIndex(token_id), &balance);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(owner), &(balance + 1));
    }

    fn remove_token_from_owner(env: &Env, owner: Address, token_id: u32) {
        let balance = Self::balance_of(env.clone(), owner.clone());
        let token_index: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerTokenIndex(token_id))
            .unwrap();
        let last_index = balance - 1;

        if token_index != last_index {
            let last_token: u32 = env
                .storage()
                .persistent()
                .get(&DataKey::OwnedToken(owner.clone(), last_index))
                .unwrap();
            env.storage().persistent().set(
                &DataKey::OwnedToken(owner.clone(), token_index),
                &last_token,
            );
            env.storage()
                .persistent()
                .set(&DataKey::OwnerTokenIndex(last_token), &token_index);
        }

        env.storage()
            .persistent()
            .remove(&DataKey::OwnedToken(owner.clone(), last_index));
        env.storage()
            .persistent()
            .remove(&DataKey::OwnerTokenIndex(token_id));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(owner), &last_index);
    }
}

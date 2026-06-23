//! # Mint/Burn Token Example
//!
//! Demonstrates controlled issuance and destruction using a simple token contract.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    TotalSupply,
    SupplyCap,
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InsufficientBalance = 5,
    SupplyCapExceeded = 6,
    Overflow = 7,
}

const EVENT_MINT: Symbol = symbol_short!("mint");
const EVENT_BURN: Symbol = symbol_short!("burn");

#[contract]
pub struct MintBurnToken;

#[contractimpl]
impl MintBurnToken {
    pub fn initialize(env: Env, admin: Address, supply_cap: i128) -> Result<(), TokenError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(TokenError::AlreadyInitialized);
        }
        if supply_cap < 0 {
            return Err(TokenError::InvalidAmount);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::SupplyCap, &supply_cap);

        Ok(())
    }

    pub fn mint(env: Env, to: Address, amount: i128) -> Result<i128, TokenError> {
        require_positive(amount)?;
        let admin = read_admin(&env)?;
        admin.require_auth();

        let cap = read_supply_cap(&env)?;
        let total_supply = read_total_supply(&env);
        let new_supply = total_supply
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;
        if cap > 0 && new_supply > cap {
            return Err(TokenError::SupplyCapExceeded);
        }

        let new_balance = read_balance(&env, &to)
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);
        env.events().publish((EVENT_MINT, to.clone()), amount);
        Ok(new_balance)
    }

    pub fn burn(env: Env, from: Address, amount: i128) -> Result<i128, TokenError> {
        require_positive(amount)?;
        from.require_auth();

        let current_balance = read_balance(&env, &from);
        if current_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        let total_supply = read_total_supply(&env);
        let new_balance = current_balance - amount;
        let new_supply = total_supply - amount;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);
        env.events().publish((EVENT_BURN, from.clone()), amount);
        Ok(new_balance)
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }

    pub fn supply_cap(env: Env) -> Option<i128> {
        let cap = env
            .storage()
            .instance()
            .get(&DataKey::SupplyCap)
            .unwrap_or(0);
        if cap == 0 {
            None
        } else {
            Some(cap)
        }
    }

    pub fn admin(env: Env) -> Result<Address, TokenError> {
        read_admin(&env)
    }
}

fn require_positive(amount: i128) -> Result<(), TokenError> {
    if amount <= 0 {
        Err(TokenError::InvalidAmount)
    } else {
        Ok(())
    }
}

fn read_admin(env: &Env) -> Result<Address, TokenError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(TokenError::NotInitialized)
}

fn read_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn read_supply_cap(env: &Env) -> Result<i128, TokenError> {
    Ok(env
        .storage()
        .instance()
        .get(&DataKey::SupplyCap)
        .unwrap_or(0))
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

#[cfg(test)]
mod test;

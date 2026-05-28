//! # Token Wrapper Pattern
//!
//! Demonstrates a 1:1 wrapper around an existing SEP-41 token. Users deposit
//! an underlying token into this contract and receive internal wrapped shares.
//! Unwrapping burns those shares and returns the underlying token.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token::TokenClient, Address,
    Env, Symbol,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Underlying,
    TotalSupply,
    Balance(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackingInfo {
    pub underlying_balance: i128,
    pub wrapped_supply: i128,
    pub surplus: i128,
    pub fully_backed: bool,
    pub exactly_backed: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum WrapperError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InsufficientWrappedBalance = 4,
    ArithmeticOverflow = 5,
    NotFullyBacked = 6,
}

const EVENT_WRAP: Symbol = symbol_short!("wrap");
const EVENT_UNWRAP: Symbol = symbol_short!("unwrap");

#[contract]
pub struct TokenWrapper;

#[contractimpl]
impl TokenWrapper {
    /// Configure the underlying token once.
    pub fn initialize(env: Env, underlying: Address) -> Result<(), WrapperError> {
        if env.storage().instance().has(&DataKey::Underlying) {
            return Err(WrapperError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&DataKey::Underlying, &underlying);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);

        Ok(())
    }

    /// Deposit underlying tokens and mint the same amount of wrapped shares.
    pub fn wrap(env: Env, user: Address, amount: i128) -> Result<i128, WrapperError> {
        require_positive(amount)?;
        let underlying = read_underlying(&env)?;
        let old_balance = read_balance(&env, &user);
        let old_supply = read_total_supply(&env);
        let new_balance = old_balance
            .checked_add(amount)
            .ok_or(WrapperError::ArithmeticOverflow)?;
        let new_supply = old_supply
            .checked_add(amount)
            .ok_or(WrapperError::ArithmeticOverflow)?;

        user.require_auth();

        let wrapper = env.current_contract_address();
        TokenClient::new(&env, &underlying).transfer(&user, &wrapper, &amount);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(user.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);

        env.events().publish((EVENT_WRAP, user), amount);
        Ok(new_balance)
    }

    /// Burn wrapped shares and return the same amount of underlying tokens.
    pub fn unwrap(env: Env, user: Address, amount: i128) -> Result<i128, WrapperError> {
        require_positive(amount)?;
        let underlying = read_underlying(&env)?;
        let old_balance = read_balance(&env, &user);
        if old_balance < amount {
            return Err(WrapperError::InsufficientWrappedBalance);
        }

        let old_supply = read_total_supply(&env);
        let wrapper = env.current_contract_address();
        let underlying_balance = TokenClient::new(&env, &underlying).balance(&wrapper);
        if underlying_balance < old_supply {
            return Err(WrapperError::NotFullyBacked);
        }

        user.require_auth();

        let new_balance = old_balance - amount;
        let new_supply = old_supply - amount;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(user.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);

        TokenClient::new(&env, &underlying).transfer(&wrapper, &user, &amount);

        env.events().publish((EVENT_UNWRAP, user), amount);
        Ok(new_balance)
    }

    /// Transfer wrapped shares without moving the underlying backing.
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), WrapperError> {
        require_positive(amount)?;
        from.require_auth();

        let from_balance = read_balance(&env, &from);
        if from_balance < amount {
            return Err(WrapperError::InsufficientWrappedBalance);
        }

        let to_balance = read_balance(&env, &to)
            .checked_add(amount)
            .ok_or(WrapperError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &to_balance);

        Ok(())
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }

    pub fn underlying(env: Env) -> Result<Address, WrapperError> {
        read_underlying(&env)
    }

    /// Compare contract-held collateral with wrapped supply.
    pub fn backing(env: Env) -> Result<BackingInfo, WrapperError> {
        let underlying = read_underlying(&env)?;
        let wrapper = env.current_contract_address();
        let underlying_balance = TokenClient::new(&env, &underlying).balance(&wrapper);
        let wrapped_supply = read_total_supply(&env);
        let fully_backed = underlying_balance >= wrapped_supply;
        let surplus = if fully_backed {
            underlying_balance - wrapped_supply
        } else {
            0
        };

        Ok(BackingInfo {
            underlying_balance,
            wrapped_supply,
            surplus,
            fully_backed,
            exactly_backed: underlying_balance == wrapped_supply,
        })
    }
}

fn require_positive(amount: i128) -> Result<(), WrapperError> {
    if amount <= 0 {
        return Err(WrapperError::InvalidAmount);
    }
    Ok(())
}

fn read_underlying(env: &Env) -> Result<Address, WrapperError> {
    env.storage()
        .instance()
        .get(&DataKey::Underlying)
        .ok_or(WrapperError::NotInitialized)
}

fn read_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

#[cfg(test)]
mod test;

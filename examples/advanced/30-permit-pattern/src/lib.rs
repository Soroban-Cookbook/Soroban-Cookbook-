#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env, IntoVal,
};

#[contract]
pub struct PermitPattern;

#[contractevent(topics = ["permit", "approve"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermitApprovedEvent {
    pub owner: Address,
    pub spender: Address,
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[contractevent(topics = ["permit", "transfer"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermitTransferEvent {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Initialized,
    Balance(Address),
    Allowance(Address, Address),
    Expiry(Address, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PermitError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InsufficientBalance = 4,
    InsufficientAllowance = 5,
    ExpiredPermit = 6,
}

#[contractimpl]
impl PermitPattern {
    pub fn initialize(env: Env, admin: Address, initial_supply: i128) -> Result<(), PermitError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(PermitError::AlreadyInitialized);
        }
        if initial_supply <= 0 {
            return Err(PermitError::InvalidAmount);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        write_balance(&env, &admin, initial_supply);
        Ok(())
    }

    pub fn permit(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), PermitError> {
        ensure_initialized(&env)?;

        if amount < 0 {
            return Err(PermitError::InvalidAmount);
        }
        if amount > 0 && expiration_ledger < env.ledger().sequence() {
            return Err(PermitError::ExpiredPermit);
        }

        owner.require_auth_for_args((spender.clone(), amount, expiration_ledger).into_val(&env));

        write_allowance(&env, &owner, &spender, amount);
        write_expiry(&env, &owner, &spender, expiration_ledger);

        PermitApprovedEvent {
            owner: owner.clone(),
            spender: spender.clone(),
            amount,
            expiration_ledger,
        }
        .publish(&env);

        Ok(())
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), PermitError> {
        from.require_auth();
        ensure_initialized(&env)?;
        require_positive(amount)?;

        let from_bal = read_balance(&env, &from);
        if from_bal < amount {
            return Err(PermitError::InsufficientBalance);
        }

        write_balance(&env, &from, from_bal - amount);
        write_balance(&env, &to, read_balance(&env, &to) + amount);

        PermitTransferEvent {
            from: from.clone(),
            to: to.clone(),
            amount,
        }
        .publish(&env);
        Ok(())
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        owner: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), PermitError> {
        spender.require_auth();
        ensure_initialized(&env)?;
        require_positive(amount)?;

        let expiry = read_expiry(&env, &owner, &spender);
        let allowance = if expiry > 0 && expiry < env.ledger().sequence() {
            0
        } else {
            read_allowance(&env, &owner, &spender)
        };

        if allowance < amount {
            return Err(PermitError::InsufficientAllowance);
        }

        let owner_bal = read_balance(&env, &owner);
        if owner_bal < amount {
            return Err(PermitError::InsufficientBalance);
        }

        write_allowance(&env, &owner, &spender, allowance - amount);
        write_balance(&env, &owner, owner_bal - amount);
        write_balance(&env, &to, read_balance(&env, &to) + amount);

        PermitTransferEvent {
            from: owner.clone(),
            to: to.clone(),
            amount,
        }
        .publish(&env);
        Ok(())
    }

    pub fn balance(env: Env, owner: Address) -> i128 {
        read_balance(&env, &owner)
    }

    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        let expiry = read_expiry(&env, &owner, &spender);
        if expiry > 0 && expiry < env.ledger().sequence() {
            return 0;
        }
        read_allowance(&env, &owner, &spender)
    }
}

fn ensure_initialized(env: &Env) -> Result<(), PermitError> {
    if env.storage().instance().has(&DataKey::Initialized) {
        Ok(())
    } else {
        Err(PermitError::NotInitialized)
    }
}

fn require_positive(amount: i128) -> Result<(), PermitError> {
    if amount <= 0 {
        Err(PermitError::InvalidAmount)
    } else {
        Ok(())
    }
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

fn write_balance(env: &Env, user: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Balance(user.clone()), &amount);
}

fn read_allowance(env: &Env, owner: &Address, spender: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Allowance(owner.clone(), spender.clone()))
        .unwrap_or(0)
}

fn write_allowance(env: &Env, owner: &Address, spender: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Allowance(owner.clone(), spender.clone()), &amount);
}

fn read_expiry(env: &Env, owner: &Address, spender: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::Expiry(owner.clone(), spender.clone()))
        .unwrap_or(0)
}

fn write_expiry(env: &Env, owner: &Address, spender: &Address, expiry: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::Expiry(owner.clone(), spender.clone()), &expiry);
}

#[cfg(test)]
mod test;

//! # Optimized Token Operations
//!
//! Demonstrates a batch-transfer optimization pattern for a Soroban token contract.

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, vec, vec::Vec, Address, Env, Symbol, symbol_short};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    TotalSupply,
    Balance(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub recipient: Address,
    pub amount: i128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InsufficientBalance = 4,
    ArithmeticOverflow = 5,
}

const EVENT_TRANSFER: Symbol = symbol_short!("transfer");
const EVENT_BATCH_TRANSFER: Symbol = symbol_short!("batch_transfer");

#[contract]
pub struct OptimizedToken;

#[contractimpl]
impl OptimizedToken {
    /// Initialize the token with a single owner balance.
    pub fn initialize(env: Env, owner: Address, total_supply: i128) -> Result<(), TokenError> {
        if env.storage().instance().has(&DataKey::TotalSupply) {
            return Err(TokenError::AlreadyInitialized);
        }

        require_positive(total_supply)?;
        env.storage().persistent().set(&DataKey::Balance(owner.clone()), &total_supply);
        env.storage().instance().set(&DataKey::TotalSupply, &total_supply);
        Ok(())
    }

    /// Transfer a single amount from one account to another.
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        require_positive(amount)?;
        from.require_auth();

        let from_balance = read_balance(&env, &from);
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        let to_balance = read_balance(&env, &to)
            .checked_add(amount)
            .ok_or(TokenError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &to_balance);
        env.events().publish((EVENT_TRANSFER, from.clone(), to.clone()), &amount);

        Ok(())
    }

    /// Naive batch transfer that re-reads and re-writes the sender balance for each recipient.
    pub fn batch_transfer_naive(
        env: Env,
        from: Address,
        payments: Vec<Payment>,
    ) -> Result<(), TokenError> {
        from.require_auth();

        for payment in payments.iter() {
            require_positive(payment.amount)?;

            let from_balance = read_balance(&env, &from);
            if from_balance < payment.amount {
                return Err(TokenError::InsufficientBalance);
            }

            let to_balance = read_balance(&env, &payment.recipient)
                .checked_add(payment.amount)
                .ok_or(TokenError::ArithmeticOverflow)?;

            env.storage()
                .persistent()
                .set(&DataKey::Balance(from.clone()), &(from_balance - payment.amount));
            env.storage()
                .persistent()
                .set(&DataKey::Balance(payment.recipient.clone()), &to_balance);
        }

        env.events().publish(
            (EVENT_BATCH_TRANSFER, from.clone()),
            &(payments.len() as i128),
        );
        Ok(())
    }

    /// Optimized batch transfer that reads the sender balance once and writes it once.
    pub fn batch_transfer_optimized(
        env: Env,
        from: Address,
        payments: Vec<Payment>,
    ) -> Result<(), TokenError> {
        from.require_auth();

        let total_amount = sum_payments(&payments)?;
        let from_balance = read_balance(&env, &from);
        if from_balance < total_amount {
            return Err(TokenError::InsufficientBalance);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - total_amount));

        for payment in payments.iter() {
            require_positive(payment.amount)?;
            let to_balance = read_balance(&env, &payment.recipient)
                .checked_add(payment.amount)
                .ok_or(TokenError::ArithmeticOverflow)?;
            env.storage()
                .persistent()
                .set(&DataKey::Balance(payment.recipient.clone()), &to_balance);
        }

        env.events().publish(
            (EVENT_BATCH_TRANSFER, from.clone()),
            &(payments.len() as i128),
        );
        Ok(())
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }
}

fn require_positive(amount: i128) -> Result<(), TokenError> {
    if amount <= 0 {
        return Err(TokenError::InvalidAmount);
    }
    Ok(())
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

fn read_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn sum_payments(payments: &Vec<Payment>) -> Result<i128, TokenError> {
    let mut total = 0i128;
    for payment in payments.iter() {
        require_positive(payment.amount)?;
        total = total
            .checked_add(payment.amount)
            .ok_or(TokenError::ArithmeticOverflow)?;
    }
    Ok(total)
}

#![no_std]

//! # Allowance Pattern
//!
//! The *allowance pattern* is the standard way one account (the `owner`) lets
//! another account (the `spender`) move a bounded amount of its tokens without
//! handing over full custody. It is the mechanism behind "approve and pull"
//! flows used by exchanges, escrow, subscription and DeFi contracts.
//!
//! This example focuses purely on the allowance lifecycle:
//!
//! * [`approve`](AllowancePattern::approve) — grant a spender an allowance that
//!   is valid up to and including an `expiration_ledger`.
//! * [`transfer_from`](AllowancePattern::transfer_from) — let the spender draw
//!   down the allowance, moving tokens out of the owner's balance.
//! * [`allowance`](AllowancePattern::allowance) — query the *spendable*
//!   allowance, with expired entries reported as `0`.
//! * [`revoke`](AllowancePattern::revoke) — explicitly cancel an allowance.
//!
//! ## Why expiration?
//!
//! A bare `(owner, spender) -> amount` mapping never forgets. A forgotten,
//! still-live approval is a classic source of token drains. Following the
//! Soroban/SEP-41 convention, every allowance carries an `expiration_ledger`;
//! once the ledger sequence passes it, the allowance is treated as `0` even if
//! a stale amount is still in storage.
//!
//! ## Edge cases handled
//!
//! * Negative amounts on `approve`/`transfer_from` are rejected.
//! * A non-zero allowance whose expiration already lies in the past is
//!   rejected, so unusable approvals never reach storage.
//! * Expired allowances are spendable as `0` without needing a clean-up call.
//! * `transfer_from` checks the allowance *and* the owner's balance, and uses
//!   checked arithmetic on the recipient's balance.
//! * Spending preserves the original expiration ledger of the allowance.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

/// Storage keys for balances, allowances, and contract metadata.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The address allowed to seed balances via [`AllowancePattern::initialize`].
    Admin,
    /// Token balance of a single account.
    Balance(Address),
    /// Allowance keyed by `(owner, spender)`.
    Allowance(Address, Address),
}

/// An allowance entry: how much `spender` may move, and the last ledger at
/// which that permission is valid.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

/// Event payload emitted by [`AllowancePattern::approve`] and
/// [`AllowancePattern::revoke`].
#[contracttype]
#[derive(Clone)]
pub struct ApproveEventData {
    pub amount: i128,
    pub expiration_ledger: u32,
}

/// Event payload emitted by [`AllowancePattern::transfer_from`].
#[contracttype]
#[derive(Clone)]
pub struct TransferEventData {
    pub amount: i128,
}

/// Errors returned by the contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AllowanceError {
    /// `initialize` was called more than once.
    AlreadyInitialized = 1,
    /// A method was called before `initialize`.
    NotInitialized = 2,
    /// A negative (or, where required, non-positive) amount was supplied.
    InvalidAmount = 3,
    /// A non-zero allowance was given an already-expired expiration ledger.
    InvalidExpiration = 4,
    /// `transfer_from` requested more than the spendable allowance.
    InsufficientAllowance = 5,
    /// The owner does not hold enough tokens for the requested transfer.
    InsufficientBalance = 6,
    /// A balance update would overflow `i128`.
    ArithmeticOverflow = 7,
}

const EVENT_NAMESPACE: Symbol = symbol_short!("events");
const EVENT_APPROVE: Symbol = symbol_short!("approve");
const EVENT_TRANSFER: Symbol = symbol_short!("transfer");

#[contract]
pub struct AllowancePattern;

#[contractimpl]
impl AllowancePattern {
    /// Initialize the contract once, crediting `initial_supply` to `admin`.
    ///
    /// The admin is simply a convenient holder of an opening balance so the
    /// allowance flows below have tokens to move; this example deliberately
    /// omits minting and other token machinery to stay focused.
    pub fn initialize(
        env: Env,
        admin: Address,
        initial_supply: i128,
    ) -> Result<(), AllowanceError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AllowanceError::AlreadyInitialized);
        }
        if initial_supply < 0 {
            return Err(AllowanceError::InvalidAmount);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(admin), &initial_supply);

        Ok(())
    }

    /// Approve `spender` to move up to `amount` of `owner`'s tokens, valid
    /// through `expiration_ledger`.
    ///
    /// Passing `amount == 0` clears the allowance (equivalent to
    /// [`revoke`](Self::revoke)). A non-zero allowance whose expiration is
    /// already in the past is rejected with [`AllowanceError::InvalidExpiration`].
    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), AllowanceError> {
        owner.require_auth();
        ensure_initialized(&env)?;

        if amount < 0 {
            return Err(AllowanceError::InvalidAmount);
        }
        // A live allowance must not already be expired, otherwise it would be
        // unusable and only waste storage.
        if amount > 0 && expiration_ledger < env.ledger().sequence() {
            return Err(AllowanceError::InvalidExpiration);
        }

        write_allowance(&env, &owner, &spender, amount, expiration_ledger);
        publish_approve(&env, owner, spender, amount, expiration_ledger);

        Ok(())
    }

    /// Explicitly revoke any allowance `owner` granted to `spender`.
    pub fn revoke(env: Env, owner: Address, spender: Address) -> Result<(), AllowanceError> {
        owner.require_auth();
        ensure_initialized(&env)?;

        write_allowance(&env, &owner, &spender, 0, 0);
        publish_approve(&env, owner, spender, 0, 0);

        Ok(())
    }

    /// Move `amount` of `owner`'s tokens to `to`, drawing on the allowance the
    /// `owner` granted to `spender`. Only `spender` authorizes the call.
    pub fn transfer_from(
        env: Env,
        spender: Address,
        owner: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), AllowanceError> {
        spender.require_auth();
        ensure_initialized(&env)?;
        require_positive(amount)?;

        let allowance = read_allowance(&env, &owner, &spender);
        let spendable = effective_allowance(&env, &allowance);
        if spendable < amount {
            return Err(AllowanceError::InsufficientAllowance);
        }

        let owner_balance = read_balance(&env, &owner);
        if owner_balance < amount {
            return Err(AllowanceError::InsufficientBalance);
        }
        let to_balance = read_balance(&env, &to);
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(AllowanceError::ArithmeticOverflow)?;

        // Spend the allowance, keeping its original expiration ledger.
        write_allowance(
            &env,
            &owner,
            &spender,
            spendable - amount,
            allowance.expiration_ledger,
        );
        env.storage()
            .persistent()
            .set(&DataKey::Balance(owner.clone()), &(owner_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_to_balance);

        publish_transfer(&env, owner, to, amount);

        Ok(())
    }

    /// Return the *spendable* allowance, reporting expired entries as `0`.
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        let allowance = read_allowance(&env, &owner, &spender);
        effective_allowance(&env, &allowance)
    }

    /// Return the raw allowance entry (amount and expiration ledger), without
    /// applying the expiration rule. Useful for inspection and tooling.
    pub fn allowance_details(env: Env, owner: Address, spender: Address) -> AllowanceValue {
        read_allowance(&env, &owner, &spender)
    }

    /// Return an account's token balance.
    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    /// Return the configured admin address.
    pub fn admin(env: Env) -> Result<Address, AllowanceError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AllowanceError::NotInitialized)
    }
}

/// Collapse a stored allowance to its spendable amount, honoring expiration.
fn effective_allowance(env: &Env, allowance: &AllowanceValue) -> i128 {
    if allowance.amount > 0 && allowance.expiration_ledger < env.ledger().sequence() {
        0
    } else {
        allowance.amount
    }
}

fn read_allowance(env: &Env, owner: &Address, spender: &Address) -> AllowanceValue {
    env.storage()
        .persistent()
        .get(&DataKey::Allowance(owner.clone(), spender.clone()))
        .unwrap_or(AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        })
}

fn write_allowance(
    env: &Env,
    owner: &Address,
    spender: &Address,
    amount: i128,
    expiration_ledger: u32,
) {
    env.storage().persistent().set(
        &DataKey::Allowance(owner.clone(), spender.clone()),
        &AllowanceValue {
            amount,
            expiration_ledger,
        },
    );
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

fn require_positive(amount: i128) -> Result<(), AllowanceError> {
    if amount <= 0 {
        return Err(AllowanceError::InvalidAmount);
    }
    Ok(())
}

fn ensure_initialized(env: &Env) -> Result<(), AllowanceError> {
    if env.storage().instance().has(&DataKey::Admin) {
        Ok(())
    } else {
        Err(AllowanceError::NotInitialized)
    }
}

fn publish_approve(
    env: &Env,
    owner: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) {
    env.events().publish(
        (EVENT_NAMESPACE, EVENT_APPROVE, owner, spender),
        ApproveEventData {
            amount,
            expiration_ledger,
        },
    );
}

fn publish_transfer(env: &Env, from: Address, to: Address, amount: i128) {
    env.events().publish(
        (EVENT_NAMESPACE, EVENT_TRANSFER, from, to),
        TransferEventData { amount },
    );
}

mod test;

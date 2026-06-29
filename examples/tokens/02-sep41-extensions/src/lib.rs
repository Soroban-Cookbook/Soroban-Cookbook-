//! # SEP-41 Extensions
//!
//! Optional extensions to a standard SEP-41 token contract.
//!
//! Key patterns shown:
//! - **Permit** (EIP-2612 equivalent): off-chain signature-based approvals so
//!   users can authorise a spender without a separate on-chain `approve` call.
//! - **Batch transfer**: send different amounts to multiple recipients in one
//!   call, debiting the sender's balance only once.
//! - **Batch approve**: set multiple allowances atomically.
//!
//! ## Design notes
//!
//! `permit` in Soroban does not use secp256k1 signatures as Ethereum does;
//! instead it relies on Soroban's built-in auth framework.  The caller
//! provides a signed authorization envelope via `require_auth_for_args` which
//! the host verifies against the owner's key material.  From the caller's
//! perspective the effect is identical to EIP-2612: a spender can be approved
//! without the owner ever submitting a separate approve transaction.
//!
//! ## Use cases
//!
//! - **Gasless approve flows**: a relayer calls `permit` on behalf of the
//!   owner, bundled with the downstream `transfer_from` in one transaction.
//! - **Batch payroll**: fund many addresses in a single call.
//! - **Bulk allowance management**: set allowances for many spenders at once.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, IntoVal,
    Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Token balance of an address.
    Balance(Address),
    /// Allowance: how much `spender` may draw from `owner`.
    Allowance(Address, Address),
    /// Expiration ledger for an allowance.
    AllowanceExpiry(Address, Address),
    /// Whether the contract has been initialised.
    Initialized,
}

// ---------------------------------------------------------------------------
// Transfer / Approval descriptors
// ---------------------------------------------------------------------------

/// A `(recipient, amount)` pair used in `batch_transfer`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    pub to: Address,
    pub amount: i128,
}

/// A `(spender, amount, expiration_ledger)` triple used in `batch_approve`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Approval {
    pub spender: Address,
    pub amount: i128,
    pub expiration_ledger: u32,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const NS: Symbol = symbol_short!("sep41ext");
const EV_XFER: Symbol = symbol_short!("transfer");
const EV_APPR: Symbol = symbol_short!("approve");
const EV_PERMIT: Symbol = symbol_short!("permit");
const EV_MINT: Symbol = symbol_short!("mint");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ExtError {
    /// The contract was already initialised.
    AlreadyInitialized = 1,
    /// The contract has not been initialised.
    NotInitialized = 2,
    /// An amount is zero or negative where a positive value is required.
    InvalidAmount = 3,
    /// The sender does not hold enough tokens.
    InsufficientBalance = 4,
    /// A drawn allowance exceeds what was granted.
    InsufficientAllowance = 5,
    /// An overflow was detected during arithmetic.
    ArithmeticOverflow = 6,
    /// The batch list is empty.
    EmptyBatch = 7,
    /// The permit's expiration ledger is already in the past.
    ExpiredPermit = 8,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct Sep41Extensions;

#[contractimpl]
impl Sep41Extensions {
    // -----------------------------------------------------------------------
    // Initialisation
    // -----------------------------------------------------------------------

    /// Initialise and mint `initial_supply` to `admin`.  Can only be called
    /// once.
    pub fn initialize(env: Env, admin: Address, initial_supply: i128) -> Result<(), ExtError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(ExtError::AlreadyInitialized);
        }
        if initial_supply <= 0 {
            return Err(ExtError::InvalidAmount);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        write_balance(&env, &admin, initial_supply);

        env.events()
            .publish((NS, EV_MINT, admin.clone()), initial_supply);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Extension 1: permit (EIP-2612 equivalent)
    // -----------------------------------------------------------------------

    /// Approve `spender` to transfer up to `amount` from `owner`, valid
    /// through `expiration_ledger`, using an off-chain signature.
    ///
    /// The owner does not need to submit a separate on-chain `approve`.
    /// Instead the owner signs an authorisation envelope off-chain; the
    /// spender (or a relayer) submits this call with that signature attached.
    ///
    /// Soroban's `require_auth_for_args` ties the authorisation to the exact
    /// arguments, preventing replay and argument substitution.
    pub fn permit(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), ExtError> {
        ensure_initialized(&env)?;

        if amount < 0 {
            return Err(ExtError::InvalidAmount);
        }
        if amount > 0 && expiration_ledger < env.ledger().sequence() {
            return Err(ExtError::ExpiredPermit);
        }

        // Bind the auth to these exact arguments so it cannot be replayed
        // with different parameters.
        owner.require_auth_for_args((spender.clone(), amount, expiration_ledger).into_val(&env));

        write_allowance(&env, &owner, &spender, amount);
        write_expiry(&env, &owner, &spender, expiration_ledger);

        env.events().publish(
            (NS, EV_PERMIT, owner.clone(), spender.clone()),
            (amount, expiration_ledger),
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Extension 2: batch transfer
    // -----------------------------------------------------------------------

    /// Transfer tokens from `from` to multiple recipients atomically.
    ///
    /// The sender's balance is read once and written once regardless of the
    /// number of recipients, making this substantially cheaper than looping
    /// over individual `transfer` calls.
    pub fn batch_transfer(
        env: Env,
        from: Address,
        transfers: Vec<Transfer>,
    ) -> Result<u32, ExtError> {
        from.require_auth();
        ensure_initialized(&env)?;

        let count = transfers.len();
        if count == 0 {
            return Err(ExtError::EmptyBatch);
        }

        // Validation pass: sum total, reject non-positive amounts.
        let mut total: i128 = 0;
        for item in transfers.iter() {
            if item.amount <= 0 {
                return Err(ExtError::InvalidAmount);
            }
            total = total
                .checked_add(item.amount)
                .ok_or(ExtError::ArithmeticOverflow)?;
        }

        // Single balance read.
        let sender_bal = read_balance(&env, &from);
        if sender_bal < total {
            return Err(ExtError::InsufficientBalance);
        }

        // Mutations: debit sender once, credit each recipient once.
        write_balance(&env, &from, sender_bal - total);

        for item in transfers.iter() {
            let old = read_balance(&env, &item.to);
            let new_bal = old
                .checked_add(item.amount)
                .ok_or(ExtError::ArithmeticOverflow)?;
            write_balance(&env, &item.to, new_bal);

            env.events()
                .publish((NS, EV_XFER, from.clone(), item.to.clone()), item.amount);
        }

        Ok(count)
    }

    // -----------------------------------------------------------------------
    // Extension 3: batch approve
    // -----------------------------------------------------------------------

    /// Set multiple allowances for `owner` in a single call.
    ///
    /// Each entry in `approvals` specifies a `spender`, the `amount` to
    /// authorise, and the `expiration_ledger` after which the allowance
    /// becomes invalid.  Passing `amount = 0` revokes that spender's
    /// allowance.
    pub fn batch_approve(
        env: Env,
        owner: Address,
        approvals: Vec<Approval>,
    ) -> Result<u32, ExtError> {
        owner.require_auth();
        ensure_initialized(&env)?;

        let count = approvals.len();
        if count == 0 {
            return Err(ExtError::EmptyBatch);
        }

        for item in approvals.iter() {
            if item.amount < 0 {
                return Err(ExtError::InvalidAmount);
            }

            write_allowance(&env, &owner, &item.spender, item.amount);
            write_expiry(&env, &owner, &item.spender, item.expiration_ledger);

            env.events().publish(
                (NS, EV_APPR, owner.clone(), item.spender.clone()),
                (item.amount, item.expiration_ledger),
            );
        }

        Ok(count)
    }

    // -----------------------------------------------------------------------
    // Standard transfer / transfer_from / approve
    // -----------------------------------------------------------------------

    /// Standard token transfer.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), ExtError> {
        from.require_auth();
        ensure_initialized(&env)?;
        require_positive(amount)?;

        let from_bal = read_balance(&env, &from);
        if from_bal < amount {
            return Err(ExtError::InsufficientBalance);
        }
        let to_bal = read_balance(&env, &to);
        let new_to = to_bal
            .checked_add(amount)
            .ok_or(ExtError::ArithmeticOverflow)?;

        write_balance(&env, &from, from_bal - amount);
        write_balance(&env, &to, new_to);

        env.events()
            .publish((NS, EV_XFER, from.clone(), to.clone()), amount);

        Ok(())
    }

    /// Approve a spender via the standard on-chain path.
    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), ExtError> {
        owner.require_auth();
        ensure_initialized(&env)?;

        if amount < 0 {
            return Err(ExtError::InvalidAmount);
        }

        write_allowance(&env, &owner, &spender, amount);
        write_expiry(&env, &owner, &spender, expiration_ledger);

        env.events().publish(
            (NS, EV_APPR, owner.clone(), spender.clone()),
            (amount, expiration_ledger),
        );

        Ok(())
    }

    /// Transfer using an existing allowance.
    pub fn transfer_from(
        env: Env,
        spender: Address,
        owner: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ExtError> {
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
            return Err(ExtError::InsufficientAllowance);
        }

        let owner_bal = read_balance(&env, &owner);
        if owner_bal < amount {
            return Err(ExtError::InsufficientBalance);
        }
        let to_bal = read_balance(&env, &to);
        let new_to = to_bal
            .checked_add(amount)
            .ok_or(ExtError::ArithmeticOverflow)?;

        write_allowance(&env, &owner, &spender, allowance - amount);
        write_balance(&env, &owner, owner_bal - amount);
        write_balance(&env, &to, new_to);

        env.events()
            .publish((NS, EV_XFER, owner.clone(), to.clone()), amount);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the token balance for `user`.
    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    /// Return the current allowance `spender` holds from `owner`.
    ///
    /// Returns `0` if the allowance has expired.
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        let expiry = read_expiry(&env, &owner, &spender);
        if expiry > 0 && expiry < env.ledger().sequence() {
            return 0;
        }
        read_allowance(&env, &owner, &spender)
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn ensure_initialized(env: &Env) -> Result<(), ExtError> {
    if env.storage().instance().has(&DataKey::Initialized) {
        Ok(())
    } else {
        Err(ExtError::NotInitialized)
    }
}

fn require_positive(amount: i128) -> Result<(), ExtError> {
    if amount <= 0 {
        Err(ExtError::InvalidAmount)
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
        .get(&DataKey::AllowanceExpiry(owner.clone(), spender.clone()))
        .unwrap_or(0)
}

fn write_expiry(env: &Env, owner: &Address, spender: &Address, expiry: u32) {
    env.storage().persistent().set(
        &DataKey::AllowanceExpiry(owner.clone(), spender.clone()),
        &expiry,
    );
}

#[cfg(test)]
mod test;

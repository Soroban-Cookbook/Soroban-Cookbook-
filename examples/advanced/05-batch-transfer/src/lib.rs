//! # Batch Transfer
//!
//! Demonstrates efficient multi-recipient token transfers on Soroban.
//!
//! Key patterns shown:
//! - Single-read / single-write for the sender balance (gas optimised)
//! - Checked arithmetic to prevent overflow on every debit and credit
//! - A dedicated `#[contracterror]` enum for clear, typed failures
//! - Events for every individual transfer within the batch
//! - An internal token ledger so the example is self-contained and fully
//!   testable without a live token contract
//!
//! ## Use cases
//! - Payroll / airdrop: send rewards to many recipients in one transaction
//! - DAO distribution: disburse governance rewards atomically
//! - NFT mint revenue split: forward proceeds to multiple beneficiaries

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Token balance of an address.
    Balance(Address),
    /// Whether the contract has been initialised.
    Initialized,
}

// ---------------------------------------------------------------------------
// Transfer descriptor
// ---------------------------------------------------------------------------

/// A single `(recipient, amount)` pair in a batch.
///
/// The `amount` field must be a strictly positive `i128`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    pub to: Address,
    pub amount: i128,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Emitted once for every successful transfer within a batch.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

/// Emitted after a batch completes, summarising the run.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchCompleteEvent {
    pub from: Address,
    pub recipient_count: u32,
    pub total_amount: i128,
}

const NS: Symbol = symbol_short!("batch");
const EV_XFER: Symbol = symbol_short!("transfer");
const EV_BATCH: Symbol = symbol_short!("complete");
const EV_MINT: Symbol = symbol_short!("mint");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BatchError {
    /// The contract has already been initialised.
    AlreadyInitialized = 1,
    /// The contract has not been initialised yet.
    NotInitialized = 2,
    /// The caller is not the admin.
    Unauthorized = 3,
    /// The sender does not have enough tokens to cover the batch.
    InsufficientBalance = 4,
    /// An amount in the batch is zero or negative.
    InvalidAmount = 5,
    /// Adding the amounts in the batch would overflow `i128`.
    TotalOverflow = 6,
    /// The batch is empty — nothing to do.
    EmptyBatch = 7,
    /// The batch exceeds the maximum allowed recipient count.
    BatchTooLarge = 8,
    /// A credit to a recipient would overflow their balance.
    RecipientOverflow = 9,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of recipients in a single batch call.
pub const MAX_BATCH_SIZE: u32 = 100;

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct BatchTransferContract;

#[contractimpl]
impl BatchTransferContract {
    // -----------------------------------------------------------------------
    // Initialisation
    // -----------------------------------------------------------------------

    /// Initialise the contract and mint an opening balance to `admin`.
    ///
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address, initial_supply: i128) -> Result<(), BatchError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(BatchError::AlreadyInitialized);
        }
        if initial_supply <= 0 {
            return Err(BatchError::InvalidAmount);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(admin.clone()), &initial_supply);

        env.events()
            .publish((NS, EV_MINT, admin.clone()), initial_supply);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Core: batch transfer
    // -----------------------------------------------------------------------

    /// Transfer tokens from `from` to multiple recipients atomically.
    ///
    /// # Gas optimisation
    ///
    /// The sender's balance is read **once** before the loop and written
    /// **once** after. Each recipient's balance is read and written exactly
    /// once. This is significantly cheaper than looping over individual
    /// `transfer` calls, each of which would incur separate storage round-trips.
    ///
    /// # Atomicity
    ///
    /// If any validation step fails (insufficient balance, overflow, invalid
    /// amount) the function returns an error and **no** storage is mutated —
    /// the entire batch is rejected.
    pub fn batch_transfer(
        env: Env,
        from: Address,
        transfers: Vec<Transfer>,
    ) -> Result<u32, BatchError> {
        // Authorization
        from.require_auth();

        ensure_initialized(&env)?;

        // Validate batch size
        let count = transfers.len();
        if count == 0 {
            return Err(BatchError::EmptyBatch);
        }
        if count > MAX_BATCH_SIZE {
            return Err(BatchError::BatchTooLarge);
        }

        // --- Validation pass: sum total and check individual amounts --------
        let mut total: i128 = 0;
        for item in transfers.iter() {
            if item.amount <= 0 {
                return Err(BatchError::InvalidAmount);
            }
            total = total
                .checked_add(item.amount)
                .ok_or(BatchError::TotalOverflow)?;
        }

        // --- Balance check (single storage read) ----------------------------
        let sender_balance = read_balance(&env, &from);
        if sender_balance < total {
            return Err(BatchError::InsufficientBalance);
        }

        // --- Mutations (all-or-nothing) -------------------------------------
        // Debit sender once
        write_balance(&env, &from, sender_balance - total);

        // Credit each recipient and emit an event
        for item in transfers.iter() {
            let old = read_balance(&env, &item.to);
            let new_bal = old
                .checked_add(item.amount)
                .ok_or(BatchError::RecipientOverflow)?;
            write_balance(&env, &item.to, new_bal);

            env.events().publish(
                (NS, EV_XFER),
                TransferEvent {
                    from: from.clone(),
                    to: item.to.clone(),
                    amount: item.amount,
                },
            );
        }

        // Batch-complete summary event
        env.events().publish(
            (NS, EV_BATCH, from.clone()),
            BatchCompleteEvent {
                from,
                recipient_count: count,
                total_amount: total,
            },
        );

        Ok(count)
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the token balance for `user`.
    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn ensure_initialized(env: &Env) -> Result<(), BatchError> {
    if env.storage().instance().has(&DataKey::Initialized) {
        Ok(())
    } else {
        Err(BatchError::NotInitialized)
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

#[cfg(test)]
mod test;

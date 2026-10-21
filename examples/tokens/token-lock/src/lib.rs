#![allow(deprecated)]
//! # Token Lock Pattern
//!
//! A time-based lock ledger: each user can hold several lock entries, and every
//! entry becomes claimable once the ledger timestamp reaches its `unlock_time`.
//!
//! The contract is an accounting primitive — it records what is locked, it does
//! not custody SEP-41 tokens. Pair it with a token contract (see
//! `examples/tokens/06-token-wrapper`) when you need real transfers.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, vec, Address, Env, Symbol, Vec,
};

/// A single lock entry.
///
/// `amount` is the portion locked until `unlock_time` (ledger timestamp).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockEntry {
    pub amount: i128,
    pub unlock_time: u64,
}

/// Storage keys.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Map: user -> list of locks (kept in persistent storage).
    Locks(Address),
    /// Total amount currently locked for the user (for fast query).
    LockedTotal(Address),
}

/// Events
const NS: Symbol = symbol_short!("tokenlock");
const EV_LOCKED: Symbol = symbol_short!("locked");
const EV_UNLOCKED: Symbol = symbol_short!("unlocked");

/// A minimal token-lock ledger.
///
/// This contract does **not** move SEP-41 tokens.
/// It only tracks locked balances in contract state.
#[contract]
pub struct TokenLockContract;

#[contractimpl]
impl TokenLockContract {
    /// Lock `amount` on behalf of `user` until `unlock_time`.
    ///
    /// `user` authorizes the lock, so a third party cannot lock someone's balance.
    pub fn lock(env: Env, user: Address, amount: i128, unlock_time: u64) {
        user.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }
        let now = env.ledger().timestamp();
        if unlock_time <= now {
            panic!("unlock_time must be in the future");
        }

        let locks_key = DataKey::Locks(user.clone());
        let mut locks: Vec<LockEntry> = env
            .storage()
            .persistent()
            .get(&locks_key)
            .unwrap_or_else(|| vec![&env]);

        locks.push_back(LockEntry {
            amount,
            unlock_time,
        });
        env.storage().persistent().set(&locks_key, &locks);

        let total_key = DataKey::LockedTotal(user.clone());
        let prev_total: i128 = env.storage().persistent().get(&total_key).unwrap_or(0);
        let new_total = prev_total
            .checked_add(amount)
            .unwrap_or_else(|| panic!("overflow"));
        env.storage().persistent().set(&total_key, &new_total);

        env.events()
            .publish((NS, EV_LOCKED, user), (amount, unlock_time));
    }

    /// Release every matured lock entry for `user` and return the total released.
    ///
    /// Entries that have not reached `unlock_time` are left untouched.
    pub fn unlock(env: Env, user: Address) -> i128 {
        user.require_auth();

        let now = env.ledger().timestamp();
        let locks_key = DataKey::Locks(user.clone());
        let locks: Vec<LockEntry> = env
            .storage()
            .persistent()
            .get(&locks_key)
            .unwrap_or_else(|| vec![&env]);

        if locks.is_empty() {
            return 0;
        }

        let mut still_locked: Vec<LockEntry> = vec![&env];
        let mut unlocked_total: i128 = 0;

        for entry in locks.iter() {
            if entry.unlock_time <= now {
                unlocked_total = unlocked_total
                    .checked_add(entry.amount)
                    .unwrap_or_else(|| panic!("overflow"));
            } else {
                still_locked.push_back(entry);
            }
        }

        env.storage().persistent().set(&locks_key, &still_locked);

        let total_key = DataKey::LockedTotal(user.clone());
        let prev_total: i128 = env.storage().persistent().get(&total_key).unwrap_or(0);
        let new_total = prev_total
            .checked_sub(unlocked_total)
            .unwrap_or_else(|| panic!("underflow"));
        env.storage().persistent().set(&total_key, &new_total);

        if unlocked_total > 0 {
            env.events()
                .publish((NS, EV_UNLOCKED, user), unlocked_total);
        }

        unlocked_total
    }

    /// Total amount still locked for `user` (read-only, no auth required).
    pub fn locked_balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::LockedTotal(user))
            .unwrap_or(0)
    }

    /// Every lock entry held by `user`, matured or not (read-only, no auth required).
    pub fn lock_schedule(env: Env, user: Address) -> Vec<LockEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::Locks(user))
            .unwrap_or_else(|| vec![&env])
    }

    /// Amount of `user`'s locked balance that is claimable right now.
    pub fn unlockable_balance(env: Env, user: Address) -> i128 {
        let now = env.ledger().timestamp();
        let locks: Vec<LockEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::Locks(user))
            .unwrap_or_else(|| vec![&env]);

        let mut claimable: i128 = 0;
        for entry in locks.iter() {
            if entry.unlock_time <= now {
                claimable = claimable
                    .checked_add(entry.amount)
                    .unwrap_or_else(|| panic!("overflow"));
            }
        }
        claimable
    }
}

#[cfg(test)]
mod test;

#![no_std]

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
const NS: Symbol = symbol_short!("token_lock");
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
    /// Lock `amount` on behalf of the caller until `unlock_time`.
    ///
    /// The caller authorizes the lock via `require_auth()`.
    pub fn lock(env: Env, amount: i128, unlock_time: u64) {
        let user = env.invoker();
        user.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }
        let now = env.ledger().timestamp();
        if unlock_time <= now {
            panic!("unlock_time must be in the future");
        }

        // Load existing locks list.
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
        let new_total = prev_total.checked_add(amount).unwrap_or_else(|| panic!("overflow"));
        env.storage().persistent().set(&total_key, &new_total);

        env.events().publish(
            (NS, EV_LOCKED, user.clone()),
            (amount, unlock_time),
        );
    }

    /// Unlock all matured lock entries for the caller.
    ///
    /// Returns the total unlocked amount.
    pub fn unlock(env: Env) -> i128 {
        let user = env.invoker();
        user.require_auth();

        let now = env.ledger().timestamp();

        let locks_key = DataKey::Locks(user.clone());
        let mut locks: Vec<LockEntry> = env
            .storage()
            .persistent()
            .get(&locks_key)
            .unwrap_or_else(|| vec![&env]);

        if locks.is_empty() {
            return 0;
        }

        let mut still_locked: Vec<LockEntry> = vec![&env];
        let mut unlocked_total: i128 = 0;

        let mut i: u32 = 0;
        while i < locks.len() {
            let entry: LockEntry = locks.get(i).unwrap();
            if entry.unlock_time <= now {
                unlocked_total = unlocked_total
                    .checked_add(entry.amount)
                    .unwrap_or_else(|| panic!("overflow"));
            } else {
                still_locked.push_back(entry);
            }
            i += 1;
        }

        // Update storage.
        env.storage().persistent().set(&locks_key, &still_locked);
        let total_key = DataKey::LockedTotal(user.clone());
        let prev_total: i128 = env.storage().persistent().get(&total_key).unwrap_or(0);
        let new_total = prev_total
            .checked_sub(unlocked_total)
            .unwrap_or_else(|| panic!("underflow"));
        env.storage().persistent().set(&total_key, &new_total);

        if unlocked_total > 0 {
            env.events().publish((NS, EV_UNLOCKED, user.clone()), unlocked_total);
        }

        unlocked_total
    }

    /// Total amount currently locked for the caller.
    pub fn locked_balance(env: Env) -> i128 {
        let user = env.invoker();
        let total_key = DataKey::LockedTotal(user);
        env.storage().persistent().get(&total_key).unwrap_or(0)
    }

    /// Return the lock entries for the caller.
    pub fn lock_schedule(env: Env) -> Vec<LockEntry> {
        let user = env.invoker();
        let locks_key = DataKey::Locks(user);
        env.storage()
            .persistent()
            .get(&locks_key)
            .unwrap_or_else(|| vec![&env])
    }

    /// Query: locked balance for an arbitrary user (read-only, no auth required).
    pub fn locked_balance_of(env: Env, user: Address) -> i128 {
        let total_key = DataKey::LockedTotal(user);
        env.storage().persistent().get(&total_key).unwrap_or(0)
    }

    /// Query: lock schedule for an arbitrary user (read-only, no auth required).
    pub fn lock_schedule_of(env: Env, user: Address) -> Vec<LockEntry> {
        let locks_key = DataKey::Locks(user);
        env.storage()
            .persistent()
            .get(&locks_key)
            .unwrap_or_else(|| vec![&env])
    }
}

#[cfg(test)]
mod test;


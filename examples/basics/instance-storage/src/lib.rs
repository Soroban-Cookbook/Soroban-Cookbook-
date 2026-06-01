//! # Instance Storage
//!
//! Demonstrates `env.storage().instance()`, the storage map attached to a
//! deployed contract instance.
//!
//! ## What is Instance Storage?
//!
//! Instance storage is scoped to the contract instance address. Its TTL is tied
//! to the contract instance itself, so a live instance keeps its instance data
//! live as well. This differs from persistent storage, where each key has an
//! independent TTL.
//!
//! ## Comparison with the Other Storage Types
//!
//! | Property                  | Persistent        | Instance          | Temporary      |
//! |---------------------------|-------------------|-------------------|----------------|
//! | Data expires into archive | Yes               | Yes               | No, deleted    |
//! | TTL management            | Per-key           | Per-instance      | Per-key        |
//! | Size model                | Unbounded keys    | Limited instance  | Unbounded keys |
//! | Use when data is          | User/entity data  | Small shared data | Short-lived    |
//!
//! ## When to Prefer Instance Over Persistent
//!
//! Choose instance storage when:
//! - The data is small and shared across most or all contract calls.
//! - The data has a known upper bound, such as admin/config/protocol metadata.
//! - You want shared TTL management for contract-wide state.
//!
//! Avoid instance storage when:
//! - The data is per-user or per-entity and can grow without a tight bound.
//! - The data is large enough that loading it on every invocation would be costly.
//! - The data is only needed for a short period; use temporary storage instead.
//!
//! ## TTL Notes
//!
//! `extend_ttl(min_ledgers, max_ledgers)` keeps the entire instance alive.
//! Because all instance data is in the instance ledger entry, a single call
//! extends the TTL for every key in instance storage.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env, Symbol};

/// Keys for all instance-storage entries in this contract.
///
/// Using a typed enum instead of raw symbols makes key collisions harder and
/// keeps the key surface explicit.
#[contracttype]
#[derive(Clone)]
pub enum InstanceKey {
    /// Running count of successful invocations.
    TxCounter,

    /// Arbitrary named configuration value.
    Config(Symbol),
}

/// Extend TTL when it falls below this many ledgers.
const TTL_THRESHOLD: u32 = 1_000;

/// Extend up to this many ledgers from the current ledger.
const TTL_EXTEND_TO: u32 = 10_000;

#[contract]
pub struct InstanceStorageContract;

#[contractimpl]
impl InstanceStorageContract {
    /// Stores any `u64` value under a named config key in instance storage.
    ///
    /// Because instance TTL is shared, a single `extend_ttl` call here refreshes
    /// the lifetime of all instance keys at once. Persistent storage requires
    /// independent TTL management for each key.
    pub fn set_instance(env: Env, key: Symbol, value: u64) {
        let storage_key = InstanceKey::Config(key);
        env.storage().instance().set(&storage_key, &value);

        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
    }

    /// Returns the `u64` stored under `key`, or `None` if not set.
    pub fn get_instance(env: Env, key: Symbol) -> Option<u64> {
        let storage_key = InstanceKey::Config(key);
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
        env.storage().instance().get(&storage_key)
    }

    /// Increments the invocation counter and returns the new value.
    ///
    /// A transaction counter is a classic instance-storage candidate: it is
    /// small, contract-wide, and useful across calls, but it does not require a
    /// separate persistent entry per user.
    pub fn increment_counter(env: Env) -> u64 {
        let count: u64 = env
            .storage()
            .instance()
            .get(&InstanceKey::TxCounter)
            .unwrap_or(0)
            + 1;

        env.storage()
            .instance()
            .set(&InstanceKey::TxCounter, &count);

        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);

        count
    }

    /// Returns the current invocation counter, or 0 if never incremented.
    pub fn get_counter(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
        env.storage()
            .instance()
            .get(&InstanceKey::TxCounter)
            .unwrap_or(0)
    }

    /// Persists a named runtime configuration value.
    ///
    /// Example: `set_config(env, symbol_short!("fee_bps"), 30)` stores a
    /// 30-basis-point fee. Runtime config is a good instance-storage fit when
    /// the number of entries is small and known ahead of time.
    pub fn set_config(env: Env, key: Symbol, value: u64) {
        Self::set_instance(env, key, value);
    }

    /// Retrieves a named runtime configuration value.
    ///
    /// Returns `None` when the key has never been set, so callers can fall back
    /// to compile-time defaults without panicking.
    pub fn get_config(env: Env, key: Symbol) -> Option<u64> {
        Self::get_instance(env, key)
    }

    /// Explicitly bumps the instance TTL.
    ///
    /// Call this from an admin or keep-alive function when the contract might
    /// go idle for long periods. Because instance storage shares one TTL, a
    /// single call here protects every key stored in the instance.
    pub fn extend_ttl(env: Env) {
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
    }
}

#[cfg(test)]
mod test;

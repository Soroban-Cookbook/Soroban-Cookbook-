//! # Instance Storage
//!
//! Demonstrates `env.storage().instance()` — the middle ground between
//! persistent and temporary storage.
//!
//! ## What is Instance Storage?
//!
//! Instance storage is scoped to the *contract instance* (the deployed address).
//! All keys in instance storage share a single TTL that covers the entire
//! instance. This differs from persistent storage, where each key has its own
//! independent TTL.
//!
//! ## Comparison with the Other Storage Types
//!
//! | Property                    | Persistent        | Instance          | Temporary        |
//! |-----------------------------|-------------------|-------------------|------------------|
//! | Survives contract upgrade   | ✅ Yes            | ❌ No             | ❌ No            |
//! | TTL management              | Per-key           | Per-instance      | Per-key          |
//! | Relative cost               | Highest           | Medium            | Lowest           |
//! | Use when data is…           | Critical / long   | Instance-lifetime | Single-ledger    |
//!
//! ## When to Prefer Instance Over Persistent
//!
//! Choose instance storage when:
//! - The data is important during the life of the instance but does *not* need
//!   to outlive a contract upgrade (e.g. a transaction counter that resets is OK).
//! - You want cheaper rent than persistent while still keeping data across calls.
//! - You're managing shared state that should expire with the instance as a whole.
//!
//! Avoid instance storage when:
//! - The data MUST survive a `upgrade()` call (use persistent instead).
//! - The data is only needed for a single invocation (use temporary instead).
//!
//! ## TTL Notes
//!
//! `extend_ttl(min_ledgers, max_ledgers)` keeps the entire instance alive.
//! Call this whenever you read or write instance data so the instance never
//! expires unexpectedly.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env, Symbol};

// ────────────────────────────────────────────────────────────────────────────
// Storage key enum
// ────────────────────────────────────────────────────────────────────────────

/// Keys for all instance-storage entries in this contract.
///
/// Using a typed enum (rather than raw Symbols) makes key collisions impossible
/// at compile time and keeps the key surface explicit.
#[contracttype]
#[derive(Clone)]
pub enum InstanceKey {
    /// Running count of successful invocations. Use case 1: transaction counter.
    TxCounter,

    /// Arbitrary named configuration value. Use case 2: cached / runtime config.
    Config(Symbol),
}

// ────────────────────────────────────────────────────────────────────────────
// TTL constants (in ledgers; ~1 ledger ≈ 5 s on Stellar mainnet)
// ────────────────────────────────────────────────────────────────────────────

/// Extend TTL when it falls below this many ledgers.
const TTL_THRESHOLD: u32 = 1_000;

/// Extend up to this many ledgers from the current ledger.
const TTL_EXTEND_TO: u32 = 10_000;

// ────────────────────────────────────────────────────────────────────────────
// Contract
// ────────────────────────────────────────────────────────────────────────────

#[contract]
pub struct InstanceStorageContract;

#[contractimpl]
impl InstanceStorageContract {
    // ── Generic key/value helpers ──────────────────────────────────────────

    /// Stores any `u64` value under a named config key in instance storage.
    ///
    /// Because instance TTL is shared, a single `extend_ttl` call here
    /// refreshes the lifetime of *all* instance keys at once — unlike
    /// persistent storage where each key must be extended individually.
    pub fn set_instance(env: Env, key: Symbol, value: u64) {
        let storage_key = InstanceKey::Config(key);
        env.storage().instance().set(&storage_key, &value);

        // One call covers the entire instance — no per-key TTL bookkeeping.
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
    }

    /// Returns the `u64` stored under `key`, or `None` if not set.
    pub fn get_instance(env: Env, key: Symbol) -> Option<u64> {
        let storage_key = InstanceKey::Config(key);
        // Extend TTL on reads too — any access should keep the instance alive.
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
        env.storage().instance().get(&storage_key)
    }

    // ── Use case 1: Transaction counter ───────────────────────────────────
    //
    // A transaction counter is a classic instance-storage candidate:
    //   • It's per-instance state (each deployed address has its own count).
    //   • It changes on every call — cheap per-instance rent beats per-key rent.
    //   • It does NOT need to survive a contract upgrade; if the contract is
    //     replaced we're happy to start the counter fresh.

    /// Increments the invocation counter and returns the new value.
    ///
    /// Persistent storage equivalent would require `extend_ttl` per key on every
    /// write; here one `extend_ttl` covers everything, reducing ledger ops.
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

        // Shared TTL refresh — covers TxCounter AND all Config(…) keys.
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

    // ── Use case 2: Cached / runtime configuration overrides ──────────────
    //
    // Sometimes a contract needs operator-tunable parameters (fee rates, limits,
    // cooldown periods) that:
    //   • Change moderately often (not every call, but not never).
    //   • Are shared across all invocations of this instance.
    //   • Can be reset on upgrade if the operator decides to re-configure.
    //
    // Instance storage is ideal here:
    //   • Cheaper than persistent for data that doesn't need upgrade durability.
    //   • Simpler TTL management than per-key persistent rent.

    /// Persists a named runtime configuration value.
    ///
    /// Example: `set_config(env, symbol_short!("fee_bps"), 30)` stores 30 bps.
    pub fn set_config(env: Env, key: Symbol, value: u64) {
        // Reuse the generic helper — both use cases share the same TTL refresh.
        Self::set_instance(env, key, value);
    }

    /// Retrieves a named runtime configuration value.
    ///
    /// Returns `None` when the key has never been set, so callers can fall back
    /// to compile-time defaults without panicking.
    pub fn get_config(env: Env, key: Symbol) -> Option<u64> {
        Self::get_instance(env, key)
    }

    // ── TTL management ─────────────────────────────────────────────────────

    /// Explicitly bumps the instance TTL.
    ///
    /// Call this from an admin or keep-alive function when the contract might
    /// go idle for long periods.  Because instance storage shares one TTL,
    /// a single call here protects every key stored in the instance.
    ///
    /// # Difference from persistent TTL
    /// With persistent storage you must call `extend_ttl` once **per key**.
    /// With instance storage this single call is sufficient for the whole state.
    pub fn extend_ttl(env: Env) {
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
    }
}

mod test;

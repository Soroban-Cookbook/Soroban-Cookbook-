#![allow(deprecated)]
//! # Pattern 2 — Versioned Storage & Migration
//!
//! This module shows how to manage storage layout changes across contract
//! upgrades without corrupting existing data.
//!
//! ## The core problem
//!
//! When you call `update_current_contract_wasm`, the new code takes over
//! immediately but the on-chain storage keys and their encoded values are
//! left exactly as the old code wrote them. If the new code reads a key and
//! expects a different shape (e.g. an extra field, a renamed variant, a
//! changed type), it will either panic or silently misinterpret old data.
//!
//! ## The solution: a `StorageVersion` sentinel key
//!
//! 1. Every deployed version of the contract defines a `CURRENT_VERSION`
//!    constant.
//! 2. A `StorageVersion` key in instance storage holds the version whose
//!    schema is currently on-chain.
//! 3. After upgrading the WASM, call `migrate()`. It reads the stored version,
//!    runs the appropriate data-transformation steps, and bumps the stored
//!    version to `CURRENT_VERSION`.
//! 4. Idempotency: calling `migrate()` when the stored version already equals
//!    `CURRENT_VERSION` is a safe no-op.
//!
//! ## Storage versioning rules (for real projects)
//!
//! - **Never rename or remove a `DataKey` variant** between versions — the
//!   encoded key bytes are baked into on-chain storage. Rename the concept in
//!   code with a new variant and write a migration step.
//! - **Never change the type stored under an existing key** without a
//!   migration step that reads the old encoding and writes the new one.
//! - **Adding a new key** is always safe — old storage simply won't have the
//!   key; use `unwrap_or` / `unwrap_or_default` defensively.
//! - **Keep a linear chain of migrations** (v1→v2, v2→v3, …) so a contract
//!   that missed one upgrade can catch up by running `migrate()` once per
//!   skipped version.
//!
//! ## Simulated schema change in this example
//!
//! - **v1 schema**: `Counter { val: i64 }` stored under `DataKey::Counter`.
//! - **v2 schema**: `CounterV2 { val: i64, last_updated: u64 }` — same `val`,
//!   plus a new `last_updated` timestamp field.
//!
//! The migration reads `Counter` from storage, constructs `CounterV2` (setting
//! `last_updated` to the current ledger timestamp), and writes `CounterV2`
//! back under the same key. Then it bumps `StorageVersion` to 2.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, BytesN, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Version constants
// ---------------------------------------------------------------------------

/// The storage-schema version produced by *this* WASM binary.
///
/// Bump this constant in every new version that changes the on-chain schema.
pub const CURRENT_VERSION: u32 = 2;

/// The version before the schema change modelled in this example.
pub const LEGACY_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const NS: Symbol = symbol_short!("ver_upg");
const EV_INIT: Symbol = symbol_short!("init");
const EV_MIGRATE: Symbol = symbol_short!("migrate");
const EV_UPGRADE: Symbol = symbol_short!("upgrade");
const EV_INCREMENT: Symbol = symbol_short!("increment");

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

/// Storage key enum for this contract.
///
/// # Key stability guarantee
///
/// The *encoded bytes* of each variant are the on-chain storage key. Adding a
/// new variant is safe. Changing or removing an existing variant is a breaking
/// storage change that requires a migration step.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Stores the admin `Address`.
    Admin,
    /// Stores a `u32` indicating the schema version currently on-chain.
    ///
    /// Absent on a freshly deployed contract (treated as version 1).
    StorageVersion,
    /// The counter value. Shape changes between v1 and v2 — see module docs.
    Counter,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Counter shape for schema version 1.
///
/// In a real upgrade scenario this type would live in the *old* crate version.
/// We keep it here so the test can write v1 data and verify the migration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CounterV1 {
    pub val: i64,
}

/// Counter shape for schema version 2 — adds `last_updated`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CounterV2 {
    pub val: i64,
    /// Ledger timestamp of the last increment (or the migration timestamp if
    /// the counter was migrated from v1 without a subsequent increment).
    pub last_updated: u64,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VersionedError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    /// `migrate()` was called but the stored version is already current.
    AlreadyMigrated = 4,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct VersionedUpgradeContract;

#[contractimpl]
impl VersionedUpgradeContract {
    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// Initialise the contract at schema version 1.
    ///
    /// Sets up the admin and writes an initial `CounterV1` with `val = 0`.
    /// Records `StorageVersion = LEGACY_VERSION` so `migrate()` knows there
    /// is work to do after the v2 WASM is deployed.
    pub fn initialize(env: Env, admin: soroban_sdk::Address) -> Result<(), VersionedError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VersionedError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        // Write v1-schema counter.
        env.storage()
            .instance()
            .set(&DataKey::Counter, &CounterV1 { val: 0 });
        // Record that storage is currently at the legacy (v1) schema.
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &LEGACY_VERSION);

        env.events()
            .publish((NS, EV_INIT, admin), env.ledger().timestamp());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Upgrade
    // -----------------------------------------------------------------------

    /// Replace this contract's WASM binary (admin-only).
    ///
    /// After calling this, the new code takes effect on the next invocation.
    /// You must then call [`migrate`] once to transform on-chain storage from
    /// the v1 schema to the v2 schema before using any v2-specific entry
    /// points.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), VersionedError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        env.events().publish(
            (NS, EV_UPGRADE, admin, new_wasm_hash.clone()),
            env.ledger().timestamp(),
        );

        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Migration
    // -----------------------------------------------------------------------

    /// Run pending storage-schema migrations up to `CURRENT_VERSION`.
    ///
    /// # Idempotency
    ///
    /// Calling `migrate()` when the stored version is already `CURRENT_VERSION`
    /// returns [`VersionedError::AlreadyMigrated`] — it is safe to call in a
    /// retry loop.
    ///
    /// # Access control
    ///
    /// The migration is admin-gated to prevent an attacker from triggering a
    /// schema change at an unexpected time (e.g. before the new WASM is
    /// actually live).
    ///
    /// # Multi-version gap handling
    ///
    /// The chain of `if stored_version == N` blocks runs in order. A contract
    /// that missed an intermediate upgrade catches up in a single `migrate()`
    /// call because each block bumps the in-memory `stored_version` so the
    /// next block's condition is immediately re-evaluated.
    pub fn migrate(env: Env) -> Result<(), VersionedError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        let stored_version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::StorageVersion)
            .unwrap_or(LEGACY_VERSION);

        if stored_version >= CURRENT_VERSION {
            return Err(VersionedError::AlreadyMigrated);
        }

        // ── v1 → v2 migration ───────────────────────────────────────────────
        //
        // Read the old CounterV1 value (if present) and rewrite it as
        // CounterV2. If the key is absent (e.g. the contract was initialised
        // without a counter), default to val = 0 so the migration is still
        // valid.
        //
        // Pattern to adapt for your own contracts:
        //   1. Read old value with the old type.
        //   2. Build the new value, filling in new fields with sensible
        //      defaults (e.g. current timestamp, zero, empty vec).
        //   3. Write the new value back under the SAME key.
        //   4. Bump the stored version counter.
        if stored_version == LEGACY_VERSION {
            let old: CounterV1 = env
                .storage()
                .instance()
                .get(&DataKey::Counter)
                .unwrap_or(CounterV1 { val: 0 });

            let new = CounterV2 {
                val: old.val,
                last_updated: env.ledger().timestamp(),
            };

            env.storage().instance().set(&DataKey::Counter, &new);
            // Bump version in-memory so a future block in this same call can
            // continue the migration chain (v2→v3, etc.).
            env.storage()
                .instance()
                .set(&DataKey::StorageVersion, &CURRENT_VERSION);
        }

        env.events().publish(
            (NS, EV_MIGRATE),
            (stored_version, CURRENT_VERSION, env.ledger().timestamp()),
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Business logic (v2)
    // -----------------------------------------------------------------------

    /// Increment the counter by `amount` and record the ledger timestamp.
    ///
    /// Requires the storage to already be at v2 schema (i.e. `migrate()` has
    /// been called). Panics with a descriptive message if called on v1 data so
    /// the problem is obvious during development.
    pub fn increment(env: Env, amount: i64) -> Result<CounterV2, VersionedError> {
        let _admin = read_admin(&env)?;

        let version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::StorageVersion)
            .unwrap_or(LEGACY_VERSION);

        if version < CURRENT_VERSION {
            // Developer error: called v2 logic before running migration.
            panic!("storage not migrated to v2; call migrate() first");
        }

        let mut counter: CounterV2 =
            env.storage()
                .instance()
                .get(&DataKey::Counter)
                .unwrap_or(CounterV2 {
                    val: 0,
                    last_updated: 0,
                });

        counter.val = counter.val.checked_add(amount).expect("counter overflow");
        counter.last_updated = env.ledger().timestamp();

        env.storage().instance().set(&DataKey::Counter, &counter);

        env.events()
            .publish((NS, EV_INCREMENT), (counter.val, counter.last_updated));

        Ok(counter)
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the current on-chain schema version.
    pub fn storage_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::StorageVersion)
            .unwrap_or(LEGACY_VERSION)
    }

    /// Return the v2 counter, or an error if storage has not been migrated.
    pub fn get_counter(env: Env) -> Result<CounterV2, VersionedError> {
        let version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::StorageVersion)
            .unwrap_or(LEGACY_VERSION);

        if version < CURRENT_VERSION {
            panic!("storage not migrated to v2; call migrate() first");
        }

        Ok(env
            .storage()
            .instance()
            .get(&DataKey::Counter)
            .unwrap_or(CounterV2 {
                val: 0,
                last_updated: 0,
            }))
    }

    /// Return the admin address, or an error if uninitialised.
    pub fn admin(env: Env) -> Result<soroban_sdk::Address, VersionedError> {
        read_admin(&env)
    }
}

// ---------------------------------------------------------------------------
// Test-only helpers
// ---------------------------------------------------------------------------

/// Write a `CounterV1` value directly into storage.
///
/// This function exists only so tests can seed the contract in a "pre-upgrade"
/// state (v1 schema) and then verify that `migrate()` transforms it correctly.
/// It is compiled out in production builds.
#[cfg(any(test, feature = "testutils"))]
#[allow(unexpected_cfgs)]
pub fn seed_v1_counter(env: &Env, val: i64) {
    env.storage()
        .instance()
        .set(&DataKey::Counter, &CounterV1 { val });
    env.storage()
        .instance()
        .set(&DataKey::StorageVersion, &LEGACY_VERSION);
}

// ---------------------------------------------------------------------------
// Helpers (private)
// ---------------------------------------------------------------------------

fn read_admin(env: &Env) -> Result<soroban_sdk::Address, VersionedError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(VersionedError::NotInitialized)
}

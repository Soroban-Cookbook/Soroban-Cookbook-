#![allow(deprecated)]
//! # Pattern 3 — Safe Initialisation Guards
//!
//! This module covers two closely related guards that prevent incorrect
//! reinitialisation of a contract:
//!
//! ## Guard A — Double-init prevention
//!
//! The `initialize` function writes an `Initialized` flag to instance storage
//! on first call and panics on any subsequent call. This is the simplest
//! possible guard and should be the default for any contract that has a
//! one-time setup phase.
//!
//! ```text
//! First call:   Initialized absent → write flag, proceed
//! Second call:  Initialized present → return AlreadyInitialized error
//! ```
//!
//! ## Guard B — Post-upgrade initialisation hook
//!
//! After a WASM upgrade the contract may need to run new one-time setup logic
//! (e.g. initialise a new storage key that v1 did not have). This is different
//! from a storage *migration* (which transforms existing values) — it is about
//! seeding *brand-new* state that the old code never wrote.
//!
//! The pattern uses the same `StorageVersion` sentinel from the
//! `versioned_upgrade` module:
//!
//! ```text
//! post_upgrade_init(expected_version):
//!   if stored_version == expected_version   → already ran, return AlreadyRan
//!   if stored_version != expected_version-1 → wrong call order, panic
//!   run new setup logic
//!   bump StorageVersion to expected_version
//! ```
//!
//! This means `post_upgrade_init` is:
//! - Idempotent: safe to retry.
//! - Ordered: cannot run "out of sequence" (e.g. skipping a version bump).
//! - Non-destructive: calling it a second time after it succeeded is a no-op.
//!
//! ## Why not just re-use `initialize`?
//!
//! Calling `initialize` after an upgrade would overwrite the admin address and
//! any other state set during first deployment — effectively resetting the
//! contract. `post_upgrade_init` is version-scoped so it can run *new* setup
//! without touching state that is already correct.
//!
//! ## Summary of entry points
//!
//! | Function | Purpose |
//! |----------|---------|
//! | `initialize` | One-time first-deploy setup; errors on repeat |
//! | `upgrade` | WASM swap (admin-only) |
//! | `post_upgrade_init` | Version-scoped post-upgrade setup; safe to retry |
//! | `is_initialized` | Query: was `initialize` called? |
//! | `setup_version` | Query: which post-upgrade init version has run? |

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, BytesN, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Version constants
// ---------------------------------------------------------------------------

/// The version number written by the *current* `post_upgrade_init` call.
///
/// In a real project, bump this in every WASM version that ships a new
/// `post_upgrade_init` body.
pub const UPGRADE_INIT_VERSION: u32 = 2;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const NS: Symbol = symbol_short!("init_grd");
const EV_INIT: Symbol = symbol_short!("init");
const EV_UPGRADE: Symbol = symbol_short!("upgrade");
const EV_POST_INIT: Symbol = symbol_short!("post_init");

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Present (value = `true`) once `initialize` has run successfully.
    Initialized,
    /// Stores the admin `Address`.
    Admin,
    /// Tracks which `post_upgrade_init` version has been run.
    ///
    /// Absent if no post-upgrade init has ever run (equivalent to version 1,
    /// i.e. the initial deployment state).
    SetupVersion,
    /// Example new-in-v2 state: a feature-flag boolean.
    FeatureFlagV2,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum InitError {
    /// `initialize` was called on a contract that is already initialised.
    AlreadyInitialized = 1,
    /// A function that requires prior initialisation was called too early.
    NotInitialized = 2,
    /// The caller is not the stored admin.
    Unauthorized = 3,
    /// `post_upgrade_init` was called but this version's init already ran.
    AlreadyRan = 4,
    /// `post_upgrade_init` was called out of sequence.
    ///
    /// This signals a logic error: e.g. the operator skipped a WASM version
    /// and the stored setup-version does not match the expected predecessor.
    OutOfSequence = 5,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct InitGuardContract;

#[contractimpl]
impl InitGuardContract {
    // -----------------------------------------------------------------------
    // Guard A: one-time initialisation
    // -----------------------------------------------------------------------

    /// First-deploy setup. Sets the admin and writes the `Initialized` flag.
    ///
    /// Returns [`InitError::AlreadyInitialized`] on any repeat call, so it
    /// can never be used to hijack the admin slot after deployment.
    ///
    /// # Design note
    ///
    /// The guard relies on `instance` storage (which persists for the lifetime
    /// of the contract) rather than a constructor argument or deploy-time
    /// parameter. This makes the guard robust against all invocation paths,
    /// including cross-contract calls.
    pub fn initialize(env: Env, admin: soroban_sdk::Address) -> Result<(), InitError> {
        // ── Guard A ──────────────────────────────────────────────────────────
        // Check the flag *before* writing anything so partial state is never
        // committed.
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(InitError::AlreadyInitialized);
        }

        admin.require_auth();

        // Write all initial state atomically (same transaction).
        env.storage().instance().set(&DataKey::Admin, &admin);
        // The flag is the source of truth — its *presence* is the guard.
        env.storage().instance().set(&DataKey::Initialized, &true);

        env.events()
            .publish((NS, EV_INIT, admin), env.ledger().timestamp());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Upgrade
    // -----------------------------------------------------------------------

    /// Replace this contract's WASM binary (admin-only).
    ///
    /// After this call, follow up with [`post_upgrade_init`] to seed any new
    /// state required by the v2 code before calling other v2 entry points.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), InitError> {
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
    // Guard B: post-upgrade initialisation hook
    // -----------------------------------------------------------------------

    /// Run one-time setup logic for `UPGRADE_INIT_VERSION`.
    ///
    /// # Calling convention
    ///
    /// Call this exactly once after deploying the v2 WASM. Subsequent calls
    /// return [`InitError::AlreadyRan`] and are otherwise harmless.
    ///
    /// # Version ordering
    ///
    /// The function verifies that the stored `SetupVersion` is exactly
    /// `UPGRADE_INIT_VERSION - 1` before running, so it cannot fire out of
    /// order. Pass `expected_version = UPGRADE_INIT_VERSION` from the client.
    ///
    /// # What it does in this example
    ///
    /// Seeds `FeatureFlagV2` to `false`. In a real contract this could seed
    /// new configuration keys, initialise a new sub-module, or grant default
    /// roles to an admin — anything that is *new* state, not a transformation
    /// of existing state (that is the job of `migrate()`).
    pub fn post_upgrade_init(env: Env, expected_version: u32) -> Result<(), InitError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        let stored: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SetupVersion)
            .unwrap_or(1); // absent → version 1 (initial deployment)

        // ── Already ran ──────────────────────────────────────────────────────
        if stored >= expected_version {
            return Err(InitError::AlreadyRan);
        }

        // ── Out-of-sequence guard ────────────────────────────────────────────
        // `expected_version` must be exactly `stored + 1`. If the operator
        // somehow calls the v3 hook before the v2 hook, this catches it.
        if expected_version != stored + 1 {
            return Err(InitError::OutOfSequence);
        }

        // ── New setup logic for this version ─────────────────────────────────
        //
        // Seed the FeatureFlagV2 key that did not exist in v1. Using
        // `set` (not `try_set` / conditional) is intentional: this is the
        // authoritative first-write and it should always succeed.
        env.storage()
            .instance()
            .set(&DataKey::FeatureFlagV2, &false);

        // Bump the stored setup version *last*, after all new state has been
        // written, so a partial failure (if any write panics) leaves the
        // version un-bumped and the call can be retried.
        env.storage()
            .instance()
            .set(&DataKey::SetupVersion, &expected_version);

        env.events().publish(
            (NS, EV_POST_INIT),
            (expected_version, env.ledger().timestamp()),
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return `true` if `initialize` has been called successfully.
    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Return which post-upgrade-init version has been applied.
    ///
    /// Returns `1` if no post-upgrade init has ever run (initial state).
    pub fn setup_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::SetupVersion)
            .unwrap_or(1)
    }

    /// Return the current value of `FeatureFlagV2`, or `None` if the v2
    /// post-upgrade init has not yet run.
    pub fn feature_flag_v2(env: Env) -> Option<bool> {
        env.storage().instance().get(&DataKey::FeatureFlagV2)
    }

    /// Return the admin address, or an error if uninitialised.
    pub fn admin(env: Env) -> Result<soroban_sdk::Address, InitError> {
        read_admin(&env)
    }
}

// ---------------------------------------------------------------------------
// Helpers (private)
// ---------------------------------------------------------------------------

fn read_admin(env: &Env) -> Result<soroban_sdk::Address, InitError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(InitError::NotInitialized)
}

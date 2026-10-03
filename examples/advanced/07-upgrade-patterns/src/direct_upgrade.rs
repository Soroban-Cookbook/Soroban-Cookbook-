#![allow(deprecated)]
//! # Pattern 1 — Direct WASM Upgrade
//!
//! The simplest possible upgrade pattern: a single admin address may swap the
//! contract's WASM binary at any time by calling [`DirectUpgradeContract::upgrade`].
//!
//! ## What `env.deployer().update_current_contract_wasm` does
//!
//! Soroban stores contracts as (address → WASM hash) pairs. This host function
//! replaces the hash recorded for *this* contract with the supplied hash. The
//! very next invocation of any entry point on this contract address will
//! execute code from the new WASM. Storage is untouched — all keys, types, and
//! values carry over exactly as-is.
//!
//! ```text
//! Before upgrade:
//!   contract_address  ──►  wasm_hash_v1  ──►  code_v1
//!
//! After upgrade(wasm_hash_v2):
//!   contract_address  ──►  wasm_hash_v2  ──►  code_v2
//!   (all storage unchanged)
//! ```
//!
//! ## When to use this pattern
//!
//! - Development / prototype: fast iteration, single trusted admin.
//! - Low-stakes contracts where an instant upgrade is acceptable.
//!
//! ## When NOT to use this pattern alone
//!
//! A direct upgrade is irreversible once executed and takes effect immediately.
//! For production contracts consider adding:
//! - A **timelock** (see `31-proxy-admin`) so stakeholders can review the new
//!   WASM hash before it becomes live.
//! - A **multi-sig** admin (see `01-multi-party-auth`) so no single key can
//!   push a rogue upgrade.
//!
//! ## Storage layout note
//!
//! This contract stores only one instance-storage key (`Admin`). If a future
//! version adds more keys, read the versioned-upgrade example to understand
//! how to migrate old data without corruption.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, BytesN, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Event namespace — every event emitted by this contract uses this as
/// the first topic so off-chain indexers can filter by contract family.
const NS: Symbol = symbol_short!("dir_upg");
const EV_INIT: Symbol = symbol_short!("init");
const EV_UPGRADE: Symbol = symbol_short!("upgrade");

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

/// All persistent state lives under one of these typed keys.
///
/// Using an enum (rather than raw `Symbol` constants) gives the compiler
/// exhaustiveness checking and makes it easy to audit every key the contract
/// touches.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Stores the `Address` of the account authorised to call `upgrade`.
    Admin,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum UpgradeError {
    /// `initialize` was called on an already-initialised contract.
    AlreadyInitialized = 1,
    /// A function that requires prior initialisation was called before it.
    NotInitialized = 2,
    /// The caller is not the stored admin address.
    Unauthorized = 3,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct DirectUpgradeContract;

#[contractimpl]
impl DirectUpgradeContract {
    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// Initialise the contract by recording the admin address.
    ///
    /// This function is *one-shot*: calling it a second time returns
    /// [`UpgradeError::AlreadyInitialized`]. See `init_guard` for a deeper
    /// treatment of initialisation safety.
    pub fn initialize(env: Env, admin: soroban_sdk::Address) -> Result<(), UpgradeError> {
        // Guard: refuse if already set up.
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(UpgradeError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);

        env.events()
            .publish((NS, EV_INIT, admin), env.ledger().timestamp());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Upgrade
    // -----------------------------------------------------------------------

    /// Replace this contract's WASM with the binary identified by
    /// `new_wasm_hash`.
    ///
    /// # Access control
    ///
    /// Only the stored admin may call this. The admin's signature is verified
    /// by the Soroban host via `require_auth` — no off-chain trust required.
    ///
    /// # Effect
    ///
    /// The swap is atomic at the ledger level: either it succeeds fully or the
    /// transaction reverts. Storage is *not* modified by this call; data
    /// migration (if needed) is a separate concern — see the
    /// `versioned_upgrade` module.
    ///
    /// # Test-environment behaviour
    ///
    /// `update_current_contract_wasm` will produce a host error in unit tests
    /// because the test environment has no real WASM registry. All guard logic
    /// (auth check, initialisation check) executes before that host call and
    /// is therefore fully testable. See `test.rs` for the pattern.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), UpgradeError> {
        let admin = read_admin(&env)?;

        // Require the admin to have signed the transaction that contains this
        // invocation. The host throws `Auth::InvalidAction` if the signature
        // is absent, before our code even returns.
        admin.require_auth();

        // Emit the event *before* the deployer call so that even if the host
        // call reverts the event is still part of the failed-transaction
        // diagnostic record.
        env.events().publish(
            (NS, EV_UPGRADE, admin, new_wasm_hash.clone()),
            env.ledger().timestamp(),
        );

        // ── The upgrade itself ──────────────────────────────────────────────
        //
        // After this line returns, the next invocation of any entry point on
        // this contract address will run code_v2. The current invocation
        // continues to completion under code_v1.
        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the current admin address, or an error if uninitialised.
    pub fn admin(env: Env) -> Result<soroban_sdk::Address, UpgradeError> {
        read_admin(&env)
    }
}

// ---------------------------------------------------------------------------
// Helpers (private)
// ---------------------------------------------------------------------------

/// Read the admin from instance storage, returning `NotInitialized` if absent.
fn read_admin(env: &Env) -> Result<soroban_sdk::Address, UpgradeError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(UpgradeError::NotInitialized)
}

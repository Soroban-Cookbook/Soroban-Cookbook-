//! # Beacon Contract
//!
//! The Beacon is the single source-of-truth for the active implementation address.
//! An authorised admin can upgrade the implementation, which instantly propagates
//! to every Proxy that holds a reference to this Beacon.
//!
//! ## Storage keys
//!
//! All entries use `persistent` storage so they survive ledger TTL extension.
//!
//! | Key constant | Type | Description |
//! |---|---|---|
//! | `ADMIN_KEY` | `Address` | Who can call `upgrade` |
//! | `IMPL_KEY` | `Address` | Current implementation address |
//! | `VERSION_KEY` | `u32` | Current version number (starts at 1) |
//! | `DataKey::VersionLog(n)` | `VersionEntry` | Historical record for version n |

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Storage key symbols
// ---------------------------------------------------------------------------

const ADMIN_KEY: Symbol = symbol_short!("admin");
const IMPL_KEY: Symbol = symbol_short!("impl");
const VERSION_KEY: Symbol = symbol_short!("version");

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Composite storage key for the version history log.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Historical entry for a specific version number.
    VersionLog(u32),
}

/// A single record in the version history stored by the Beacon.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionEntry {
    /// The implementation address associated with this version.
    pub implementation: Address,
    /// Ledger timestamp when this version was activated.
    pub activated_at: u64,
    /// Freeform label supplied by the upgrader (max 9 chars due to Symbol constraints).
    pub label: Symbol,
}

// ---------------------------------------------------------------------------
// Contract
//
// `#[contract]` is always present so the SDK macro generates BeaconContractClient
// for use in tests.  When compiling to cdylib (WASM), build with:
//   --no-default-features --features beacon
// to ensure only this contract's symbols are exported.
// ---------------------------------------------------------------------------

/// The Beacon contract.
///
/// Stores the live implementation pointer and a full version history.
/// Any number of Proxy contracts can point to this single Beacon, so a single
/// call to `upgrade` propagates to all of them simultaneously.
#[contract]
pub struct BeaconContract;

#[contractimpl]
impl BeaconContract {
    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// Initialise the Beacon with an admin and the first implementation.
    ///
    /// Can only be called once. Subsequent calls panic with `"Already initialized"`.
    ///
    /// # Arguments
    /// * `admin` - address authorised to call `upgrade`
    /// * `implementation` - address of the initial implementation contract
    /// * `label` - human-readable tag for version 1 (e.g. `Symbol::new(&env, "v1")`)
    pub fn init(env: Env, admin: Address, implementation: Address, label: Symbol) {
        if env.storage().persistent().has(&ADMIN_KEY) {
            panic!("Already initialized");
        }

        env.storage().persistent().set(&ADMIN_KEY, &admin);
        env.storage().persistent().set(&IMPL_KEY, &implementation);
        env.storage().persistent().set(&VERSION_KEY, &1u32);

        // Log version 1.
        env.storage().persistent().set(
            &DataKey::VersionLog(1),
            &VersionEntry {
                implementation: implementation.clone(),
                activated_at: env.ledger().timestamp(),
                label,
            },
        );

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("beacon"), symbol_short!("init")),
            (implementation, 1u32),
        );
    }

    // -----------------------------------------------------------------------
    // Upgrade
    // -----------------------------------------------------------------------

    /// Upgrade the implementation to a new address.
    ///
    /// Only the admin can call this. The version counter is incremented and a
    /// `VersionEntry` is appended to the history log.
    ///
    /// # Arguments
    /// * `new_implementation` - address of the new implementation contract
    /// * `label` - human-readable tag for this version
    ///
    /// # Panics
    /// * `"Not initialized"` if `init` has not been called
    /// * auth failure if caller is not the admin
    pub fn upgrade(env: Env, new_implementation: Address, label: Symbol) {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN_KEY)
            .unwrap_or_else(|| panic!("Not initialized"));

        admin.require_auth();

        let prev_version: u32 = env
            .storage()
            .persistent()
            .get(&VERSION_KEY)
            .unwrap_or(0);

        let new_version = prev_version
            .checked_add(1)
            .unwrap_or_else(|| panic!("Version overflow"));

        env.storage()
            .persistent()
            .set(&IMPL_KEY, &new_implementation);
        env.storage()
            .persistent()
            .set(&VERSION_KEY, &new_version);

        env.storage().persistent().set(
            &DataKey::VersionLog(new_version),
            &VersionEntry {
                implementation: new_implementation.clone(),
                activated_at: env.ledger().timestamp(),
                label,
            },
        );

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("beacon"), symbol_short!("upgraded")),
            (new_implementation, new_version),
        );
    }

    // -----------------------------------------------------------------------
    // Admin transfer
    // -----------------------------------------------------------------------

    /// Transfer admin rights to a new address.
    ///
    /// Only the current admin can call this.
    ///
    /// # Arguments
    /// * `new_admin` - the address that will become the new admin
    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN_KEY)
            .unwrap_or_else(|| panic!("Not initialized"));

        admin.require_auth();

        env.storage().persistent().set(&ADMIN_KEY, &new_admin);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("beacon"), symbol_short!("adm_xfr")),
            new_admin,
        );
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the address of the current implementation.
    pub fn get_implementation(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&IMPL_KEY)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    /// Return the current version number.
    pub fn get_version(env: Env) -> u32 {
        env.storage()
            .persistent()
            .get(&VERSION_KEY)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    /// Return the admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&ADMIN_KEY)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    /// Return the `VersionEntry` for a specific version number.
    ///
    /// Panics with `"Version not found"` if the requested version does not exist.
    pub fn get_version_entry(env: Env, version: u32) -> VersionEntry {
        env.storage()
            .persistent()
            .get(&DataKey::VersionLog(version))
            .unwrap_or_else(|| panic!("Version not found"))
    }
}

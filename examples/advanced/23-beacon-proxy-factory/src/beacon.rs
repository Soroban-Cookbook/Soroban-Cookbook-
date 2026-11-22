#![allow(deprecated)]
//! # Beacon Contract
//!
//! The Beacon is the single source-of-truth for the active implementation address.
//! An authorised admin can upgrade the implementation, which propagates to every
//! Proxy that holds a reference to this Beacon.
//!
//! Upgrades use a two-step commitment: the admin first calls `stage_upgrade`,
//! which records the new implementation and starts a timelock, and only after
//! the delay has elapsed can `execute_upgrade` apply it. During the delay the
//! staged upgrade can be aborted with `cancel_upgrade`, and proxy operators have
//! a window to review it or re-point their proxies to a different beacon.
//!
//! In the factory pattern the beacon admin is set to the factory contract itself,
//! so only the factory can trigger upgrades (ensuring the factory's proxy registry
//! remains the single point of control).
//!
//! ## Storage keys (all `persistent`)
//!
//! | Key | Type | Description |
//! |---|---|---|
//! | `ADMIN_KEY` | `Address` | Who can stage, execute and cancel upgrades |
//! | `IMPL_KEY` | `Address` | Current implementation address |
//! | `VERSION_KEY` | `u32` | Current version number (starts at 1) |
//! | `DELAY_KEY` | `u64` | Upgrade timelock delay in seconds |
//! | `PENDING_KEY` | `PendingUpgrade` | Staged upgrade awaiting execution |
//! | `DataKey::VersionLog(n)` | `VersionEntry` | Historical record for version n |

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Storage key symbols
// ---------------------------------------------------------------------------

const ADMIN_KEY: Symbol = symbol_short!("admin");
const IMPL_KEY: Symbol = symbol_short!("impl");
const VERSION_KEY: Symbol = symbol_short!("version");
const DELAY_KEY: Symbol = symbol_short!("upg_delay");
const PENDING_KEY: Symbol = symbol_short!("pending");

/// Default timelock between `stage_upgrade` and `execute_upgrade` (24 hours).
pub const DEFAULT_UPGRADE_DELAY: u64 = 24 * 60 * 60;

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

/// An upgrade that has been staged but not yet executed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingUpgrade {
    /// The implementation address that will become active.
    pub implementation: Address,
    /// Label for the new version.
    pub label: Symbol,
    /// Earliest ledger timestamp at which `execute_upgrade` may be called.
    pub execute_after: u64,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// The Beacon contract.
///
/// Stores the live implementation pointer and a full version history.
/// Any number of Proxy contracts can point to this single Beacon, so a single
/// executed upgrade propagates to all of them simultaneously.
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
    /// * `admin` - address authorised to stage, execute and cancel upgrades (typically the factory)
    /// * `implementation` - address of the initial implementation contract
    /// * `label` - human-readable tag for version 1 (e.g. `Symbol::new(&env, "v1")`)
    pub fn init(env: Env, admin: Address, implementation: Address, label: Symbol) {
        if env.storage().persistent().has(&ADMIN_KEY) {
            panic!("Already initialized");
        }
        admin.require_auth();

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

    /// Stage an upgrade to a new implementation (step 1 of 2).
    ///
    /// Only the admin can call this. The upgrade is recorded together with the
    /// earliest time it may be executed (`now + upgrade delay`). Proxies keep
    /// using the current implementation until `execute_upgrade` is called.
    ///
    /// # Arguments
    /// * `new_implementation` - address of the new implementation contract
    /// * `label` - human-readable tag for this version
    ///
    /// # Panics
    /// * `"Not initialized"` if `init` has not been called
    /// * `"Upgrade already staged"` if another upgrade is pending
    /// * auth failure if caller is not the admin
    pub fn stage_upgrade(env: Env, new_implementation: Address, label: Symbol) {
        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        if env.storage().persistent().has(&PENDING_KEY) {
            panic!("Upgrade already staged");
        }

        let execute_after = env
            .ledger()
            .timestamp()
            .checked_add(Self::get_upgrade_delay(env.clone()))
            .unwrap_or_else(|| panic!("Timestamp overflow"));

        env.storage().persistent().set(
            &PENDING_KEY,
            &PendingUpgrade {
                implementation: new_implementation.clone(),
                label,
                execute_after,
            },
        );

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("beacon"), symbol_short!("staged")),
            (new_implementation, execute_after),
        );
    }

    /// Apply a staged upgrade once its timelock has expired (step 2 of 2).
    ///
    /// Only the admin can call this. The version counter is incremented and a
    /// `VersionEntry` is appended to the history log.
    ///
    /// # Panics
    /// * `"Not initialized"` if `init` has not been called
    /// * `"No upgrade staged"` if there is nothing to execute
    /// * `"Upgrade timelock not expired"` if called before `execute_after`
    /// * auth failure if caller is not the admin
    pub fn execute_upgrade(env: Env) {
        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        let pending: PendingUpgrade = env
            .storage()
            .persistent()
            .get(&PENDING_KEY)
            .unwrap_or_else(|| panic!("No upgrade staged"));

        if env.ledger().timestamp() < pending.execute_after {
            panic!("Upgrade timelock not expired");
        }

        env.storage().persistent().remove(&PENDING_KEY);

        let new_implementation = pending.implementation;
        let label = pending.label;

        let prev_version: u32 = env.storage().persistent().get(&VERSION_KEY).unwrap_or(0);

        let new_version = prev_version
            .checked_add(1)
            .unwrap_or_else(|| panic!("Version overflow"));

        env.storage()
            .persistent()
            .set(&IMPL_KEY, &new_implementation);
        env.storage().persistent().set(&VERSION_KEY, &new_version);

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

    /// Abort a staged upgrade before it is executed.
    ///
    /// Only the admin can call this.
    ///
    /// # Panics
    /// * `"No upgrade staged"` if there is nothing to cancel
    /// * auth failure if caller is not the admin
    pub fn cancel_upgrade(env: Env) {
        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        let pending: PendingUpgrade = env
            .storage()
            .persistent()
            .get(&PENDING_KEY)
            .unwrap_or_else(|| panic!("No upgrade staged"));

        env.storage().persistent().remove(&PENDING_KEY);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("beacon"), symbol_short!("cancelled")),
            pending.implementation,
        );
    }

    /// Set the timelock delay (in seconds) applied to future staged upgrades.
    ///
    /// Only the admin can call this. The delay cannot be changed while an
    /// upgrade is staged, so a pending upgrade keeps the delay it was staged with.
    ///
    /// # Panics
    /// * `"Upgrade already staged"` if an upgrade is pending
    /// * auth failure if caller is not the admin
    pub fn set_upgrade_delay(env: Env, delay: u64) {
        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        if env.storage().persistent().has(&PENDING_KEY) {
            panic!("Upgrade already staged");
        }

        env.storage().persistent().set(&DELAY_KEY, &delay);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("beacon"), symbol_short!("delay_set")),
            delay,
        );
    }

    // -----------------------------------------------------------------------
    // Admin transfer
    // -----------------------------------------------------------------------

    /// Transfer admin rights to a new address.
    ///
    /// Only the current admin can call this.
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

    /// Return the timelock delay (in seconds) applied to staged upgrades.
    pub fn get_upgrade_delay(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DELAY_KEY)
            .unwrap_or(DEFAULT_UPGRADE_DELAY)
    }

    /// Return the staged upgrade, if any.
    pub fn get_pending_upgrade(env: Env) -> Option<PendingUpgrade> {
        env.storage().persistent().get(&PENDING_KEY)
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

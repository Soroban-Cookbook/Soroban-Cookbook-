//! # Pause / Unpause Pattern
//!
//! Demonstrates an emergency shutdown mechanism where an admin can halt
//! sensitive state-changing operations and later resume them.
//!
//! ## Operational Guidance
//!
//! - **When to pause:** during incident response, vulnerability disclosure,
//!   planned upgrades, or any situation that requires halting user-facing
//!   mutations while preserving read access.
//! - **What to guard:** state-changing operations that move funds, update
//!   balances, or modify critical state. Read-only inspection functions
//!   generally remain available.
//! - **Pausing is a mitigation lever, not a substitute for secure design.**
//!   It buys time for operators to react but does not fix underlying issues.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PauseError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller is not the admin.
    NotAuthorized = 3,
    /// Operation rejected because the contract is paused.
    ContractPaused = 4,
    /// Contract is already in the requested pause state.
    AlreadyInState = 5,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Paused,
    Counter,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const CONTRACT_NS: Symbol = symbol_short!("pausable");
const ACTION_PAUSE: Symbol = symbol_short!("pause");
const ACTION_UNPAUSE: Symbol = symbol_short!("unpause");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct PausableContract;

#[contractimpl]
impl PausableContract {
    /// Initialize the contract with an admin. Starts in unpaused state.
    pub fn initialize(env: Env, admin: Address) -> Result<(), PauseError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(PauseError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::Counter, &0u64);
        Ok(())
    }

    // ── Admin actions ───────────────────────────────────────────────────

    /// Pause the contract. Only the admin may call this.
    pub fn pause(env: Env) -> Result<(), PauseError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(PauseError::NotInitialized)?;
        admin.require_auth();

        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(PauseError::AlreadyInState);
        }

        env.storage().instance().set(&DataKey::Paused, &true);

        env.events()
            .publish((CONTRACT_NS, ACTION_PAUSE, admin), env.ledger().timestamp());

        Ok(())
    }

    /// Unpause the contract. Only the admin may call this.
    pub fn unpause(env: Env) -> Result<(), PauseError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(PauseError::NotInitialized)?;
        admin.require_auth();

        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if !paused {
            return Err(PauseError::AlreadyInState);
        }

        env.storage().instance().set(&DataKey::Paused, &false);

        env.events().publish(
            (CONTRACT_NS, ACTION_UNPAUSE, admin),
            env.ledger().timestamp(),
        );

        Ok(())
    }

    // ── Guarded mutable operations ──────────────────────────────────────

    /// Increment the counter by one. **Guarded** — fails while paused.
    pub fn increment(env: Env) -> Result<u64, PauseError> {
        Self::require_not_paused(&env)?;

        let mut count: u64 = env.storage().instance().get(&DataKey::Counter).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&DataKey::Counter, &count);
        Ok(count)
    }

    /// Reset the counter to zero. **Guarded** — fails while paused.
    pub fn reset(env: Env) -> Result<(), PauseError> {
        Self::require_not_paused(&env)?;

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(PauseError::NotInitialized)?;
        admin.require_auth();

        env.storage().instance().set(&DataKey::Counter, &0u64);
        Ok(())
    }

    // ── Read-only operations (available even while paused) ──────────────

    /// Return the current counter value.
    pub fn get_counter(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::Counter).unwrap_or(0)
    }

    /// Return whether the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    // ── Internal helpers ────────────────────────────────────────────────

    fn require_not_paused(env: &Env) -> Result<(), PauseError> {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(PauseError::ContractPaused);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;

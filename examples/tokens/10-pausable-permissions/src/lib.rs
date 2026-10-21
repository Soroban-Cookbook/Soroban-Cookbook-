//! # Pausable Permissions
//!
//! A permission system for pausing contracts, demonstrating three complementary
//! pause mechanisms that can be composed together:
//!
//! ## Mechanisms
//!
//! ### 1. Pauser Role
//! A dedicated `pauser` address (separate from the admin) can pause the contract.
//! Only the admin can assign or revoke the pauser role, and only the admin can
//! unpause. This separates the ability to halt the contract from full admin power.
//!
//! ### 2. Multi-Sig Pause
//! A pause requires `M` of `N` designated guardians to submit a pause vote before
//! the contract halts. Prevents a single compromised key from pausing production
//! systems unilaterally.
//!
//! ### 3. Time-Limited Pause
//! A pause automatically expires after a configurable duration. Once
//! `pause_expires_at` passes, guarded operations resume without any unpause
//! transaction, removing the risk of an indefinitely locked contract.
//!
//! ## Storage Layout
//!
//! | Key | Type | Storage | Purpose |
//! |-----|------|---------|---------|
//! | `Admin` | `Address` | Instance | contract owner |
//! | `Pauser` | `Address` | Instance | dedicated pause role |
//! | `Paused` | `bool` | Instance | pause flag |
//! | `PauseExpiresAt` | `u64` | Instance | timestamp when pause auto-lifts (0 = no expiry) |
//! | `Guardians` | `Vec<Address>` | Instance | multi-sig guardian list |
//! | `Threshold` | `u32` | Instance | votes required to pause |
//! | `PauseVotes` | `Vec<Address>` | Instance | guardians who have voted |
//!
//! ## Patterns reused from basics
//! - Auth: `address.require_auth()` — see `examples/basics/03-authentication`
//! - Events: `(namespace, action, actor)` topic layout — see `examples/basics/04-events`

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PauseError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller is not the admin.
    NotAdmin = 3,
    /// Caller is not the designated pauser.
    NotPauser = 4,
    /// Operation rejected because the contract is currently paused.
    ContractPaused = 5,
    /// Contract is already in the requested pause/unpause state.
    AlreadyInState = 6,
    /// Guardian is not in the authorized guardian list.
    NotGuardian = 7,
    /// Guardian has already cast a pause vote.
    AlreadyVoted = 8,
    /// Threshold or guardian list is invalid (e.g. threshold > guardians).
    InvalidConfig = 9,
    /// The pause duration must be greater than zero.
    InvalidDuration = 10,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The admin address (can assign pauser, set guardians, unpause).
    Admin,
    /// The dedicated pauser address (can pause without multi-sig).
    Pauser,
    /// Whether the contract is currently paused.
    Paused,
    /// Timestamp at which an active pause automatically expires (0 = no expiry).
    PauseExpiresAt,
    /// Ordered list of multi-sig guardian addresses.
    Guardians,
    /// Number of guardian votes required to trigger a multi-sig pause.
    Threshold,
    /// Guardians who have already voted for the current pending pause.
    PauseVotes,
}

// ---------------------------------------------------------------------------
// Event topics
// ---------------------------------------------------------------------------

const NS: Symbol = symbol_short!("pauseperm");
const EVT_PAUSE: Symbol = symbol_short!("paused");
const EVT_UNPAUSE: Symbol = symbol_short!("unpaused");
const EVT_ROLE_SET: Symbol = symbol_short!("role_set");
const EVT_VOTE: Symbol = symbol_short!("voted");
const EVT_EXPIRE: Symbol = symbol_short!("expired");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct PausablePermissions;

#[contractimpl]
impl PausablePermissions {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    /// Initialize the contract with an admin. Starts unpaused with no pauser
    /// and no guardians configured.
    pub fn initialize(env: Env, admin: Address) -> Result<(), PauseError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(PauseError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::PauseExpiresAt, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::PauseVotes, &Vec::<Address>::new(&env));
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Mechanism 1 — Pauser Role
    // -----------------------------------------------------------------------

    /// Assign the pauser role to `pauser`. Admin only.
    ///
    /// The pauser can halt the contract via `pause_as_pauser` without
    /// requiring multi-sig votes. Only the admin can unpause.
    pub fn set_pauser(env: Env, admin: Address, pauser: Address) -> Result<(), PauseError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        env.storage().instance().set(&DataKey::Pauser, &pauser);

        env.events()
            .publish((NS, EVT_ROLE_SET, admin, pauser), symbol_short!("pauser"));
        Ok(())
    }

    /// Pause the contract using the pauser role.
    ///
    /// Only the address assigned via `set_pauser` can call this.
    /// Use `pause_for` to apply a time-limited pause.
    pub fn pause_as_pauser(env: Env, pauser: Address) -> Result<(), PauseError> {
        pauser.require_auth();
        let stored_pauser: Address = env
            .storage()
            .instance()
            .get(&DataKey::Pauser)
            .ok_or(PauseError::NotPauser)?;
        if pauser != stored_pauser {
            return Err(PauseError::NotPauser);
        }
        Self::require_not_paused(&env)?;

        env.storage().instance().set(&DataKey::Paused, &true);
        env.storage()
            .instance()
            .set(&DataKey::PauseExpiresAt, &0u64);

        env.events()
            .publish((NS, EVT_PAUSE, pauser), env.ledger().timestamp());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Mechanism 2 — Multi-Sig Pause
    // -----------------------------------------------------------------------

    /// Configure the guardian list and vote threshold for multi-sig pause.
    ///
    /// Admin only. `threshold` must be ≥ 1 and ≤ `guardians.len()`.
    /// Resets any pending pause votes.
    pub fn set_guardians(
        env: Env,
        admin: Address,
        guardians: Vec<Address>,
        threshold: u32,
    ) -> Result<(), PauseError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if threshold == 0 || threshold > guardians.len() {
            return Err(PauseError::InvalidConfig);
        }

        env.storage()
            .instance()
            .set(&DataKey::Guardians, &guardians);
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        // Reset any in-progress vote when config changes.
        env.storage()
            .instance()
            .set(&DataKey::PauseVotes, &Vec::<Address>::new(&env));

        Ok(())
    }

    /// Cast a guardian vote to pause the contract.
    ///
    /// Once `threshold` unique guardians have voted, the contract is paused
    /// automatically and the vote tally is reset.
    pub fn guardian_vote_pause(env: Env, guardian: Address) -> Result<(), PauseError> {
        guardian.require_auth();
        Self::require_not_paused(&env)?;

        // Verify the caller is a registered guardian.
        let guardians: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Guardians)
            .unwrap_or_else(|| Vec::new(&env));
        if !guardians.contains(&guardian) {
            return Err(PauseError::NotGuardian);
        }

        // Prevent double-voting.
        let mut votes: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::PauseVotes)
            .unwrap_or_else(|| Vec::new(&env));
        if votes.contains(&guardian) {
            return Err(PauseError::AlreadyVoted);
        }

        votes.push_back(guardian.clone());
        env.storage().instance().set(&DataKey::PauseVotes, &votes);

        env.events().publish((NS, EVT_VOTE, guardian), votes.len());

        // Check if threshold is met.
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .unwrap_or(u32::MAX);

        if votes.len() >= threshold {
            env.storage().instance().set(&DataKey::Paused, &true);
            env.storage()
                .instance()
                .set(&DataKey::PauseExpiresAt, &0u64);
            // Reset votes after triggering pause.
            env.storage()
                .instance()
                .set(&DataKey::PauseVotes, &Vec::<Address>::new(&env));

            env.events().publish(
                (NS, EVT_PAUSE, env.current_contract_address()),
                env.ledger().timestamp(),
            );
        }

        Ok(())
    }

    /// Return the number of guardian votes currently pending.
    pub fn pending_votes(env: Env) -> u32 {
        let votes: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::PauseVotes)
            .unwrap_or_else(|| Vec::new(&env));
        votes.len()
    }

    // -----------------------------------------------------------------------
    // Mechanism 3 — Time-Limited Pause
    // -----------------------------------------------------------------------

    /// Pause for a fixed duration (seconds). Admin or pauser may call this.
    ///
    /// After `duration` seconds the pause automatically lifts — guarded
    /// operations check `pause_expires_at` and resume without a transaction.
    pub fn pause_for(env: Env, caller: Address, duration: u64) -> Result<(), PauseError> {
        caller.require_auth();
        if duration == 0 {
            return Err(PauseError::InvalidDuration);
        }
        Self::require_admin_or_pauser(&env, &caller)?;
        Self::require_not_paused(&env)?;

        let expires_at = env.ledger().timestamp() + duration;
        env.storage().instance().set(&DataKey::Paused, &true);
        env.storage()
            .instance()
            .set(&DataKey::PauseExpiresAt, &expires_at);

        env.events().publish((NS, EVT_PAUSE, caller), expires_at);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Shared unpause — admin only
    // -----------------------------------------------------------------------

    /// Unpause the contract manually. Admin only.
    ///
    /// Also clears any pending pause votes and expiry timestamp.
    pub fn unpause(env: Env, admin: Address) -> Result<(), PauseError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let currently_paused = Self::is_paused_now(&env);
        if !currently_paused {
            return Err(PauseError::AlreadyInState);
        }

        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::PauseExpiresAt, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::PauseVotes, &Vec::<Address>::new(&env));

        env.events()
            .publish((NS, EVT_UNPAUSE, admin), env.ledger().timestamp());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Guard helper — used by protected operations
    // -----------------------------------------------------------------------

    /// Gate a protected operation. Returns `ContractPaused` if currently paused
    /// (accounting for time-limited expiry). Emits an expiry event the first
    /// time a pause auto-lifts.
    pub fn assert_not_paused(env: Env) -> Result<(), PauseError> {
        if Self::is_paused_now(&env) {
            return Err(PauseError::ContractPaused);
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Read-only queries
    // -----------------------------------------------------------------------

    /// Return `true` if the contract is currently paused (respects expiry).
    pub fn is_paused(env: Env) -> bool {
        Self::is_paused_now(&env)
    }

    /// Return the timestamp at which a time-limited pause expires (0 = no expiry).
    pub fn pause_expires_at(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::PauseExpiresAt)
            .unwrap_or(0)
    }

    /// Return the current pauser address, if set.
    pub fn get_pauser(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Pauser)
    }

    /// Return the configured guardian list.
    pub fn get_guardians(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Guardians)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Return the multi-sig pause threshold.
    pub fn get_threshold(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Threshold)
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Effective pause check: respects time-limited expiry.
    ///
    /// If the pause has expired (timestamp-based), clears the pause flag and
    /// emits an expiry event so off-chain indexers can track the transition.
    fn is_paused_now(env: &Env) -> bool {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);

        if !paused {
            return false;
        }

        let expires_at: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PauseExpiresAt)
            .unwrap_or(0);

        // 0 means no expiry — pause is indefinite until manually lifted.
        if expires_at == 0 {
            return true;
        }

        if env.ledger().timestamp() >= expires_at {
            // Auto-lift: clear pause state.
            env.storage().instance().set(&DataKey::Paused, &false);
            env.storage()
                .instance()
                .set(&DataKey::PauseExpiresAt, &0u64);
            env.events()
                .publish((NS, EVT_EXPIRE), env.ledger().timestamp());
            return false;
        }

        true
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), PauseError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(PauseError::NotInitialized)?;
        if caller != &admin {
            return Err(PauseError::NotAdmin);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), PauseError> {
        if Self::is_paused_now(env) {
            return Err(PauseError::ContractPaused);
        }
        Ok(())
    }

    fn require_admin_or_pauser(env: &Env, caller: &Address) -> Result<(), PauseError> {
        // Try admin first.
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(PauseError::NotInitialized)?;
        if caller == &admin {
            return Ok(());
        }
        // Then try pauser.
        let pauser: Option<Address> = env.storage().instance().get(&DataKey::Pauser);
        if let Some(p) = pauser {
            if caller == &p {
                return Ok(());
            }
        }
        Err(PauseError::NotPauser)
    }
}

#[cfg(test)]
mod test;

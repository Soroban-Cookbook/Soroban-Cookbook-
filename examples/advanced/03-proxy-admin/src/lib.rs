//! # Proxy Admin Controls
//!
//! Demonstrates governance and safety controls around Soroban contract
//! upgrades. The pattern combines four independent safety layers:
//!
//! 1. **Admin authentication** — only the stored admin address may propose or
//!    execute upgrades.
//! 2. **Proposal workflow** — upgrades are proposed with a new WASM hash and
//!    must be explicitly confirmed before execution.
//! 3. **Timelock** — a configurable delay (in seconds) must pass between
//!    proposal and execution, giving stakeholders a review window.
//! 4. **Emergency pause** — the admin can halt all non-admin operations
//!    instantly; the pause itself is not subject to the timelock.
//!
//! ## Upgrade lifecycle
//!
//! ```text
//! admin calls propose_upgrade(new_hash, delay)
//!         │
//!         ▼
//!   ProposalState::Pending  ──── delay passes ────▶  ProposalState::Ready
//!         │                                                    │
//!   admin calls cancel_upgrade                        admin calls execute_upgrade
//!         │                                                    │
//!   ProposalState removed                          WASM replaced, proposal removed
//! ```

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, BytesN, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum timelock delay: 60 seconds.
pub const MIN_DELAY: u64 = 60;
/// Maximum timelock delay: 7 days.
pub const MAX_DELAY: u64 = 604_800;

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    /// Stores the pending `UpgradeProposal`, if any.
    Proposal,
    /// `true` when the contract is paused.
    Paused,
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A pending upgrade proposal.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    /// SHA-256 hash of the new WASM binary.
    pub new_wasm_hash: BytesN<32>,
    /// Ledger timestamp after which execution is allowed.
    pub execute_after: u64,
}

/// Observable state of the upgrade proposal slot.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalState {
    /// No proposal exists.
    None,
    /// Proposal exists but the timelock has not yet elapsed.
    Pending,
    /// Timelock has elapsed; execution is allowed.
    Ready,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdminError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    /// Caller is not the stored admin.
    Unauthorized = 3,
    DelayOutOfRange = 4,
    /// A proposal already exists; cancel it first.
    ProposalAlreadyExists = 5,
    NoProposal = 6,
    /// Timelock has not yet elapsed.
    TooEarly = 7,
    /// Contract is paused; non-admin operations are blocked.
    ContractPaused = 8,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const NS: Symbol = symbol_short!("prx_adm");
const EV_INIT: Symbol = symbol_short!("init");
const EV_PROPOSE: Symbol = symbol_short!("propose");
const EV_CANCEL: Symbol = symbol_short!("cancel");
const EV_EXECUTE: Symbol = symbol_short!("execute");
const EV_PAUSE: Symbol = symbol_short!("pause");
const EV_UNPAUSE: Symbol = symbol_short!("unpause");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ProxyAdmin;

#[contractimpl]
impl ProxyAdmin {
    /// Initialise the contract with an admin address.
    pub fn initialize(env: Env, admin: soroban_sdk::Address) -> Result<(), AdminError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AdminError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);

        env.events()
            .publish((NS, EV_INIT, admin), env.ledger().timestamp());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Upgrade workflow
    // -----------------------------------------------------------------------

    /// Propose a WASM upgrade. The upgrade cannot be executed until `delay`
    /// seconds have passed. Only one proposal may be active at a time.
    ///
    /// `delay` must be within `MIN_DELAY..=MAX_DELAY`.
    pub fn propose_upgrade(
        env: Env,
        new_wasm_hash: BytesN<32>,
        delay: u64,
    ) -> Result<(), AdminError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        if !(MIN_DELAY..=MAX_DELAY).contains(&delay) {
            return Err(AdminError::DelayOutOfRange);
        }
        if env.storage().instance().has(&DataKey::Proposal) {
            return Err(AdminError::ProposalAlreadyExists);
        }

        let execute_after = env.ledger().timestamp() + delay;
        let proposal = UpgradeProposal {
            new_wasm_hash: new_wasm_hash.clone(),
            execute_after,
        };
        env.storage().instance().set(&DataKey::Proposal, &proposal);

        env.events()
            .publish((NS, EV_PROPOSE, admin, new_wasm_hash), execute_after);
        Ok(())
    }

    /// Cancel the pending upgrade proposal.
    pub fn cancel_upgrade(env: Env) -> Result<(), AdminError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        if !env.storage().instance().has(&DataKey::Proposal) {
            return Err(AdminError::NoProposal);
        }
        env.storage().instance().remove(&DataKey::Proposal);

        env.events()
            .publish((NS, EV_CANCEL, admin), env.ledger().timestamp());
        Ok(())
    }

    /// Execute the pending upgrade once the timelock has elapsed.
    ///
    /// Calls `env.deployer().update_current_contract_wasm()` which replaces
    /// the running WASM binary in-place. The proposal is removed after a
    /// successful upgrade to prevent replay.
    pub fn execute_upgrade(env: Env) -> Result<(), AdminError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        let proposal: UpgradeProposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal)
            .ok_or(AdminError::NoProposal)?;

        let now = env.ledger().timestamp();
        if now < proposal.execute_after {
            return Err(AdminError::TooEarly);
        }

        // Remove before upgrading to prevent any re-entrancy replay.
        env.storage().instance().remove(&DataKey::Proposal);

        env.deployer()
            .update_current_contract_wasm(proposal.new_wasm_hash.clone());

        env.events()
            .publish((NS, EV_EXECUTE, admin, proposal.new_wasm_hash), now);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Emergency pause
    // -----------------------------------------------------------------------

    /// Pause the contract. Non-admin entry points should call
    /// `require_unpaused` before executing any logic.
    pub fn pause(env: Env) -> Result<(), AdminError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        env.storage().instance().set(&DataKey::Paused, &true);
        env.events()
            .publish((NS, EV_PAUSE, admin), env.ledger().timestamp());
        Ok(())
    }

    /// Unpause the contract.
    pub fn unpause(env: Env) -> Result<(), AdminError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        env.storage().instance().set(&DataKey::Paused, &false);
        env.events()
            .publish((NS, EV_UNPAUSE, admin), env.ledger().timestamp());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    pub fn proposal_state(env: Env) -> ProposalState {
        match env
            .storage()
            .instance()
            .get::<DataKey, UpgradeProposal>(&DataKey::Proposal)
        {
            None => ProposalState::None,
            Some(p) => {
                if env.ledger().timestamp() < p.execute_after {
                    ProposalState::Pending
                } else {
                    ProposalState::Ready
                }
            }
        }
    }

    pub fn get_proposal(env: Env) -> Option<UpgradeProposal> {
        env.storage().instance().get(&DataKey::Proposal)
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn admin(env: Env) -> Result<soroban_sdk::Address, AdminError> {
        read_admin(&env)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_admin(env: &Env) -> Result<soroban_sdk::Address, AdminError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(AdminError::NotInitialized)
}

/// Convenience guard for non-admin entry points in contracts that embed this
/// pattern. Returns `Err(AdminError::ContractPaused)` when paused.
pub fn require_unpaused(env: &Env) -> Result<(), AdminError> {
    let paused: bool = env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        Err(AdminError::ContractPaused)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test;

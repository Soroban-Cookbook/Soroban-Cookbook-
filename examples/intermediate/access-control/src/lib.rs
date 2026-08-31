#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccessControlError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    InvalidRole = 3,
    InvalidThreshold = 4,
    ProposalNotFound = 5,
    ProposalAlreadyExecuted = 6,
    ProposalNotReady = 7,
    DuplicateApproval = 8,
    InsufficientApprovals = 9,
    NotASigner = 10,
    TimelockActive = 11,
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Role {
    User = 0,
    Operator = 1,
    Auditor = 2,
    Admin = 3,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Initialized,
    UserRole(Address),
    TimelockDelay,
    MinDelay,
    MaxDelay,
    Paused,
    ProposalCount,
    Proposal(u32),
    Signers,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleChangeEvent {
    pub operator: Address,
    pub account: Address,
    pub old_role: Role,
    pub new_role: Role,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub action: Symbol,
    pub execute_at: u64,
    pub executed: bool,
    pub approvals: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalEventData {
    pub proposal_id: u32,
    pub action: Symbol,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelockEventData {
    pub action: Symbol,
    pub delay: u64,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CONTRACT_NS: Symbol = symbol_short!("access");
const ACTION_ROLE_CHANGE: Symbol = symbol_short!("role_chg");
const ACTION_PROPOSAL: Symbol = symbol_short!("proposal");
const ACTION_TIMELOCK: Symbol = symbol_short!("timelock");

const DEFAULT_MIN_DELAY: u64 = 60;
const DEFAULT_MAX_DELAY: u64 = 86_400;
const ABSOLUTE_MIN_DELAY: u64 = 30;
const ABSOLUTE_MAX_DELAY: u64 = 604_800;

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AccessControl;

#[contractimpl]
impl AccessControl {
    /// Initialize the contract with an admin, initial signers, threshold, and default timelock delay.
    pub fn initialize(
        env: Env,
        admin: Address,
        threshold: u32,
        signers: Vec<Address>,
        timelock_delay: u64,
    ) -> Result<(), AccessControlError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(AccessControlError::AlreadyInitialized);
        }

        if threshold == 0 || threshold > signers.len() {
            return Err(AccessControlError::InvalidThreshold);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage()
            .persistent()
            .set(&DataKey::UserRole(admin.clone()), &Role::Admin);
        env.storage()
            .instance()
            .set(&DataKey::TimelockDelay, &timelock_delay);
        env.storage()
            .instance()
            .set(&DataKey::MinDelay, &DEFAULT_MIN_DELAY);
        env.storage()
            .instance()
            .set(&DataKey::MaxDelay, &DEFAULT_MAX_DELAY);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);
        env.storage().instance().set(&DataKey::Signers, &signers);

        env.events().publish(
            (CONTRACT_NS, ACTION_ROLE_CHANGE, admin.clone()),
            RoleChangeEvent {
                operator: admin.clone(),
                account: admin,
                old_role: Role::User,
                new_role: Role::Admin,
            },
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // RBAC
    // -----------------------------------------------------------------------

    /// Grant a role to an account. Only Admin can grant.
    pub fn grant_role(
        env: Env,
        caller: Address,
        account: Address,
        role: Role,
    ) -> Result<(), AccessControlError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let old_role = Self::get_role_internal(&env, &account);
        env.storage()
            .persistent()
            .set(&DataKey::UserRole(account.clone()), &role);

        env.events().publish(
            (CONTRACT_NS, ACTION_ROLE_CHANGE, account.clone()),
            RoleChangeEvent {
                operator: caller,
                account,
                old_role,
                new_role: role,
            },
        );

        Ok(())
    }

    /// Revoke a role from an account. Only Admin can revoke non-Admin roles.
    pub fn revoke_role(
        env: Env,
        caller: Address,
        account: Address,
    ) -> Result<(), AccessControlError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let target_role = Self::get_role_internal(&env, &account);
        if target_role == Role::Admin {
            return Err(AccessControlError::InvalidRole);
        }

        env.storage()
            .persistent()
            .remove(&DataKey::UserRole(account.clone()));

        env.events().publish(
            (CONTRACT_NS, ACTION_ROLE_CHANGE, account.clone()),
            RoleChangeEvent {
                operator: caller,
                account,
                old_role: target_role,
                new_role: Role::User,
            },
        );

        Ok(())
    }

    /// Check if an account has at least the required role.
    pub fn has_role(env: Env, account: Address, role: Role) -> bool {
        let user_role = Self::get_role_internal(&env, &account);
        user_role as u32 >= role as u32
    }

    /// Require the caller to have one of the allowed roles.
    pub fn require_role(
        env: Env,
        caller: Address,
        allowed: Vec<Role>,
    ) -> Result<(), AccessControlError> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let user_role = Self::get_role_internal(&env, &caller);
        for allowed_role in allowed.iter() {
            if user_role as u32 >= allowed_role as u32 {
                return Ok(());
            }
        }
        Err(AccessControlError::Unauthorized)
    }

    // -----------------------------------------------------------------------
    // Timelock
    // -----------------------------------------------------------------------

    /// Update the global timelock delay (must be within bounds).
    pub fn set_timelock_delay(
        env: Env,
        caller: Address,
        delay: u64,
    ) -> Result<(), AccessControlError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let min_delay: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MinDelay)
            .unwrap_or(DEFAULT_MIN_DELAY);
        let max_delay: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MaxDelay)
            .unwrap_or(DEFAULT_MAX_DELAY);

        if delay < ABSOLUTE_MIN_DELAY
            || delay > ABSOLUTE_MAX_DELAY
            || delay < min_delay
            || delay > max_delay
        {
            return Err(AccessControlError::Unauthorized);
        }

        env.storage()
            .instance()
            .set(&DataKey::TimelockDelay, &delay);

        env.events().publish(
            (CONTRACT_NS, ACTION_TIMELOCK, caller),
            TimelockEventData {
                action: symbol_short!("set_delay"),
                delay,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Get the current timelock delay.
    pub fn get_timelock_delay(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TimelockDelay)
            .unwrap_or(DEFAULT_MIN_DELAY)
    }

    /// Emergency pause to halt sensitive operations.
    pub fn set_pause(env: Env, caller: Address, paused: bool) -> Result<(), AccessControlError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().instance().set(&DataKey::Paused, &paused);

        env.events().publish(
            (CONTRACT_NS, ACTION_TIMELOCK, caller),
            TimelockEventData {
                action: if paused {
                    symbol_short!("pause")
                } else {
                    symbol_short!("unpause")
                },
                delay: 0,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Check if the contract is paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Multi-Sig Proposal + Timelock
    // -----------------------------------------------------------------------

    /// Create a new proposal. Any signer can create a proposal.
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        action: Symbol,
    ) -> Result<u32, AccessControlError> {
        proposer.require_auth();
        Self::require_initialized(&env)?;

        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(AccessControlError::TimelockActive);
        }

        let mut proposal_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0);
        proposal_count += 1;

        let delay: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TimelockDelay)
            .unwrap_or(DEFAULT_MIN_DELAY);
        let execute_at = env.ledger().timestamp() + delay;

        let proposal = Proposal {
            id: proposal_count,
            proposer: proposer.clone(),
            action: action.clone(),
            execute_at,
            executed: false,
            approvals: Vec::new(&env),
        };

        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &proposal_count);
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_count), &proposal);

        env.events().publish(
            (CONTRACT_NS, ACTION_PROPOSAL, proposer),
            ProposalEventData {
                proposal_id: proposal_count,
                action,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(proposal_count)
    }

    /// Approve a proposal. Must be an authorized signer.
    pub fn approve(env: Env, caller: Address, proposal_id: u32) -> Result<(), AccessControlError> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let key = DataKey::Proposal(proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(AccessControlError::ProposalNotFound)?;

        if proposal.executed {
            return Err(AccessControlError::ProposalAlreadyExecuted);
        }

        if proposal.approvals.contains(&caller) {
            return Err(AccessControlError::DuplicateApproval);
        }

        let signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or(Vec::new(&env));

        if !signers.contains(&caller) {
            return Err(AccessControlError::NotASigner);
        }

        proposal.approvals.push_back(caller.clone());
        env.storage().persistent().set(&key, &proposal);

        Ok(())
    }

    /// Execute a proposal after the timelock delay has passed and threshold is met.
    pub fn execute(
        env: Env,
        caller: Address,
        proposal_id: u32,
    ) -> Result<bool, AccessControlError> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let key = DataKey::Proposal(proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(AccessControlError::ProposalNotFound)?;

        if proposal.executed {
            return Err(AccessControlError::ProposalAlreadyExecuted);
        }

        if env.ledger().timestamp() < proposal.execute_at {
            return Err(AccessControlError::ProposalNotReady);
        }

        let signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or(Vec::new(&env));

        if proposal.approvals.len() < 2 || proposal.approvals.len() < signers.len() / 2 + 1 {
            return Err(AccessControlError::InsufficientApprovals);
        }

        proposal.executed = true;
        env.storage().persistent().set(&key, &proposal);

        Ok(true)
    }

    /// Get a proposal by id.
    pub fn get_proposal(env: Env, proposal_id: u32) -> Option<Proposal> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
    }

    /// Add a signer to the multisig set. Admin only.
    pub fn add_signer(
        env: Env,
        caller: Address,
        signer: Address,
    ) -> Result<(), AccessControlError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let mut signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or(Vec::new(&env));

        if !signers.contains(&signer) {
            signers.push_back(signer.clone());
            env.storage().instance().set(&DataKey::Signers, &signers);
        }

        Ok(())
    }

    /// Remove a signer from the multisig set. Admin only.
    pub fn remove_signer(
        env: Env,
        caller: Address,
        signer: Address,
    ) -> Result<(), AccessControlError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let mut signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or(Vec::new(&env));

        if let Some(pos) = signers.iter().position(|x| x == signer) {
            signers.remove(pos as u32);
            env.storage().instance().set(&DataKey::Signers, &signers);
        }

        Ok(())
    }

    /// Check if an address is a signer.
    pub fn is_signer(env: Env, account: Address) -> bool {
        let signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or(Vec::new(&env));
        signers.contains(&account)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), AccessControlError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(AccessControlError::Unauthorized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), AccessControlError> {
        Self::require_initialized(env)?;
        let role: Role = env
            .storage()
            .persistent()
            .get(&DataKey::UserRole(caller.clone()))
            .unwrap_or(Role::User);
        if role as u32 >= Role::Admin as u32 {
            Ok(())
        } else {
            Err(AccessControlError::Unauthorized)
        }
    }

    fn get_role_internal(env: &Env, account: &Address) -> Role {
        env.storage()
            .persistent()
            .get(&DataKey::UserRole(account.clone()))
            .unwrap_or(Role::User)
    }
}

#[cfg(test)]
mod test;

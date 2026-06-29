//! # Voting Time Constraints
//!
//! Demonstrates time-bounded governance voting on Soroban:
//!
//! - **Voting period**: proposals are open for a fixed window
//! - **Proposal deadline**: votes are rejected after the deadline
//! - **Grace period**: a quiet window after voting before execution
//! - **Early closure**: admin can close a proposal early if quorum is met
//!
//! ## Storage layout
//! - `Instance`: admin address, global config
//! - `Persistent`: per-proposal state keyed by `ProposalId`
//!
//! ## Auth model
//! - Creating a proposal: any address (one proposal per address per period)
//! - Voting: any address (once per proposal)
//! - Closing / executing: admin or auto (after deadline + grace)

#![no_std]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default voting window in seconds (3 days).
const DEFAULT_VOTING_PERIOD: u64 = 3 * 24 * 3_600;
/// Default grace period after voting closes before execution (1 day).
const DEFAULT_GRACE_PERIOD: u64 = 24 * 3_600;
/// Minimum quorum (votes for early closure) as a fraction of `u32::MAX`.
/// Represents 10% if total possible voters were `u32::MAX`. In practice the
/// admin sets the `quorum_threshold` absolute count on each proposal.
const MIN_QUORUM: u32 = 1;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VotingError {
    /// Contract has not been initialized.
    NotInitialized = 1,
    /// Contract already initialized.
    AlreadyInitialized = 2,
    /// Caller is not the admin.
    Unauthorized = 3,
    /// Proposal does not exist.
    ProposalNotFound = 4,
    /// Voting period has not started yet or has already ended.
    VotingClosed = 5,
    /// Caller has already voted on this proposal.
    AlreadyVoted = 6,
    /// Proposal is not in the correct state for this operation.
    InvalidState = 7,
    /// Grace period has not elapsed; execution is not yet allowed.
    GracePeriodActive = 8,
    /// Quorum has not been reached for early closure.
    QuorumNotMet = 9,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Unique identifier for a proposal (up to 9-char Symbol).
pub type ProposalId = Symbol;

/// Possible lifecycle states for a proposal.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalState {
    /// Voting is open.
    Active = 0,
    /// Voting window closed; grace period running.
    GracePeriod = 1,
    /// Grace period elapsed; proposal ready to execute.
    Executable = 2,
    /// Proposal was executed.
    Executed = 3,
    /// Proposal was cancelled or closed without reaching quorum.
    Cancelled = 4,
}

/// Full on-chain state of a proposal.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    /// Address that created the proposal.
    pub creator: Address,
    /// Ledger timestamp when voting opens.
    pub voting_start: u64,
    /// Ledger timestamp when voting closes (deadline).
    pub voting_deadline: u64,
    /// Ledger timestamp when the grace period ends (earliest execution time).
    pub executable_after: u64,
    /// Minimum yes-votes required to consider the proposal passing.
    pub quorum_threshold: u32,
    /// Number of yes votes cast.
    pub votes_for: u32,
    /// Number of no votes cast.
    pub votes_against: u32,
    /// Current lifecycle state.
    pub state: ProposalState,
}

/// Storage keys.
#[contracttype]
pub enum DataKey {
    /// Instance key: admin address.
    Admin,
    /// Instance key: default voting period in seconds.
    VotingPeriod,
    /// Instance key: default grace period in seconds.
    GracePeriod,
    /// Persistent key: proposal state.
    Proposal(ProposalId),
    /// Persistent key: vote record for (proposal, voter).
    Vote(ProposalId, Address),
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const CONTRACT_NS: Symbol = symbol_short!("gov");
const EV_PROPOSED: Symbol = symbol_short!("proposed");
const EV_VOTED: Symbol = symbol_short!("voted");
const EV_CLOSED: Symbol = symbol_short!("closed");
const EV_EXECUTED: Symbol = symbol_short!("executed");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct VotingContract;

#[contractimpl]
impl VotingContract {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    /// Initialize the contract.
    ///
    /// - `admin`: address that can close proposals early and execute them.
    /// - `voting_period`: seconds a proposal stays open (0 = use default).
    /// - `grace_period`: seconds between deadline and earliest execution (0 = use default).
    pub fn initialize(
        env: Env,
        admin: Address,
        voting_period: u64,
        grace_period: u64,
    ) -> Result<(), VotingError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VotingError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);

        let vp = if voting_period == 0 {
            DEFAULT_VOTING_PERIOD
        } else {
            voting_period
        };
        let gp = if grace_period == 0 {
            DEFAULT_GRACE_PERIOD
        } else {
            grace_period
        };

        env.storage().instance().set(&DataKey::VotingPeriod, &vp);
        env.storage().instance().set(&DataKey::GracePeriod, &gp);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Proposal lifecycle
    // -----------------------------------------------------------------------

    /// Create a new proposal.
    ///
    /// - `proposal_id`: unique Symbol identifying this proposal.
    /// - `quorum_threshold`: minimum yes-votes for the proposal to pass.
    ///
    /// The voting window opens immediately and closes after `voting_period` seconds.
    pub fn create_proposal(
        env: Env,
        creator: Address,
        proposal_id: ProposalId,
        quorum_threshold: u32,
    ) -> Result<(), VotingError> {
        Self::require_initialized(&env)?;
        creator.require_auth();

        if env
            .storage()
            .persistent()
            .has(&DataKey::Proposal(proposal_id.clone()))
        {
            return Err(VotingError::InvalidState);
        }

        let quorum = quorum_threshold.max(MIN_QUORUM);
        let now = env.ledger().timestamp();
        let voting_period: u64 = env
            .storage()
            .instance()
            .get(&DataKey::VotingPeriod)
            .unwrap_or(DEFAULT_VOTING_PERIOD);
        let grace_period: u64 = env
            .storage()
            .instance()
            .get(&DataKey::GracePeriod)
            .unwrap_or(DEFAULT_GRACE_PERIOD);

        let deadline = now + voting_period;
        let executable_after = deadline + grace_period;

        let proposal = Proposal {
            creator: creator.clone(),
            voting_start: now,
            voting_deadline: deadline,
            executable_after,
            quorum_threshold: quorum,
            votes_for: 0,
            votes_against: 0,
            state: ProposalState::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id.clone()), &proposal);
        // Keep alive well beyond the voting window.
        env.storage().persistent().extend_ttl(
            &DataKey::Proposal(proposal_id.clone()),
            17_280,
            120_960,
        );

        env.events().publish(
            (CONTRACT_NS, EV_PROPOSED, creator, proposal_id),
            (now, deadline, executable_after, quorum),
        );

        Ok(())
    }

    /// Cast a vote on an active proposal.
    ///
    /// - `support`: `true` = vote for, `false` = vote against.
    ///
    /// Reverts if:
    /// - the proposal is not in `Active` state,
    /// - the voting deadline has passed,
    /// - the caller has already voted.
    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: ProposalId,
        support: bool,
    ) -> Result<(), VotingError> {
        Self::require_initialized(&env)?;
        voter.require_auth();

        let vote_key = DataKey::Vote(proposal_id.clone(), voter.clone());
        if env.storage().persistent().has(&vote_key) {
            return Err(VotingError::AlreadyVoted);
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id.clone()))
            .ok_or(VotingError::ProposalNotFound)?;

        let now = env.ledger().timestamp();

        // Reject votes outside the active window.
        if proposal.state != ProposalState::Active || now > proposal.voting_deadline {
            return Err(VotingError::VotingClosed);
        }

        // Record the vote.
        env.storage().persistent().set(&vote_key, &support);
        env.storage()
            .persistent()
            .extend_ttl(&vote_key, 17_280, 120_960);

        if support {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id.clone()), &proposal);

        env.events().publish(
            (CONTRACT_NS, EV_VOTED, voter, proposal_id),
            (support, proposal.votes_for, proposal.votes_against),
        );

        Ok(())
    }

    /// Attempt early closure of an active proposal (admin only).
    ///
    /// Closes the proposal immediately if `votes_for >= quorum_threshold`.
    /// Transitions state to `GracePeriod` so the grace window still applies.
    pub fn close_early(
        env: Env,
        admin: Address,
        proposal_id: ProposalId,
    ) -> Result<(), VotingError> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;
        admin.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id.clone()))
            .ok_or(VotingError::ProposalNotFound)?;

        if proposal.state != ProposalState::Active {
            return Err(VotingError::InvalidState);
        }

        if proposal.votes_for < proposal.quorum_threshold {
            return Err(VotingError::QuorumNotMet);
        }

        let now = env.ledger().timestamp();
        let grace_period: u64 = env
            .storage()
            .instance()
            .get(&DataKey::GracePeriod)
            .unwrap_or(DEFAULT_GRACE_PERIOD);

        // Override the deadline and recompute executable_after from now.
        proposal.voting_deadline = now;
        proposal.executable_after = now + grace_period;
        proposal.state = ProposalState::GracePeriod;

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id.clone()), &proposal);

        env.events().publish(
            (CONTRACT_NS, EV_CLOSED, admin, proposal_id),
            (now, proposal.executable_after),
        );

        Ok(())
    }

    /// Finalise the state of a proposal after its deadline.
    ///
    /// Anyone can call this. Transitions `Active` → `GracePeriod` once
    /// `voting_deadline` has passed, and `GracePeriod` → `Executable` once
    /// `executable_after` has passed.
    pub fn finalize(env: Env, proposal_id: ProposalId) -> Result<ProposalState, VotingError> {
        Self::require_initialized(&env)?;

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id.clone()))
            .ok_or(VotingError::ProposalNotFound)?;

        let now = env.ledger().timestamp();

        let new_state = match proposal.state {
            ProposalState::Active if now > proposal.voting_deadline => ProposalState::GracePeriod,
            ProposalState::GracePeriod if now >= proposal.executable_after => {
                ProposalState::Executable
            }
            other => other,
        };

        if new_state != proposal.state {
            proposal.state = new_state;
            env.storage()
                .persistent()
                .set(&DataKey::Proposal(proposal_id), &proposal);
        }

        Ok(new_state)
    }

    /// Execute an `Executable` proposal (admin only).
    ///
    /// In a real DAO this would dispatch to a target contract. Here it
    /// records the execution and transitions state to `Executed`.
    pub fn execute(env: Env, admin: Address, proposal_id: ProposalId) -> Result<(), VotingError> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;
        admin.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id.clone()))
            .ok_or(VotingError::ProposalNotFound)?;

        if proposal.state != ProposalState::Executable {
            return Err(VotingError::InvalidState);
        }

        proposal.state = ProposalState::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id.clone()), &proposal);

        env.events().publish(
            (CONTRACT_NS, EV_EXECUTED, admin, proposal_id),
            env.ledger().timestamp(),
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the full proposal record.
    pub fn get_proposal(env: Env, proposal_id: ProposalId) -> Result<Proposal, VotingError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::ProposalNotFound)
    }

    /// Check whether a given address has voted on a proposal.
    pub fn has_voted(env: Env, proposal_id: ProposalId, voter: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Vote(proposal_id, voter))
    }

    /// Return the current admin.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), VotingError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(VotingError::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), VotingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VotingError::NotInitialized)?;
        if caller != &admin {
            return Err(VotingError::Unauthorized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;

//! # Simple Voting Contract
//!
//! Demonstrates a basic on-chain governance voting system on Soroban.
//!
//! ## Features
//!
//! - **Create Proposal**: Admin creates proposals with a title and voting deadline.
//! - **Cast Vote**: Authenticated users cast a single vote (For/Against/Abstain) per proposal.
//! - **Tally Votes**: Anyone can query the current vote counts for a proposal.
//! - **Execute Result**: After the deadline, finalize and execute the proposal outcome.
//!
//! ## Design Patterns
//!
//! - Admin-gated proposal creation
//! - One-address-one-vote enforcement via persistent storage
//! - Time-based deadline enforcement using `env.ledger().timestamp()`
//! - Event emission for indexer-friendly audit trails

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The possible choices a voter can make.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VoteChoice {
    For = 0,
    Against = 1,
    Abstain = 2,
}

/// Status of a proposal.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalStatus {
    Active = 0,
    Passed = 1,
    Rejected = 2,
    Executed = 3,
}

/// Core proposal data stored on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub title: String,
    pub proposer: Address,
    pub deadline: u64,
    pub votes_for: u32,
    pub votes_against: u32,
    pub votes_abstain: u32,
    pub status: ProposalStatus,
}

// ---------------------------------------------------------------------------
// Storage Keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PropCount,
    Proposal(u32),
    Vote(u32, Address),
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VotingError {
    /// Contract already initialized.
    AlreadyInit = 1,
    /// Caller is not the admin.
    NotAdmin = 2,
    /// Proposal not found.
    NotFound = 3,
    /// Voting period has ended.
    VoteEnded = 4,
    /// Voting period has not ended yet.
    NotEnded = 5,
    /// Caller already voted on this proposal.
    AlreadyVoted = 6,
    /// Proposal already executed.
    Executed = 7,
    /// Deadline must be in the future.
    BadDeadline = 8,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct VotingContract;

#[contractimpl]
impl VotingContract {
    // ==================== INITIALIZATION ====================

    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), VotingError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VotingError::AlreadyInit);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::PropCount, &0u32);

        env.events()
            .publish((symbol_short!("voting"), symbol_short!("init")), admin);

        Ok(())
    }

    /// Returns the admin address.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    // ==================== PROPOSAL MANAGEMENT ====================

    /// Create a new proposal (admin-only). Returns the proposal ID.
    pub fn create_prop(
        env: Env,
        admin: Address,
        title: String,
        deadline: u64,
    ) -> Result<u32, VotingError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if deadline <= env.ledger().timestamp() {
            return Err(VotingError::BadDeadline);
        }

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PropCount)
            .unwrap_or(0);
        let proposal_id = count + 1;

        let proposal = Proposal {
            id: proposal_id,
            title,
            proposer: admin.clone(),
            deadline,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            status: ProposalStatus::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::PropCount, &proposal_id);

        env.events().publish(
            (symbol_short!("voting"), symbol_short!("propose")),
            proposal_id,
        );

        Ok(proposal_id)
    }

    /// Get proposal details by ID.
    pub fn get_prop(env: Env, proposal_id: u32) -> Result<Proposal, VotingError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::NotFound)
    }

    /// Get total number of proposals.
    pub fn prop_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::PropCount)
            .unwrap_or(0)
    }

    // ==================== VOTING ====================

    /// Cast a vote on a proposal. Each address can vote only once per proposal.
    pub fn cast_vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        choice: VoteChoice,
    ) -> Result<(), VotingError> {
        voter.require_auth();

        let vote_key = DataKey::Vote(proposal_id, voter.clone());
        if env.storage().persistent().has(&vote_key) {
            return Err(VotingError::AlreadyVoted);
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::NotFound)?;

        if proposal.status != ProposalStatus::Active {
            return Err(VotingError::Executed);
        }

        if env.ledger().timestamp() >= proposal.deadline {
            return Err(VotingError::VoteEnded);
        }

        match choice {
            VoteChoice::For => proposal.votes_for += 1,
            VoteChoice::Against => proposal.votes_against += 1,
            VoteChoice::Abstain => proposal.votes_abstain += 1,
        }

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().persistent().set(&vote_key, &choice);

        env.events().publish(
            (symbol_short!("voting"), symbol_short!("vote")),
            (proposal_id, choice),
        );

        Ok(())
    }

    /// Check if an address has voted on a proposal.
    pub fn has_voted(env: Env, voter: Address, proposal_id: u32) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Vote(proposal_id, voter))
    }

    /// Get the vote choice of an address for a proposal.
    pub fn get_vote(env: Env, voter: Address, proposal_id: u32) -> Option<VoteChoice> {
        env.storage()
            .persistent()
            .get(&DataKey::Vote(proposal_id, voter))
    }

    // ==================== TALLY ====================

    /// Get the current vote tally: (votes_for, votes_against, votes_abstain).
    pub fn tally(env: Env, proposal_id: u32) -> Result<(u32, u32, u32), VotingError> {
        let proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::NotFound)?;

        Ok((
            proposal.votes_for,
            proposal.votes_against,
            proposal.votes_abstain,
        ))
    }

    // ==================== EXECUTION ====================

    /// Execute/finalize a proposal after the voting deadline.
    /// Passed if votes_for > votes_against, otherwise Rejected.
    pub fn execute(env: Env, proposal_id: u32) -> Result<ProposalStatus, VotingError> {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::NotFound)?;

        if proposal.status != ProposalStatus::Active {
            return Err(VotingError::Executed);
        }

        if env.ledger().timestamp() < proposal.deadline {
            return Err(VotingError::NotEnded);
        }

        let new_status = if proposal.votes_for > proposal.votes_against {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        proposal.status = new_status;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (symbol_short!("voting"), symbol_short!("execute")),
            (proposal_id, new_status),
        );

        Ok(new_status)
    }

    // ==================== HELPERS ====================

    fn require_admin(env: &Env, caller: &Address) -> Result<(), VotingError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VotingError::NotAdmin)?;
        if *caller != stored_admin {
            return Err(VotingError::NotAdmin);
        }
        Ok(())
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod test;

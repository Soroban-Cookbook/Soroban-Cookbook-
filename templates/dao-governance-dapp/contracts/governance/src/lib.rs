#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalStatus {
    Active = 1,
    Passed = 2,
    Defeated = 3,
    Executed = 4,
    Cancelled = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub description: String,
    pub target: Address,
    pub amount: i128,
    pub votes_for: i128,
    pub votes_against: i128,
    pub end_ledger: u32,
    pub status: ProposalStatus,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    QuorumVotes,
    VotingPeriodLedgers,
    NextProposalId,
    Proposal(u32),
    HasVoted(u32, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GovernanceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ProposalNotFound = 4,
    VotingEnded = 5,
    VotingStillActive = 6,
    AlreadyVoted = 7,
    InvalidAmount = 8,
    ProposalNotPassed = 9,
    AlreadyExecuted = 10,
    ArithmeticOverflow = 11,
}

const INSTANCE_BUMP_AMOUNT: u32 = 518_400;
const INSTANCE_LIFETIME_THRESHOLD: u32 = 120_960;

const STORAGE_BUMP_AMOUNT: u32 = 518_400;
const STORAGE_LIFETIME_THRESHOLD: u32 = 120_960;

#[contract]
pub struct DAOGovernanceContract;

#[contractimpl]
impl DAOGovernanceContract {
    /// Initialize DAO Governance parameters
    pub fn initialize(
        env: Env,
        admin: Address,
        quorum_votes: i128,
        voting_period_ledgers: u32,
    ) -> Result<(), GovernanceError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(GovernanceError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::QuorumVotes, &quorum_votes);
        env.storage().instance().set(&DataKey::VotingPeriodLedgers, &voting_period_ledgers);
        env.storage().instance().set(&DataKey::NextProposalId, &1u32);

        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish((symbol_short!("init"), admin), (quorum_votes, voting_period_ledgers));
        Ok(())
    }

    /// Submit a new proposal
    pub fn propose(
        env: Env,
        proposer: Address,
        description: String,
        target: Address,
        amount: i128,
    ) -> Result<u32, GovernanceError> {
        proposer.require_auth();

        if amount < 0 {
            return Err(GovernanceError::InvalidAmount);
        }

        let proposal_id: u32 = env.storage().instance().get(&DataKey::NextProposalId).unwrap_or(1);
        let next_id = proposal_id.checked_add(1).ok_or(GovernanceError::ArithmeticOverflow)?;
        env.storage().instance().set(&DataKey::NextProposalId, &next_id);

        let voting_period: u32 = env
            .storage()
            .instance()
            .get(&DataKey::VotingPeriodLedgers)
            .unwrap_or(17280); // ~1 day

        let end_ledger = env
            .ledger()
            .sequence()
            .checked_add(voting_period)
            .ok_or(GovernanceError::ArithmeticOverflow)?;

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.clone(),
            description,
            target,
            amount,
            votes_for: 0,
            votes_against: 0,
            end_ledger,
            status: ProposalStatus::Active,
        };

        let prop_key = DataKey::Proposal(proposal_id);
        env.storage().persistent().set(&prop_key, &proposal);
        env.storage().persistent().extend_ttl(&prop_key, STORAGE_LIFETIME_THRESHOLD, STORAGE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("propose"), proposer),
            (proposal_id, end_ledger),
        );

        Ok(proposal_id)
    }

    /// Cast vote on a proposal (support: true = for, false = against)
    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        support: bool,
        weight: i128,
    ) -> Result<(), GovernanceError> {
        voter.require_auth();

        if weight <= 0 {
            return Err(GovernanceError::InvalidAmount);
        }

        let prop_key = DataKey::Proposal(proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&prop_key)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::VotingEnded);
        }

        if env.ledger().sequence() > proposal.end_ledger {
            return Err(GovernanceError::VotingEnded);
        }

        let voted_key = DataKey::HasVoted(proposal_id, voter.clone());
        if env.storage().persistent().has(&voted_key) {
            return Err(GovernanceError::AlreadyVoted);
        }

        if support {
            proposal.votes_for = proposal
                .votes_for
                .checked_add(weight)
                .ok_or(GovernanceError::ArithmeticOverflow)?;
        } else {
            proposal.votes_against = proposal
                .votes_against
                .checked_add(weight)
                .ok_or(GovernanceError::ArithmeticOverflow)?;
        }

        env.storage().persistent().set(&voted_key, &true);
        env.storage().persistent().set(&prop_key, &proposal);

        env.storage().persistent().extend_ttl(&voted_key, STORAGE_LIFETIME_THRESHOLD, STORAGE_BUMP_AMOUNT);
        env.storage().persistent().extend_ttl(&prop_key, STORAGE_LIFETIME_THRESHOLD, STORAGE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("vote"), voter, proposal_id),
            (support, weight),
        );

        Ok(())
    }

    /// Execute a passed proposal
    pub fn execute(env: Env, executor: Address, proposal_id: u32) -> Result<(), GovernanceError> {
        executor.require_auth();

        let prop_key = DataKey::Proposal(proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&prop_key)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.status == ProposalStatus::Executed {
            return Err(GovernanceError::AlreadyExecuted);
        }

        if env.ledger().sequence() <= proposal.end_ledger {
            return Err(GovernanceError::VotingStillActive);
        }

        let quorum: i128 = env.storage().instance().get(&DataKey::QuorumVotes).unwrap_or(1000);
        let total_votes = proposal
            .votes_for
            .checked_add(proposal.votes_against)
            .ok_or(GovernanceError::ArithmeticOverflow)?;

        if total_votes < quorum || proposal.votes_for <= proposal.votes_against {
            proposal.status = ProposalStatus::Defeated;
            env.storage().persistent().set(&prop_key, &proposal);
            return Err(GovernanceError::ProposalNotPassed);
        }

        proposal.status = ProposalStatus::Executed;
        env.storage().persistent().set(&prop_key, &proposal);

        env.storage().persistent().extend_ttl(&prop_key, STORAGE_LIFETIME_THRESHOLD, STORAGE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("execute"), executor, proposal_id),
            (proposal.target, proposal.amount),
        );

        Ok(())
    }

    /// Query proposal details
    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, GovernanceError> {
        let prop_key = DataKey::Proposal(proposal_id);
        env.storage().persistent().get(&prop_key).ok_or(GovernanceError::ProposalNotFound)
    }
}

#[cfg(test)]
mod test;

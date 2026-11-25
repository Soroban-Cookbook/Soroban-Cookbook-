//! # Governance Contract
//!
//! A governance contract that can create and execute proposals to withdraw
//! funds from a Treasury contract. Demonstrates cross-contract calls.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

use crate::treasury::{self, TreasuryClient};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GovernanceError {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    ProposalNotFound = 3,
    AlreadyVoted = 4,
    AlreadyExecuted = 5,
    InsufficientVotes = 6,
    TreasuryCallFailed = 7,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Treasury,
    Voters,
    Threshold,
    Proposal(u32),
    ProposalCount,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub recipient: Address,
    pub amount: i128,
    pub votes: Vec<Address>,
    pub executed: bool,
}

#[contract]
pub struct Governance;

#[contractimpl]
impl Governance {
    /// Initialize governance with treasury address, voters, and vote threshold
    pub fn initialize(
        env: Env,
        treasury: Address,
        voters: Vec<Address>,
        threshold: u32,
    ) -> Result<(), GovernanceError> {
        if env.storage().instance().has(&DataKey::Treasury) {
            return Err(GovernanceError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Treasury, &treasury);
        env.storage().instance().set(&DataKey::Voters, &voters);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);

        Ok(())
    }

    /// Create a proposal to withdraw from treasury
    pub fn propose(
        env: Env,
        proposer: Address,
        recipient: Address,
        amount: i128,
    ) -> Result<u32, GovernanceError> {
        proposer.require_auth();

        let voters: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Voters)
            .ok_or(GovernanceError::NotAuthorized)?;

        if !voters.contains(&proposer) {
            return Err(GovernanceError::NotAuthorized);
        }

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0);

        let proposal = Proposal {
            recipient,
            amount,
            votes: Vec::new(&env),
            executed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(count), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &(count + 1));

        env.events().publish(
            (soroban_sdk::symbol_short!("propose"),),
            (count, amount),
        );

        Ok(count)
    }

    /// Vote on a proposal
    pub fn vote(env: Env, proposal_id: u32, voter: Address) -> Result<(), GovernanceError> {
        voter.require_auth();

        let voters: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Voters)
            .ok_or(GovernanceError::NotAuthorized)?;

        if !voters.contains(&voter) {
            return Err(GovernanceError::NotAuthorized);
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.executed {
            return Err(GovernanceError::AlreadyExecuted);
        }

        if proposal.votes.contains(&voter) {
            return Err(GovernanceError::AlreadyVoted);
        }

        proposal.votes.push_back(voter);
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (soroban_sdk::symbol_short!("vote"),),
            proposal_id,
        );

        Ok(())
    }

    /// Execute a proposal if threshold is met - calls Treasury contract
    pub fn execute(env: Env, proposal_id: u32, executor: Address) -> Result<(), GovernanceError> {
        executor.require_auth();

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .ok_or(GovernanceError::NotAuthorized)?;

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.executed {
            return Err(GovernanceError::AlreadyExecuted);
        }

        if proposal.votes.len() < threshold {
            return Err(GovernanceError::InsufficientVotes);
        }

        // This is the cross-contract call!
        let treasury_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Treasury)
            .ok_or(GovernanceError::TreasuryCallFailed)?;

        let treasury_client = TreasuryClient::new(&env, &treasury_addr);

        // Get the treasury admin (which should be this governance contract)
        let admin = treasury_client
            .admin()
            .ok_or(GovernanceError::TreasuryCallFailed)?;

        // Call treasury withdraw - this demonstrates cross-contract invocation
        treasury_client
            .try_withdraw(&admin, &proposal.recipient, &proposal.amount)
            .map_err(|_| GovernanceError::TreasuryCallFailed)?;

        proposal.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (soroban_sdk::symbol_short!("execute"),),
            (proposal_id, proposal.amount),
        );

        Ok(())
    }

    /// Get proposal details
    pub fn get_proposal(env: Env, proposal_id: u32) -> Option<Proposal> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
    }

    /// Get vote count for a proposal
    pub fn vote_count(env: Env, proposal_id: u32) -> u32 {
        let proposal: Option<Proposal> = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id));

        match proposal {
            Some(p) => p.votes.len(),
            None => 0,
        }
    }

    /// Check if proposal has been executed
    pub fn is_executed(env: Env, proposal_id: u32) -> bool {
        let proposal: Option<Proposal> = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id));

        match proposal {
            Some(p) => p.executed,
            None => false,
        }
    }
}

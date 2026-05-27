//! # Multi-Party Authorization Patterns
//!
//! Demonstrates patterns for requiring multiple parties to authorize actions,
//! including threshold signatures, sequential approvals, and weighted voting.

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AuthError {
    InvalidThreshold = 1,
    NotAuthorized = 2,
    ProposalNotFound = 3,
    AlreadyApproved = 4,
    AlreadyExecuted = 5,
    ThresholdNotMet = 6,
    AlreadyInitialized = 7,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Threshold,
    Signers,
    Proposal(u32),
    ProposalCount,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub approvals: Vec<Address>,
    pub executed: bool,
}

#[contract]
pub struct MultiPartyAuth;

#[contractimpl]
impl MultiPartyAuth {
    /// Initialize with required threshold and authorized signers
    pub fn initialize(env: Env, threshold: u32, signers: Vec<Address>) -> Result<(), AuthError> {
        if env.storage().instance().has(&DataKey::Threshold) {
            return Err(AuthError::AlreadyInitialized);
        }

        if threshold == 0 || threshold > signers.len() {
            return Err(AuthError::InvalidThreshold);
        }
        
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        env.storage().instance().set(&DataKey::Signers, &signers);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);
        
        Ok(())
    }

    /// Create a proposal that requires multi-party approval
    pub fn create_proposal(env: Env, proposer: Address) -> Result<u32, AuthError> {
        proposer.require_auth();

        let signers: Vec<Address> = env.storage().instance()
            .get(&DataKey::Signers)
            .ok_or(AuthError::NotAuthorized)?;
            
        if !signers.contains(&proposer) {
            return Err(AuthError::NotAuthorized);
        }

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0);
        let proposal_id = count;

        let proposal = Proposal {
            approvals: Vec::new(&env),
            executed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &(count + 1));

        Ok(proposal_id)
    }

    /// Approve a proposal (each signer calls this)
    pub fn approve(env: Env, proposal_id: u32, signer: Address) -> Result<(), AuthError> {
        signer.require_auth();

        let signers: Vec<Address> = env.storage().instance()
            .get(&DataKey::Signers)
            .ok_or(AuthError::NotAuthorized)?;
            
        if !signers.contains(&signer) {
            return Err(AuthError::NotAuthorized);
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(AuthError::ProposalNotFound)?;

        if proposal.executed {
            return Err(AuthError::AlreadyExecuted);
        }

        if proposal.approvals.contains(&signer) {
            return Err(AuthError::AlreadyApproved);
        }

        proposal.approvals.push_back(signer);
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
            
        Ok(())
    }

    /// Execute proposal if threshold is met
    pub fn execute(env: Env, proposal_id: u32, executor: Address) -> Result<bool, AuthError> {
        executor.require_auth();

        let threshold: u32 = env.storage().instance()
            .get(&DataKey::Threshold)
            .ok_or(AuthError::NotAuthorized)?;
            
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(AuthError::ProposalNotFound)?;

        if proposal.executed {
            return Err(AuthError::AlreadyExecuted);
        }

        if proposal.approvals.len() < threshold {
            return Err(AuthError::ThresholdNotMet);
        }

        proposal.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        Ok(true)
    }

    /// Get proposal status
    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, AuthError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(AuthError::ProposalNotFound)
    }

    /// Require multiple addresses to authorize in a single call
    pub fn multi_auth_action(_env: Env, signers: Vec<Address>) -> bool {
        for signer in signers.iter() {
            signer.require_auth();
        }
        true
    }

    /// Require authorization from all configured signers
    pub fn require_all_signers(env: Env) -> Result<bool, AuthError> {
        let signers: Vec<Address> = env.storage().instance()
            .get(&DataKey::Signers)
            .ok_or(AuthError::NotAuthorized)?;
            
        for signer in signers.iter() {
            signer.require_auth();
        }
        Ok(true)
    }
}

#[cfg(test)]
mod test;

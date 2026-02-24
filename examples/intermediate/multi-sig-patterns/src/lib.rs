//! # Multi-Party Authorization Patterns
//!
//! Demonstrates patterns for requiring multiple parties to authorize actions,
//! including threshold signatures, sequential approvals, and weighted voting.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

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
    pub fn initialize(env: Env, threshold: u32, signers: Vec<Address>) {
        if threshold == 0 || threshold > signers.len() {
            panic!("Invalid threshold");
        }
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        env.storage().instance().set(&DataKey::Signers, &signers);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);
    }

    /// Create a proposal that requires multi-party approval
    pub fn create_proposal(env: Env, proposer: Address) -> u32 {
        proposer.require_auth();

        let signers: Vec<Address> = env.storage().instance().get(&DataKey::Signers).unwrap();
        if !signers.contains(&proposer) {
            panic!("Not an authorized signer");
        }

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap();
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

        proposal_id
    }

    /// Approve a proposal (each signer calls this)
    pub fn approve(env: Env, proposal_id: u32, signer: Address) {
        signer.require_auth();

        let signers: Vec<Address> = env.storage().instance().get(&DataKey::Signers).unwrap();
        if !signers.contains(&signer) {
            panic!("Not an authorized signer");
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap();

        if proposal.executed {
            panic!("Already executed");
        }

        if proposal.approvals.contains(&signer) {
            panic!("Already approved");
        }

        proposal.approvals.push_back(signer);
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    /// Execute proposal if threshold is met
    pub fn execute(env: Env, proposal_id: u32, executor: Address) -> bool {
        executor.require_auth();

        let threshold: u32 = env.storage().instance().get(&DataKey::Threshold).unwrap();
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap();

        if proposal.executed {
            panic!("Already executed");
        }

        if proposal.approvals.len() < threshold {
            panic!("Threshold not met");
        }

        proposal.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        true
    }

    /// Get proposal status
    pub fn get_proposal(env: Env, proposal_id: u32) -> Proposal {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap()
    }

    /// Require multiple addresses to authorize in a single call
    pub fn multi_auth_action(_env: Env, signers: Vec<Address>) -> bool {
        for signer in signers.iter() {
            signer.require_auth();
        }
        true
    }

    /// Require authorization from all configured signers
    pub fn require_all_signers(env: Env) -> bool {
        let signers: Vec<Address> = env.storage().instance().get(&DataKey::Signers).unwrap();
        for signer in signers.iter() {
            signer.require_auth();
        }
        true
    }
}

#[cfg(test)]
mod test;

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Bytes, Env, Symbol, Vec,
};

const MIN_LEAD_TIME: u64 = 10;
const MIN_DURATION: u64 = 20;
const MAX_DURATION: u64 = 60 * 60 * 24 * 7;
const MIN_QUORUM_BPS: u32 = 100;
const MAX_QUORUM_BPS: u32 = 10_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    NextProposalId,
    Proposal(u32),
    TopicIndex(Symbol),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub topic: Symbol,
    pub starts_at: u64,
    pub ends_at: u64,
    pub quorum_bps: u32,
    pub metadata_hash: Bytes,
    pub active: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProposalError {
    InvalidWindow = 1,
    InvalidDuration = 2,
    InvalidQuorum = 3,
    InvalidMetadata = 4,
    TopicConflict = 5,
    ProposalNotFound = 6,
    ProposalAlreadyClosed = 7,
}

#[contract]
pub struct ProposalValidation;

#[contractimpl]
impl ProposalValidation {
    pub fn create_proposal(
        env: Env,
        topic: Symbol,
        starts_at: u64,
        ends_at: u64,
        quorum_bps: u32,
        metadata_hash: Bytes,
    ) -> Result<u32, ProposalError> {
        Self::validate_input(&env, starts_at, ends_at, quorum_bps, metadata_hash.clone())?;
        Self::validate_topic_conflicts(&env, topic, starts_at, ends_at)?;

        let proposal_id = Self::next_proposal_id(&env);

        let proposal = Proposal {
            id: proposal_id,
            topic,
            starts_at,
            ends_at,
            quorum_bps,
            metadata_hash,
            active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        let mut topic_index: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::TopicIndex(topic))
            .unwrap_or_else(|| Vec::new(&env));
        topic_index.push_back(proposal_id);
        env.storage()
            .persistent()
            .set(&DataKey::TopicIndex(topic), &topic_index);

        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("created"), topic),
            proposal_id,
        );

        Ok(proposal_id)
    }

    pub fn get_proposal(env: Env, proposal_id: u32) -> Option<Proposal> {
        env.storage().persistent().get(&DataKey::Proposal(proposal_id))
    }

    pub fn close_proposal(env: Env, proposal_id: u32) -> Result<(), ProposalError> {
        let mut proposal = Self::get_proposal(env.clone(), proposal_id)
            .ok_or(ProposalError::ProposalNotFound)?;

        if !proposal.active {
            return Err(ProposalError::ProposalAlreadyClosed);
        }

        proposal.active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("closed"), proposal.topic),
            proposal_id,
        );

        Ok(())
    }

    pub fn validate_window(
        env: Env,
        starts_at: u64,
        ends_at: u64,
    ) -> Result<(), ProposalError> {
        Self::validate_window_internal(&env, starts_at, ends_at)
    }

    fn validate_input(
        env: &Env,
        starts_at: u64,
        ends_at: u64,
        quorum_bps: u32,
        metadata_hash: Bytes,
    ) -> Result<(), ProposalError> {
        Self::validate_window_internal(env, starts_at, ends_at)?;

        if !(MIN_QUORUM_BPS..=MAX_QUORUM_BPS).contains(&quorum_bps) {
            return Err(ProposalError::InvalidQuorum);
        }

        if metadata_hash.is_empty() {
            return Err(ProposalError::InvalidMetadata);
        }

        Ok(())
    }

    fn validate_window_internal(
        env: &Env,
        starts_at: u64,
        ends_at: u64,
    ) -> Result<(), ProposalError> {
        let now = env.ledger().timestamp();

        if starts_at <= now + MIN_LEAD_TIME || ends_at <= starts_at {
            return Err(ProposalError::InvalidWindow);
        }

        let duration = ends_at - starts_at;
        if !(MIN_DURATION..=MAX_DURATION).contains(&duration) {
            return Err(ProposalError::InvalidDuration);
        }

        Ok(())
    }

    fn validate_topic_conflicts(
        env: &Env,
        topic: Symbol,
        starts_at: u64,
        ends_at: u64,
    ) -> Result<(), ProposalError> {
        let topic_index: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::TopicIndex(topic))
            .unwrap_or_else(|| Vec::new(env));

        for proposal_id in topic_index.iter() {
            let proposal: Proposal = env
                .storage()
                .persistent()
                .get(&DataKey::Proposal(proposal_id))
                .ok_or(ProposalError::ProposalNotFound)?;

            if proposal.active && windows_overlap(starts_at, ends_at, proposal.starts_at, proposal.ends_at)
            {
                return Err(ProposalError::TopicConflict);
            }
        }

        Ok(())
    }

    fn next_proposal_id(env: &Env) -> u32 {
        let next: u32 = env.storage().instance().get(&DataKey::NextProposalId).unwrap_or(1);
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &(next + 1));
        next
    }
}

fn windows_overlap(start_a: u64, end_a: u64, start_b: u64, end_b: u64) -> bool {
    start_a < end_b && start_b < end_a
}

#[cfg(test)]
mod test;

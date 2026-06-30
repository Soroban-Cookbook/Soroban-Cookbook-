#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env, String,
    Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalState {
    Draft = 1,
    Active = 2,
    Passed = 3,
    Failed = 4,
    Executed = 5,
    Cancelled = 6,
    Expired = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub description: String,
    pub target_contract: Address,
    pub action: Symbol,
    pub action_args: Vec<soroban_sdk::Val>,
    pub votes_yes: i128,
    pub votes_no: i128,
    pub state: ProposalState,
    pub voting_end_ledger: u32,
    pub execution_end_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    ProposalCount,
    Proposal(u32),
    Voted(u32, Address),
    MinQuorum,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ProposalNotFound = 3,
    InvalidState = 4,
    VotingEnded = 5,
    VotingNotEnded = 6,
    ExecutionEnded = 7,
    ExecutionNotEnded = 8,
    AlreadyVoted = 9,
    Unauthorized = 10,
    QuorumNotMet = 11,
    ProposalFailed = 12,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceInitialized {
    pub admin: Address,
    pub min_quorum: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCreated {
    pub proposer: Address,
    pub proposal_id: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalSubmitted {
    pub proposal_id: u32,
    pub voting_end_ledger: u32,
    pub execution_end_ledger: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteCast {
    pub proposal_id: u32,
    pub voter: Address,
    pub approve: bool,
    pub weight: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalExecuted {
    pub proposal_id: u32,
    pub executor: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCancelled {
    pub proposal_id: u32,
    pub cancelled_by: Address,
}

#[contract]
pub struct ProposalLifecycleContract;

#[contractimpl]
impl ProposalLifecycleContract {
    pub fn initialize(env: Env, admin: Address, min_quorum: i128) -> Result<(), ProposalError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ProposalError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::MinQuorum, &min_quorum);

        GovernanceInitialized { admin, min_quorum }.publish(&env);
        Ok(())
    }

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        description: String,
        target_contract: Address,
        action: Symbol,
        action_args: Vec<soroban_sdk::Val>,
    ) -> Result<u32, ProposalError> {
        proposer.require_auth();

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .ok_or(ProposalError::NotInitialized)?;

        let proposal = Proposal {
            id: count,
            proposer: proposer.clone(),
            description,
            target_contract,
            action,
            action_args,
            votes_yes: 0,
            votes_no: 0,
            state: ProposalState::Draft,
            voting_end_ledger: 0,
            execution_end_ledger: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(count), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &(count + 1));

        ProposalCreated {
            proposer,
            proposal_id: count,
        }
        .publish(&env);

        Ok(count)
    }

    pub fn submit_proposal(
        env: Env,
        proposer: Address,
        proposal_id: u32,
        voting_duration: u32,
        execution_duration: u32,
    ) -> Result<(), ProposalError> {
        proposer.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(ProposalError::ProposalNotFound)?;

        if proposal.proposer != proposer {
            return Err(ProposalError::Unauthorized);
        }

        if proposal.state != ProposalState::Draft {
            return Err(ProposalError::InvalidState);
        }

        let voting_end = env.ledger().sequence() + voting_duration;
        let execution_end = voting_end + execution_duration;

        proposal.state = ProposalState::Active;
        proposal.voting_end_ledger = voting_end;
        proposal.execution_end_ledger = execution_end;

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        ProposalSubmitted {
            proposal_id,
            voting_end_ledger: voting_end,
            execution_end_ledger: execution_end,
        }
        .publish(&env);

        Ok(())
    }

    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        approve: bool,
        weight: i128,
    ) -> Result<(), ProposalError> {
        voter.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(ProposalError::ProposalNotFound)?;

        let current_state = Self::resolve_proposal_state(&env, &proposal);
        if current_state != ProposalState::Active {
            return Err(ProposalError::InvalidState);
        }

        if env.ledger().sequence() > proposal.voting_end_ledger {
            return Err(ProposalError::VotingEnded);
        }

        let vote_key = DataKey::Voted(proposal_id, voter.clone());
        if env.storage().persistent().has(&vote_key) {
            return Err(ProposalError::AlreadyVoted);
        }

        if approve {
            proposal.votes_yes += weight;
        } else {
            proposal.votes_no += weight;
        }

        env.storage().persistent().set(&vote_key, &true);
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        VoteCast {
            proposal_id,
            voter,
            approve,
            weight,
        }
        .publish(&env);

        Ok(())
    }

    pub fn execute_proposal(
        env: Env,
        executor: Address,
        proposal_id: u32,
    ) -> Result<(), ProposalError> {
        executor.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(ProposalError::ProposalNotFound)?;

        let current_state = Self::resolve_proposal_state(&env, &proposal);
        if current_state != ProposalState::Passed {
            if current_state == ProposalState::Expired {
                return Err(ProposalError::ExecutionEnded);
            }
            if current_state == ProposalState::Active {
                return Err(ProposalError::VotingNotEnded);
            }
            return Err(ProposalError::InvalidState);
        }

        env.invoke_contract::<()>(
            &proposal.target_contract,
            &proposal.action,
            proposal.action_args.clone(),
        );

        proposal.state = ProposalState::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        ProposalExecuted {
            proposal_id,
            executor,
        }
        .publish(&env);

        Ok(())
    }

    pub fn cancel_proposal(
        env: Env,
        caller: Address,
        proposal_id: u32,
    ) -> Result<(), ProposalError> {
        caller.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(ProposalError::ProposalNotFound)?;

        let current_state = Self::resolve_proposal_state(&env, &proposal);
        if current_state == ProposalState::Executed
            || current_state == ProposalState::Cancelled
            || current_state == ProposalState::Expired
        {
            return Err(ProposalError::InvalidState);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ProposalError::NotInitialized)?;

        if caller != proposal.proposer && caller != admin {
            return Err(ProposalError::Unauthorized);
        }

        proposal.state = ProposalState::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        ProposalCancelled {
            proposal_id,
            cancelled_by: caller,
        }
        .publish(&env);

        Ok(())
    }

    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, ProposalError> {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(ProposalError::ProposalNotFound)?;

        proposal.state = Self::resolve_proposal_state(&env, &proposal);
        Ok(proposal)
    }

    pub fn get_proposal_state(env: Env, proposal_id: u32) -> Result<ProposalState, ProposalError> {
        let proposal = Self::get_proposal(env.clone(), proposal_id)?;
        Ok(proposal.state)
    }

    fn resolve_proposal_state(env: &Env, proposal: &Proposal) -> ProposalState {
        if proposal.state == ProposalState::Draft
            || proposal.state == ProposalState::Executed
            || proposal.state == ProposalState::Cancelled
        {
            return proposal.state;
        }

        let seq = env.ledger().sequence();
        if proposal.state == ProposalState::Active {
            if seq <= proposal.voting_end_ledger {
                return ProposalState::Active;
            }

            let min_quorum: i128 = env
                .storage()
                .instance()
                .get(&DataKey::MinQuorum)
                .unwrap_or(0);

            let total_votes = proposal.votes_yes + proposal.votes_no;
            if total_votes < min_quorum {
                return ProposalState::Failed;
            }

            if proposal.votes_yes > proposal.votes_no {
                if seq <= proposal.execution_end_ledger {
                    return ProposalState::Passed;
                } else {
                    return ProposalState::Expired;
                }
            } else {
                return ProposalState::Failed;
            }
        }

        proposal.state
    }
}

#[cfg(test)]
mod test;

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalState {
    Active = 1,
    Passed = 2,
    Failed = 3,
    Executed = 4,
    Cancelled = 5,
    Expired = 6,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalKind {
    Transfer = 1,
    Upgrade = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub proposer: Address,
    pub kind: ProposalKind,
    pub state: ProposalState,
    pub votes_yes: i128,
    pub votes_no: i128,
    pub transfer_amount: i128,
    pub recipient: Address,
    pub upgrade_hash: BytesN<32>,
    pub voting_end_ledger: u32,
    pub exec_end_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    MinQuorum,
    VotingDuration,
    ExecDuration,
    ProposalCount,
    TreasuryBalance,
    Proposal(u32),
    Voted(u32, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DaoError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ProposalNotFound = 3,
    InvalidAmount = 4,
    VotingEnded = 5,
    VotingNotEnded = 6,
    ExecutionEnded = 7,
    AlreadyVoted = 8,
    Unauthorized = 9,
    InvalidState = 10,
    InsufficientTreasuryBalance = 11,
    AlreadyExecuted = 12,
}

#[contract]
pub struct DaoContract;

#[contractimpl]
impl DaoContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        min_quorum: i128,
        voting_duration: u32,
        exec_duration: u32,
    ) -> Result<(), DaoError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(DaoError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::MinQuorum, &min_quorum);
        env.storage()
            .instance()
            .set(&DataKey::VotingDuration, &voting_duration);
        env.storage()
            .instance()
            .set(&DataKey::ExecDuration, &exec_duration);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);
        env.storage()
            .persistent()
            .set(&DataKey::TreasuryBalance, &0i128);
        Ok(())
    }

    pub fn admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    pub fn proposal_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0)
    }

    pub fn treasury_balance(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TreasuryBalance)
            .unwrap_or(0)
    }

    pub fn deposit(env: Env, depositor: Address, amount: i128) -> Result<(), DaoError> {
        Self::require_initialized(&env)?;
        depositor.require_auth();
        if amount <= 0 {
            return Err(DaoError::InvalidAmount);
        }
        let balance = Self::treasury_balance(env.clone());
        let updated = balance.checked_add(amount).ok_or(DaoError::InvalidAmount)?;
        env.storage()
            .persistent()
            .set(&DataKey::TreasuryBalance, &updated);
        Ok(())
    }

    pub fn propose_transfer(
        env: Env,
        proposer: Address,
        recipient: Address,
        amount: i128,
    ) -> Result<u32, DaoError> {
        Self::require_initialized(&env)?;
        proposer.require_auth();
        if amount <= 0 {
            return Err(DaoError::InvalidAmount);
        }
        Self::create_proposal(
            env.clone(),
            proposer,
            ProposalKind::Transfer,
            amount,
            recipient,
            BytesN::from_array(&env, &[0u8; 32]),
        )
    }

    pub fn propose_upgrade(
        env: Env,
        proposer: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<u32, DaoError> {
        Self::require_initialized(&env)?;
        proposer.require_auth();
        Self::create_proposal(
            env,
            proposer.clone(),
            ProposalKind::Upgrade,
            0,
            proposer,
            new_wasm_hash,
        )
    }

    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        approve: bool,
        weight: i128,
    ) -> Result<(), DaoError> {
        Self::require_initialized(&env)?;
        voter.require_auth();
        if weight <= 0 {
            return Err(DaoError::InvalidAmount);
        }

        let mut proposal = Self::load_proposal(&env, proposal_id)?;
        if proposal.state == ProposalState::Cancelled || proposal.state == ProposalState::Executed {
            return Err(DaoError::InvalidState);
        }

        let current = env.ledger().sequence();
        if current > proposal.voting_end_ledger {
            return Err(DaoError::VotingEnded);
        }

        let voted_key = DataKey::Voted(proposal_id, voter.clone());
        if env.storage().persistent().has(&voted_key) {
            return Err(DaoError::AlreadyVoted);
        }

        if approve {
            proposal.votes_yes = proposal
                .votes_yes
                .checked_add(weight)
                .ok_or(DaoError::InvalidAmount)?;
        } else {
            proposal.votes_no = proposal
                .votes_no
                .checked_add(weight)
                .ok_or(DaoError::InvalidAmount)?;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().persistent().set(&voted_key, &true);
        Ok(())
    }

    pub fn execute(env: Env, executor: Address, proposal_id: u32) -> Result<(), DaoError> {
        Self::require_initialized(&env)?;
        executor.require_auth();

        let mut proposal = Self::load_proposal(&env, proposal_id)?;
        if proposal.state == ProposalState::Executed {
            return Err(DaoError::AlreadyExecuted);
        }
        if proposal.state == ProposalState::Cancelled {
            return Err(DaoError::InvalidState);
        }

        let state = Self::resolve_state(&env, &proposal);
        if state != ProposalState::Passed {
            return Err(DaoError::InvalidState);
        }

        let current = env.ledger().sequence();
        if current <= proposal.voting_end_ledger {
            return Err(DaoError::VotingNotEnded);
        }
        if current > proposal.exec_end_ledger {
            return Err(DaoError::ExecutionEnded);
        }

        if proposal.kind == ProposalKind::Transfer {
            let balance = Self::treasury_balance(env.clone());
            if balance < proposal.transfer_amount {
                return Err(DaoError::InsufficientTreasuryBalance);
            }
            env.storage().persistent().set(
                &DataKey::TreasuryBalance,
                &(balance - proposal.transfer_amount),
            );
        }

        proposal.state = ProposalState::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (symbol_short!("dao"), symbol_short!("exec"), proposal_id),
            executor,
        );
        Ok(())
    }

    pub fn cancel(env: Env, caller: Address, proposal_id: u32) -> Result<(), DaoError> {
        Self::require_initialized(&env)?;
        caller.require_auth();

        let admin = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
            .ok_or(DaoError::NotInitialized)?;

        let mut proposal = Self::load_proposal(&env, proposal_id)?;
        if proposal.state == ProposalState::Executed {
            return Err(DaoError::InvalidState);
        }
        if proposal.state == ProposalState::Cancelled {
            return Ok(());
        }

        if caller != proposal.proposer && caller != admin {
            return Err(DaoError::Unauthorized);
        }

        proposal.state = ProposalState::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        Ok(())
    }

    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, DaoError> {
        Self::require_initialized(&env)?;
        let mut proposal = Self::load_proposal(&env, proposal_id)?;
        proposal.state = Self::resolve_state(&env, &proposal);
        Ok(proposal)
    }

    pub fn proposal_state(env: Env, proposal_id: u32) -> Result<ProposalState, DaoError> {
        let proposal = Self::load_proposal(&env, proposal_id)?;
        Ok(Self::resolve_state(&env, &proposal))
    }

    pub fn has_voted(env: Env, proposal_id: u32, voter: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Voted(proposal_id, voter))
            .unwrap_or(false)
    }

    fn create_proposal(
        env: Env,
        proposer: Address,
        kind: ProposalKind,
        transfer_amount: i128,
        recipient: Address,
        upgrade_hash: BytesN<32>,
    ) -> Result<u32, DaoError> {
        let voting_duration: u32 = env
            .storage()
            .instance()
            .get(&DataKey::VotingDuration)
            .ok_or(DaoError::NotInitialized)?;
        let exec_duration: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ExecDuration)
            .ok_or(DaoError::NotInitialized)?;

        let proposal_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .ok_or(DaoError::NotInitialized)?;

        let current = env.ledger().sequence();
        let voting_end = current.saturating_add(voting_duration);
        let exec_end = voting_end.saturating_add(exec_duration);

        let proposal = Proposal {
            proposer: proposer.clone(),
            kind,
            state: ProposalState::Active,
            votes_yes: 0,
            votes_no: 0,
            transfer_amount,
            recipient,
            upgrade_hash,
            voting_end_ledger: voting_end,
            exec_end_ledger: exec_end,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &(proposal_id + 1));

        env.events().publish(
            (symbol_short!("dao"), symbol_short!("prop"), proposal_id),
            proposer,
        );
        Ok(proposal_id)
    }

    fn resolve_state(env: &Env, proposal: &Proposal) -> ProposalState {
        if proposal.state == ProposalState::Cancelled || proposal.state == ProposalState::Executed {
            return proposal.state;
        }

        let current = env.ledger().sequence();
        let min_quorum: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinQuorum)
            .unwrap_or(0);

        if current <= proposal.voting_end_ledger {
            return ProposalState::Active;
        }

        let total_votes = proposal.votes_yes.saturating_add(proposal.votes_no);
        let passed = total_votes >= min_quorum && proposal.votes_yes > proposal.votes_no;

        if current > proposal.exec_end_ledger {
            if passed {
                return ProposalState::Expired;
            }
            return ProposalState::Failed;
        }

        if passed {
            ProposalState::Passed
        } else {
            ProposalState::Failed
        }
    }

    fn load_proposal(env: &Env, proposal_id: u32) -> Result<Proposal, DaoError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(DaoError::ProposalNotFound)
    }

    fn require_initialized(env: &Env) -> Result<(), DaoError> {
        if env.storage().instance().has(&DataKey::Admin) {
            Ok(())
        } else {
            Err(DaoError::NotInitialized)
        }
    }
}

#[cfg(test)]
mod test;

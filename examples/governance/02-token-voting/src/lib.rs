#![no_std]

use soroban_sdk::{contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Val, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BalanceCheckpoint {
    pub ledger: u32,
    pub balance: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegateCheckpoint {
    pub ledger: u32,
    pub delegatee: Address,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalState {
    Draft = 0,
    Active = 1,
    Passed = 2,
    Failed = 3,
    Executed = 4,
    Cancelled = 5,
    Expired = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub description: String,
    pub target_contract: Address,
    pub action: Symbol,
    pub action_args: Vec<Val>,
    pub votes_yes: i128,
    pub votes_no: i128,
    pub state: ProposalState,
    pub voting_end_ledger: u32,
    pub execution_end_ledger: u32,
    pub snapshot_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    MinQuorum,
    TotalSupply,
    Balance(Address),
    BalanceHistory(Address),
    DelegatedTo(Address),
    DelegateHistory(Address),
    ProposalCount,
    Proposal(u32),
    Vote(u32, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VotingError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidState = 4,
    VotingEnded = 5,
    ExecutionEnded = 6,
    AlreadyVoted = 7,
    AlreadyDelegated = 8,
    DelegationNotFound = 9,
    InsufficientBalance = 10,
    NoVotingPower = 11,
    QuorumNotMet = 12,
    ProposalNotFound = 13,
    ProposalFailed = 14,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceInitialized {
    pub admin: Address,
    pub min_quorum: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMinted {
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenTransferred {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationSet {
    pub delegator: Address,
    pub delegatee: Address,
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
    pub snapshot_ledger: u32,
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
pub struct TokenVotingContract;

#[contractimpl]
impl TokenVotingContract {
    pub fn initialize(env: Env, admin: Address, min_quorum: i128) -> Result<(), VotingError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VotingError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);
        env.storage().instance().set(&DataKey::MinQuorum, &min_quorum);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);

        GovernanceInitialized { admin, min_quorum }.publish(&env);
        Ok(())
    }

    pub fn mint(
        env: Env,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), VotingError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        if amount <= 0 {
            return Err(VotingError::InsufficientBalance);
        }

        let old_supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        let new_supply = old_supply + amount;
        env.storage().instance().set(&DataKey::TotalSupply, &new_supply);

        let current_balance = Self::current_balance(&env, &to);
        Self::set_balance(&env, &to, current_balance + amount);

        TokenMinted { to, amount }.publish(&env);
        Ok(())
    }

    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), VotingError> {
        from.require_auth();
        if amount <= 0 {
            return Err(VotingError::InsufficientBalance);
        }

        let from_balance = Self::current_balance(&env, &from);
        if from_balance < amount {
            return Err(VotingError::InsufficientBalance);
        }

        let to_balance = Self::current_balance(&env, &to);
        Self::set_balance(&env, &from, from_balance - amount);
        Self::set_balance(&env, &to, to_balance + amount);

        TokenTransferred { from, to, amount }.publish(&env);
        Ok(())
    }

    pub fn get_balance(env: Env, account: Address) -> i128 {
        Self::current_balance(&env, &account)
    }

    pub fn get_balance_at(env: Env, account: Address, ledger: u32) -> i128 {
        Self::balance_at(&env, &account, ledger)
    }

    pub fn delegate(
        env: Env,
        delegator: Address,
        delegatee: Address,
    ) -> Result<(), VotingError> {
        delegator.require_auth();
        if delegator == delegatee {
            return Err(VotingError::AlreadyDelegated);
        }

        Self::record_delegate(&env, &delegator, &delegatee);
        DelegationSet { delegator, delegatee }.publish(&env);
        Ok(())
    }

    pub fn get_delegate(env: Env, account: Address) -> Option<Address> {
        env.storage().persistent().get(&DataKey::DelegatedTo(account))
    }

    pub fn get_delegate_at(
        env: Env,
        account: Address,
        ledger: u32,
    ) -> Option<Address> {
        Self::delegate_at(&env, &account, ledger)
    }

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        description: String,
        target_contract: Address,
        action: Symbol,
        action_args: Vec<Val>,
    ) -> Result<u32, VotingError> {
        proposer.require_auth();

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .ok_or(VotingError::NotInitialized)?;

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
            snapshot_ledger: 0,
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
    ) -> Result<(), VotingError> {
        proposer.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::ProposalNotFound)?;

        if proposal.proposer != proposer {
            return Err(VotingError::Unauthorized);
        }
        if proposal.state != ProposalState::Draft {
            return Err(VotingError::InvalidState);
        }

        let snapshot = env.ledger().sequence();
        let voting_end = snapshot + voting_duration;
        let execution_end = voting_end + execution_duration;

        proposal.state = ProposalState::Active;
        proposal.snapshot_ledger = snapshot;
        proposal.voting_end_ledger = voting_end;
        proposal.execution_end_ledger = execution_end;

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        ProposalSubmitted {
            proposal_id,
            snapshot_ledger: snapshot,
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
    ) -> Result<(), VotingError> {
        voter.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::ProposalNotFound)?;

        let current_state = Self::resolve_proposal_state(&env, &proposal);
        if current_state != ProposalState::Active {
            return Err(VotingError::InvalidState);
        }
        if env.ledger().sequence() > proposal.voting_end_ledger {
            return Err(VotingError::VotingEnded);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Vote(proposal_id, voter.clone()))
        {
            return Err(VotingError::AlreadyVoted);
        }
        if let Some(delegatee) = Self::delegate_at(&env, &voter, proposal.snapshot_ledger) {
            if delegatee != voter {
                return Err(VotingError::AlreadyDelegated);
            }
        }

        let weight = Self::balance_at(&env, &voter, proposal.snapshot_ledger);
        if weight <= 0 {
            return Err(VotingError::NoVotingPower);
        }

        if approve {
            proposal.votes_yes += weight;
        } else {
            proposal.votes_no += weight;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Vote(proposal_id, voter.clone()), &true);
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

    pub fn vote_for(
        env: Env,
        delegatee: Address,
        proposal_id: u32,
        delegator: Address,
        approve: bool,
    ) -> Result<(), VotingError> {
        delegatee.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::ProposalNotFound)?;

        let current_state = Self::resolve_proposal_state(&env, &proposal);
        if current_state != ProposalState::Active {
            return Err(VotingError::InvalidState);
        }
        if env.ledger().sequence() > proposal.voting_end_ledger {
            return Err(VotingError::VotingEnded);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Vote(proposal_id, delegator.clone()))
        {
            return Err(VotingError::AlreadyVoted);
        }

        let delegatee_at_snapshot = Self::delegate_at(&env, &delegator, proposal.snapshot_ledger);
        if delegatee_at_snapshot != Some(delegatee.clone()) {
            return Err(VotingError::DelegationNotFound);
        }

        let weight = Self::balance_at(&env, &delegator, proposal.snapshot_ledger);
        if weight <= 0 {
            return Err(VotingError::NoVotingPower);
        }

        if approve {
            proposal.votes_yes += weight;
        } else {
            proposal.votes_no += weight;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Vote(proposal_id, delegator.clone()), &true);
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        VoteCast {
            proposal_id,
            voter: delegator,
            approve,
            weight,
        }
        .publish(&env);

        Ok(())
    }

    pub fn get_proposal(
        env: Env,
        proposal_id: u32,
    ) -> Result<Proposal, VotingError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::ProposalNotFound)
    }

    pub fn get_proposal_state(
        env: Env,
        proposal_id: u32,
    ) -> Result<ProposalState, VotingError> {
        let proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::ProposalNotFound)?;
        Ok(Self::resolve_proposal_state(&env, &proposal))
    }

    pub fn execute_proposal(
        env: Env,
        executor: Address,
        proposal_id: u32,
    ) -> Result<(), VotingError> {
        executor.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::ProposalNotFound)?;

        let current_state = Self::resolve_proposal_state(&env, &proposal);
        if current_state != ProposalState::Passed {
            return Err(VotingError::InvalidState);
        }
        if env.ledger().sequence() > proposal.execution_end_ledger {
            return Err(VotingError::ExecutionEnded);
        }

        env.invoke_contract::<()>(&proposal.target_contract, &proposal.action, proposal.action_args.clone());

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
    ) -> Result<(), VotingError> {
        caller.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(VotingError::ProposalNotFound)?;

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VotingError::NotInitialized)?;

        if caller != proposal.proposer && caller != admin {
            return Err(VotingError::Unauthorized);
        }

        let state = Self::resolve_proposal_state(&env, &proposal);
        if !(state == ProposalState::Draft || state == ProposalState::Active) {
            return Err(VotingError::InvalidState);
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

    fn require_admin(env: &Env, admin: &Address) -> Result<(), VotingError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VotingError::NotInitialized)?;

        if &stored_admin != admin {
            return Err(VotingError::Unauthorized);
        }
        Ok(())
    }

    fn current_balance(env: &Env, account: &Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(account.clone()))
            .unwrap_or(0)
    }

    fn set_balance(env: &Env, account: &Address, amount: i128) {
        env.storage()
            .persistent()
            .set(&DataKey::Balance(account.clone()), &amount);
        Self::record_balance_checkpoint(env, account, amount);
    }

    fn record_balance_checkpoint(env: &Env, account: &Address, balance: i128) {
        let mut history: Vec<BalanceCheckpoint> = env
            .storage()
            .persistent()
            .get(&DataKey::BalanceHistory(account.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        history.push_back(BalanceCheckpoint {
            ledger: env.ledger().sequence(),
            balance,
        });
        env.storage()
            .persistent()
            .set(&DataKey::BalanceHistory(account.clone()), &history);
    }

    fn balance_at(env: &Env, account: &Address, ledger: u32) -> i128 {
        let history: Vec<BalanceCheckpoint> = env
            .storage()
            .persistent()
            .get(&DataKey::BalanceHistory(account.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        let mut idx = history.len();
        while idx > 0 {
            idx -= 1;
            let checkpoint = history.get_unchecked(idx);
            if checkpoint.ledger <= ledger {
                return checkpoint.balance;
            }
        }
        0
    }

    fn record_delegate(env: &Env, delegator: &Address, delegatee: &Address) {
        env.storage()
            .persistent()
            .set(&DataKey::DelegatedTo(delegator.clone()), delegatee);

        let mut history: Vec<DelegateCheckpoint> = env
            .storage()
            .persistent()
            .get(&DataKey::DelegateHistory(delegator.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        history.push_back(DelegateCheckpoint {
            ledger: env.ledger().sequence(),
            delegatee: delegatee.clone(),
        });
        env.storage()
            .persistent()
            .set(&DataKey::DelegateHistory(delegator.clone()), &history);
    }

    fn delegate_at(env: &Env, account: &Address, ledger: u32) -> Option<Address> {
        let history: Vec<DelegateCheckpoint> = env
            .storage()
            .persistent()
            .get(&DataKey::DelegateHistory(account.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        let mut idx = history.len();
        while idx > 0 {
            idx -= 1;
            let checkpoint = history.get_unchecked(idx);
            if checkpoint.ledger <= ledger {
                return Some(checkpoint.delegatee.clone());
            }
        }
        None
    }

    fn resolve_proposal_state(env: &Env, proposal: &Proposal) -> ProposalState {
        match proposal.state {
            ProposalState::Draft | ProposalState::Cancelled | ProposalState::Executed => {
                proposal.state
            }
            ProposalState::Active => {
                let now = env.ledger().sequence();
                if now <= proposal.voting_end_ledger {
                    ProposalState::Active
                } else if !Self::quorum_met(env, proposal)
                    || proposal.votes_yes <= proposal.votes_no
                {
                    ProposalState::Failed
                } else if now <= proposal.execution_end_ledger {
                    ProposalState::Passed
                } else {
                    ProposalState::Expired
                }
            }
            ProposalState::Passed => {
                if env.ledger().sequence() > proposal.execution_end_ledger {
                    ProposalState::Expired
                } else {
                    ProposalState::Passed
                }
            }
            ProposalState::Failed => ProposalState::Failed,
            ProposalState::Expired => ProposalState::Expired,
        }
    }

    fn quorum_met(env: &Env, proposal: &Proposal) -> bool {
        let min_quorum: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinQuorum)
            .unwrap_or(0);
        proposal.votes_yes + proposal.votes_no >= min_quorum
    }
}

#![allow(deprecated)]
//! Governance Integration Tests
//!
//! 30 tests across 8 categories covering the full governance stack:
//!
//! | # | Category                 | Tests |
//! |---|--------------------------|-------|
//! | 1 | Proposal lifecycle       | 5     |
//! | 2 | Voting                   | 4     |
//! | 3 | DAO treasury             | 3     |
//! | 4 | Authorization            | 2     |
//! | 5 | Multiple / concurrent    | 4     |
//! | 6 | Delegation (inline mock) | 4     |
//! | 7 | Voting-time-constraints  | 4     |
//! | 8 | End-to-end               | 4     |
//!
//! Run with:
//!   cargo test -p integration-tests governance

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, testutils::Ledger as _, Address,
    Env, IntoVal, String, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Inline mock target – governance proposals invoke this contract
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum MockDataKey {
    Value,
}

#[contract]
pub struct MockTargetContract;

#[contractimpl]
impl MockTargetContract {
    pub fn set_value(env: Env, value: u32) {
        env.storage().instance().set(&MockDataKey::Value, &value);
    }

    pub fn get_value(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&MockDataKey::Value)
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Inline delegation mock – tracks per-address delegated vote weights
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DelegationKey {
    Delegate(Address),
    Delegator(Address),
}

#[contract]
pub struct DelegationContract;

#[contractimpl]
impl DelegationContract {
    pub fn delegate(env: Env, delegator: Address, delegatee: Address, weight: i128) {
        delegator.require_auth();
        if let Some(old) = env
            .storage()
            .persistent()
            .get::<_, Address>(&DelegationKey::Delegator(delegator.clone()))
        {
            let old_w: i128 = env
                .storage()
                .persistent()
                .get(&DelegationKey::Delegate(old.clone()))
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&DelegationKey::Delegate(old), &(old_w - weight));
        }
        env.storage()
            .persistent()
            .set(&DelegationKey::Delegator(delegator), &delegatee.clone());
        let cur: i128 = env
            .storage()
            .persistent()
            .get(&DelegationKey::Delegate(delegatee.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DelegationKey::Delegate(delegatee), &(cur + weight));
    }

    pub fn undelegate(env: Env, delegator: Address, weight: i128) {
        delegator.require_auth();
        if let Some(delegatee) = env
            .storage()
            .persistent()
            .get::<_, Address>(&DelegationKey::Delegator(delegator.clone()))
        {
            let old_w: i128 = env
                .storage()
                .persistent()
                .get(&DelegationKey::Delegate(delegatee.clone()))
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&DelegationKey::Delegate(delegatee), &(old_w - weight));
            env.storage()
                .persistent()
                .remove(&DelegationKey::Delegator(delegator));
        }
    }

    pub fn voting_weight(env: Env, delegatee: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DelegationKey::Delegate(delegatee))
            .unwrap_or(0)
    }

    pub fn get_delegatee(env: Env, delegator: Address) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DelegationKey::Delegator(delegator))
    }
}

// ===========================================================================
// Category 1 – Proposal Lifecycle (5 tests)
// ===========================================================================

// 1-1  Initialize and create a draft proposal.
#[test]
fn test_gov_proposal_creation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let target = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &50i128);

    let description = String::from_str(&env, "Proposal 1: fund community initiative");
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &target,
        &Symbol::new(&env, "set_value"),
        &Vec::from_array(&env, [1u32.into_val(&env)]),
    );

    assert_eq!(proposal_id, 0);
    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.proposer, proposer);
    assert_eq!(proposal.state, proposal_lifecycle::ProposalState::Draft);
    assert_eq!(proposal.votes_yes, 0);
    assert_eq!(proposal.votes_no, 0);
}

// 1-2  create_proposal errors when contract is not initialized.
#[test]
fn test_gov_create_proposal_not_initialized_error() {
    let env = Env::default();
    env.mock_all_auths();

    let proposer = Address::generate(&env);
    let target = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);

    let result = client.try_create_proposal(
        &proposer,
        &String::from_str(&env, "Should fail"),
        &target,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    assert_eq!(
        result,
        Err(Ok(proposal_lifecycle::ProposalError::NotInitialized))
    );
}

// 1-3  Submit moves a proposal from Draft to Active.
#[test]
fn test_gov_proposal_submit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let target = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &50i128);

    let proposal_id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Submit test"),
        &target,
        &Symbol::new(&env, "set_value"),
        &Vec::new(&env),
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.state, proposal_lifecycle::ProposalState::Active);
    assert_eq!(proposal.voting_end_ledger, 150);
    assert_eq!(proposal.execution_end_ledger, 250);
}

// 1-4  A passed proposal can be executed against a real target contract.
#[test]
fn test_gov_proposal_execute() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let gov_client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    gov_client.initialize(&admin, &50i128);

    let target_id = env.register_contract(None, MockTargetContract);

    let proposal_id = gov_client.create_proposal(
        &proposer,
        &String::from_str(&env, "Set value to 42"),
        &target_id,
        &Symbol::new(&env, "set_value"),
        &Vec::from_array(&env, [42u32.into_val(&env)]),
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    gov_client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);
    gov_client.vote(&voter, &proposal_id, &true, &60i128);

    // Advance past voting window, within execution window.
    env.ledger().with_mut(|l| l.sequence_number = 160);
    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Passed
    );

    gov_client.execute_proposal(&voter, &proposal_id);
    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Executed
    );

    let stored_val = env.as_contract(&target_id, || {
        env.storage()
            .instance()
            .get::<_, u32>(&MockDataKey::Value)
            .unwrap()
    });
    assert_eq!(stored_val, 42);
}

// 1-5  A proposal rejected by quorum stays Failed; a draft can be cancelled.
#[test]
fn test_gov_proposal_reject_and_cancel() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let gov_client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    gov_client.initialize(&admin, &100i128);

    let proposal_id = gov_client.create_proposal(
        &proposer,
        &String::from_str(&env, "Doomed proposal"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    gov_client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);
    // Only 30 votes – quorum of 100 not met.
    gov_client.vote(&voter, &proposal_id, &true, &30i128);

    env.ledger().with_mut(|l| l.sequence_number = 160);
    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Failed
    );

    // A fresh draft proposal can be cancelled by the proposer.
    let prop2 = gov_client.create_proposal(
        &proposer,
        &String::from_str(&env, "Draft to cancel"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    gov_client.cancel_proposal(&proposer, &prop2);
    assert_eq!(
        gov_client.get_proposal_state(&prop2),
        proposal_lifecycle::ProposalState::Cancelled
    );
}

// ===========================================================================
// Category 2 – Voting (4 tests)
// ===========================================================================

// 2-1  A voter casts a yes vote and the weight is recorded correctly.
#[test]
fn test_gov_vote_successful() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &50i128);

    let proposal_id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Vote test"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    client.vote(&voter, &proposal_id, &true, &75i128);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.votes_yes, 75);
    assert_eq!(proposal.votes_no, 0);
}

// 2-2  Voting twice with the same address is rejected.
#[test]
fn test_gov_vote_duplicate_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &10i128);

    let proposal_id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Duplicate vote test"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);
    client.vote(&voter, &proposal_id, &true, &20i128);

    let result = client.try_vote(&voter, &proposal_id, &false, &20i128);
    assert_eq!(
        result,
        Err(Ok(proposal_lifecycle::ProposalError::AlreadyVoted))
    );
}

// 2-3  Voting after the deadline is rejected.
#[test]
fn test_gov_vote_deadline_enforcement() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &10i128);

    let proposal_id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Deadline test"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);
    // voting_end_ledger = 150; advance past it.
    env.ledger().with_mut(|l| l.sequence_number = 155);

    let result = client.try_vote(&voter, &proposal_id, &true, &20i128);
    assert_eq!(
        result,
        Err(Ok(proposal_lifecycle::ProposalError::InvalidState))
    );
}

// 2-4  A proposal that ends below quorum resolves to Failed.
#[test]
fn test_gov_vote_quorum_not_met() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &200i128);

    let proposal_id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Quorum miss test"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);
    client.vote(&voter, &proposal_id, &true, &50i128); // 50 < quorum 200

    env.ledger().with_mut(|l| l.sequence_number = 160);
    assert_eq!(
        client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Failed
    );
}

// ===========================================================================
// Category 3 – DAO Treasury (3 tests)
// ===========================================================================

// 3-1  Depositing increases the treasury balance.
#[test]
fn test_dao_treasury_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);

    let dao_id = env.register_contract(None, dao_treasury::DaoContract);
    let client = dao_treasury::DaoContractClient::new(&env, &dao_id);
    client.initialize(&admin, &10i128, &50u32, &100u32);

    assert_eq!(client.treasury_balance(), 0);
    client.deposit(&depositor, &500i128);
    assert_eq!(client.treasury_balance(), 500);
    client.deposit(&depositor, &250i128);
    assert_eq!(client.treasury_balance(), 750);
}

// 3-2  A Transfer proposal, once passed and executed, moves funds from the treasury.
#[test]
fn test_dao_treasury_withdrawal_via_governance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let recipient = Address::generate(&env);

    let dao_id = env.register_contract(None, dao_treasury::DaoContract);
    let client = dao_treasury::DaoContractClient::new(&env, &dao_id);
    client.initialize(&admin, &50i128, &50u32, &100u32);
    client.deposit(&depositor, &1000i128);

    let proposal_id = client.propose_transfer(&proposer, &recipient, &400i128);
    client.vote(&voter, &proposal_id, &true, &60i128);

    env.ledger().with_mut(|l| l.sequence_number = 60);
    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.state, dao_treasury::ProposalState::Passed);

    client.execute(&voter, &proposal_id);
    assert_eq!(client.treasury_balance(), 600);
    assert_eq!(
        client.get_proposal(&proposal_id).state,
        dao_treasury::ProposalState::Executed
    );
}

// 3-3  Executing a Transfer whose amount exceeds the treasury balance fails.
#[test]
fn test_dao_treasury_over_withdrawal_guard() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let recipient = Address::generate(&env);

    let dao_id = env.register_contract(None, dao_treasury::DaoContract);
    let client = dao_treasury::DaoContractClient::new(&env, &dao_id);
    client.initialize(&admin, &10i128, &50u32, &100u32);
    client.deposit(&depositor, &100i128);

    // Propose more than what is in the treasury.
    let proposal_id = client.propose_transfer(&proposer, &recipient, &500i128);
    client.vote(&voter, &proposal_id, &true, &20i128);

    env.ledger().with_mut(|l| l.sequence_number = 60);
    let result = client.try_execute(&voter, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(dao_treasury::DaoError::InsufficientTreasuryBalance))
    );
}

// ===========================================================================
// Category 4 – Authorization (2 tests)
// ===========================================================================

// 4-1  A random address cannot cancel another proposer's active proposal.
#[test]
fn test_gov_auth_invalid_proposer_cancel() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let attacker = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &10i128);

    let proposal_id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Auth cancel test"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    let result = client.try_cancel_proposal(&attacker, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(proposal_lifecycle::ProposalError::Unauthorized))
    );
}

// 4-2  execute_proposal while voting is still active returns VotingNotEnded.
#[test]
fn test_gov_auth_invalid_executor_while_active() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &10i128);

    let proposal_id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Early execute test"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);
    client.vote(&voter, &proposal_id, &true, &20i128);

    // Still at ledger 100 – voting window is open.
    let result = client.try_execute_proposal(&voter, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(proposal_lifecycle::ProposalError::VotingNotEnded))
    );
}

// ===========================================================================
// Category 5 – Multiple / Concurrent Proposals (4 tests)
// ===========================================================================

// 5-1  Multiple proposals receive independent sequential IDs.
#[test]
fn test_gov_multiple_proposals_independent_ids() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &10i128);

    let sym = Symbol::new(&env, "noop");
    let id0 = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Prop 0"),
        &dummy,
        &sym,
        &Vec::new(&env),
    );
    let id1 = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Prop 1"),
        &dummy,
        &sym,
        &Vec::new(&env),
    );
    let id2 = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Prop 2"),
        &dummy,
        &sym,
        &Vec::new(&env),
    );

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(
        client.get_proposal(&id0).description,
        String::from_str(&env, "Prop 0")
    );
    assert_eq!(
        client.get_proposal(&id1).description,
        String::from_str(&env, "Prop 1")
    );
    assert_eq!(
        client.get_proposal(&id2).description,
        String::from_str(&env, "Prop 2")
    );
}

// 5-2  Two concurrent proposals can have different outcomes simultaneously.
#[test]
fn test_gov_concurrent_proposals_different_outcomes() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter_a = Address::generate(&env);
    let voter_b = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &50i128);

    let sym = Symbol::new(&env, "noop");
    let desc = String::from_str(&env, "Concurrent");
    let id_pass = client.create_proposal(&proposer, &desc, &dummy, &sym, &Vec::new(&env));
    let id_fail = client.create_proposal(&proposer, &desc, &dummy, &sym, &Vec::new(&env));

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &id_pass, &50u32, &100u32);
    client.submit_proposal(&proposer, &id_fail, &50u32, &100u32);

    client.vote(&voter_a, &id_pass, &true, &60i128); // 60 ≥ quorum 50 → Pass
    client.vote(&voter_b, &id_fail, &true, &20i128); // 20 < quorum 50 → Fail

    env.ledger().with_mut(|l| l.sequence_number = 160);
    assert_eq!(
        client.get_proposal_state(&id_pass),
        proposal_lifecycle::ProposalState::Passed
    );
    assert_eq!(
        client.get_proposal_state(&id_fail),
        proposal_lifecycle::ProposalState::Failed
    );
}

// 5-3  Three concurrent DAO Transfer proposals track votes independently.
#[test]
fn test_gov_concurrent_dao_proposals_independent_votes() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let depositor = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let voter3 = Address::generate(&env);
    let recipient = Address::generate(&env);

    let dao_id = env.register_contract(None, dao_treasury::DaoContract);
    let client = dao_treasury::DaoContractClient::new(&env, &dao_id);
    client.initialize(&admin, &10i128, &50u32, &100u32);
    client.deposit(&depositor, &3000i128);

    let pid0 = client.propose_transfer(&proposer, &recipient, &100i128);
    let pid1 = client.propose_transfer(&proposer, &recipient, &200i128);
    let pid2 = client.propose_transfer(&proposer, &recipient, &300i128);

    client.vote(&voter1, &pid0, &true, &20i128);
    client.vote(&voter2, &pid1, &false, &20i128);
    client.vote(&voter3, &pid2, &true, &20i128);

    let p0 = client.get_proposal(&pid0);
    let p1 = client.get_proposal(&pid1);
    let p2 = client.get_proposal(&pid2);

    assert_eq!((p0.votes_yes, p0.votes_no), (20, 0));
    assert_eq!((p1.votes_yes, p1.votes_no), (0, 20));
    assert_eq!((p2.votes_yes, p2.votes_no), (20, 0));
}

// 5-4  Voting on one proposal does not affect any other proposal.
#[test]
fn test_gov_vote_isolation_between_proposals() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let dummy = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    client.initialize(&admin, &10i128);

    let sym = Symbol::new(&env, "noop");
    let desc = String::from_str(&env, "Isolation test");
    let id_a = client.create_proposal(&proposer, &desc, &dummy, &sym, &Vec::new(&env));
    let id_b = client.create_proposal(&proposer, &desc, &dummy, &sym, &Vec::new(&env));

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &id_a, &50u32, &100u32);
    client.submit_proposal(&proposer, &id_b, &50u32, &100u32);

    client.vote(&voter, &id_a, &true, &30i128);

    assert_eq!(client.get_proposal(&id_a).votes_yes, 30);
    assert_eq!(client.get_proposal(&id_b).votes_yes, 0);
}

// ===========================================================================
// Category 6 – Delegation (4 tests)
// ===========================================================================

// 6-1  A delegator can create a delegation and the weight is tracked.
#[test]
fn test_gov_delegation_create() {
    let env = Env::default();
    env.mock_all_auths();

    let delegator = Address::generate(&env);
    let delegatee = Address::generate(&env);

    let del_id = env.register_contract(None, DelegationContract);
    let client = DelegationContractClient::new(&env, &del_id);

    client.delegate(&delegator, &delegatee, &100i128);

    assert_eq!(client.voting_weight(&delegatee), 100);
    assert_eq!(client.get_delegatee(&delegator), Some(delegatee));
}

// 6-2  Removing a delegation zeroes the delegatee's weight.
#[test]
fn test_gov_delegation_remove() {
    let env = Env::default();
    env.mock_all_auths();

    let delegator = Address::generate(&env);
    let delegatee = Address::generate(&env);

    let del_id = env.register_contract(None, DelegationContract);
    let client = DelegationContractClient::new(&env, &del_id);

    client.delegate(&delegator, &delegatee, &80i128);
    assert_eq!(client.voting_weight(&delegatee), 80);

    client.undelegate(&delegator, &80i128);
    assert_eq!(client.voting_weight(&delegatee), 0);
    assert_eq!(client.get_delegatee(&delegator), None);
}

// 6-3  A delegatee uses their accumulated weight to vote on a governance proposal.
#[test]
fn test_gov_delegated_weight_voting() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let delegator1 = Address::generate(&env);
    let delegator2 = Address::generate(&env);
    let delegatee = Address::generate(&env);
    let dummy = Address::generate(&env);

    let del_id = env.register_contract(None, DelegationContract);
    let del_client = DelegationContractClient::new(&env, &del_id);
    del_client.delegate(&delegator1, &delegatee, &40i128);
    del_client.delegate(&delegator2, &delegatee, &35i128);
    let total_weight = del_client.voting_weight(&delegatee);
    assert_eq!(total_weight, 75);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let gov_client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    gov_client.initialize(&admin, &50i128);

    let proposal_id = gov_client.create_proposal(
        &proposer,
        &String::from_str(&env, "Delegated vote proposal"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    env.ledger().with_mut(|l| l.sequence_number = 100);
    gov_client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    gov_client.vote(&delegatee, &proposal_id, &true, &total_weight);
    assert_eq!(gov_client.get_proposal(&proposal_id).votes_yes, 75);

    env.ledger().with_mut(|l| l.sequence_number = 160);
    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Passed
    );
}

// 6-4  Delegating zero weight leaves the delegatee with no voting power.
#[test]
fn test_gov_delegation_zero_weight_edge_case() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let delegator = Address::generate(&env);
    let delegatee = Address::generate(&env);
    let dummy = Address::generate(&env);

    let del_id = env.register_contract(None, DelegationContract);
    let del_client = DelegationContractClient::new(&env, &del_id);
    del_client.delegate(&delegator, &delegatee, &0i128);
    assert_eq!(del_client.voting_weight(&delegatee), 0);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let gov_client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    gov_client.initialize(&admin, &10i128);

    let proposal_id = gov_client.create_proposal(
        &proposer,
        &String::from_str(&env, "Zero weight vote"),
        &dummy,
        &Symbol::new(&env, "noop"),
        &Vec::new(&env),
    );
    env.ledger().with_mut(|l| l.sequence_number = 100);
    gov_client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);
    gov_client.vote(&delegatee, &proposal_id, &true, &0i128); // 0 < quorum 10

    env.ledger().with_mut(|l| l.sequence_number = 160);
    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Failed
    );
}

// ===========================================================================
// Category 7 – Voting-time-constraints (4 tests)
// ===========================================================================

fn setup_vtc(env: &Env, voting_period: u64, grace_period: u64) -> (Address, Address) {
    let id = env.register_contract(None, voting_time_constraints::VotingContract);
    let admin = Address::generate(env);
    voting_time_constraints::VotingContractClient::new(env, &id).initialize(
        &admin,
        &voting_period,
        &grace_period,
    );
    (id, admin)
}

// 7-1  Initialize voting-time-constraints and create a proposal.
#[test]
fn test_vtc_proposal_creation() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let (vtc_id, _admin) = setup_vtc(&env, 300u64, 60u64);
    let client = voting_time_constraints::VotingContractClient::new(&env, &vtc_id);

    let proposal_id = Symbol::new(&env, "prop_vtc_1");
    client.create_proposal(&creator, &proposal_id, &1u32);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(
        proposal.state,
        voting_time_constraints::ProposalState::Active
    );
    assert_eq!(proposal.quorum_threshold, 1);
    assert_eq!(proposal.votes_for, 0);
}

// 7-2  Voting after the deadline is rejected.
#[test]
fn test_vtc_post_deadline_vote_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let voter = Address::generate(&env);
    let (vtc_id, _admin) = setup_vtc(&env, 100u64, 60u64);
    let client = voting_time_constraints::VotingContractClient::new(&env, &vtc_id);

    let proposal_id = Symbol::new(&env, "prop_vtc_2");
    client.create_proposal(&creator, &proposal_id, &1u32);

    env.ledger().with_mut(|l| l.timestamp += 200);

    let result = client.try_vote(&voter, &proposal_id, &true);
    assert_eq!(
        result,
        Err(Ok(voting_time_constraints::VotingError::VotingClosed))
    );
}

// 7-3  Votes cast within the window are counted correctly.
#[test]
fn test_vtc_vote_within_window_accepted() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let voter_a = Address::generate(&env);
    let voter_b = Address::generate(&env);
    let (vtc_id, _admin) = setup_vtc(&env, 3600u64, 60u64);
    let client = voting_time_constraints::VotingContractClient::new(&env, &vtc_id);

    let proposal_id = Symbol::new(&env, "prop_vtc_3");
    client.create_proposal(&creator, &proposal_id, &1u32);

    client.vote(&voter_a, &proposal_id, &true);
    client.vote(&voter_b, &proposal_id, &false);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.votes_for, 1);
    assert_eq!(proposal.votes_against, 1);
}

// 7-4  Full lifecycle: create → vote → finalize (GracePeriod) → finalize (Executable) → execute.
#[test]
fn test_vtc_full_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let voter = Address::generate(&env);
    let (vtc_id, admin) = setup_vtc(&env, 200u64, 50u64);
    let client = voting_time_constraints::VotingContractClient::new(&env, &vtc_id);

    let proposal_id = Symbol::new(&env, "prop_vtc_4");
    client.create_proposal(&creator, &proposal_id, &1u32);
    client.vote(&voter, &proposal_id, &true);

    // Advance past the voting deadline.
    env.ledger().with_mut(|l| l.timestamp += 210);
    let state = client.finalize(&proposal_id);
    assert_eq!(state, voting_time_constraints::ProposalState::GracePeriod);

    // Advance past the grace period.
    env.ledger().with_mut(|l| l.timestamp += 60);
    let state = client.finalize(&proposal_id);
    assert_eq!(state, voting_time_constraints::ProposalState::Executable);

    client.execute(&admin, &proposal_id);
    assert_eq!(
        client.get_proposal(&proposal_id).state,
        voting_time_constraints::ProposalState::Executed
    );
}

// ===========================================================================
// Category 8 – End-to-end (4 tests)
// ===========================================================================

// 8-1  Cross-contract governance action: governance invokes MockTargetContract.
#[test]
fn test_e2e_cross_contract_governance_action() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let gov_client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    gov_client.initialize(&admin, &100i128);

    let target_id = env.register_contract(None, MockTargetContract);

    let proposal_id = gov_client.create_proposal(
        &proposer,
        &String::from_str(&env, "Cross-contract: set value to 99"),
        &target_id,
        &Symbol::new(&env, "set_value"),
        &Vec::from_array(&env, [99u32.into_val(&env)]),
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    gov_client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);
    gov_client.vote(&voter1, &proposal_id, &true, &70i128);
    gov_client.vote(&voter2, &proposal_id, &true, &50i128);

    env.ledger().with_mut(|l| l.sequence_number = 160);
    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Passed
    );

    gov_client.execute_proposal(&voter1, &proposal_id);
    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Executed
    );

    let val = env.as_contract(&target_id, || {
        env.storage()
            .instance()
            .get::<_, u32>(&MockDataKey::Value)
            .unwrap()
    });
    assert_eq!(val, 99);
}

// 8-2  Full community vote workflow: five voters with different weights.
#[test]
fn test_e2e_full_community_vote_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voters: [(Address, i128); 5] = [
        (Address::generate(&env), 20),
        (Address::generate(&env), 30),
        (Address::generate(&env), 10),
        (Address::generate(&env), 50),
        (Address::generate(&env), 15),
    ];

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let gov_client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    gov_client.initialize(&admin, &100i128);

    let target_id = env.register_contract(None, MockTargetContract);

    let proposal_id = gov_client.create_proposal(
        &proposer,
        &String::from_str(&env, "Community upgrade proposal"),
        &target_id,
        &Symbol::new(&env, "set_value"),
        &Vec::from_array(&env, [7u32.into_val(&env)]),
    );

    env.ledger().with_mut(|l| l.sequence_number = 200);
    gov_client.submit_proposal(&proposer, &proposal_id, &100u32, &200u32);

    let mut total_yes: i128 = 0;
    for (voter, weight) in &voters {
        gov_client.vote(voter, &proposal_id, &true, weight);
        total_yes += weight;
    }
    assert_eq!(total_yes, 125); // 125 ≥ quorum 100

    env.ledger().with_mut(|l| l.sequence_number = 310);
    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Passed
    );

    gov_client.execute_proposal(&voters[0].0, &proposal_id);

    let stored_val = env.as_contract(&target_id, || {
        env.storage()
            .instance()
            .get::<_, u32>(&MockDataKey::Value)
            .unwrap()
    });
    assert_eq!(stored_val, 7);
}

// 8-3  DAO treasury end-to-end: deposit → propose → vote → execute → balance verified.
#[test]
fn test_e2e_dao_treasury_full_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let recipient = Address::generate(&env);

    let dao_id = env.register_contract(None, dao_treasury::DaoContract);
    let client = dao_treasury::DaoContractClient::new(&env, &dao_id);
    client.initialize(&admin, &30i128, &40u32, &80u32);

    client.deposit(&depositor, &2000i128);
    assert_eq!(client.treasury_balance(), 2000);

    let proposal_id = client.propose_transfer(&proposer, &recipient, &800i128);
    assert_eq!(client.proposal_count(), 1);

    client.vote(&voter, &proposal_id, &true, &50i128); // 50 ≥ quorum 30

    env.ledger().with_mut(|l| l.sequence_number = 50);
    assert_eq!(
        client.get_proposal(&proposal_id).state,
        dao_treasury::ProposalState::Passed
    );

    client.execute(&voter, &proposal_id);
    assert_eq!(client.treasury_balance(), 1200);
    assert_eq!(
        client.get_proposal(&proposal_id).state,
        dao_treasury::ProposalState::Executed
    );
}

// 8-4  Simple voting end-to-end: admin creates proposal, users vote, result tallied.
#[test]
fn test_e2e_simple_voting_full_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let voter_a = Address::generate(&env);
    let voter_b = Address::generate(&env);
    let voter_c = Address::generate(&env);

    let sv_id = env.register_contract(None, simple_voting::VotingContract);
    let client = simple_voting::VotingContractClient::new(&env, &sv_id);
    client.initialize(&admin);

    let deadline: u64 = env.ledger().timestamp() + 1_000;
    let proposal_id = client.create_prop(
        &admin,
        &String::from_str(&env, "Adopt new fee model"),
        &deadline,
    );

    client.cast_vote(&voter_a, &proposal_id, &simple_voting::VoteChoice::For);
    client.cast_vote(&voter_b, &proposal_id, &simple_voting::VoteChoice::For);
    client.cast_vote(&voter_c, &proposal_id, &simple_voting::VoteChoice::Against);

    let (votes_for, votes_against, votes_abstain) = client.tally(&proposal_id);
    assert_eq!(votes_for, 2);
    assert_eq!(votes_against, 1);
    assert_eq!(votes_abstain, 0);

    env.ledger().with_mut(|l| l.timestamp += 1_100);
    let status = client.execute(&proposal_id);
    assert_eq!(status, simple_voting::ProposalStatus::Passed);
    assert_eq!(
        client.get_prop(&proposal_id).status,
        simple_voting::ProposalStatus::Passed
    );
}

#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env, IntoVal, String, Symbol, Vec,
};

#[contract]
pub struct DummyTargetContract;

#[contractimpl]
impl DummyTargetContract {
    pub fn execute_action(env: Env, value: u32) {
        env.storage()
            .instance()
            .set(&symbol_short!("executed"), &value);
    }
}

fn setup_test_env(
    env: &Env,
) -> (
    Address,
    ProposalLifecycleContractClient<'_>,
    Address,
    DummyTargetContractClient<'_>,
) {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let proposer = Address::generate(env);

    let contract_id = env.register_contract(None, ProposalLifecycleContract);
    let client = ProposalLifecycleContractClient::new(env, &contract_id);

    let dummy_id = env.register_contract(None, DummyTargetContract);
    let dummy_client = DummyTargetContractClient::new(env, &dummy_id);

    (admin, client, proposer, dummy_client)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let (admin, client, _, _) = setup_test_env(&env);

    client.initialize(&admin, &100i128);

    // Check that we cannot initialize again
    let res = client.try_initialize(&admin, &100i128);
    assert!(res.is_err());
}

#[test]
fn test_create_proposal() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);

    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    assert_eq!(proposal_id, 0);

    let prop = client.get_proposal(&0);
    assert_eq!(prop.proposer, proposer);
    assert_eq!(prop.state, ProposalState::Draft);
    assert_eq!(prop.votes_yes, 0);
    assert_eq!(prop.votes_no, 0);
}

#[test]
fn test_submit_proposal_success() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    let prop = client.get_proposal(&proposal_id);
    assert_eq!(prop.state, ProposalState::Active);
    assert_eq!(prop.voting_end_ledger, 150);
    assert_eq!(prop.execution_end_ledger, 250);
}

#[test]
fn test_submit_proposal_unauthorized() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    let hacker = Address::generate(&env);
    let res = client.try_submit_proposal(&hacker, &proposal_id, &50u32, &100u32);
    assert_eq!(res, Err(Ok(ProposalError::Unauthorized)));
}

#[test]
fn test_submit_proposal_invalid_state() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    // Submit again should fail
    let res = client.try_submit_proposal(&proposer, &proposal_id, &50u32, &100u32);
    assert_eq!(res, Err(Ok(ProposalError::InvalidState)));
}

#[test]
fn test_vote_success() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    client.vote(&voter1, &proposal_id, &true, &60i128);
    client.vote(&voter2, &proposal_id, &false, &30i128);

    let prop = client.get_proposal(&proposal_id);
    assert_eq!(prop.votes_yes, 60);
    assert_eq!(prop.votes_no, 30);
}

#[test]
fn test_vote_invalid_state() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    let voter = Address::generate(&env);
    // Proposal is in Draft state, cannot vote
    let res = client.try_vote(&voter, &proposal_id, &true, &50i128);
    assert_eq!(res, Err(Ok(ProposalError::InvalidState)));
}

#[test]
fn test_vote_already_voted() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    let voter = Address::generate(&env);
    client.vote(&voter, &proposal_id, &true, &50i128);

    // Vote again should fail
    let res = client.try_vote(&voter, &proposal_id, &true, &50i128);
    assert_eq!(res, Err(Ok(ProposalError::AlreadyVoted)));
}

#[test]
fn test_vote_ended() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32); // end = 150

    env.ledger().with_mut(|l| l.sequence_number = 151);

    let voter = Address::generate(&env);
    let res = client.try_vote(&voter, &proposal_id, &true, &50i128);
    assert_eq!(res, Err(Ok(ProposalError::InvalidState))); // resolves to Failed/Passed since voting ended
}

#[test]
fn test_execute_success() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128); // Quorum = 100

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [888u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32); // vote end = 150, exec end = 250

    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    client.vote(&voter1, &proposal_id, &true, &80i128);
    client.vote(&voter2, &proposal_id, &true, &30i128); // total = 110 (quorum met)

    env.ledger().with_mut(|l| l.sequence_number = 155);

    assert_eq!(
        client.get_proposal_state(&proposal_id),
        ProposalState::Passed
    );

    client.execute_proposal(&voter1, &proposal_id);

    assert_eq!(
        client.get_proposal_state(&proposal_id),
        ProposalState::Executed
    );

    // Check dummy target contract was executed
    let is_executed: u32 = env.as_contract(&dummy.address, || {
        env.storage()
            .instance()
            .get(&symbol_short!("executed"))
            .unwrap()
    });
    assert_eq!(is_executed, 888);
}

#[test]
fn test_execute_quorum_not_met() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128); // Quorum = 100

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    let voter1 = Address::generate(&env);
    client.vote(&voter1, &proposal_id, &true, &50i128); // 50 < 100

    env.ledger().with_mut(|l| l.sequence_number = 155);

    assert_eq!(
        client.get_proposal_state(&proposal_id),
        ProposalState::Failed
    );

    let res = client.try_execute_proposal(&voter1, &proposal_id);
    assert_eq!(res, Err(Ok(ProposalError::InvalidState)));
}

#[test]
fn test_execute_failed_proposal() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128); // Quorum = 100

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32);

    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    client.vote(&voter1, &proposal_id, &true, &40i128);
    client.vote(&voter2, &proposal_id, &false, &70i128); // total = 110 (quorum met, but No > Yes)

    env.ledger().with_mut(|l| l.sequence_number = 155);

    assert_eq!(
        client.get_proposal_state(&proposal_id),
        ProposalState::Failed
    );

    let res = client.try_execute_proposal(&voter1, &proposal_id);
    assert_eq!(res, Err(Ok(ProposalError::InvalidState)));
}

#[test]
fn test_execute_expired() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32); // vote end = 150, exec end = 250

    let voter1 = Address::generate(&env);
    client.vote(&voter1, &proposal_id, &true, &120i128);

    env.ledger().with_mut(|l| l.sequence_number = 251); // Expired

    assert_eq!(
        client.get_proposal_state(&proposal_id),
        ProposalState::Expired
    );

    let res = client.try_execute_proposal(&voter1, &proposal_id);
    assert_eq!(res, Err(Ok(ProposalError::ExecutionEnded)));
}

#[test]
fn test_cancel_by_proposer() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    client.cancel_proposal(&proposer, &proposal_id);

    assert_eq!(
        client.get_proposal_state(&proposal_id),
        ProposalState::Cancelled
    );
}

#[test]
fn test_cancel_by_admin() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    client.cancel_proposal(&admin, &proposal_id);

    assert_eq!(
        client.get_proposal_state(&proposal_id),
        ProposalState::Cancelled
    );
}

#[test]
fn test_cancel_unauthorized() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup_test_env(&env);
    client.initialize(&admin, &100i128);

    let description = String::from_str(&env, "Test Proposal");
    let action_args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(
        &proposer,
        &description,
        &dummy.address,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    let hacker = Address::generate(&env);
    let res = client.try_cancel_proposal(&hacker, &proposal_id);
    assert_eq!(res, Err(Ok(ProposalError::Unauthorized)));
}

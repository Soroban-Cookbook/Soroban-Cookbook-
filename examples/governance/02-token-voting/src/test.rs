#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::{Address as _, Ledger}, Address, Env, IntoVal, String, Symbol, Vec};

#[contract]
pub struct DummyTargetContract;

#[contractimpl]
impl DummyTargetContract {
    pub fn execute_action(env: Env, value: u32) {
        env.storage().instance().set(&symbol_short!("executed"), &value);
    }
}

fn setup(env: &Env) -> (Address, TokenVotingContractClient<'_>, Address, DummyTargetContractClient<'_>) {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let user = Address::generate(env);
    let contract_id = env.register_contract(None, TokenVotingContract);
    let client = TokenVotingContractClient::new(env, &contract_id);

    let target_id = env.register_contract(None, DummyTargetContract);
    let target_client = DummyTargetContractClient::new(env, &target_id);

    (admin, client, user, target_client)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let (admin, client, _, _) = setup(&env);

    client.initialize(&admin, &100i128);
    let res = client.try_initialize(&admin, &100i128);
    assert!(res.is_err());
}

#[test]
fn test_mint_and_balance_snapshot() {
    let env = Env::default();
    let (admin, client, user, _) = setup(&env);

    client.initialize(&admin, &100i128);
    client.mint(&admin, &user, &150i128);

    assert_eq!(client.get_balance(&user), 150);
    assert_eq!(client.get_balance_at(&user, &env.ledger().sequence()), 150);
}

#[test]
fn test_transfer_updates_history() {
    let env = Env::default();
    let (admin, client, alice, _) = setup(&env);

    let bob = Address::generate(&env);
    client.initialize(&admin, &100i128);
    client.mint(&admin, &alice, &120i128);

    env.ledger().with_mut(|l| l.sequence_number = 101);
    client.transfer(&alice, &bob, &50i128);

    assert_eq!(client.get_balance(&alice), 70);
    assert_eq!(client.get_balance_at(&alice, &100u32), 120);
    assert_eq!(client.get_balance_at(&alice, &101u32), 70);
}

#[test]
fn test_delegate_and_delegate_at() {
    let env = Env::default();
    let (admin, client, delegator, _) = setup(&env);

    let delegatee = Address::generate(&env);
    client.initialize(&admin, &100i128);
    client.mint(&admin, &delegator, &80i128);

    env.ledger().with_mut(|l| l.sequence_number = 200);
    client.delegate(&delegator, &delegatee).unwrap();

    assert_eq!(client.get_delegate(&delegator), Some(delegatee.clone()));
    assert_eq!(client.get_delegate_at(&delegator, &200u32), Some(delegatee));
}

#[test]
fn test_proposal_snapshot_locks_voting_power() {
    let env = Env::default();
    let (admin, client, alice, dummy) = setup(&env);
    let bob = Address::generate(&env);

    client.initialize(&admin, &100i128);
    client.mint(&admin, &alice, &100i128);

    let desc = String::from_str(&env, "Snapshot Proposal");
    let args = Vec::from_array(&env, [42u32.into_val(&env)]);
    let proposal_id = client.create_proposal(&alice, &desc, &dummy.address, &Symbol::new(&env, "execute_action"), &args);

    env.ledger().with_mut(|l| l.sequence_number = 50);
    client.submit_proposal(&alice, &proposal_id, &20u32, &50u32);

    env.ledger().with_mut(|l| l.sequence_number = 51);
    client.transfer(&alice, &bob, &30i128).unwrap();

    client.vote(&alice, &proposal_id, &true);
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.votes_yes, 100);
}

#[test]
fn test_vote_for_delegated_balance() {
    let env = Env::default();
    let (admin, client, delegator, dummy) = setup(&env);
    let delegatee = Address::generate(&env);

    client.initialize(&admin, &100i128);
    client.mint(&admin, &delegator, &120i128);

    client.delegate(&delegator, &delegatee).unwrap();
    let desc = String::from_str(&env, "Delegate Proposal");
    let args = Vec::from_array(&env, [11u32.into_val(&env)]);
    let proposal_id = client.create_proposal(&delegatee, &desc, &dummy.address, &Symbol::new(&env, "execute_action"), &args);

    env.ledger().with_mut(|l| l.sequence_number = 500);
    client.submit_proposal(&delegatee, &proposal_id, &10u32, &20u32);
    client.vote_for(&delegatee, &proposal_id, &delegator, &true).unwrap();

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.votes_yes, 120);
}

#[test]
fn test_double_vote_fails() {
    let env = Env::default();
    let (admin, client, voter, dummy) = setup(&env);

    client.initialize(&admin, &100i128);
    client.mint(&admin, &voter, &70i128);

    let desc = String::from_str(&env, "Double Vote");
    let args = Vec::from_array(&env, [99u32.into_val(&env)]);
    let proposal_id = client.create_proposal(&voter, &desc, &dummy.address, &Symbol::new(&env, "execute_action"), &args);

    env.ledger().with_mut(|l| l.sequence_number = 600);
    client.submit_proposal(&voter, &proposal_id, &10u32, &20u32);

    client.vote(&voter, &proposal_id, &true).unwrap();
    let result = client.try_vote(&voter, &proposal_id, &true);
    assert_eq!(result, Err(Ok(VotingError::AlreadyVoted)));
}

#[test]
fn test_quorum_not_met() {
    let env = Env::default();
    let (admin, client, voter, dummy) = setup(&env);

    client.initialize(&admin, &100i128);
    client.mint(&admin, &voter, &40i128);

    let desc = String::from_str(&env, "Quorum Fail");
    let args = Vec::from_array(&env, [7u32.into_val(&env)]);
    let proposal_id = client.create_proposal(&voter, &desc, &dummy.address, &Symbol::new(&env, "execute_action"), &args);

    env.ledger().with_mut(|l| l.sequence_number = 700);
    client.submit_proposal(&voter, &proposal_id, &10u32, &20u32);
    client.vote(&voter, &proposal_id, &true).unwrap();

    env.ledger().with_mut(|l| l.sequence_number = 711);
    assert_eq!(client.get_proposal_state(&proposal_id).unwrap(), ProposalState::Failed);
}

#[test]
fn test_execute_proposal_success() {
    let env = Env::default();
    let (admin, client, voter, dummy) = setup(&env);

    client.initialize(&admin, &100i128);
    client.mint(&admin, &voter, &120i128);

    let desc = String::from_str(&env, "Execute Proposal");
    let args = Vec::from_array(&env, [123u32.into_val(&env)]);
    let proposal_id = client.create_proposal(&voter, &desc, &dummy.address, &Symbol::new(&env, "execute_action"), &args);

    env.ledger().with_mut(|l| l.sequence_number = 800);
    client.submit_proposal(&voter, &proposal_id, &10u32, &20u32);
    client.vote(&voter, &proposal_id, &true).unwrap();

    env.ledger().with_mut(|l| l.sequence_number = 811);
    assert_eq!(client.get_proposal_state(&proposal_id).unwrap(), ProposalState::Passed);
    client.execute_proposal(&voter, &proposal_id).unwrap();
    assert_eq!(client.get_proposal_state(&proposal_id).unwrap(), ProposalState::Executed);

    let executed: u32 = env.as_contract(&dummy.address, || {
        env.storage().instance().get(&symbol_short!("executed")).unwrap()
    });
    assert_eq!(executed, 123);
}

#[test]
fn test_execute_expired() {
    let env = Env::default();
    let (admin, client, voter, dummy) = setup(&env);

    client.initialize(&admin, &100i128);
    client.mint(&admin, &voter, &130i128);

    let desc = String::from_str(&env, "Expire Proposal");
    let args = Vec::from_array(&env, [321u32.into_val(&env)]);
    let proposal_id = client.create_proposal(&voter, &desc, &dummy.address, &Symbol::new(&env, "execute_action"), &args);

    env.ledger().with_mut(|l| l.sequence_number = 900);
    client.submit_proposal(&voter, &proposal_id, &10u32, &15u32);
    client.vote(&voter, &proposal_id, &true).unwrap();

    env.ledger().with_mut(|l| l.sequence_number = 926);
    let result = client.try_execute_proposal(&voter, &proposal_id);
    assert_eq!(result, Err(Ok(VotingError::ExecutionEnded)));
}

#[test]
fn test_cancel_proposal() {
    let env = Env::default();
    let (admin, client, proposer, dummy) = setup(&env);

    client.initialize(&admin, &100i128);

    let desc = String::from_str(&env, "Cancel Proposal");
    let args = Vec::from_array(&env, [111u32.into_val(&env)]);
    let proposal_id = client.create_proposal(&proposer, &desc, &dummy.address, &Symbol::new(&env, "execute_action"), &args);

    client.cancel_proposal(&proposer, &proposal_id).unwrap();
    assert_eq!(client.get_proposal_state(&proposal_id).unwrap(), ProposalState::Cancelled);
}

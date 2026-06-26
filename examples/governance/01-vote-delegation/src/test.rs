//! Unit tests for the Vote Delegation contract.

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register_contract(None, VoteDelegationContract);
    let client = VoteDelegationContractClient::new(&env, &contract_id);

    client.init(&admin);

    // Initializing again should fail
    let second_admin = Address::generate(&env);
    let res = client.try_init(&second_admin);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        DelegationError::AlreadyInitialized
    );
}

#[test]
fn test_set_balance() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_id = env.register_contract(None, VoteDelegationContract);
    let client = VoteDelegationContractClient::new(&env, &contract_id);

    client.init(&admin);

    env.mock_all_auths();
    client.set_balance(&admin, &user, &100);

    assert_eq!(client.get_balance(&user), 100);
    assert_eq!(client.get_voting_power(&user), 100);
}

#[test]
fn test_basic_delegation() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let contract_id = env.register_contract(None, VoteDelegationContract);
    let client = VoteDelegationContractClient::new(&env, &contract_id);

    client.init(&admin);

    env.mock_all_auths();
    client.set_balance(&admin, &alice, &100);
    client.set_balance(&admin, &bob, &50);

    // Alice delegates to Bob
    client.delegate(&alice, &bob);

    assert_eq!(client.get_delegate(&alice), Some(bob.clone()));
    // Alice's active power goes to 0
    assert_eq!(client.get_voting_power(&alice), 0);
    // Bob's active power becomes Bob's balance (50) + Alice's balance (100) = 150
    assert_eq!(client.get_voting_power(&bob), 150);
}

#[test]
fn test_chain_delegation() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    let contract_id = env.register_contract(None, VoteDelegationContract);
    let client = VoteDelegationContractClient::new(&env, &contract_id);

    client.init(&admin);

    env.mock_all_auths();
    client.set_balance(&admin, &alice, &100);
    client.set_balance(&admin, &bob, &50);
    client.set_balance(&admin, &charlie, &30);

    // Alice delegates to Bob
    client.delegate(&alice, &bob);
    // Bob delegates to Charlie
    client.delegate(&bob, &charlie);

    assert_eq!(client.get_voting_power(&alice), 0);
    assert_eq!(client.get_voting_power(&bob), 0);
    // Charlie has Charlie (30) + Bob (50) + Alice (100) = 180
    assert_eq!(client.get_voting_power(&charlie), 180);
}

#[test]
fn test_cycle_detection_direct() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let contract_id = env.register_contract(None, VoteDelegationContract);
    let client = VoteDelegationContractClient::new(&env, &contract_id);

    client.init(&admin);

    env.mock_all_auths();
    client.set_balance(&admin, &alice, &100);
    client.set_balance(&admin, &bob, &50);

    // Alice delegates to Bob
    client.delegate(&alice, &bob);

    // Bob trying to delegate to Alice should cause a DelegationCycle error
    let res = client.try_delegate(&bob, &alice);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        DelegationError::DelegationCycle
    );
}

#[test]
fn test_cycle_detection_indirect() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    let contract_id = env.register_contract(None, VoteDelegationContract);
    let client = VoteDelegationContractClient::new(&env, &contract_id);

    client.init(&admin);

    env.mock_all_auths();
    client.set_balance(&admin, &alice, &100);
    client.set_balance(&admin, &bob, &50);
    client.set_balance(&admin, &charlie, &30);

    // Alice -> Bob -> Charlie
    client.delegate(&alice, &bob);
    client.delegate(&bob, &charlie);

    // Charlie trying to delegate to Alice should fail
    let res = client.try_delegate(&charlie, &alice);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        DelegationError::DelegationCycle
    );
}

#[test]
fn test_self_delegation() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);

    let contract_id = env.register_contract(None, VoteDelegationContract);
    let client = VoteDelegationContractClient::new(&env, &contract_id);

    client.init(&admin);

    env.mock_all_auths();
    client.set_balance(&admin, &alice, &100);

    // Alice delegating to herself should fail
    let res = client.try_delegate(&alice, &alice);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        DelegationError::SelfDelegation
    );
}

#[test]
fn test_undelegate() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let contract_id = env.register_contract(None, VoteDelegationContract);
    let client = VoteDelegationContractClient::new(&env, &contract_id);

    client.init(&admin);

    env.mock_all_auths();
    client.set_balance(&admin, &alice, &100);
    client.set_balance(&admin, &bob, &50);

    // Delegate
    client.delegate(&alice, &bob);
    assert_eq!(client.get_voting_power(&bob), 150);

    // Undelegate
    client.undelegate(&alice);
    assert_eq!(client.get_delegate(&alice), None);
    assert_eq!(client.get_voting_power(&alice), 100);
    assert_eq!(client.get_voting_power(&bob), 50);
}

#[test]
fn test_max_depth_exceeded() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register_contract(None, VoteDelegationContract);
    let client = VoteDelegationContractClient::new(&env, &contract_id);

    client.init(&admin);
    env.mock_all_auths();

    // Create 7 users to exceed the MAX_DEPTH of 5
    let u1 = Address::generate(&env);
    let u2 = Address::generate(&env);
    let u3 = Address::generate(&env);
    let u4 = Address::generate(&env);
    let u5 = Address::generate(&env);
    let u6 = Address::generate(&env);
    let u7 = Address::generate(&env);

    client.set_balance(&admin, &u1, &10);
    client.set_balance(&admin, &u2, &10);
    client.set_balance(&admin, &u3, &10);
    client.set_balance(&admin, &u4, &10);
    client.set_balance(&admin, &u5, &10);
    client.set_balance(&admin, &u6, &10);
    client.set_balance(&admin, &u7, &10);

    // Chain delegation up to depth 5: u1 -> u2 -> u3 -> u4 -> u5 -> u6
    client.delegate(&u1, &u2);
    client.delegate(&u2, &u3);
    client.delegate(&u3, &u4);
    client.delegate(&u4, &u5);
    client.delegate(&u5, &u6);

    // Trying to delegate u6 -> u7 should exceed MAX_DEPTH (depth 6)
    let res = client.try_delegate(&u6, &u7);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        DelegationError::MaxDelegationDepthExceeded
    );
}

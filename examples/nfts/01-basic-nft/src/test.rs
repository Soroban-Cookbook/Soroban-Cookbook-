extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String, Vec};

fn setup() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, BasicNftContract);
    let client = BasicNftContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client
        .initialize(
            &admin,
            &String::from_str(&env, "Basic Collection"),
            &String::from_str(&env, "BASIC"),
        )
        .unwrap();

    (env, contract_id, admin)
}

#[test]
fn test_initialize_success() {
    let (env, contract_id, _admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    assert_eq!(client.name().unwrap(), String::from_str(&env, "Basic Collection"));
    assert_eq!(client.symbol().unwrap(), String::from_str(&env, "BASIC"));
    assert_eq!(client.total_supply(), 0);
}

#[test]
fn test_mint_and_query_owner_balance() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    client.mint(&admin, &owner, &1u32).unwrap();

    assert_eq!(client.owner_of(&1u32).unwrap(), owner);
    assert_eq!(client.balance_of(&owner), 1);
    assert_eq!(client.total_supply(), 1);
}

#[test]
fn test_mint_duplicate_fails() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.mint(&admin, &owner, &1u32).unwrap();

    let result = client.mint(&admin, &owner, &1u32);
    assert_eq!(result, Err(NftError::TokenAlreadyExists));
}

#[test]
fn test_transfer_updates_balances_and_owner() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.mint(&admin, &alice, &1u32).unwrap();
    client.transfer(&alice, &bob, &1u32).unwrap();

    assert_eq!(client.owner_of(&1u32).unwrap(), bob);
    assert_eq!(client.balance_of(&alice), 0);
    assert_eq!(client.balance_of(&bob), 1);
}

#[test]
fn test_approve_and_transfer_from() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let spender = Address::generate(&env);

    client.mint(&admin, &alice, &1u32).unwrap();
    client.approve(&alice, &spender, &1u32).unwrap();
    client.transfer_from(&spender, &alice, &bob, &1u32).unwrap();

    assert_eq!(client.owner_of(&1u32).unwrap(), bob);
    assert_eq!(client.balance_of(&alice), 0);
    assert_eq!(client.balance_of(&bob), 1);
}

#[test]
fn test_transfer_from_operator() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let operator = Address::generate(&env);

    client.mint(&admin, &alice, &1u32).unwrap();
    client.set_approval_for_all(&alice, &operator, &true).unwrap();
    client.transfer_from(&operator, &alice, &bob, &1u32).unwrap();

    assert_eq!(client.owner_of(&1u32).unwrap(), bob);
    assert_eq!(client.balance_of(&bob), 1);
}

#[test]
fn test_transfer_from_unapproved_fails() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let attacker = Address::generate(&env);

    client.mint(&admin, &alice, &1u32).unwrap();

    let result = client.transfer_from(&attacker, &alice, &bob, &1u32);
    assert_eq!(result, Err(NftError::NotApproved));
}

#[test]
fn test_token_enumeration_global_and_owner() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.mint(&admin, &alice, &1u32).unwrap();
    client.mint(&admin, &bob, &2u32).unwrap();
    client.mint(&admin, &alice, &3u32).unwrap();

    assert_eq!(client.token_by_index(&0u32).unwrap(), 1u32);
    assert_eq!(client.token_by_index(&1u32).unwrap(), 2u32);
    assert_eq!(client.token_by_index(&2u32).unwrap(), 3u32);

    let alice_tokens = client.tokens_of_owner(&alice);
    assert_eq!(alice_tokens.len(), 2);
    assert!(alice_tokens.contains(&1u32));
    assert!(alice_tokens.contains(&3u32));
}

#[test]
fn test_owner_approval_round_trip() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let spender = Address::generate(&env);

    client.mint(&admin, &alice, &1u32).unwrap();
    client.approve(&alice, &spender, &1u32).unwrap();

    assert_eq!(client.get_approved(&1u32), Some(spender));
}

#[test]
fn test_set_approval_for_all_toggle() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let operator = Address::generate(&env);

    client.mint(&admin, &alice, &1u32).unwrap();
    client.set_approval_for_all(&alice, &operator, &true).unwrap();
    assert!(client.is_approved_for_all(&alice, &operator));

    client.set_approval_for_all(&alice, &operator, &false).unwrap();
    assert!(!client.is_approved_for_all(&alice, &operator));
}

#[test]
fn test_transfer_clears_approval() {
    let (env, contract_id, admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let spender = Address::generate(&env);

    client.mint(&admin, &alice, &1u32).unwrap();
    client.approve(&alice, &spender, &1u32).unwrap();
    client.transfer(&alice, &bob, &1u32).unwrap();

    assert_eq!(client.get_approved(&1u32), None);
}

#[test]
fn test_mint_requires_admin() {
    let (env, contract_id, _admin) = setup();
    let client = BasicNftContractClient::new(&env, &contract_id);

    let attacker = Address::generate(&env);
    let owner = Address::generate(&env);

    let result = client.mint(&attacker, &owner, &1u32);
    assert_eq!(result, Err(NftError::NotAdmin));
}

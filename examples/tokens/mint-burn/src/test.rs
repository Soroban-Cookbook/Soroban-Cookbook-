//! Unit tests for the mint/burn token contract.

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
fn test_initialize_and_mint_burn_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = env.register_contract(None, MintBurnToken);
    let client = MintBurnTokenClient::new(&env, &contract_id);

    client.try_initialize(&admin, &1000).unwrap().unwrap();
    assert_eq!(client.total_supply(), 0);
    assert_eq!(client.balance(&user), 0);

    let minted = client.try_mint(&user, &500).unwrap().unwrap();
    assert_eq!(minted, 500);

    assert_eq!(client.balance(&user), 500);
    assert_eq!(client.total_supply(), 500);
    assert_eq!(client.supply_cap(), Some(1000));

    let burned = client.try_burn(&user, &200).unwrap().unwrap();
    assert_eq!(burned, 300);

    assert_eq!(client.balance(&user), 300);
    assert_eq!(client.total_supply(), 300);
}

#[test]
fn test_mint_respects_supply_cap() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let contract_id = env.register_contract(None, MintBurnToken);
    let client = MintBurnTokenClient::new(&env, &contract_id);

    client.try_initialize(&admin, &250).unwrap().unwrap();
    client.try_mint(&alice, &200).unwrap().unwrap();
    assert_eq!(
        client.try_mint(&alice, &100),
        Err(Ok(TokenError::SupplyCapExceeded))
    );
}

#[test]
fn test_burn_requires_owner_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let contract_id = env.register_contract(None, MintBurnToken);
    let client = MintBurnTokenClient::new(&env, &contract_id);

    env.mock_all_auths();
    client.try_initialize(&admin, &0).unwrap().unwrap();
    client.try_mint(&alice, &100).unwrap().unwrap();
    env.set_auths(&[]);

    assert!(client.try_burn(&alice, &50).is_err());
    let _ = bob;
}

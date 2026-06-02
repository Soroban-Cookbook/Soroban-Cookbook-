//! Unit tests for the mint/burn token contract.

use super::*;
use soroban_sdk::{symbol_short, Address, Env};

#[test]
fn test_initialize_and_mint_burn_flow() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = env.register_contract(None, MintBurnToken);
    let client = MintBurnTokenClient::new(&env, &contract_id);

    client.initialize(&admin, &1000).unwrap();
    assert_eq!(client.total_supply(), 0);
    assert_eq!(client.balance(&user), 0);

    env.mock_all_auths();
    let minted = client.mint(&user, &500).unwrap();
    assert_eq!(minted, 500);

    assert_eq!(client.balance(&user), 500);
    assert_eq!(client.total_supply(), 500);
    assert_eq!(client.supply_cap(), Some(1000));

    let burned = client.burn(&user, &200).unwrap();
    assert_eq!(burned, 300);
    env.set_auths(&[]);

    assert_eq!(client.balance(&user), 300);
    assert_eq!(client.total_supply(), 300);
}

#[test]
fn test_mint_respects_supply_cap() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let contract_id = env.register_contract(None, MintBurnToken);
    let client = MintBurnTokenClient::new(&env, &contract_id);

    client.initialize(&admin, &250).unwrap();
    env.authenticate_transaction(&admin, || {
        client.mint(&alice, &200).unwrap();
    });
    env.authenticate_transaction(&admin, || {
        let err = client.mint(&alice, &100);
        assert!(err.is_err());
    });
}

#[test]
fn test_burn_requires_owner_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let contract_id = env.register_contract(None, MintBurnToken);
    let client = MintBurnTokenClient::new(&env, &contract_id);

    client.initialize(&admin, &0).unwrap();
    env.authenticate_transaction(&admin, || {
        client.mint(&alice, &100).unwrap();
    });

    env.authenticate_transaction(&bob, || {
        let result = client.burn(&alice, &50);
        assert!(result.is_err());
    });
}

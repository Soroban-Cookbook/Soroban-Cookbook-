#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env};

fn setup(supply: i128) -> (Env, PermitPatternClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PermitPattern, ());
    let client = PermitPatternClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &supply);
    (env, client, admin)
}

#[test]
fn test_initialize_sets_balance() {
    let (env, client, admin) = setup(1_000);
    assert_eq!(client.balance(&admin), 1_000);
    let _ = env;
}

#[test]
fn test_initialize_requires_admin_auth() {
    let env = Env::default();
    let contract_id = env.register(PermitPattern, ());
    let client = PermitPatternClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    assert!(client.try_initialize(&admin, &1_000).is_err());

    env.mock_all_auths();
    client.initialize(&admin, &1_000);
    assert_eq!(client.balance(&admin), 1_000);
}

#[test]
fn test_permit_sets_allowance() {
    let (env, client, owner) = setup(1_000);
    let spender = Address::generate(&env);

    client.permit(&owner, &spender, &500, &100_000);
    assert_eq!(client.allowance(&owner, &spender), 500);
}

#[test]
fn test_permit_expired_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PermitPattern, ());
    let client = PermitPatternClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.initialize(&owner, &1_000);

    let spender = Address::generate(&env);
    env.ledger().set_sequence_number(2);

    let result = client.try_permit(&owner, &spender, &100, &1);
    assert_eq!(result, Err(Ok(PermitError::ExpiredPermit)));
}

#[test]
fn test_transfer_from_uses_permit_allowance() {
    let (env, client, owner) = setup(1_000);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.permit(&owner, &spender, &400, &100_000);
    client.transfer_from(&spender, &owner, &recipient, &300);

    assert_eq!(client.allowance(&owner, &spender), 100);
    assert_eq!(client.balance(&owner), 700);
    assert_eq!(client.balance(&recipient), 300);
}

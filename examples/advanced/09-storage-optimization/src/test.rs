#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

fn setup() -> (Env, Address, StorageOptimizationClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StorageOptimization);
    let client = StorageOptimizationClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

fn create_users(env: &Env, n: u32) -> Vec<Address> {
    let mut users: Vec<Address> = Vec::new(env);
    for _ in 0..n {
        users.push_back(Address::generate(env));
    }
    users
}

#[test]
fn test_initialize() {
    let (_env, admin, client) = setup();
    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert!(!config.paused);
    assert_eq!(config.fee_bps, 25);
    assert_eq!(config.min_deposit, 100);
    assert_eq!(config.max_deposit, 1_000_000);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_double_initialize() {
    let (_env, admin, client) = setup();
    client.initialize(&admin);
}

#[test]
fn test_deposit_packed_storage() {
    let (_env, _admin, client) = setup();
    let user = Address::generate(&_env);
    client.deposit(&user, &500);
    assert_eq!(client.get_balance(&user), 500);
}

#[test]
fn test_deposit_multiple_updates_single_entry() {
    let (_env, _admin, client) = setup();
    let user = Address::generate(&_env);
    client.deposit(&user, &200);
    client.deposit(&user, &300);
    client.deposit(&user, &100);
    assert_eq!(client.get_balance(&user), 600);
}

#[test]
fn test_withdraw() {
    let (_env, _admin, client) = setup();
    let user = Address::generate(&_env);
    client.deposit(&user, &1000);
    let remaining = client.withdraw(&user, &400);
    assert_eq!(remaining, 600);
    assert_eq!(client.get_balance(&user), 600);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_withdraw_insufficient() {
    let (_env, _admin, client) = setup();
    let user = Address::generate(&_env);
    client.deposit(&user, &100);
    client.withdraw(&user, &200);
}

#[test]
fn test_get_user_data_packed_fields() {
    let (_env, _admin, client) = setup();
    let user = Address::generate(&_env);
    client.deposit(&user, &500);

    let data = client.get_user_data(&user);
    assert_eq!(data.balance, 500);
    assert_eq!(data.nonce, 1);
    assert_eq!(data.flags, 0);
    assert_eq!(data.delegate, user);
}

#[test]
fn test_batch_get_balances() {
    let (_env, _admin, client) = setup();
    let users = create_users(&_env, 3);
    client.deposit(&users.get(0).unwrap(), &100);
    client.deposit(&users.get(1).unwrap(), &200);
    client.deposit(&users.get(2).unwrap(), &300);

    let balances = client.batch_get_balances(&users);
    assert_eq!(balances.len(), 3);
    assert_eq!(balances.get(0).unwrap(), 100);
    assert_eq!(balances.get(1).unwrap(), 200);
    assert_eq!(balances.get(2).unwrap(), 300);
}

#[test]
fn test_batch_deposit() {
    let (_env, _admin, client) = setup();
    let users = create_users(&_env, 3);
    let deposits = Vec::from_array(
        &_env,
        [
            (users.get(0).unwrap(), 100i128),
            (users.get(1).unwrap(), 200i128),
            (users.get(2).unwrap(), 300i128),
        ],
    );

    let count = client.batch_deposit(&deposits);
    assert_eq!(count, 3);
    assert_eq!(client.get_balance(&users.get(0).unwrap()), 100);
    assert_eq!(client.get_balance(&users.get(1).unwrap()), 200);
    assert_eq!(client.get_balance(&users.get(2).unwrap()), 300);
}

#[test]
fn test_update_fee() {
    let (_env, _admin, client) = setup();
    client.update_fee(&100);
    let config = client.get_config();
    assert_eq!(config.fee_bps, 100);
}

#[test]
#[should_panic(expected = "Fee too high")]
fn test_update_fee_too_high() {
    let (_env, _admin, client) = setup();
    client.update_fee(&10001);
}

#[test]
fn test_set_paused() {
    let (_env, _admin, client) = setup();
    assert!(!client.get_config().paused);
    client.set_paused(&true);
    assert!(client.get_config().paused);
    client.set_paused(&false);
    assert!(!client.get_config().paused);
}

#[test]
#[should_panic(expected = "Contract is paused")]
fn test_deposit_when_paused() {
    let (_env, _admin, client) = setup();
    client.set_paused(&true);
    let user = Address::generate(&_env);
    client.deposit(&user, &100);
}

#[test]
fn test_deposit_bounds() {
    let (_env, _admin, client) = setup();
    let user = Address::generate(&_env);

    client.deposit(&user, &100);
    assert_eq!(client.get_balance(&user), 100);
}

#[test]
fn test_nonce_increments() {
    let (_env, _admin, client) = setup();
    let user = Address::generate(&_env);

    client.deposit(&user, &200);
    let data1 = client.get_user_data(&user);
    assert_eq!(data1.nonce, 1);

    client.deposit(&user, &150);
    let data2 = client.get_user_data(&user);
    assert_eq!(data2.nonce, 2);

    client.withdraw(&user, &50);
    let data3 = client.get_user_data(&user);
    assert_eq!(data3.nonce, 3);
}

#[test]
fn test_empty_balance() {
    let (_env, _admin, client) = setup();
    let user = Address::generate(&_env);
    assert_eq!(client.get_balance(&user), 0);
}

#[test]
fn test_batch_get_count() {
    let (_env, _admin, client) = setup();
    assert_eq!(client.get_batch_count(), 0);

    let user = Address::generate(&_env);
    let deposits = Vec::from_array(&_env, [(user, 500i128)]);
    client.batch_deposit(&deposits);
    assert_eq!(client.get_batch_count(), 1);
}

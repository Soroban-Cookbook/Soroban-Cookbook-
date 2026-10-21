#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup() -> (Env, Address, TokenLockContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TokenLockContract);
    let client = TokenLockContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    (env, user, client)
}

#[test]
fn test_lock_records_entry_and_balance() {
    let (_env, user, client) = setup();

    client.lock(&user, &100, &1_500);

    assert_eq!(client.locked_balance(&user), 100);
    let schedule = client.lock_schedule(&user);
    assert_eq!(schedule.len(), 1);
    assert_eq!(schedule.get(0).unwrap().amount, 100);
    assert_eq!(schedule.get(0).unwrap().unlock_time, 1_500);
}

#[test]
fn test_unlock_releases_matured_entries_only() {
    let (env, user, client) = setup();

    client.lock(&user, &100, &1_500);
    client.lock(&user, &50, &1_200);
    assert_eq!(client.locked_balance(&user), 150);

    // t=1_250: only the entry maturing at 1_200 is claimable.
    env.ledger().with_mut(|l| l.timestamp = 1_250);
    assert_eq!(client.unlockable_balance(&user), 50);
    assert_eq!(client.unlock(&user), 50);

    assert_eq!(client.locked_balance(&user), 100);
    let schedule = client.lock_schedule(&user);
    assert_eq!(schedule.len(), 1);
    assert_eq!(schedule.get(0).unwrap().unlock_time, 1_500);

    // t=1_600: the remaining entry matures too.
    env.ledger().with_mut(|l| l.timestamp = 1_600);
    assert_eq!(client.unlock(&user), 100);
    assert_eq!(client.locked_balance(&user), 0);
    assert!(client.lock_schedule(&user).is_empty());
}

#[test]
fn test_unlock_before_maturity_is_a_noop() {
    let (_env, user, client) = setup();

    client.lock(&user, &100, &1_500);
    assert_eq!(client.unlock(&user), 0);
    assert_eq!(client.locked_balance(&user), 100);
}

#[test]
fn test_balances_are_per_user() {
    let (env, alice, client) = setup();
    let bob = Address::generate(&env);

    client.lock(&alice, &100, &1_500);
    client.lock(&bob, &70, &1_500);

    assert_eq!(client.locked_balance(&alice), 100);
    assert_eq!(client.locked_balance(&bob), 70);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_lock_rejects_non_positive_amount() {
    let (_env, user, client) = setup();
    client.lock(&user, &0, &1_500);
}

#[test]
#[should_panic(expected = "unlock_time must be in the future")]
fn test_lock_rejects_past_unlock_time() {
    let (_env, user, client) = setup();
    client.lock(&user, &100, &900);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_lock_requires_user_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TokenLockContract);
    let client = TokenLockContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    env.set_auths(&[]); // no auth granted for `user`
    client.lock(&user, &100, &1_500);
}

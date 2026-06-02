use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup() -> (Env, Address, Address, OracleContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, OracleContract);
    let client = OracleContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let updater = Address::generate(&env);
    let max_age: u64 = 300; // 5 minutes
    client.initialize(&admin, &updater, &max_age);
    (env, admin, updater, client)
}

// ── initialization ──────────────────────────────────────────────────────────

#[test]
fn test_initialize_success() {
    let (_env, _admin, _updater, _client) = setup();
}

#[test]
fn test_initialize_twice_fails() {
    let (env, admin, updater, client) = setup();
    let result = client.try_initialize(&admin, &updater, &300);
    assert_eq!(result, Err(Ok(OracleError::AlreadyInitialized)));
    let _ = env;
}

// ── submit ──────────────────────────────────────────────────────────────────

#[test]
fn test_submit_authorized() {
    let (_env, _admin, updater, client) = setup();
    client.submit(&updater, &42_i128);
    assert_eq!(client.get_value(), 42_i128);
}

#[test]
fn test_submit_unauthorized() {
    let (env, _admin, _updater, client) = setup();
    let stranger = Address::generate(&env);
    let result = client.try_submit(&stranger, &99_i128);
    assert_eq!(result, Err(Ok(OracleError::NotAuthorized)));
}

#[test]
fn test_submit_updates_value() {
    let (_env, _admin, updater, client) = setup();
    client.submit(&updater, &100_i128);
    assert_eq!(client.get_value(), 100_i128);
    client.submit(&updater, &200_i128);
    assert_eq!(client.get_value(), 200_i128);
}

// ── timestamp ───────────────────────────────────────────────────────────────

#[test]
fn test_timestamp_stored() {
    let (env, _admin, updater, client) = setup();
    let before = env.ledger().timestamp();
    client.submit(&updater, &10_i128);
    let ts = client.get_timestamp();
    assert_eq!(ts, before);
}

#[test]
fn test_timestamp_updates() {
    let (env, _admin, updater, client) = setup();
    client.submit(&updater, &10_i128);
    let ts1 = client.get_timestamp();
    env.ledger().with_mut(|l| l.timestamp += 60);
    client.submit(&updater, &20_i128);
    let ts2 = client.get_timestamp();
    assert!(ts2 > ts1);
}

// ── freshness ───────────────────────────────────────────────────────────────

#[test]
fn test_fresh_immediately_after_submit() {
    let (_env, _admin, updater, client) = setup();
    client.submit(&updater, &10_i128);
    assert!(client.is_fresh());
}

#[test]
fn test_stale_after_max_age() {
    let (env, _admin, updater, client) = setup();
    client.submit(&updater, &10_i128);
    // advance past max_age (300s)
    env.ledger().with_mut(|l| l.timestamp += 301);
    assert!(!client.is_fresh());
}

#[test]
fn test_strict_get_succeeds_when_fresh() {
    let (_env, _admin, updater, client) = setup();
    client.submit(&updater, &42_i128);
    assert_eq!(client.get_value_strict(), 42_i128);
}

#[test]
fn test_strict_get_fails_when_stale() {
    let (env, _admin, updater, client) = setup();
    client.submit(&updater, &42_i128);
    env.ledger().with_mut(|l| l.timestamp += 301);
    let result = client.try_get_value_strict();
    assert_eq!(result, Err(Ok(OracleError::StaleData)));
}

// ── no-data reads ───────────────────────────────────────────────────────────

#[test]
fn test_get_value_no_data() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, OracleContract);
    let client = OracleContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let updater = Address::generate(&env);
    client.initialize(&admin, &updater, &300);
    let result = client.try_get_value();
    assert_eq!(result, Err(Ok(OracleError::NoData)));
}

#[test]
fn test_is_fresh_no_data() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, OracleContract);
    let client = OracleContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let updater = Address::generate(&env);
    client.initialize(&admin, &updater, &300);
    let result = client.try_is_fresh();
    assert_eq!(result, Err(Ok(OracleError::NoData)));
}

// ── updater rotation ────────────────────────────────────────────────────────

#[test]
fn test_rotate_updater() {
    let (env, _admin, updater, client) = setup();
    let new_updater = Address::generate(&env);
    client.set_updater(&new_updater);

    // old updater should fail
    let result = client.try_submit(&updater, &10_i128);
    assert_eq!(result, Err(Ok(OracleError::NotAuthorized)));

    // new updater should succeed
    client.submit(&new_updater, &99_i128);
    assert_eq!(client.get_value(), 99_i128);
}

// ── auth guard on set_updater ───────────────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_updater_unauthorized() {
    let env = Env::default();
    // no mock_all_auths
    let contract_id = env.register_contract(None, OracleContract);
    let client = OracleContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let updater = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin, &updater, &300);
    env.set_auths(&[]);
    let stranger = Address::generate(&env);
    client.set_updater(&stranger);
}

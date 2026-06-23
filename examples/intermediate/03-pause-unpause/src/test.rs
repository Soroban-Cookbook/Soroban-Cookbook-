use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, Address, PausableContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PausableContract);
    let client = PausableContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

// ── initialization ──────────────────────────────────────────────────────────

#[test]
fn test_initialize_success() {
    let (_env, _admin, client) = setup();
    assert!(!client.is_paused());
    assert_eq!(client.get_counter(), 0);
}

#[test]
fn test_initialize_twice_fails() {
    let (env, admin, client) = setup();
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(PauseError::AlreadyInitialized)));
    let _ = env;
}

// ── pause / unpause ─────────────────────────────────────────────────────────

#[test]
fn test_admin_can_pause() {
    let (_env, _admin, client) = setup();
    client.pause();
    assert!(client.is_paused());
}

#[test]
fn test_admin_can_unpause() {
    let (_env, _admin, client) = setup();
    client.pause();
    assert!(client.is_paused());
    client.unpause();
    assert!(!client.is_paused());
}

#[test]
fn test_pause_already_paused() {
    let (_env, _admin, client) = setup();
    client.pause();
    let result = client.try_pause();
    assert_eq!(result, Err(Ok(PauseError::AlreadyInState)));
}

#[test]
fn test_unpause_already_unpaused() {
    let (_env, _admin, client) = setup();
    let result = client.try_unpause();
    assert_eq!(result, Err(Ok(PauseError::AlreadyInState)));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_non_admin_cannot_pause() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PausableContract);
    let client = PausableContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);
    env.set_auths(&[]);
    client.pause();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_non_admin_cannot_unpause() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PausableContract);
    let client = PausableContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);
    client.pause();
    env.set_auths(&[]);
    client.unpause();
}

// ── guarded operations ──────────────────────────────────────────────────────

#[test]
fn test_increment_works_when_unpaused() {
    let (_env, _admin, client) = setup();
    assert_eq!(client.increment(), 1);
    assert_eq!(client.increment(), 2);
    assert_eq!(client.get_counter(), 2);
}

#[test]
fn test_increment_fails_when_paused() {
    let (_env, _admin, client) = setup();
    client.pause();
    let result = client.try_increment();
    assert_eq!(result, Err(Ok(PauseError::ContractPaused)));
}

#[test]
fn test_increment_works_again_after_unpause() {
    let (_env, _admin, client) = setup();
    client.increment();
    client.pause();
    let result = client.try_increment();
    assert_eq!(result, Err(Ok(PauseError::ContractPaused)));
    client.unpause();
    assert_eq!(client.increment(), 2);
}

#[test]
fn test_reset_fails_when_paused() {
    let (_env, _admin, client) = setup();
    client.increment();
    client.pause();
    let result = client.try_reset();
    assert_eq!(result, Err(Ok(PauseError::ContractPaused)));
}

// ── read-only while paused ──────────────────────────────────────────────────

#[test]
fn test_read_only_works_while_paused() {
    let (_env, _admin, client) = setup();
    client.increment();
    client.increment();
    client.pause();
    assert_eq!(client.get_counter(), 2);
    assert!(client.is_paused());
}

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup() -> (Env, Address, CircuitBreakerContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CircuitBreakerContract);
    let client = CircuitBreakerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

#[test]
fn test_initialize_and_defaults() {
    let (_env, _admin, client) = setup();
    assert_eq!(client.get_state(), CircuitState::Active);
    assert_eq!(client.get_failure_count(), 0);
    assert_eq!(client.get_failure_threshold(), 3);
    assert_eq!(client.get_recovery_window(), 100);
}

#[test]
fn test_unauthorized_initialize_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CircuitBreakerContract);
    let client = CircuitBreakerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    // Do not mock auths - this should fail with Abort since auth is not provided
    let result = client.try_initialize(&admin);
    assert!(matches!(result, Err(Err(_))));
}

#[test]
fn test_pause_and_resume_manual() {
    let (_env, _admin, client) = setup();
    client.set_pause(&true);
    assert_eq!(client.get_state(), CircuitState::Paused);
    client.set_pause(&false);
    assert_eq!(client.get_state(), CircuitState::Active);
}

#[test]
fn test_configure_rejects_invalid_values() {
    let (_env, _admin, client) = setup();
    let result = client.try_configure(&0, &10);
    assert_eq!(result, Err(Ok(CircuitError::InvalidThreshold)));
    let result = client.try_configure(&2, &0);
    assert_eq!(result, Err(Ok(CircuitError::InvalidRecoveryWindow)));
}

#[test]
fn test_auto_pause_after_threshold_failures() {
    let (_env, _admin, client) = setup();
    let caller = Address::generate(&_env);
    client.fail(&caller);
    client.fail(&caller);
    client.fail(&caller);
    assert_eq!(client.get_state(), CircuitState::Paused);
    assert_eq!(client.get_failure_count(), 3);
}

#[test]
fn test_execute_is_blocked_when_paused() {
    let (_env, _admin, client) = setup();
    let caller = Address::generate(&_env);
    client.fail(&caller);
    client.fail(&caller);
    client.fail(&caller);
    let result = client.try_execute(&caller);
    assert_eq!(result, Err(Ok(CircuitError::CircuitPaused)));
}

#[test]
fn test_recovery_window_reopens_circuit() {
    let (env, _admin, client) = setup();
    let caller = Address::generate(&env);
    client.fail(&caller);
    client.fail(&caller);
    client.fail(&caller);
    env.ledger().with_mut(|l| l.timestamp += 101);
    client.execute(&caller);
    assert_eq!(client.get_state(), CircuitState::Active);
    assert_eq!(client.get_failure_count(), 0);
}

#[test]
fn test_success_resets_failure_count() {
    let (_env, _admin, client) = setup();
    let caller = Address::generate(&_env);
    client.fail(&caller);
    client.fail(&caller);
    client.execute(&caller);
    assert_eq!(client.get_failure_count(), 0);
}

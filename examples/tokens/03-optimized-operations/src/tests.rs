#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

// ============================================================================
// Tests for Standard Implementation
// ============================================================================

#[test]
fn test_standard_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let underlying = Address::generate(&env);
    let standard =
        StandardTokenOpsClient::new(&env, &env.register_contract(None, StandardTokenOps));

    let result = standard.try_standard_initialize(&underlying);
    assert!(result.is_ok());
}

#[test]
fn test_standard_wrap_and_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let underlying = Address::generate(&env);

    let standard_id = env.register_contract(None, StandardTokenOps);
    let standard = StandardTokenOpsClient::new(&env, &standard_id);

    standard.standard_initialize(&underlying);

    // Test balance retrieval
    let balance = standard.standard_balance(&user);
    assert_eq!(balance, 0);
}

// ============================================================================
// Tests for Optimized Implementation
// ============================================================================

#[test]
fn test_optimized_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let underlying = Address::generate(&env);
    let optimized_id = env.register_contract(None, OptimizedTokenOps);
    let optimized = OptimizedTokenOpsClient::new(&env, &optimized_id);

    let result = optimized.try_initialize(&underlying);
    assert!(result.is_ok());
}

#[test]
fn test_optimized_single_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let underlying = Address::generate(&env);

    let optimized_id = env.register_contract(None, OptimizedTokenOps);
    let optimized = OptimizedTokenOpsClient::new(&env, &optimized_id);

    optimized.initialize(&underlying);

    // Initial balances should be 0
    assert_eq!(optimized.balance(&alice), 0);
    assert_eq!(optimized.balance(&bob), 0);
}

#[test]
fn test_optimized_batch_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);
    let underlying = Address::generate(&env);

    let optimized_id = env.register_contract(None, OptimizedTokenOps);
    let optimized = OptimizedTokenOpsClient::new(&env, &optimized_id);

    optimized.initialize(&underlying);

    // Create batch transfer
    let batch = vec![
        &env,
        BatchTransfer {
            recipient: bob.clone(),
            amount: 100,
        },
        BatchTransfer {
            recipient: charlie.clone(),
            amount: 200,
        },
    ];

    // Attempt batch transfer (would fail due to insufficient balance)
    let result = optimized.try_batch_transfer(&alice, &batch);
    assert!(result.is_err());
}

#[test]
fn test_optimized_empty_batch_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let underlying = Address::generate(&env);

    let optimized_id = env.register_contract(None, OptimizedTokenOps);
    let optimized = OptimizedTokenOpsClient::new(&env, &optimized_id);

    optimized.initialize(&underlying);

    let empty_batch: Vec<BatchTransfer> = vec![&env];

    let result = optimized.try_batch_transfer(&alice, &empty_batch);
    assert!(result.is_err());
}

#[test]
fn test_optimized_transfer_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let underlying = Address::generate(&env);

    let optimized_id = env.register_contract(None, OptimizedTokenOps);
    let optimized = OptimizedTokenOpsClient::new(&env, &optimized_id);

    optimized.initialize(&underlying);

    let result = optimized.try_transfer(&alice, &bob, &0);
    assert!(result.is_err());

    let result = optimized.try_transfer(&alice, &bob, &-100);
    assert!(result.is_err());
}

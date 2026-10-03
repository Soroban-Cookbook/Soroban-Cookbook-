extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, BatchBuilderContractClient<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(BatchBuilderContract, ());
    let client = BatchBuilderContractClient::new(&env, &contract_id);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.initialize();
    client.set_balance(&alice, &1_000);
    client.set_balance(&bob, &100);
    (env, client, alice, bob)
}

#[test]
fn test_builder_adds_operations_and_estimates_gas() {
    let (_env, client, alice, bob) = setup();
    let batch_id = client.begin_batch();

    client.add_transfer(&batch_id, &alice, &bob, &50);
    client.add_mint(&batch_id, &bob, &10);

    assert_eq!(client.batch_len(&batch_id), 2);
    assert_eq!(
        client.estimate_gas(&batch_id),
        BASE_GAS_UNITS + GAS_PER_TRANSFER + GAS_PER_MINT
    );
}

#[test]
fn test_validate_and_execute_batch() {
    let (_env, client, alice, bob) = setup();
    let batch_id = client.begin_batch();

    client.add_transfer(&batch_id, &alice, &bob, &50);
    client.add_mint(&batch_id, &bob, &10);

    let gas = client.validate_batch(&batch_id);
    assert_eq!(gas, BASE_GAS_UNITS + GAS_PER_TRANSFER + GAS_PER_MINT);
    assert!(client.is_validated(&batch_id));

    let executed = client.execute_batch(&batch_id);
    assert_eq!(executed, 2);
    assert_eq!(client.get_balance(&alice), 950);
    assert_eq!(client.get_balance(&bob), 160);
}

#[test]
fn test_validate_rejects_insufficient_balance() {
    let (_env, client, alice, bob) = setup();
    let batch_id = client.begin_batch();

    client.add_transfer(&batch_id, &alice, &bob, &5_000);
    let result = client.try_validate_batch(&batch_id);
    assert_eq!(result, Err(Ok(BuilderError::InsufficientBalance)));
}

#[test]
fn test_validate_rejects_duplicate_operation() {
    let (_env, client, alice, bob) = setup();
    let batch_id = client.begin_batch();

    client.add_transfer(&batch_id, &alice, &bob, &10);
    client.add_transfer(&batch_id, &alice, &bob, &10);
    let result = client.try_validate_batch(&batch_id);
    assert_eq!(result, Err(Ok(BuilderError::DuplicateOperation)));
}

#[test]
fn test_execute_requires_validation() {
    let (_env, client, alice, bob) = setup();
    let batch_id = client.begin_batch();
    client.add_transfer(&batch_id, &alice, &bob, &10);

    let result = client.try_execute_batch(&batch_id);
    assert_eq!(result, Err(Ok(BuilderError::NotValidated)));
}

#[test]
fn test_batch_size_limit() {
    let (_env, client, alice, bob) = setup();
    let batch_id = client.begin_batch();

    for _ in 0..MAX_BATCH_SIZE {
        client.add_transfer(&batch_id, &alice, &bob, &1);
    }

    let result = client.try_add_transfer(&batch_id, &alice, &bob, &1);
    assert_eq!(result, Err(Ok(BuilderError::BatchTooLarge)));
}

#[test]
fn test_empty_batch_validation_fails() {
    let (_env, client, _alice, _bob) = setup();
    let batch_id = client.begin_batch();
    let result = client.try_validate_batch(&batch_id);
    assert_eq!(result, Err(Ok(BuilderError::EmptyBatch)));
}

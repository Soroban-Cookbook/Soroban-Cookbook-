extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

fn setup() -> (Env, BatchOperationsClient<'static>, Address, Address, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, BatchOperations);
    let client = BatchOperationsClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.set_balance(&alice, &100).unwrap();
    client.set_balance(&bob, &20).unwrap();
    client.set_balance(&carol, &0).unwrap();

    (env, client, alice, bob, carol)
}

#[test]
fn test_atomic_executes_all_operations() {
    let (env, client, alice, bob, carol) = setup();

    let ops = Vec::from_array(
        &env,
        [
            BatchOperation::Transfer(alice.clone(), bob.clone(), 30),
            BatchOperation::Credit(carol.clone(), 10),
            BatchOperation::Debit(bob.clone(), 5),
        ],
    );

    client.execute_batch_atomic(&ops).unwrap();

    assert_eq!(client.get_balance(&alice), 70);
    assert_eq!(client.get_balance(&bob), 45);
    assert_eq!(client.get_balance(&carol), 10);
}

#[test]
fn test_atomic_rolls_back_on_failure() {
    let (env, client, alice, bob, _carol) = setup();

    let ops = Vec::from_array(
        &env,
        [
            BatchOperation::Transfer(alice.clone(), bob.clone(), 30),
            BatchOperation::Debit(bob.clone(), 10_000),
        ],
    );

    assert_eq!(
        client.execute_batch_atomic(&ops),
        Err(Ok(BatchError::InsufficientBalance))
    );

    assert_eq!(client.get_balance(&alice), 100);
    assert_eq!(client.get_balance(&bob), 20);
}

#[test]
fn test_atomic_rolls_back_pause_toggle() {
    let (env, client, alice, bob, _carol) = setup();

    let ops = Vec::from_array(
        &env,
        [
            BatchOperation::SetPaused(true),
            BatchOperation::Transfer(alice.clone(), bob.clone(), 5),
        ],
    );

    assert_eq!(
        client.execute_batch_atomic(&ops),
        Err(Ok(BatchError::ContractPaused))
    );

    assert!(!client.is_paused());
    assert_eq!(client.get_balance(&alice), 100);
    assert_eq!(client.get_balance(&bob), 20);
}

#[test]
fn test_partial_executes_valid_and_skips_invalid() {
    let (env, client, alice, bob, carol) = setup();

    let ops = Vec::from_array(
        &env,
        [
            BatchOperation::Transfer(alice.clone(), bob.clone(), 40),
            BatchOperation::Debit(carol.clone(), 1),
            BatchOperation::Credit(carol.clone(), 20),
        ],
    );

    let result = client.execute_batch_partial(&ops);
    assert_eq!(result.applied, 2);
    assert_eq!(result.failed, 1);
    assert_eq!(result.statuses.len(), 3);
    assert_eq!(result.statuses.get(0).unwrap(), OpStatus::Applied);
    assert_eq!(
        result.statuses.get(1).unwrap(),
        OpStatus::Skipped(BatchError::InsufficientBalance)
    );
    assert_eq!(result.statuses.get(2).unwrap(), OpStatus::Applied);

    assert_eq!(client.get_balance(&alice), 60);
    assert_eq!(client.get_balance(&bob), 60);
    assert_eq!(client.get_balance(&carol), 20);
}

#[test]
fn test_partial_continue_after_failure() {
    let (env, client, alice, bob, _carol) = setup();

    let ops = Vec::from_array(
        &env,
        [
            BatchOperation::Debit(bob.clone(), 100),
            BatchOperation::Transfer(alice.clone(), bob.clone(), 25),
        ],
    );

    let result = client.execute_batch_partial(&ops);
    assert_eq!(result.applied, 1);
    assert_eq!(result.failed, 1);

    assert_eq!(client.get_balance(&alice), 75);
    assert_eq!(client.get_balance(&bob), 45);
}

#[test]
fn test_paused_contract_blocks_financial_ops() {
    let (env, client, alice, bob, _carol) = setup();

    client
        .execute_batch_atomic(&Vec::from_array(&env, [BatchOperation::SetPaused(true)]))
        .unwrap();

    assert_eq!(
        client.execute_batch_atomic(&Vec::from_array(
            &env,
            [BatchOperation::Transfer(alice.clone(), bob.clone(), 1)]
        )),
        Err(Ok(BatchError::ContractPaused))
    );
}

#[test]
fn test_unpause_restores_operations() {
    let (env, client, alice, bob, _carol) = setup();

    client
        .execute_batch_atomic(&Vec::from_array(&env, [BatchOperation::SetPaused(true)]))
        .unwrap();
    client
        .execute_batch_atomic(&Vec::from_array(&env, [BatchOperation::SetPaused(false)]))
        .unwrap();

    client
        .execute_batch_atomic(&Vec::from_array(
            &env,
            [BatchOperation::Transfer(alice.clone(), bob.clone(), 10)],
        ))
        .unwrap();

    assert_eq!(client.get_balance(&alice), 90);
    assert_eq!(client.get_balance(&bob), 30);
}

#[test]
fn test_invalid_amount_rejected() {
    let (env, client, alice, _bob, _carol) = setup();

    let ops = Vec::from_array(&env, [BatchOperation::Credit(alice.clone(), 0)]);
    assert_eq!(client.execute_batch_atomic(&ops), Err(Ok(BatchError::InvalidAmount)));
}

#[test]
fn test_transfer_overflow_rejected() {
    let (env, client, alice, bob, _carol) = setup();
    client.set_balance(&bob, &i128::MAX).unwrap();

    let ops = Vec::from_array(&env, [BatchOperation::Transfer(alice.clone(), bob.clone(), 1)]);
    assert_eq!(
        client.execute_batch_atomic(&ops),
        Err(Ok(BatchError::ArithmeticOverflow))
    );
}

#[test]
fn test_empty_batch_is_noop() {
    let (env, client, alice, bob, _carol) = setup();

    client.execute_batch_atomic(&Vec::new(&env)).unwrap();
    let partial = client.execute_batch_partial(&Vec::new(&env));

    assert_eq!(partial.applied, 0);
    assert_eq!(partial.failed, 0);
    assert_eq!(client.get_balance(&alice), 100);
    assert_eq!(client.get_balance(&bob), 20);
}

#[test]
fn test_set_balance_rejects_negative_values() {
    let (env, client, _alice, _bob, _carol) = setup();
    let dave = Address::generate(&env);

    assert_eq!(
        client.set_balance(&dave, &-1),
        Err(Ok(BatchError::InvalidAmount))
    );
}

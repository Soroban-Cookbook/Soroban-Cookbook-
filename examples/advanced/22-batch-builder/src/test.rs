extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, BatchBuilderContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(BatchBuilderContract, ());
    let client = BatchBuilderContractClient::new(&env, &contract_id);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let admin = Address::generate(&env);
    // 0 → use the default MAX_BATCH_SIZE (20)
    client.initialize(&admin, &0);
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

/// `add_*` helpers must reject the (limit + 1)-th operation with `TooManyOps`.
/// Uses the default limit of `MAX_BATCH_SIZE` (20).
#[test]
fn test_add_op_rejects_oversized_batch() {
    let (_env, client, alice, bob) = setup();
    let batch_id = client.begin_batch();

    // Fill exactly MAX_BATCH_SIZE slots; each amount is distinct to avoid the
    // duplicate-operation check.
    for i in 1..=(MAX_BATCH_SIZE as i128) {
        client.add_transfer(&batch_id, &alice, &bob, &i);
    }

    // The (MAX_BATCH_SIZE + 1)-th add must be rejected with TooManyOps.
    let result = client.try_add_transfer(&batch_id, &alice, &bob, &999);
    assert_eq!(result, Err(Ok(BuilderError::TooManyOps)));
}

/// `execute_batch` enforces the size limit independently of `push_op`.
/// A contract initialized with a generous limit builds and validates a large
/// batch successfully; the same op count is blocked at the add stage on a
/// contract with the default limit.
#[test]
fn test_execute_batch_rejects_oversized() {
    // --- Part 1: generous limit — batch executes successfully ---------------
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(BatchBuilderContract, ());
    let client = BatchBuilderContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let generous_limit: u32 = MAX_BATCH_SIZE + 5;
    client.initialize(&admin, &generous_limit);
    let total: i128 = (1..=(generous_limit as i128)).sum();
    client.set_balance(&alice, &total);
    client.set_balance(&bob, &0);

    let batch_id = client.begin_batch();
    for i in 1..=(generous_limit as i128) {
        client.add_transfer(&batch_id, &alice, &bob, &i);
    }
    client.validate_batch(&batch_id);
    let executed = client.execute_batch(&batch_id);
    assert_eq!(executed, generous_limit);

    // --- Part 2: default limit — (limit + 1)-th add returns TooManyOps -----
    let env2 = Env::default();
    env2.mock_all_auths();
    let cid2 = env2.register(BatchBuilderContract, ());
    let client2 = BatchBuilderContractClient::new(&env2, &cid2);
    let admin2 = Address::generate(&env2);
    let alice2 = Address::generate(&env2);
    let bob2 = Address::generate(&env2);

    client2.initialize(&admin2, &0); // default limit = MAX_BATCH_SIZE (20)
    client2.set_balance(&alice2, &100_000);
    client2.set_balance(&bob2, &0);

    let bid2 = client2.begin_batch();
    for i in 1..=(MAX_BATCH_SIZE as i128) {
        client2.add_transfer(&bid2, &alice2, &bob2, &i);
    }
    // Exactly at the limit → next add must return TooManyOps.
    let result = client2.try_add_transfer(&bid2, &alice2, &bob2, &999);
    assert_eq!(result, Err(Ok(BuilderError::TooManyOps)));

    // Validate and execute the 20-op batch — both must succeed.
    client2.validate_batch(&bid2);
    let executed2 = client2.execute_batch(&bid2);
    assert_eq!(executed2, MAX_BATCH_SIZE);
}

#[test]
fn test_empty_batch_validation_fails() {
    let (_env, client, _alice, _bob) = setup();
    let batch_id = client.begin_batch();
    let result = client.try_validate_batch(&batch_id);
    assert_eq!(result, Err(Ok(BuilderError::EmptyBatch)));
}

/// A custom `max_ops` supplied at initialize time is respected.
#[test]
fn test_custom_max_ops_at_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(BatchBuilderContract, ());
    let client = BatchBuilderContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Initialize with a small custom limit of 3.
    client.initialize(&admin, &3);
    client.set_balance(&alice, &10_000);
    client.set_balance(&bob, &0);

    let batch_id = client.begin_batch();
    client.add_transfer(&batch_id, &alice, &bob, &1);
    client.add_transfer(&batch_id, &alice, &bob, &2);
    client.add_transfer(&batch_id, &alice, &bob, &3);

    // 4th add must be rejected — limit is 3, not the default 20.
    let result = client.try_add_transfer(&batch_id, &alice, &bob, &4);
    assert_eq!(result, Err(Ok(BuilderError::TooManyOps)));
}

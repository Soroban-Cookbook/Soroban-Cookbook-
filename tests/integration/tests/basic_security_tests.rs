#![allow(deprecated)]
//! Integration tests covering security edge cases, unauthorized access attempts,
//! and potential exploit vectors across basic cookbook examples.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, Env};

// ---------------------------------------------------------------------------
// Security Test 1: Authentication Negative Amount & Overflow Prevention
// ---------------------------------------------------------------------------

#[test]
fn test_auth_contract_negative_amount_exploit_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let client = authentication::AuthContractClient::new(&env, &auth_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client.initialize(&admin);
    client.set_balance(&admin, &user1, &500);
    client.set_balance(&admin, &user2, &300);

    // Attempting a negative transfer should be blocked with InvalidAmount
    let res = client.try_transfer(&user1, &user2, &-100);
    assert!(res.is_err());

    // Attempting a zero transfer should also be blocked
    let res_zero = client.try_transfer(&user1, &user2, &0);
    assert!(res_zero.is_err());

    // Verify balances remained untouched
    assert_eq!(client.get_balance(&user1), 500);
    assert_eq!(client.get_balance(&user2), 300);

    // Spender negative allowance check
    let res_approve = client.try_approve(&user1, &user2, &-50);
    assert!(res_approve.is_err());

    // Spender transfer_from negative exploit check
    client.approve(&user1, &user2, &200);
    let res_tf = client.try_transfer_from(&user2, &user1, &admin, &-50);
    assert!(res_tf.is_err());
}

// ---------------------------------------------------------------------------
// Security Test 2: Error Handling Negative Input Safeguards
// ---------------------------------------------------------------------------

#[test]
fn test_error_handling_negative_deposit_withdraw_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let err_id = env.register_contract(None, error_handling::ErrorDemoContract);
    let client = error_handling::ErrorDemoContractClient::new(&env, &err_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);

    // Negative deposit must fail
    let res_dep = client.try_deposit(&user, &-100);
    assert_eq!(res_dep, Err(Ok(error_handling::ContractError::ZeroAmount)));

    // Negative withdraw must fail
    let res_wd = client.try_withdraw(&user, &-100);
    assert_eq!(res_wd, Err(Ok(error_handling::ContractError::ZeroAmount)));

    assert_eq!(client.balance(&user), 0);
}

// ---------------------------------------------------------------------------
// Security Test 3: Unauthorized Access Bypass Attempts (require_auth verification)
// ---------------------------------------------------------------------------

#[test]
fn test_unauthorized_admin_and_user_actions_rejected() {
    let env = Env::default();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let client = authentication::AuthContractClient::new(&env, &auth_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // First call initialize - we mock auths only for this setup step
    env.mock_all_auths();
    client.initialize(&admin);

    // Now remove auth mock to test actual validation checks
    env.mock_auths(&[]);

    // Admin action without proper authorization must panic at Host level
    let res_admin = client.try_admin_action(&admin, &100);
    assert!(res_admin.is_err());

    // User transfer without signature must fail
    let res_xfer = client.try_transfer(&user, &admin, &50);
    assert!(res_xfer.is_err());
}

// ---------------------------------------------------------------------------
// Security Test 4: Queue Bounds, Underflow, & Drop Policy Verification
// ---------------------------------------------------------------------------

#[test]
fn test_fifo_queue_bounds_and_empty_state_panics() {
    let env = Env::default();
    let q_id = env.register_contract(None, fifo_queue::QueueContract);
    let client = fifo_queue::QueueContractClient::new(&env, &q_id);

    // Dequeuing from an empty queue must panic
    let res_deq = client.try_dequeue();
    assert!(res_deq.is_err());

    // Peeking on empty queue must panic
    let res_peek = client.try_peek();
    assert!(res_peek.is_err());

    // Fill the queue
    let item = symbol_short!("task");
    client.enqueue(&item);
    assert_eq!(client.size(), 1);
    assert_eq!(client.peek(), item);

    let removed = client.dequeue();
    assert_eq!(removed, item);
    assert!(client.is_empty());
}

#[test]
fn test_bounded_queue_capacity_enforcement_and_policies() {
    let env = Env::default();

    // Bounded Queue Contract (Capacity = 2, Policy = DropNewest)
    let bq_id = env.register_contract(None, queue_variants::BoundedQueueContract);
    let bq_client = queue_variants::BoundedQueueContractClient::new(&env, &bq_id);

    bq_client.initialize(&2, &queue_variants::DropPolicy::DropNewest);

    let val1 = Bytes::from_slice(&env, b"one");
    let val2 = Bytes::from_slice(&env, b"two");
    let val3 = Bytes::from_slice(&env, b"three");

    bq_client.push(&val1);
    bq_client.push(&val2);
    assert_eq!(bq_client.len(), 2);

    // Pushing third element on DropNewest policy must panic
    let res_push = bq_client.try_push(&val3);
    assert!(res_push.is_err());

    // Bounded Queue Contract (Capacity = 2, Policy = DropOldest)
    let bq_id_2 = env.register_contract(None, queue_variants::BoundedQueueContract);
    let bq_client_2 = queue_variants::BoundedQueueContractClient::new(&env, &bq_id_2);

    bq_client_2.initialize(&2, &queue_variants::DropPolicy::DropOldest);
    bq_client_2.push(&val1);
    bq_client_2.push(&val2);

    // Pushing third element on DropOldest overwrites the oldest (val1)
    bq_client_2.push(&val3);
    assert_eq!(bq_client_2.len(), 2);
    assert_eq!(bq_client_2.pop(), val2); // val1 was dropped, so val2 is now head
    assert_eq!(bq_client_2.pop(), val3);
}

#[test]
fn test_circular_buffer_capacity_overwrites() {
    let env = Env::default();
    let cb_id = env.register_contract(None, queue_variants::CircularBufferContract);
    let cb_client = queue_variants::CircularBufferContractClient::new(&env, &cb_id);

    cb_client.initialize(&2);

    let val1 = Bytes::from_slice(&env, b"one");
    let val2 = Bytes::from_slice(&env, b"two");
    let val3 = Bytes::from_slice(&env, b"three");

    cb_client.push(&val1);
    cb_client.push(&val2);

    // Pushes when full should overwrite the oldest element safely
    cb_client.push(&val3);
    assert_eq!(cb_client.len(), 2);
    assert_eq!(cb_client.pop(), val2);
    assert_eq!(cb_client.pop(), val3);
}

// ---------------------------------------------------------------------------
// Security Test 5: Lazy Cache Hits, Misses, and Invalidation Integrity
// ---------------------------------------------------------------------------

#[test]
fn test_lazy_cache_integrity_and_eviction() {
    let env = Env::default();
    let cache_id = env.register_contract(None, lazy_cache::LazyCacheContract);
    let client = lazy_cache::LazyCacheContractClient::new(&env, &cache_id);

    // Save item in persistent storage
    client.set_item(&1, &100);
    client.set_item(&2, &200);

    // Check initial stats (current_size, hits, misses)
    let stats = client.cache_stats();
    assert_eq!(stats, (0, 0, 0));

    // First retrieve: cache miss
    let item1 = client.get_item(&1);
    assert_eq!(item1, Some(100));
    assert_eq!(client.cache_stats(), (1, 0, 1));

    // Second retrieve: cache hit
    let item1_cached = client.get_item(&1);
    assert_eq!(item1_cached, Some(100));
    assert_eq!(client.cache_stats(), (1, 1, 1));

    // Manual invalidation
    client.invalidate_cache(&1);
    assert_eq!(client.cache_stats(), (0, 1, 1));

    // Try retrieve again: miss
    let item1_miss = client.get_item(&1);
    assert_eq!(item1_miss, Some(100));
    assert_eq!(client.cache_stats(), (1, 1, 2));
}

// ---------------------------------------------------------------------------
// Security Test 6: Compressed Storage Run-Length Encoding Accuracy
// ---------------------------------------------------------------------------

#[test]
fn test_compressed_storage_integrity() {
    let env = Env::default();
    let store_id = env.register_contract(None, compressed_storage::CompressedStorageContract);
    let client = compressed_storage::CompressedStorageContractClient::new(&env, &store_id);

    let key = Address::generate(&env);

    // Payload with high repeatability to test RLE compression ratio
    let mut payload = Bytes::new(&env);
    for _ in 0..10 {
        payload.push_back(0xAA);
    }
    for _ in 0..5 {
        payload.push_back(0xBB);
    }

    let raw_len = client.store_raw(&key, &payload);
    let comp_len = client.store_compressed(&key, &payload);

    // Verify sizes
    assert_eq!(raw_len, 15);
    // Be-encoded original length (4 bytes) + 2 repeating sequences of AA (x10) and BB (x5) (2 * 2 = 4 bytes) = 8 bytes
    assert_eq!(comp_len, 8);

    // Retrieve and compare
    let raw_out = client.get_raw(&key).unwrap();
    let decomp_out = client.get_decompressed(&key).unwrap();

    assert_eq!(raw_out, payload);
    assert_eq!(decomp_out, payload);
}

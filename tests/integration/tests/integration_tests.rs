//! Integration Tests for Soroban Cookbook Basic Examples
//!
//! This test suite demonstrates cross-contract interactions and end-to-end scenarios
//! combining multiple basic examples using WASM binaries.

#![cfg(test)]

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, Env, IntoVal, String, Symbol, Vec};

/// Test 1: Multi-Contract Workflow - Hello World + Storage + Events
/// 
/// Scenario: A user greeting system that stores greetings and emits events
#[test]
fn test_greeting_system_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    // Register contracts from WASM
    let hello_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/hello_world.wasm"));
    let hello_id = env.register_contract_wasm(None, hello_wasm);

    let storage_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/storage_patterns.wasm"));
    let storage_id = env.register_contract_wasm(None, storage_wasm);

    let events_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/events.wasm"));
    let events_id = env.register_contract_wasm(None, events_wasm);

    let user = Address::generate(&env);

    // Step 1: Generate greeting
    let greeting: String = env.invoke_contract(
        &hello_id,
        &symbol_short!("hello"),
        Vec::from_array(&env, [symbol_short!("Alice").into_val(&env)]),
    );
    assert_eq!(greeting, String::from_bytes(&env, b"Hello, Alice!"));

    // Step 2: Store greeting count in persistent storage
    let greeting_key = symbol_short!("greet_cnt");
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env), 1u64.into_val(&env)]),
    );

    let count: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env)]),
    );
    assert_eq!(count, 1);

    // Step 3: Emit audit event for the greeting
    env.invoke_contract::<()>(
        &events_id,
        &Symbol::new(&env, "admin_action"),
        Vec::from_array(&env, [user.into_val(&env), symbol_short!("greet").into_val(&env)]),
    );

    // Step 4: Increment greeting count
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env), 2u64.into_val(&env)]),
    );

    let new_count: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env)]),
    );
    assert_eq!(new_count, 2);

    // Verify storage persistence
    let has_key: bool = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "has_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env)]),
    );
    assert!(has_key);
}

/// Test 2: Authentication + Storage Integration
/// 
/// Scenario: Authenticated users can store and retrieve their own data
#[test]
fn test_authenticated_storage_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/authentication.wasm"));
    let auth_id = env.register_contract_wasm(None, auth_wasm);

    let storage_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/storage_patterns.wasm"));
    let storage_id = env.register_contract_wasm(None, storage_wasm);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Step 1: Authenticate users
    let result1: bool = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "basic_auth"),
        Vec::from_array(&env, [user1.into_val(&env)]),
    );
    assert!(result1);

    let result2: bool = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "basic_auth"),
        Vec::from_array(&env, [user2.into_val(&env)]),
    );
    assert!(result2);

    // Step 2: Each user stores their data
    let user1_key = symbol_short!("user1");
    let user2_key = symbol_short!("user2");

    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [user1_key.into_val(&env), 100u64.into_val(&env)]),
    );

    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [user2_key.into_val(&env), 200u64.into_val(&env)]),
    );

    // Step 3: Verify data isolation
    let user1_data: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [user1_key.into_val(&env)]),
    );
    assert_eq!(user1_data, 100);

    let user2_data: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [user2_key.into_val(&env)]),
    );
    assert_eq!(user2_data, 200);
}

/// Test 3: Cross-Contract Event Tracking
/// 
/// Scenario: Track operations across multiple contracts with events
#[test]
fn test_cross_contract_event_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/authentication.wasm"));
    let auth_id = env.register_contract_wasm(None, auth_wasm);

    let events_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/events.wasm"));
    let events_id = env.register_contract_wasm(None, events_wasm);

    let storage_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/storage_patterns.wasm"));
    let storage_id = env.register_contract_wasm(None, storage_wasm);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Step 1: Initialize admin
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "set_admin"),
        Vec::from_array(&env, [admin.clone().into_val(&env), admin.clone().into_val(&env)]),
    );

    // Step 2: Emit admin action event
    env.invoke_contract::<()>(
        &events_id,
        &Symbol::new(&env, "admin_action"),
        Vec::from_array(&env, [admin.into_val(&env), symbol_short!("init").into_val(&env)]),
    );

    // Step 3: Store configuration
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_instance"),
        Vec::from_array(&env, [symbol_short!("config").into_val(&env), 42u64.into_val(&env)]),
    );

    // Step 4: Emit config update event
    env.invoke_contract::<()>(
        &events_id,
        &Symbol::new(&env, "update_config"),
        Vec::from_array(&env, [
            symbol_short!("config").into_val(&env),
            0u64.into_val(&env),
            42u64.into_val(&env),
        ]),
    );

    // Step 5: User performs transfer
    env.invoke_contract::<()>(
        &events_id,
        &symbol_short!("transfer"),
        Vec::from_array(&env, [
            user.into_val(&env),
            admin.into_val(&env),
            1000i128.into_val(&env),
            1u64.into_val(&env),
        ]),
    );

    // Verify storage state
    let config: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_instance"),
        Vec::from_array(&env, [symbol_short!("config").into_val(&env)]),
    );
    assert_eq!(config, 42);
}

/// Test 4: Storage Type Comparison - End-to-End
/// 
/// Scenario: Demonstrate differences between persistent, temporary, and instance storage
#[test]
fn test_storage_types_comparison() {
    let env = Env::default();

    let storage_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/storage_patterns.wasm"));
    let storage_id = env.register_contract_wasm(None, storage_wasm);

    let key = symbol_short!("testkey");

    // Test 1: Persistent storage survives
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [key.into_val(&env), 100u64.into_val(&env)]),
    );

    let has_pers: bool = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "has_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert!(has_pers);

    let pers_val: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(pers_val, 100);

    // Test 2: Temporary storage (same ledger)
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_temporary"),
        Vec::from_array(&env, [key.into_val(&env), 200u64.into_val(&env)]),
    );

    let has_temp: bool = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "has_temporary"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert!(has_temp);

    let temp_val: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_temporary"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(temp_val, 200);

    // Test 3: Instance storage
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_instance"),
        Vec::from_array(&env, [key.into_val(&env), 300u64.into_val(&env)]),
    );

    let has_inst: bool = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "has_instance"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert!(has_inst);

    let inst_val: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_instance"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(inst_val, 300);

    // Test 4: All three storage types are independent
    let pers_check: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(pers_check, 100);

    // Test 5: Remove operations
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "remove_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );

    let has_after_remove: bool = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "has_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert!(!has_after_remove);
}

/// Test 5: Complex Multi-Party Workflow
/// 
/// Scenario: Multiple users interact with authentication, storage, and events
#[test]
fn test_multi_party_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy contracts
    let auth_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/authentication.wasm"));
    let auth_id = env.register_contract_wasm(None, auth_wasm);

    let storage_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/storage_patterns.wasm"));
    let storage_id = env.register_contract_wasm(None, storage_wasm);

    let events_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/events.wasm"));
    let events_id = env.register_contract_wasm(None, events_wasm);

    let hello_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/hello_world.wasm"));
    let hello_id = env.register_contract_wasm(None, hello_wasm);

    // Create multiple users
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Step 1: Setup - Admin initialization
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "set_admin"),
        Vec::from_array(&env, [admin.clone().into_val(&env), admin.clone().into_val(&env)]),
    );

    env.invoke_contract::<()>(
        &events_id,
        &Symbol::new(&env, "admin_action"),
        Vec::from_array(&env, [admin.into_val(&env), symbol_short!("setup").into_val(&env)]),
    );

    // Step 2: Alice joins and gets greeted
    let alice_greeting: String = env.invoke_contract(
        &hello_id,
        &symbol_short!("hello"),
        Vec::from_array(&env, [symbol_short!("Alice").into_val(&env)]),
    );
    assert_eq!(alice_greeting, String::from_bytes(&env, b"Hello, Alice!"));

    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [symbol_short!("alice").into_val(&env), 100u64.into_val(&env)]),
    );

    // Step 3: Bob joins and gets greeted
    let bob_greeting: String = env.invoke_contract(
        &hello_id,
        &symbol_short!("hello"),
        Vec::from_array(&env, [symbol_short!("Bob").into_val(&env)]),
    );
    assert_eq!(bob_greeting, String::from_bytes(&env, b"Hello, Bob!"));

    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [symbol_short!("bob").into_val(&env), 200u64.into_val(&env)]),
    );

    // Step 4: Alice transfers to Bob
    env.invoke_contract::<()>(
        &events_id,
        &symbol_short!("transfer"),
        Vec::from_array(&env, [
            alice.into_val(&env),
            bob.into_val(&env),
            50i128.into_val(&env),
            1u64.into_val(&env),
        ]),
    );

    // Step 5: Update balances
    let alice_balance: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [symbol_short!("alice").into_val(&env)]),
    );

    let bob_balance: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [symbol_short!("bob").into_val(&env)]),
    );

    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [
            symbol_short!("alice").into_val(&env),
            (alice_balance - 50).into_val(&env),
        ]),
    );

    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [
            symbol_short!("bob").into_val(&env),
            (bob_balance + 50).into_val(&env),
        ]),
    );

    // Step 6: Verify final state
    let final_alice: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [symbol_short!("alice").into_val(&env)]),
    );
    assert_eq!(final_alice, 50);

    let final_bob: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [symbol_short!("bob").into_val(&env)]),
    );
    assert_eq!(final_bob, 250);
}

/// Test 6: State Management Across Contracts
/// 
/// Scenario: Coordinate state changes across multiple contracts
#[test]
fn test_coordinated_state_management() {
    let env = Env::default();
    env.mock_all_auths();

    let storage_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/storage_patterns.wasm"));
    let storage_id = env.register_contract_wasm(None, storage_wasm);

    let events_wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/events.wasm"));
    let events_id = env.register_contract_wasm(None, events_wasm);

    let admin = Address::generate(&env);

    // Scenario: System configuration update workflow

    // Step 1: Read current config
    let config_key = symbol_short!("max_val");
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_instance"),
        Vec::from_array(&env, [config_key.into_val(&env), 1000u64.into_val(&env)]),
    );

    let old_value: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_instance"),
        Vec::from_array(&env, [config_key.into_val(&env)]),
    );

    // Step 2: Update config
    let new_value = 2000u64;
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_instance"),
        Vec::from_array(&env, [config_key.into_val(&env), new_value.into_val(&env)]),
    );

    // Step 3: Emit config change event
    env.invoke_contract::<()>(
        &events_id,
        &Symbol::new(&env, "update_config"),
        Vec::from_array(&env, [
            config_key.into_val(&env),
            old_value.into_val(&env),
            new_value.into_val(&env),
        ]),
    );

    // Step 4: Emit admin action
    env.invoke_contract::<()>(
        &events_id,
        &Symbol::new(&env, "admin_action"),
        Vec::from_array(&env, [admin.into_val(&env), symbol_short!("cfg_upd").into_val(&env)]),
    );

    // Step 5: Verify new config
    let updated_value: u64 = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_instance"),
        Vec::from_array(&env, [config_key.into_val(&env)]),
    );
    assert_eq!(updated_value, new_value);

    // Step 6: Store audit trail in persistent storage
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [symbol_short!("audit").into_val(&env), 1u64.into_val(&env)]),
    );

    let has_audit: bool = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "has_persistent"),
        Vec::from_array(&env, [symbol_short!("audit").into_val(&env)]),
    );
    assert!(has_audit);
}

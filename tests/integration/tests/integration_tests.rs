//! Integration Tests for Soroban Cookbook Basic Examples
//!
//! This test suite demonstrates cross-contract interactions and end-to-end
//! scenarios combining multiple basic examples.  Contracts are registered
//! natively (no WASM binary required) so the tests work with any Rust
//! toolchain without special build-time flags.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use multi_party_auth::MultiPartyAuthContract;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Test 1: Multi-Contract Workflow — Hello World + Storage + Events counter
// ---------------------------------------------------------------------------

#[test]
fn test_greeting_system_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let hello_id = env.register_contract(None, hello_world::HelloContract);
    let storage_id = env.register_contract(None, storage_patterns::StorageContract);
    let events_id = env.register_contract(None, events_counter::Contract);

    // Step 1: Generate greeting
    let greeting: Vec<Symbol> = env.invoke_contract(
        &hello_id,
        &symbol_short!("hello"),
        Vec::from_array(&env, [symbol_short!("Alice").into_val(&env)]),
    );
    assert_eq!(greeting.get(0).unwrap(), symbol_short!("Hello"));
    assert_eq!(greeting.get(1).unwrap(), symbol_short!("Alice"));

    // Step 2: Store greeting count in persistent storage
    let greeting_key = symbol_short!("greet_cnt");
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env), 1u64.into_val(&env)]),
    );

    let count: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env)]),
    );
    assert_eq!(count, Some(1));

    // Step 3: Use events counter to track greeting calls
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));

    let event_count: u32 =
        env.invoke_contract(&events_id, &Symbol::new(&env, "get_number"), Vec::new(&env));
    assert_eq!(event_count, 1);

    // Step 4: Increment greeting count
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env), 2u64.into_val(&env)]),
    );

    let new_count: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env)]),
    );
    assert_eq!(new_count, Some(2));

    let has_key: bool = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "has_persistent"),
        Vec::from_array(&env, [greeting_key.into_val(&env)]),
    );
    assert!(has_key);
}

// ---------------------------------------------------------------------------
// Test 2: Authentication + Storage Integration
// ---------------------------------------------------------------------------

#[test]
fn test_authenticated_storage_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let storage_id = env.register_contract(None, storage_patterns::StorageContract);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Step 1: Initialize authentication contract
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );

    // Step 2: Admin sets balances for both users
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                user1.clone().into_val(&env),
                500i128.into_val(&env),
            ],
        ),
    );
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                user2.clone().into_val(&env),
                300i128.into_val(&env),
            ],
        ),
    );

    // Step 3: Verify balances
    let bal1: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [user1.clone().into_val(&env)]),
    );
    assert_eq!(bal1, 500);

    let bal2: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [user2.clone().into_val(&env)]),
    );
    assert_eq!(bal2, 300);

    // Step 4: Each user stores their own metadata in storage contract
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

    // Step 5: Verify data isolation
    let user1_data: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [user1_key.into_val(&env)]),
    );
    assert_eq!(user1_data, Some(100));

    let user2_data: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [user2_key.into_val(&env)]),
    );
    assert_eq!(user2_data, Some(200));

    // Step 6: Perform auth transfer and verify updated balances
    env.invoke_contract::<()>(
        &auth_id,
        &symbol_short!("transfer"),
        Vec::from_array(
            &env,
            [
                user1.clone().into_val(&env),
                user2.clone().into_val(&env),
                100i128.into_val(&env),
            ],
        ),
    );

    let new_bal1: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [user1.into_val(&env)]),
    );
    assert_eq!(new_bal1, 400);

    let new_bal2: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [user2.into_val(&env)]),
    );
    assert_eq!(new_bal2, 400);
}

// ---------------------------------------------------------------------------
// Test 3: Cross-Contract Coordination — Auth + Events + Storage
// ---------------------------------------------------------------------------

#[test]
fn test_cross_contract_event_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let events_id = env.register_contract(None, events_counter::Contract);
    let storage_id = env.register_contract(None, storage_patterns::StorageContract);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Step 1: Initialize auth contract
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );

    // Step 2: Admin performs an action
    let action_result: u32 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "admin_action"),
        Vec::from_array(&env, [admin.clone().into_val(&env), 42u32.into_val(&env)]),
    );
    assert_eq!(action_result, 84); // admin_action returns value * 2

    // Step 3: Use events counter to track admin actions
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));

    // Step 4: Store configuration in instance storage
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_instance"),
        Vec::from_array(
            &env,
            [symbol_short!("config").into_val(&env), 42u64.into_val(&env)],
        ),
    );

    // Step 5: Increment event counter again for config change
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));

    // Step 6: Set user balance and emit event via auth
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                user.clone().into_val(&env),
                1000i128.into_val(&env),
            ],
        ),
    );
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "emit_event"),
        Vec::from_array(
            &env,
            [user.into_val(&env), symbol_short!("deposit").into_val(&env)],
        ),
    );

    // Verify storage state
    let config: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_instance"),
        Vec::from_array(&env, [symbol_short!("config").into_val(&env)]),
    );
    assert_eq!(config, Some(42));

    // Verify event counter
    let evt_count: u32 =
        env.invoke_contract(&events_id, &Symbol::new(&env, "get_number"), Vec::new(&env));
    assert_eq!(evt_count, 2);
}

// ---------------------------------------------------------------------------
// Test 4: Storage Type Comparison — End-to-End
// ---------------------------------------------------------------------------

#[test]
fn test_storage_types_comparison() {
    let env = Env::default();

    let storage_id = env.register_contract(None, storage_patterns::StorageContract);

    let key = symbol_short!("testkey");

    // Test 1: Persistent storage
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

    let pers_val: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(pers_val, Some(100));

    // Test 2: Temporary storage
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

    let temp_val: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_temporary"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(temp_val, Some(200));

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

    let inst_val: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_instance"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(inst_val, Some(300));

    // Test 4: All three storage types are independent
    let pers_check: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(pers_check, Some(100));

    let temp_check: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_temporary"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(temp_check, Some(200));

    // Test 5: Remove persistent
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

// ---------------------------------------------------------------------------
// Test 5: Complex Multi-Party Workflow
// ---------------------------------------------------------------------------

#[test]
fn test_multi_party_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let storage_id = env.register_contract(None, storage_patterns::StorageContract);
    let events_id = env.register_contract(None, events_counter::Contract);
    let hello_id = env.register_contract(None, hello_world::HelloContract);

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Step 1: Setup — initialize auth and set balances
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                alice.clone().into_val(&env),
                100i128.into_val(&env),
            ],
        ),
    );
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                bob.clone().into_val(&env),
                200i128.into_val(&env),
            ],
        ),
    );

    // Step 2: Alice gets greeted
    let alice_greeting: Vec<Symbol> = env.invoke_contract(
        &hello_id,
        &symbol_short!("hello"),
        Vec::from_array(&env, [symbol_short!("Alice").into_val(&env)]),
    );
    assert_eq!(alice_greeting.get(0).unwrap(), symbol_short!("Hello"));
    assert_eq!(alice_greeting.get(1).unwrap(), symbol_short!("Alice"));

    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(
            &env,
            [symbol_short!("alice").into_val(&env), 100u64.into_val(&env)],
        ),
    );

    // Step 3: Bob gets greeted
    let bob_greeting: Vec<Symbol> = env.invoke_contract(
        &hello_id,
        &symbol_short!("hello"),
        Vec::from_array(&env, [symbol_short!("Bob").into_val(&env)]),
    );
    assert_eq!(bob_greeting.get(0).unwrap(), symbol_short!("Hello"));
    assert_eq!(bob_greeting.get(1).unwrap(), symbol_short!("Bob"));

    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(
            &env,
            [symbol_short!("bob").into_val(&env), 200u64.into_val(&env)],
        ),
    );

    // Step 4: Track greetings via events counter (2 greetings)
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));
    let greet_count: u32 =
        env.invoke_contract(&events_id, &Symbol::new(&env, "get_number"), Vec::new(&env));
    assert_eq!(greet_count, 2);

    // Step 5: Alice transfers to Bob
    env.invoke_contract::<()>(
        &auth_id,
        &symbol_short!("transfer"),
        Vec::from_array(
            &env,
            [
                alice.clone().into_val(&env),
                bob.clone().into_val(&env),
                50i128.into_val(&env),
            ],
        ),
    );

    // Step 6: Verify final balances
    let final_alice_bal: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [alice.into_val(&env)]),
    );
    assert_eq!(final_alice_bal, 50);

    let final_bob_bal: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [bob.into_val(&env)]),
    );
    assert_eq!(final_bob_bal, 250);

    let alice_meta: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [symbol_short!("alice").into_val(&env)]),
    );
    assert_eq!(alice_meta, Some(100));

    let bob_meta: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [symbol_short!("bob").into_val(&env)]),
    );
    assert_eq!(bob_meta, Some(200));
}

// ---------------------------------------------------------------------------
// Test 6: Coordinated State Management
// ---------------------------------------------------------------------------

#[test]
fn test_coordinated_state_management() {
    let env = Env::default();
    env.mock_all_auths();

    let storage_id = env.register_contract(None, storage_patterns::StorageContract);
    let events_id = env.register_contract(None, events_counter::Contract);

    // Step 1: Store initial config
    let config_key = symbol_short!("max_val");
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_instance"),
        Vec::from_array(&env, [config_key.into_val(&env), 1000u64.into_val(&env)]),
    );

    let old_value: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_instance"),
        Vec::from_array(&env, [config_key.into_val(&env)]),
    );
    assert_eq!(old_value, Some(1000));

    // Step 2: Update config
    let new_value = 2000u64;
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_instance"),
        Vec::from_array(&env, [config_key.into_val(&env), new_value.into_val(&env)]),
    );

    // Step 3: Track config changes via events counter
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));

    // Step 4: Verify config updated
    let updated_value: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_instance"),
        Vec::from_array(&env, [config_key.into_val(&env)]),
    );
    assert_eq!(updated_value, Some(new_value));

    // Step 5: Store audit trail in persistent storage
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(
            &env,
            [symbol_short!("audit").into_val(&env), 1u64.into_val(&env)],
        ),
    );
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));

    let has_audit: bool = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "has_persistent"),
        Vec::from_array(&env, [symbol_short!("audit").into_val(&env)]),
    );
    assert!(has_audit);

    // Step 6: Use temporary storage for in-flight data
    let tx_key = symbol_short!("pending");
    env.invoke_contract::<()>(
        &storage_id,
        &Symbol::new(&env, "set_temporary"),
        Vec::from_array(&env, [tx_key.into_val(&env), 999u64.into_val(&env)]),
    );
    let pending: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_temporary"),
        Vec::from_array(&env, [tx_key.into_val(&env)]),
    );
    assert_eq!(pending, Some(999));

    let evt_count: u32 =
        env.invoke_contract(&events_id, &Symbol::new(&env, "get_number"), Vec::new(&env));
    assert_eq!(evt_count, 2);
}

// ---------------------------------------------------------------------------
// Multi-Party Authorization Integration Tests
//
// Setup overview:
//   - Three signers (signer1, signer2, signer3) are generated for each test.
//   - `setup_proposal` stores the threshold and the valid-signer list on-chain.
//   - `env.mock_all_auths()` lets the test environment satisfy every
//     `require_auth()` call automatically, so we can focus on threshold logic.
//   - `env.auths()` is used to assert exactly which addresses were required
//     to authorize a given call.
//   - For cross-function auth checks a fresh `env.auths()` snapshot is taken
//     after each call to confirm auth state is not shared between functions.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Test 7: 2-of-3 threshold — exactly two signers meet the threshold
// ---------------------------------------------------------------------------

/// Verifies that a proposal configured with a 2-of-3 threshold passes when
/// exactly two valid signers authorize the call.
#[test]
fn test_integration_two_of_three_auth_passes() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);

    let all_signers =
        Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "integ_2of3");

    // Configure the proposal: threshold = 2, valid signers = all three.
    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "setup_proposal"),
        Vec::from_array(
            &env,
            [
                proposal_id.clone().into_val(&env),
                2u32.into_val(&env),
                all_signers.into_val(&env),
            ],
        ),
    );

    // Only signer1 and signer3 approve — threshold of 2 is met.
    let approvers = Vec::from_array(&env, [signer1.clone(), signer3.clone()]);

    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "proposal_approval"),
        Vec::from_array(
            &env,
            [
                proposal_id.clone().into_val(&env),
                approvers.clone().into_val(&env),
            ],
        ),
    );

    // Both approvers must have been required to authorize this call.
    let auths = env.auths();
    assert!(auths.iter().any(|(addr, _)| addr == signer1));
    assert!(auths.iter().any(|(addr, _)| addr == signer3));
    // signer2 was NOT required (only 2-of-3 needed).
    assert!(!auths.iter().any(|(addr, _)| addr == signer2));
}

/// Verifies that a 2-of-3 proposal panics when only one signer approves.
#[test]
#[should_panic(expected = "Threshold not met")]
fn test_integration_two_of_three_auth_fails_below_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);

    let all_signers =
        Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "integ_2of3f");

    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "setup_proposal"),
        Vec::from_array(
            &env,
            [
                proposal_id.clone().into_val(&env),
                2u32.into_val(&env),
                all_signers.into_val(&env),
            ],
        ),
    );

    // Only one signer — below the threshold of 2.
    let approvers = Vec::from_array(&env, [signer1.clone()]);

    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "proposal_approval"),
        Vec::from_array(
            &env,
            [
                proposal_id.into_val(&env),
                approvers.into_val(&env),
            ],
        ),
    );
}

// ---------------------------------------------------------------------------
// Test 8: 3-of-3 threshold — all three signers must authorize
// ---------------------------------------------------------------------------

/// Verifies that a 3-of-3 proposal passes only when all three signers approve.
#[test]
fn test_integration_three_of_three_auth_passes() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);

    let all_signers =
        Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "integ_3of3");

    // Threshold = 3: every signer must approve.
    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "setup_proposal"),
        Vec::from_array(
            &env,
            [
                proposal_id.clone().into_val(&env),
                3u32.into_val(&env),
                all_signers.clone().into_val(&env),
            ],
        ),
    );

    let approvers = Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);

    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "proposal_approval"),
        Vec::from_array(
            &env,
            [
                proposal_id.clone().into_val(&env),
                approvers.clone().into_val(&env),
            ],
        ),
    );

    // All three must appear in the auth record.
    let auths = env.auths();
    assert_eq!(
        auths,
        std::vec![
            (
                signer1.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        Symbol::new(&env, "proposal_approval"),
                        (proposal_id.clone(), approvers.clone()).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                }
            ),
            (
                signer2.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        Symbol::new(&env, "proposal_approval"),
                        (proposal_id.clone(), approvers.clone()).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                }
            ),
            (
                signer3.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        Symbol::new(&env, "proposal_approval"),
                        (proposal_id.clone(), approvers.clone()).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                }
            ),
        ]
    );
}

/// Verifies that a 3-of-3 proposal panics when only two of three signers approve.
#[test]
#[should_panic(expected = "Threshold not met")]
fn test_integration_three_of_three_auth_fails_when_one_missing() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);

    let all_signers =
        Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "integ_3of3f");

    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "setup_proposal"),
        Vec::from_array(
            &env,
            [
                proposal_id.clone().into_val(&env),
                3u32.into_val(&env),
                all_signers.into_val(&env),
            ],
        ),
    );

    // signer3 is absent — threshold of 3 is not met.
    let approvers = Vec::from_array(&env, [signer1.clone(), signer2.clone()]);

    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "proposal_approval"),
        Vec::from_array(
            &env,
            [
                proposal_id.into_val(&env),
                approvers.into_val(&env),
            ],
        ),
    );
}

// ---------------------------------------------------------------------------
// Test 9: Cross-function auth isolation
//
// Authorization granted for one function must NOT carry over to a different
// function that requires its own independent authorization.  Each call to
// `env.auths()` returns only the authorizations recorded since the last
// snapshot, so we verify that each function's auth set is distinct.
// ---------------------------------------------------------------------------

/// Verifies that `multi_sig_transfer` and `proposal_approval` each require
/// their own independent authorization and do not share auth state.
#[test]
fn test_integration_cross_function_auth_isolation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let recipient = Address::generate(&env);

    let all_signers =
        Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "integ_xfn");

    // Setup a 2-of-3 proposal.
    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "setup_proposal"),
        Vec::from_array(
            &env,
            [
                proposal_id.clone().into_val(&env),
                2u32.into_val(&env),
                all_signers.clone().into_val(&env),
            ],
        ),
    );

    // --- Call 1: multi_sig_transfer (requires ALL three signers) ---
    let transfer_signers =
        Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);

    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "multi_sig_transfer"),
        Vec::from_array(
            &env,
            [
                transfer_signers.clone().into_val(&env),
                recipient.clone().into_val(&env),
                50i128.into_val(&env),
            ],
        ),
    );

    // Snapshot auth after the transfer call: all three signers required.
    let transfer_auths = env.auths();
    assert_eq!(transfer_auths.len(), 3);
    assert!(transfer_auths.iter().any(|(a, _)| a == signer1));
    assert!(transfer_auths.iter().any(|(a, _)| a == signer2));
    assert!(transfer_auths.iter().any(|(a, _)| a == signer3));
    // Confirm every auth entry names `multi_sig_transfer`.
    for (_, inv) in &transfer_auths {
        if let AuthorizedFunction::Contract((_, fn_name, _)) = &inv.function {
            assert_eq!(fn_name, &Symbol::new(&env, "multi_sig_transfer"));
        }
    }

    // --- Call 2: proposal_approval (requires only 2-of-3) ---
    // Auth state is reset between calls; signer3 is intentionally absent.
    let approvers = Vec::from_array(&env, [signer1.clone(), signer2.clone()]);

    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "proposal_approval"),
        Vec::from_array(
            &env,
            [
                proposal_id.clone().into_val(&env),
                approvers.clone().into_val(&env),
            ],
        ),
    );

    // Snapshot auth after the approval call: only two signers required.
    let approval_auths = env.auths();
    assert_eq!(approval_auths.len(), 2);
    assert!(approval_auths.iter().any(|(a, _)| a == signer1));
    assert!(approval_auths.iter().any(|(a, _)| a == signer2));
    // signer3 was NOT required for proposal_approval.
    assert!(!approval_auths.iter().any(|(a, _)| a == signer3));
    // Confirm every auth entry names `proposal_approval`.
    for (_, inv) in &approval_auths {
        if let AuthorizedFunction::Contract((_, fn_name, _)) = &inv.function {
            assert_eq!(fn_name, &Symbol::new(&env, "proposal_approval"));
        }
    }
}

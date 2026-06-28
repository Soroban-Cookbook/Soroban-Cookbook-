//! Integration Tests for Soroban Cookbook Advanced Examples
//!
//! This test suite demonstrates cross-contract interactions and end-to-end
//! scenarios combining multiple examples.  Contracts are registered
//! natively (no WASM binary required) so the tests work with any Rust
//! toolchain without special build-time flags.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, BytesN, Env, IntoVal, Symbol, Vec};

// ---------------------------------------------------------------------------
// Test 1: Multi-Contract Workflow — Hello World + Storage + Events counter
// ---------------------------------------------------------------------------

#[test]
fn test_greeting_system_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let hello_id = env.register_contract(None, hello_world::HelloContract);
    let storage_id = env.register_contract(None, storage_patterns::StorageContract);
    let events_id = env.register_contract(None, events_structured::EventsContract);

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
// Test 5: Validation + Custom Errors Integration
// ---------------------------------------------------------------------------

#[test]
fn test_validation_and_errors_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let validation_id = env.register_contract(None, validation_patterns::ValidationContract);
    let errors_id = env.register_contract(None, custom_errors::CustomErrorsContract);

    let owner = Address::generate(&env);

    // Step 1: Initialize validation contract
    let _: Result<(), soroban_validation::ValidationError> = env.invoke_contract(
        &validation_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [owner.clone().into_val(&env)]),
    );

    // Step 2: Test validation parameters (Success)
    let _: Result<(), soroban_validation::ValidationError> = env.invoke_contract(
        &validation_id,
        &Symbol::new(&env, "validate_amount_parameters"),
        Vec::from_array(
            &env,
            [
                100i128.into_val(&env),
                50i128.into_val(&env),
                200i128.into_val(&env),
            ],
        ),
    );

    // Step 3: Test custom errors (Failure)
    let errors_client = custom_errors::CustomErrorsContractClient::new(&env, &errors_id);
    let error_result = errors_client.try_validate_input(&0i64);
    assert_eq!(
        error_result,
        Err(Ok(custom_errors::ContractError::InvalidInput))
    );
}

// ---------------------------------------------------------------------------
// Test 6: Ajo Factory + Authentication Lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_ajo_factory_lifecycle_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let factory_id = env.register_contract(None, ajo_factory::AjoFactory);
    let auth_id = env.register_contract(None, authentication::AuthContract);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    // Step 1: Initialize auth contract
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );

    // Step 2: Initialize Ajo Factory (wasm hash placeholder — deploy tested in WASM CI build)
    let wasm_hash = soroban_sdk::BytesN::from_array(&env, &[2u8; 32]);

    env.invoke_contract::<Result<(), ajo_factory::FactoryError>>(
        &factory_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [wasm_hash.into_val(&env)]),
    )
    .unwrap();

    // Step 3: Register Ajo template natively and verify auth + factory state
    let ajo_id = env.register_contract(None, ajo_factory::Ajo);
    env.invoke_contract::<Result<(), ajo::AjoError>>(
        &ajo_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(
            &env,
            [
                1000i128.into_val(&env),
                10u32.into_val(&env),
                creator.clone().into_val(&env),
            ],
        ),
    )
    .unwrap();

    // Step 4: Verify factory initialized and auth contract is active
    let deployed_ajos: Vec<Address> = env.invoke_contract(
        &factory_id,
        &Symbol::new(&env, "get_deployed_ajos"),
        Vec::new(&env),
    );
    assert_eq!(deployed_ajos.len(), 0);

    let admin_bal: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [admin.into_val(&env)]),
    );
    assert_eq!(admin_bal, 0);
}

// ---------------------------------------------------------------------------
// Test 7: Multi-Sig Governance + Events Tracking
// ---------------------------------------------------------------------------

#[test]
fn test_multi_sig_governance_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let events_id = env.register_contract(None, events_structured::EventsContract);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone(), signer2.clone()]);

    // Step 1: Initialize multi-sig
    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [2u32.into_val(&env), signers.into_val(&env)]),
    )
    .unwrap();

    // Step 2: Create a proposal
    let proposal_id: u32 = env
        .invoke_contract::<Result<u32, multi_sig_patterns::AuthError>>(
            &multisig_id,
            &Symbol::new(&env, "create_proposal"),
            Vec::from_array(&env, [signer1.clone().into_val(&env)]),
        )
        .unwrap();

    // Step 3: Track governance action via events counter
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));

    // Step 4: Approve from both signers
    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [proposal_id.into_val(&env), signer1.clone().into_val(&env)],
        ),
    )
    .unwrap();
    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [proposal_id.into_val(&env), signer2.clone().into_val(&env)],
        ),
    )
    .unwrap();

    // Step 5: Execute
    let success: bool = env
        .invoke_contract::<Result<bool, multi_sig_patterns::AuthError>>(
            &multisig_id,
            &Symbol::new(&env, "execute"),
            Vec::from_array(&env, [proposal_id.into_val(&env), signer1.into_val(&env)]),
        )
        .unwrap();
    assert!(success);

    // Verify events tracking
    let evt_count: u32 =
        env.invoke_contract(&events_id, &Symbol::new(&env, "get_number"), Vec::new(&env));
    assert_eq!(evt_count, 1);
}

// ---------------------------------------------------------------------------
// Test 3: Cross-Contract Coordination — Auth + Events + Storage
// ---------------------------------------------------------------------------

#[test]
fn test_cross_contract_event_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let events_id = env.register_contract(None, events_structured::EventsContract);
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
    let events_id = env.register_contract(None, events_structured::EventsContract);
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
    let events_id = env.register_contract(None, events_structured::EventsContract);

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
// Test 7: Multi-Party Auth — 2-of-3 proposal approval
// ---------------------------------------------------------------------------

#[test]
fn test_multi_party_auth_2_of_3() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, multi_party_auth::MultiPartyAuthContract);
    let client = multi_party_auth::MultiPartyAuthContractClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);

    let all_signers =
        soroban_sdk::Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "prop_2of3");

    // Setup 2-of-3 threshold
    client.setup_proposal(&proposal_id, &2u32, &all_signers);

    // Only signer1 and signer2 approve — threshold met
    let approvers = soroban_sdk::Vec::from_array(&env, [signer1.clone(), signer2.clone()]);
    client.proposal_approval(&proposal_id, &approvers);

    // Verify both signers were required to authorize
    let auths = env.auths();
    let auth_addresses: std::vec::Vec<Address> =
        auths.iter().map(|(addr, _)| addr.clone()).collect();
    assert!(auth_addresses.contains(&signer1));
    assert!(auth_addresses.contains(&signer2));
    assert!(!auth_addresses.contains(&signer3));
}

// ---------------------------------------------------------------------------
// Test 8: Multi-Party Auth — 3-of-3 proposal approval
// ---------------------------------------------------------------------------

#[test]
fn test_multi_party_auth_3_of_3() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, multi_party_auth::MultiPartyAuthContract);
    let client = multi_party_auth::MultiPartyAuthContractClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);

    let all_signers =
        soroban_sdk::Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "prop_3of3");

    // Setup 3-of-3 threshold — all must approve
    client.setup_proposal(&proposal_id, &3u32, &all_signers);

    let approvers =
        soroban_sdk::Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    client.proposal_approval(&proposal_id, &approvers);

    let auths = env.auths();
    let auth_addresses: std::vec::Vec<Address> =
        auths.iter().map(|(addr, _)| addr.clone()).collect();
    assert!(auth_addresses.contains(&signer1));
    assert!(auth_addresses.contains(&signer2));
    assert!(auth_addresses.contains(&signer3));
}

// ---------------------------------------------------------------------------
// Test 9: Multi-Party Auth — cross-function auth check (escrow + proposal)
// ---------------------------------------------------------------------------

#[test]
fn test_multi_party_auth_cross_function() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, multi_party_auth::MultiPartyAuthContract);
    let client = multi_party_auth::MultiPartyAuthContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let signer3 = Address::generate(&env);

    // --- Escrow flow ---
    // Step 1: buyer funds escrow (requires buyer auth)
    client.sequential_auth_escrow(&buyer, &seller, &500i128);

    let step_key = multi_party_auth::DataKey::EscrowStep(buyer.clone(), seller.clone());
    let step: u32 = env.as_contract(&contract_id, || {
        env.storage().instance().get(&step_key).unwrap_or(0)
    });
    assert_eq!(step, 2);

    // Step 2: joint release (requires both buyer and seller auth)
    client.sequential_auth_escrow(&buyer, &seller, &500i128);

    let step_after: u32 = env.as_contract(&contract_id, || {
        env.storage().instance().get(&step_key).unwrap_or(0)
    });
    assert_eq!(step_after, 0);

    // --- Proposal flow on the same contract instance ---
    let all_signers =
        soroban_sdk::Vec::from_array(&env, [buyer.clone(), seller.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "cross_prop");

    client.setup_proposal(&proposal_id, &2u32, &all_signers);

    // buyer and seller (who just completed escrow) now co-approve a proposal
    let approvers = soroban_sdk::Vec::from_array(&env, [buyer.clone(), seller.clone()]);
    client.proposal_approval(&proposal_id, &approvers);

    let auths = env.auths();
    let auth_addresses: std::vec::Vec<Address> =
        auths.iter().map(|(addr, _)| addr.clone()).collect();
    assert!(auth_addresses.contains(&buyer));
    assert!(auth_addresses.contains(&seller));
}

// ---------------------------------------------------------------------------
// Oracle Pattern Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_oracle_basic_operation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let updater = Address::generate(&env);
    let oracle_id = env.register_contract(None, oracle_pattern::OracleContract);
    let client = oracle_pattern::OracleContractClient::new(&env, &oracle_id);

    client.initialize(&admin, &updater, &3600); // 1 hour max age

    client.submit(&updater, &1500);
    let value = client.get_value().unwrap();
    assert_eq!(value, 1500);
}

#[test]
fn test_oracle_strict_mode_fresh() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let updater = Address::generate(&env);
    let oracle_id = env.register_contract(None, oracle_pattern::OracleContract);
    let client = oracle_pattern::OracleContractClient::new(&env, &oracle_id);

    client.initialize(&admin, &updater, &3600);

    env.ledger().set_timestamp(1000);
    client.submit(&updater, &2000);

    env.ledger().set_timestamp(2000); // Still fresh
    let value = client.get_value_strict().unwrap();
    assert_eq!(value, 2000);
}

#[test]
fn test_oracle_rotate_updater() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let updater1 = Address::generate(&env);
    let updater2 = Address::generate(&env);
    let oracle_id = env.register_contract(None, oracle_pattern::OracleContract);
    let client = oracle_pattern::OracleContractClient::new(&env, &oracle_id);

    client.initialize(&admin, &updater1, &3600);
    client.set_updater(&admin, &updater2);

    client.submit(&updater2, &3000);
    let value = client.get_value().unwrap();
    assert_eq!(value, 3000);
}

// ---------------------------------------------------------------------------
// Timelock Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_timelock_queue_and_execute() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let timelock_id = env.register_contract(None, timelock::TimelockContract);
    let client = timelock::TimelockContractClient::new(&env, &timelock_id);

    client.initialize(&admin);

    let op_id = Bytes::from_slice(&env, b"test_op_1");
    client.queue(&op_id, &60); // 60s delay

    let state = client.get_state(&op_id);
    assert_eq!(state, timelock::OperationState::Pending);

    env.ledger().set_timestamp(env.ledger().timestamp() + 100);
    let state_after = client.get_state(&op_id);
    assert_eq!(state_after, timelock::OperationState::Ready);

    client.execute(&op_id);
    let state_final = client.get_state(&op_id);
    assert_eq!(state_final, timelock::OperationState::Unknown);
}

#[test]
fn test_timelock_cancel_operation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let timelock_id = env.register_contract(None, timelock::TimelockContract);
    let client = timelock::TimelockContractClient::new(&env, &timelock_id);

    client.initialize(&admin);

    let op_id = Bytes::from_slice(&env, b"test_op_cancel");
    client.queue(&op_id, &60);
    client.cancel(&op_id);

    let state = client.get_state(&op_id);
    assert_eq!(state, timelock::OperationState::Unknown);
}

#[test]
fn test_timelock_pause_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let timelock_id = env.register_contract(None, timelock::TimelockContract);
    let client = timelock::TimelockContractClient::new(&env, &timelock_id);

    client.initialize(&admin);
    assert!(!client.is_paused());

    client.set_pause(&true);
    assert!(client.is_paused());

    client.set_pause(&false);
    assert!(!client.is_paused());
}

// ---------------------------------------------------------------------------
// Proxy Admin Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_proxy_admin_proposal_state() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proxy_admin_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let client = proxy_admin::ProxyAdminClient::new(&env, &proxy_admin_id);

    client.initialize(&admin).unwrap();

    let wasm_hash = BytesN::from_array(&env, &[0x42; 32]);
    client.propose_upgrade(&wasm_hash, &60).unwrap();

    let state = client.proposal_state();
    assert_eq!(state, proxy_admin::ProposalState::Pending);

    env.ledger().set_timestamp(env.ledger().timestamp() + 100);
    let state_after = client.proposal_state();
    assert_eq!(state_after, proxy_admin::ProposalState::Ready);
}

#[test]
fn test_proxy_admin_cancel_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proxy_admin_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let client = proxy_admin::ProxyAdminClient::new(&env, &proxy_admin_id);

    client.initialize(&admin).unwrap();

    let wasm_hash = BytesN::from_array(&env, &[0x42; 32]);
    client.propose_upgrade(&wasm_hash, &60).unwrap();
    client.cancel_upgrade().unwrap();

    let state = client.proposal_state();
    assert_eq!(state, proxy_admin::ProposalState::None);
}

#[test]
fn test_proxy_admin_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proxy_admin_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let client = proxy_admin::ProxyAdminClient::new(&env, &proxy_admin_id);

    client.initialize(&admin).unwrap();
    assert!(!client.is_paused());

    client.pause().unwrap();
    assert!(client.is_paused());

    client.unpause().unwrap();
    assert!(!client.is_paused());
}

// ---------------------------------------------------------------------------
// RBAC Modifiers Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_rbac_grant_and_revoke_role() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let rbac_id = env.register_contract(None, rbac_modifiers::RbacContract);
    let client = rbac_modifiers::RbacContractClient::new(&env, &rbac_id);

    client.initialize(&admin);

    client.grant_role(&admin, &rbac_modifiers::ROLE_MINTER, &user);
    let has_role = client.has_role(&rbac_modifiers::ROLE_MINTER, &user);
    assert!(has_role);

    client.revoke_role(&admin, &rbac_modifiers::ROLE_MINTER, &user);
    let has_role_after = client.has_role(&rbac_modifiers::ROLE_MINTER, &user);
    assert!(!has_role_after);
}

#[test]
fn test_rbac_protected_operations() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rbac_id = env.register_contract(None, rbac_modifiers::RbacContract);
    let client = rbac_modifiers::RbacContractClient::new(&env, &rbac_id);

    client.initialize(&admin);
    client.grant_role(&admin, &rbac_modifiers::ROLE_MINTER, &minter);
    client.grant_role(&admin, &rbac_modifiers::ROLE_PAUSER, &pauser);

    client.protected_mint(&minter, &Address::generate(&env), &100);
    client.pause(&pauser);
    assert!(client.is_paused());

    client.unpause(&pauser);
    assert!(!client.is_paused());
}

#[test]
fn test_rbac_renounce_role() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let rbac_id = env.register_contract(None, rbac_modifiers::RbacContract);
    let client = rbac_modifiers::RbacContractClient::new(&env, &rbac_id);

    client.initialize(&admin);
    client.grant_role(&admin, &rbac_modifiers::ROLE_MINTER, &user);
    client.renounce_role(&user, &rbac_modifiers::ROLE_MINTER);

    let has_role = client.has_role(&rbac_modifiers::ROLE_MINTER, &user);
    assert!(!has_role);
}

// ---------------------------------------------------------------------------
// Registry Access Controls Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_registry_access_basic_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let user = Address::generate(&env);
    let registry_id = env.register_contract(None, registry_access_controls::RegistryContract);
    let client = registry_access_controls::RegistryContractClient::new(&env, &registry_id);

    client.init(&owner, &false, &100);
    client.register(&user, &100);
    assert!(client.is_registered(&user));
}

#[test]
fn test_registry_whitelist_mode() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let registry_id = env.register_contract(None, registry_access_controls::RegistryContract);
    let client = registry_access_controls::RegistryContractClient::new(&env, &registry_id);

    client.init(&owner, &true, &0);
    client.add_whitelist(&user1);

    client.register(&user1, &0);
    assert!(client.is_registered(&user1));
}

#[test]
fn test_registry_removal_request() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let reporter = Address::generate(&env);
    let target = Address::generate(&env);
    let registry_id = env.register_contract(None, registry_access_controls::RegistryContract);
    let client = registry_access_controls::RegistryContractClient::new(&env, &registry_id);

    client.init(&owner, &false, &0);
    client.register(&target, &0);

    client.request_removal(&reporter, &target, &symbol_short!("fraud"));
    client.resolve_removal(&target, &true);

    assert!(!client.is_registered(&target));
}

// ---------------------------------------------------------------------------
// Contract Registry Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_contract_registry_register() {
    let env = Env::default();
    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let client = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    let name = symbol_short!("test_contract");
    let category = symbol_short!("defi");
    let version = symbol_short!("v1.0");
    let addr = Address::generate(&env);

    client.register(&name, &category, &version, &addr).unwrap();

    let metadata = client.get_by_name(&name).unwrap();
    assert_eq!(metadata.name, name);
    assert_eq!(metadata.category, category);
}

#[test]
fn test_contract_registry_list_categories() {
    let env = Env::default();
    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let client = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    let cat1 = symbol_short!("defi");
    let cat2 = symbol_short!("nft");

    client.register(&symbol_short!("c1"), &cat1, &symbol_short!("v1"), &Address::generate(&env)).unwrap();
    client.register(&symbol_short!("c2"), &cat2, &symbol_short!("v1"), &Address::generate(&env)).unwrap();

    let cats = client.list_categories();
    assert_eq!(cats.len(), 2);
}

#[test]
fn test_contract_registry_list_by_category() {
    let env = Env::default();
    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let client = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    let cat = symbol_short!("defi");

    client.register(&symbol_short!("c1"), &cat, &symbol_short!("v1"), &Address::generate(&env)).unwrap();
    client.register(&symbol_short!("c2"), &cat, &symbol_short!("v1"), &Address::generate(&env)).unwrap();

    let names = client.list_by_category(&cat);
    assert_eq!(names.len(), 2);
}

// ---------------------------------------------------------------------------
// Upgradeable Proxy Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_upgradeable_proxy_basic_forwarding() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proxy_id = env.register_contract(None, upgradeable_proxy::ProxyContract);
    let client = upgradeable_proxy::ProxyContractClient::new(&env, &proxy_id);

    // Use the same proxy contract as implementation for testing
    client.init(&admin, &proxy_id);

    let sum = client.add(&100, &200);
    assert_eq!(sum, 300);

    let diff = client.subtract(&500, &200);
    assert_eq!(diff, 300);
}

#[test]
fn test_upgradeable_proxy_get_implementation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let impl_addr = Address::generate(&env);
    let proxy_id = env.register_contract(None, upgradeable_proxy::ProxyContract);
    let client = upgradeable_proxy::ProxyContractClient::new(&env, &proxy_id);

    client.init(&admin, &impl_addr);
    let stored_impl = client.get_implementation();
    assert_eq!(stored_impl, impl_addr);
}

// ---------------------------------------------------------------------------
// Cross-Contract Integration Testing (Factory + Registry)
// ---------------------------------------------------------------------------

#[test]
fn test_factory_and_registry_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let reg_id = env.register_contract(None, cross_contract_integration_testing::Registry);
    let factory_id = env.register_contract(None, cross_contract_integration_testing::Factory);
    let reg_client = cross_contract_integration_testing::RegistryClient::new(&env, &reg_id);
    let factory_client = cross_contract_integration_testing::FactoryClient::new(&env, &factory_id);

    let wasm_hash = BytesN::from_array(&env, &[0x11; 32]);
    factory_client.initialize(&wasm_hash, &reg_id);

    let creator = Address::generate(&env);
    let deployed_addr = factory_client.create_instance(&42, &symbol_short!("my_contract"), &creator);
    let lookup = reg_client.lookup(&symbol_short!("my_contract"));

    assert_eq!(lookup, Some(deployed_addr));
}

#[test]
fn test_multi_oracle_integration() {
    let env = Env::default();
    env.mock_all_auths();

    // Create two oracles
    let admin = Address::generate(&env);
    let updater = Address::generate(&env);

    let oracle1_id = env.register_contract(None, oracle_pattern::OracleContract);
    let oracle2_id = env.register_contract(None, oracle_pattern::OracleContract);
    let client1 = oracle_pattern::OracleContractClient::new(&env, &oracle1_id);
    let client2 = oracle_pattern::OracleContractClient::new(&env, &oracle2_id);

    client1.initialize(&admin, &updater, &3600);
    client2.initialize(&admin, &updater, &3600);

    client1.submit(&updater, &100);
    client2.submit(&updater, &102);

    let v1 = client1.get_value().unwrap();
    let v2 = client2.get_value().unwrap();
    let avg = (v1 + v2) / 2;
    assert_eq!(avg, 101);
}

#[test]
fn test_timelock_and_rbac_integration() {
    let env = Env::default();
    env.mock_all_auths();

    // Set up both
    let admin = Address::generate(&env);
    let timelock_id = env.register_contract(None, timelock::TimelockContract);
    let rbac_id = env.register_contract(None, rbac_modifiers::RbacContract);
    let timelock_client = timelock::TimelockContractClient::new(&env, &timelock_id);
    let rbac_client = rbac_modifiers::RbacContractClient::new(&env, &rbac_id);

    timelock_client.initialize(&admin);
    rbac_client.initialize(&admin);

    // Queue an "action"
    let op_id = Bytes::from_slice(&env, b"grant_role_op");
    timelock_client.queue(&op_id, &60);

    env.ledger().set_timestamp(env.ledger().timestamp() + 100);
    timelock_client.execute(&op_id);

    // Then grant role via RBAC
    let user = Address::generate(&env);
    rbac_client.grant_role(&admin, &rbac_modifiers::ROLE_MINTER, &user);
    assert!(rbac_client.has_role(&rbac_modifiers::ROLE_MINTER, &user));
}

#[test]
fn test_registry_and_oracle_integration() {
    let env = Env::default();
    env.mock_all_auths();

    // Registry for contracts, oracle for price
    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let oracle_id = env.register_contract(None, oracle_pattern::OracleContract);
    let reg_client = contract_registry::ContractRegistryClient::new(&env, &reg_id);
    let oracle_client = oracle_pattern::OracleContractClient::new(&env, &oracle_id);

    let admin = Address::generate(&env);
    let updater = Address::generate(&env);
    oracle_client.initialize(&admin, &updater, &3600).unwrap();

    // Register oracle in contract registry
    reg_client.register(&symbol_short!("price_oracle"), &symbol_short!("oracles"), &symbol_short!("v1"), &oracle_id).unwrap();

    // Submit price to oracle
    oracle_client.submit(&updater, &50000).unwrap();

    // Verify both
    let registered = reg_client.get_by_name(&symbol_short!("price_oracle")).unwrap();
    assert_eq!(registered.address, oracle_id);
    let price = oracle_client.get_value().unwrap();
    assert_eq!(price, 50000);
}


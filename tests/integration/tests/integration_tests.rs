//! Integration Tests for Soroban Cookbook Basic Examples
//!
//! This test suite demonstrates cross-contract interactions and end-to-end
//! scenarios combining multiple basic examples.  Contracts are registered
//! natively (no WASM binary required) so the tests work with any Rust
//! toolchain without special build-time flags.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{
    symbol_short, testutils::Address as _, testutils::Ledger as _, Address, Env, IntoVal, Symbol,
    Vec,
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
// Test 10: Timelock lifecycle — queue, wait, execute, cancel
// ---------------------------------------------------------------------------

#[test]
fn test_timelock_queue_and_execute() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, timelock::TimelockContract);
    let client = timelock::TimelockContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let op_id = soroban_sdk::Bytes::from_array(&env, &[1u8; 32]);
    // delay = 60 s (minimum)
    client.queue(&op_id, &60u64);

    // Verify pending
    let state = client.get_state(&op_id);
    assert_eq!(state, timelock::OperationState::Pending);

    // Advance time past the delay
    env.ledger().with_mut(|l| l.timestamp += 70);
    let state_after = client.get_state(&op_id);
    assert_eq!(state_after, timelock::OperationState::Ready);

    client.execute(&op_id);

    // After execution the key is removed — state is Unknown
    let final_state = client.get_state(&op_id);
    assert_eq!(final_state, timelock::OperationState::Unknown);
}

#[test]
fn test_timelock_cancel() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, timelock::TimelockContract);
    let client = timelock::TimelockContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let op_id = soroban_sdk::Bytes::from_array(&env, &[2u8; 32]);
    client.queue(&op_id, &60u64);

    // Cancel before it's ready
    client.cancel(&op_id);
    let state = client.get_state(&op_id);
    assert_eq!(state, timelock::OperationState::Unknown);
}

// ---------------------------------------------------------------------------
// Test 12: Error handling — deposit/withdraw lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_error_handling_deposit_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, error_handling::ErrorDemoContract);
    let client = error_handling::ErrorDemoContractClient::new(&env, &id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);

    // Deposit
    let new_bal = client.deposit(&user, &500i128);
    assert_eq!(new_bal, 500);
    assert_eq!(client.balance(&user), 500);

    // Withdraw partial
    let after_withdraw = client.withdraw(&user, &200i128);
    assert_eq!(after_withdraw, 300);
    assert_eq!(client.balance(&user), 300);

    // Withdraw too much — typed error
    let err = client.try_withdraw(&user, &999i128);
    assert_eq!(
        err,
        Err(Ok(error_handling::ContractError::InsufficientBalance))
    );
}

#[test]
fn test_error_handling_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, error_handling::ErrorDemoContract);
    let client = error_handling::ErrorDemoContractClient::new(&env, &id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin);

    let err = client.try_deposit(&user, &0i128);
    assert_eq!(err, Err(Ok(error_handling::ContractError::ZeroAmount)));
}

#[test]
fn test_error_handling_paused_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, error_handling::ErrorDemoContract);
    let client = error_handling::ErrorDemoContractClient::new(&env, &id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin);

    client.pause(&admin);
    assert!(client.is_paused());

    let err = client.try_deposit(&user, &100i128);
    assert_eq!(err, Err(Ok(error_handling::ContractError::ContractPaused)));

    client.unpause(&admin);
    assert!(!client.is_paused());

    // Deposit works after unpause
    let bal = client.deposit(&user, &100i128);
    assert_eq!(bal, 100);
}

// ---------------------------------------------------------------------------
// Test 15: Multi-user balance management — 5-user scenario
// ---------------------------------------------------------------------------

#[test]
fn test_multi_user_balance_management() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let admin = Address::generate(&env);
    let users: [Address; 5] = core::array::from_fn(|_| Address::generate(&env));

    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );

    // Fund each user
    for (i, user) in users.iter().enumerate() {
        let amount = ((i + 1) * 100) as i128;
        env.invoke_contract::<()>(
            &auth_id,
            &Symbol::new(&env, "set_balance"),
            Vec::from_array(
                &env,
                [
                    admin.clone().into_val(&env),
                    user.clone().into_val(&env),
                    amount.into_val(&env),
                ],
            ),
        );
    }

    // Verify balances
    for (i, user) in users.iter().enumerate() {
        let expected = ((i + 1) * 100) as i128;
        let bal: i128 = env.invoke_contract(
            &auth_id,
            &Symbol::new(&env, "get_balance"),
            Vec::from_array(&env, [user.clone().into_val(&env)]),
        );
        assert_eq!(bal, expected);
    }

    // User 4 (500) transfers 150 to User 0 (100)
    env.invoke_contract::<()>(
        &auth_id,
        &symbol_short!("transfer"),
        Vec::from_array(
            &env,
            [
                users[4].clone().into_val(&env),
                users[0].clone().into_val(&env),
                150i128.into_val(&env),
            ],
        ),
    );

    let bal0: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [users[0].clone().into_val(&env)]),
    );
    let bal4: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [users[4].clone().into_val(&env)]),
    );
    assert_eq!(bal0, 250);
    assert_eq!(bal4, 350);
}

// ---------------------------------------------------------------------------
// Test 16: Cross-contract events — hello + storage + events all in one tx
// ---------------------------------------------------------------------------

#[test]
fn test_cross_contract_full_pipeline() {
    let env = Env::default();
    env.mock_all_auths();

    let hello_id = env.register_contract(None, hello_world::HelloContract);
    let storage_id = env.register_contract(None, storage_patterns::StorageContract);
    let events_id = env.register_contract(None, events_structured::EventsContract);

    // Greet 3 different users and record counts
    let names = [
        symbol_short!("Alice"),
        symbol_short!("Bob"),
        symbol_short!("Carol"),
    ];
    for (i, name) in names.iter().enumerate() {
        let greeting: Vec<Symbol> = env.invoke_contract(
            &hello_id,
            &symbol_short!("hello"),
            Vec::from_array(&env, [name.into_val(&env)]),
        );
        assert_eq!(greeting.get(0).unwrap(), symbol_short!("Hello"));
        assert_eq!(greeting.get(1).unwrap(), *name);

        // Store visit count
        let key = symbol_short!("visits");
        env.invoke_contract::<()>(
            &storage_id,
            &Symbol::new(&env, "set_persistent"),
            Vec::from_array(&env, [key.into_val(&env), ((i + 1) as u64).into_val(&env)]),
        );
        env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));
    }

    let final_visits: Option<u64> = env.invoke_contract(
        &storage_id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [symbol_short!("visits").into_val(&env)]),
    );
    assert_eq!(final_visits, Some(3));

    let event_count: u32 =
        env.invoke_contract(&events_id, &Symbol::new(&env, "get_number"), Vec::new(&env));
    assert_eq!(event_count, 3);
}

// ---------------------------------------------------------------------------
// Test 17: Storage contract — all three types coexist without collision
// ---------------------------------------------------------------------------

#[test]
fn test_storage_isolation_three_types() {
    let env = Env::default();

    let id = env.register_contract(None, storage_patterns::StorageContract);

    let key = symbol_short!("shared");

    // Write distinct values to each storage type using the same key
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [key.into_val(&env), 11u64.into_val(&env)]),
    );
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "set_temporary"),
        Vec::from_array(&env, [key.into_val(&env), 22u64.into_val(&env)]),
    );
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "set_instance"),
        Vec::from_array(&env, [key.into_val(&env), 33u64.into_val(&env)]),
    );

    let p: Option<u64> = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    let t: Option<u64> = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_temporary"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    let i: Option<u64> = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_instance"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );

    assert_eq!(p, Some(11));
    assert_eq!(t, Some(22));
    assert_eq!(i, Some(33));
}

// ---------------------------------------------------------------------------
// Test 18: Auth — allowance-based transfer with multi-user scenario
// ---------------------------------------------------------------------------

#[test]
fn test_auth_allowance_multi_user() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, authentication::AuthContract);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    // Initialize and fund
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );
    for (user, amt) in [(&alice, 1000i128), (&bob, 500i128), (&carol, 0i128)] {
        env.invoke_contract::<()>(
            &id,
            &Symbol::new(&env, "set_balance"),
            Vec::from_array(
                &env,
                [
                    admin.clone().into_val(&env),
                    user.clone().into_val(&env),
                    amt.into_val(&env),
                ],
            ),
        );
    }

    // Alice approves Carol to spend 200 on her behalf
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [
                alice.clone().into_val(&env),
                carol.clone().into_val(&env),
                200i128.into_val(&env),
            ],
        ),
    );

    // Carol uses allowance to send 150 from Alice to Bob
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "transfer_from"),
        Vec::from_array(
            &env,
            [
                carol.clone().into_val(&env),
                alice.clone().into_val(&env),
                bob.clone().into_val(&env),
                150i128.into_val(&env),
            ],
        ),
    );

    let alice_bal: i128 = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [alice.into_val(&env)]),
    );
    let bob_bal: i128 = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [bob.into_val(&env)]),
    );
    assert_eq!(alice_bal, 850);
    assert_eq!(bob_bal, 650);
}

// ---------------------------------------------------------------------------
// Test 19: Event system — multiple event types in sequence
// ---------------------------------------------------------------------------

#[test]
fn test_events_multiple_types() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, events_structured::EventsContract);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Emit transfer event
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "transfer"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                user.clone().into_val(&env),
                1000i128.into_val(&env),
                0u64.into_val(&env),
            ],
        ),
    );

    // Emit config update
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "update_config"),
        Vec::from_array(
            &env,
            [
                Symbol::new(&env, "max_supply").into_val(&env),
                100u64.into_val(&env),
                200u64.into_val(&env),
            ],
        ),
    );

    // Emit admin action
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "admin_action"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                symbol_short!("upgrade").into_val(&env),
            ],
        ),
    );

    // Increment counter 3 times to represent 3 event categories
    for _ in 0..3 {
        env.invoke_contract::<()>(&id, &symbol_short!("increment"), Vec::new(&env));
    }

    let count: u32 = env.invoke_contract(&id, &Symbol::new(&env, "get_number"), Vec::new(&env));
    assert_eq!(count, 3);
}

// ---------------------------------------------------------------------------
// Test 20: Full workflow — auth + timelock + events (production-like scenario)
// ---------------------------------------------------------------------------

#[test]
fn test_full_auth_timelock_events_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let timelock_id = env.register_contract(None, timelock::TimelockContract);
    let events_id = env.register_contract(None, events_structured::EventsContract);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Step 1: Initialize contracts
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );
    timelock::TimelockContractClient::new(&env, &timelock_id).initialize(&admin);

    // Step 2: Fund user via auth contract
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                user.clone().into_val(&env),
                1_000i128.into_val(&env),
            ],
        ),
    );

    // Step 3: Queue a timelock operation representing a governance action
    let op_id = soroban_sdk::Bytes::from_array(&env, &[42u8; 32]);
    timelock::TimelockContractClient::new(&env, &timelock_id).queue(&op_id, &60u64);

    // Step 4: Track the queued operation in events
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));

    // Step 5: Advance time and execute
    env.ledger().with_mut(|l| l.timestamp += 70);
    timelock::TimelockContractClient::new(&env, &timelock_id).execute(&op_id);

    // Step 6: Track execution
    env.invoke_contract::<()>(&events_id, &symbol_short!("increment"), Vec::new(&env));

    // Step 7: User performs transfer post-governance
    env.invoke_contract::<()>(
        &auth_id,
        &symbol_short!("transfer"),
        Vec::from_array(
            &env,
            [
                user.clone().into_val(&env),
                admin.clone().into_val(&env),
                100i128.into_val(&env),
            ],
        ),
    );

    // Verify final state
    let user_bal: i128 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [user.into_val(&env)]),
    );
    assert_eq!(user_bal, 900);

    let event_count: u32 =
        env.invoke_contract(&events_id, &Symbol::new(&env, "get_number"), Vec::new(&env));
    assert_eq!(event_count, 2);

    let final_state = timelock::TimelockContractClient::new(&env, &timelock_id).get_state(&op_id);
    assert_eq!(final_state, timelock::OperationState::Unknown);
}

// ---------------------------------------------------------------------------
// Test 21: Multi-user scenario — 3-way transfer chain
// ---------------------------------------------------------------------------

#[test]
fn test_three_way_transfer_chain() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, authentication::AuthContract);
    let admin = Address::generate(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);

    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );

    // Fund A with 600
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                a.clone().into_val(&env),
                600i128.into_val(&env),
            ],
        ),
    );

    // A → B: 200
    env.invoke_contract::<()>(
        &id,
        &symbol_short!("transfer"),
        Vec::from_array(
            &env,
            [
                a.clone().into_val(&env),
                b.clone().into_val(&env),
                200i128.into_val(&env),
            ],
        ),
    );

    // B → C: 100
    env.invoke_contract::<()>(
        &id,
        &symbol_short!("transfer"),
        Vec::from_array(
            &env,
            [
                b.clone().into_val(&env),
                c.clone().into_val(&env),
                100i128.into_val(&env),
            ],
        ),
    );

    // C → A: 50
    env.invoke_contract::<()>(
        &id,
        &symbol_short!("transfer"),
        Vec::from_array(
            &env,
            [
                c.clone().into_val(&env),
                a.clone().into_val(&env),
                50i128.into_val(&env),
            ],
        ),
    );

    let get_bal = |user: &Address| -> i128 {
        env.invoke_contract(
            &id,
            &Symbol::new(&env, "get_balance"),
            Vec::from_array(&env, [user.clone().into_val(&env)]),
        )
    };

    // A: 600 - 200 + 50 = 450
    // B: 200 - 100 = 100
    // C: 100 - 50 = 50
    assert_eq!(get_bal(&a), 450);
    assert_eq!(get_bal(&b), 100);
    assert_eq!(get_bal(&c), 50);
}

// ---------------------------------------------------------------------------
// Test 25: Auth Context — invoker auth + cross-contract scenario
// ---------------------------------------------------------------------------

#[test]
fn test_auth_context_invoker_and_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let ctx_id = env.register_contract(None, auth_context::AuthContextContract);
    let auth_id = env.register_contract(None, authentication::AuthContract);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // AuthContextContract: get_invoker requires the caller to authorize
    let returned: Address = env.invoke_contract(
        &ctx_id,
        &Symbol::new(&env, "get_invoker"),
        Vec::from_array(&env, [user.clone().into_val(&env)]),
    );
    assert_eq!(returned, user);

    // AuthContextContract: get_current_address returns the contract itself
    let contract_addr: Address = env.invoke_contract(
        &ctx_id,
        &Symbol::new(&env, "get_current_address"),
        Vec::new(&env),
    );
    assert_eq!(contract_addr, ctx_id);

    // Cross-contract: initialize auth and confirm admin action also requires auth
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );
    let result: u32 = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "admin_action"),
        Vec::from_array(&env, [admin.clone().into_val(&env), 7u32.into_val(&env)]),
    );
    assert_eq!(result, 14); // admin_action returns value * 2
}

// ---------------------------------------------------------------------------
// Test 26: Type Conversions — numbers, strings, collections
// ---------------------------------------------------------------------------

#[test]
fn test_type_conversions_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, type_conversions::TypeConversionsContract);
    let client = type_conversions::TypeConversionsContractClient::new(&env, &id);

    // Number conversion: i128 → u32 (target_type = 1)
    let v = client.convert_numbers(&1000i128, &1u32);
    assert_eq!(v, 1000);

    // String/Symbol conversion
    let input = soroban_sdk::String::from_str(&env, "hello");
    let (_s, sym) = client.convert_strings(&input, &true);
    assert_eq!(sym, soroban_sdk::symbol_short!("hello"));

    // Collection conversion: Vec<i32> → Vec<i64>
    let native: soroban_sdk::Vec<i32> = soroban_sdk::Vec::from_array(&env, [1i32, 2i32, 3i32]);
    let result: soroban_sdk::Vec<i64> = client.convert_collections(&native);
    assert_eq!(result.len(), 3);
    assert_eq!(result.get(0).unwrap(), 1i64);
}

// ---------------------------------------------------------------------------
// Test 27: Soroban Types — address, bytes, symbols storage
// ---------------------------------------------------------------------------

#[test]
fn test_soroban_types_storage() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, soroban_types_example::SorobanTypesContract);
    let client = soroban_types_example::SorobanTypesContractClient::new(&env, &id);

    let owner = Address::generate(&env);

    // Store and retrieve address
    client.store_address(&owner);
    let retrieved = client.get_address();
    assert_eq!(retrieved, owner);

    // Verify address comparison
    let same = client.verify_address(&owner, &owner);
    assert!(same);

    let other = Address::generate(&env);
    let different = client.verify_address(&owner, &other);
    assert!(!different);

    // Store and retrieve bytes
    let data = soroban_sdk::Bytes::from_array(&env, &[1u8, 2u8, 3u8]);
    client.store_bytes(&data);
    let retrieved_bytes = client.get_bytes();
    assert_eq!(retrieved_bytes, data);
}

// ---------------------------------------------------------------------------
// Test 28: Enum Types — role dispatch and state machine
// ---------------------------------------------------------------------------

#[test]
fn test_enum_types_role_dispatch() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, enum_types::EnumContract);
    let client = enum_types::EnumContractClient::new(&env, &id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);
    assert_eq!(client.get_state(), enum_types::ContractState::Active);

    // Admin is Owner role
    assert_eq!(client.get_user_role(&admin), enum_types::UserRole::Owner);

    // Grant Admin role to user
    client.set_user_role(&admin, &user, &enum_types::UserRole::Admin);
    assert_eq!(client.get_user_role(&user), enum_types::UserRole::Admin);

    // Regular user role query
    let nobody = Address::generate(&env);
    assert_eq!(client.get_user_role(&nobody), enum_types::UserRole::None);
}

// ---------------------------------------------------------------------------
// Test 29: Custom Structs — user profile lifecycle (multi-user scenario)
// ---------------------------------------------------------------------------

#[test]
fn test_custom_structs_multi_user_profiles() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, custom_structs::CustomStructsContract);
    let client = custom_structs::CustomStructsContractClient::new(&env, &id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Create profiles for two users
    let alice_name = soroban_sdk::String::from_str(&env, "Alice");
    let bob_name = soroban_sdk::String::from_str(&env, "Bob");

    let alice_profile = client.create_user_profile(&alice, &alice_name, &None);
    client.create_user_profile(&bob, &bob_name, &None);

    // Retrieve and verify profiles are independent
    let alice_profile_fetched = client.get_user_profile(&alice);
    let bob_profile = client.get_user_profile(&bob);

    assert_eq!(alice_profile.name, alice_name);
    assert_eq!(alice_profile_fetched.name, alice_name);
    assert_eq!(bob_profile.name, bob_name);
    assert_ne!(alice_profile.address, bob_profile.address);
}

// ---------------------------------------------------------------------------
// Test 30: Primitive Types — safe arithmetic across multiple users
// ---------------------------------------------------------------------------

#[test]
fn test_primitive_types_safe_arithmetic() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, primitive_types::PrimitiveTypesContract);
    let client = primitive_types::PrimitiveTypesContractClient::new(&env, &id);

    client.initialize();

    // u32 arithmetic
    assert_eq!(client.add_u32(&10u32, &20u32), 30);
    assert_eq!(client.sub_u32(&50u32, &15u32), 35);
    assert_eq!(client.mul_u32(&6u32, &7u32), 42);
    assert_eq!(client.div_u32(&100u32, &4u32), 25);

    // u64 arithmetic
    assert_eq!(client.add_u64(&1_000_000u64, &2_000_000u64), 3_000_000);

    // Overflow protection: u32::MAX + 1 should error
    let overflow = client.try_add_u32(&u32::MAX, &1u32);
    assert!(overflow.is_err());
}

// ---------------------------------------------------------------------------
// Test 31: Data Types — all primitive type round-trips
// ---------------------------------------------------------------------------

#[test]
fn test_data_types_round_trips() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, data_types::DataTypesContract);
    let client = data_types::DataTypesContractClient::new(&env, &id);

    assert_eq!(client.store_u32(&42u32), 42u32);
    assert_eq!(client.store_u64(&999_999u64), 999_999u64);
    assert_eq!(client.store_i128(&-100_000i128), -100_000i128);

    // safe_add: i128 + i128
    assert_eq!(client.safe_add(&500i128, &300i128), 800i128);

    let sym = soroban_sdk::symbol_short!("test");
    assert_eq!(client.store_symbol(&sym), sym);

    let s = soroban_sdk::String::from_str(&env, "hello");
    assert_eq!(client.store_string(&s), s);
}

// ---------------------------------------------------------------------------
// Test 32: Collection Types — Vec and Map operations
// ---------------------------------------------------------------------------

#[test]
fn test_collection_types_vec_and_map() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, collection_types::CollectionTypesContract);
    let client = collection_types::CollectionTypesContractClient::new(&env, &id);

    // Vec operations
    client.vec_push(&10i128);
    client.vec_push(&20i128);
    client.vec_push(&30i128);

    let list = client.vec_list();
    assert_eq!(list.len(), 3);

    let popped = client.vec_pop();
    assert_eq!(popped, Some(30i128));

    // Sum
    let items = soroban_sdk::Vec::from_array(&env, [1i128, 2i128, 3i128, 4i128]);
    let total = client.vec_sum(&items);
    assert_eq!(total, 10i128);

    // Filter positives
    let mixed = soroban_sdk::Vec::from_array(&env, [-5i128, 3i128, -1i128, 7i128]);
    let positive = client.vec_filter_positive(&mixed);
    assert_eq!(positive.len(), 2);

    // Max
    let max = client.vec_max(&items);
    assert_eq!(max, Some(4i128));
}

// ---------------------------------------------------------------------------
// NFT and Governance Integration Tests (10 new tests to reach 22 total)
// ---------------------------------------------------------------------------

// --- Helper contracts for Fractional NFT and Governance tests ---

#[soroban_sdk::contracttype]
#[derive(Clone)]
pub enum FractionalDataKey {
    NftContract,
    TokenId,
    ShareSupply,
    ShareBalance(Address),
    Initialized,
}

#[soroban_sdk::contract]
pub struct FractionalNftContract;

#[soroban_sdk::contractimpl]
impl FractionalNftContract {
    pub fn initialize(
        env: Env,
        nft_contract: Address,
        token_id: u32,
        total_shares: i128,
        owner: Address,
    ) {
        if env
            .storage()
            .instance()
            .has(&FractionalDataKey::Initialized)
        {
            panic!("already initialized");
        }

        env.storage()
            .instance()
            .set(&FractionalDataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&FractionalDataKey::NftContract, &nft_contract);
        env.storage()
            .instance()
            .set(&FractionalDataKey::TokenId, &token_id);
        env.storage()
            .instance()
            .set(&FractionalDataKey::ShareSupply, &total_shares);
        env.storage().persistent().set(
            &FractionalDataKey::ShareBalance(owner.clone()),
            &total_shares,
        );

        let nft_client = basic_nft::BasicNftContractClient::new(&env, &nft_contract);
        nft_client.transfer_from(
            &env.current_contract_address(),
            &owner,
            &env.current_contract_address(),
            &token_id,
        );
    }

    pub fn transfer_shares(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&FractionalDataKey::ShareBalance(from.clone()))
            .unwrap_or(0);
        if from_bal < amount {
            panic!("insufficient balance");
        }
        let to_bal: i128 = env
            .storage()
            .persistent()
            .get(&FractionalDataKey::ShareBalance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&FractionalDataKey::ShareBalance(from), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&FractionalDataKey::ShareBalance(to), &(to_bal + amount));
    }

    pub fn redeem(env: Env, redeemer: Address) {
        redeemer.require_auth();
        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&FractionalDataKey::ShareSupply)
            .unwrap();
        let bal: i128 = env
            .storage()
            .persistent()
            .get(&FractionalDataKey::ShareBalance(redeemer.clone()))
            .unwrap_or(0);
        if bal != total_shares {
            panic!("must own all shares to redeem");
        }

        env.storage()
            .persistent()
            .remove(&FractionalDataKey::ShareBalance(redeemer.clone()));

        let nft_contract: Address = env
            .storage()
            .instance()
            .get(&FractionalDataKey::NftContract)
            .unwrap();
        let token_id: u32 = env
            .storage()
            .instance()
            .get(&FractionalDataKey::TokenId)
            .unwrap();

        let nft_client = basic_nft::BasicNftContractClient::new(&env, &nft_contract);
        nft_client.transfer(&env.current_contract_address(), &redeemer, &token_id);
    }

    pub fn balance_of(env: Env, address: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&FractionalDataKey::ShareBalance(address))
            .unwrap_or(0)
    }
}

#[soroban_sdk::contract]
pub struct GovDummyContract;

#[soroban_sdk::contractimpl]
impl GovDummyContract {
    pub fn execute_action(env: Env, value: u32) {
        env.storage()
            .instance()
            .set(&symbol_short!("executed"), &value);
    }
}

// --- Integration Tests ---

// 1. NFT Mint and Direct Transfer
#[test]
fn test_nft_mint_and_direct_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let nft_id = env.register_contract(None, basic_nft::BasicNftContract);
    let client = basic_nft::BasicNftContractClient::new(&env, &nft_id);

    client.initialize(
        &admin,
        &soroban_sdk::String::from_str(&env, "Test NFT"),
        &soroban_sdk::String::from_str(&env, "TNFT"),
    );

    // Mint to Alice
    client.mint(&admin, &alice, &1u32);
    assert_eq!(client.owner_of(&1u32), alice);
    assert_eq!(client.balance_of(&alice), 1);

    // Transfer from Alice to Bob
    client.transfer(&alice, &bob, &1u32);
    assert_eq!(client.owner_of(&1u32), bob);
    assert_eq!(client.balance_of(&alice), 0);
    assert_eq!(client.balance_of(&bob), 1);
}

// 2. NFT Approved Transfer From
#[test]
fn test_nft_approved_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    let nft_id = env.register_contract(None, basic_nft::BasicNftContract);
    let client = basic_nft::BasicNftContractClient::new(&env, &nft_id);

    client.initialize(
        &admin,
        &soroban_sdk::String::from_str(&env, "Test NFT"),
        &soroban_sdk::String::from_str(&env, "TNFT"),
    );
    client.mint(&admin, &alice, &1u32);

    // Alice approves Bob for token 1
    client.approve(&alice, &bob, &1u32);
    assert_eq!(client.get_approved(&1u32).unwrap(), bob);

    // Bob transfers token 1 from Alice to Charlie
    client.transfer_from(&bob, &alice, &charlie, &1u32);
    assert_eq!(client.owner_of(&1u32), charlie);
    assert!(client.get_approved(&1u32).is_none());
}

// 3. NFT Operator Transfer
#[test]
fn test_nft_operator_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    let nft_id = env.register_contract(None, basic_nft::BasicNftContract);
    let client = basic_nft::BasicNftContractClient::new(&env, &nft_id);

    client.initialize(
        &admin,
        &soroban_sdk::String::from_str(&env, "Test NFT"),
        &soroban_sdk::String::from_str(&env, "TNFT"),
    );
    client.mint(&admin, &alice, &1u32);

    // Alice sets Bob as operator
    client.set_approval_for_all(&alice, &bob, &true);
    assert!(client.is_approved_for_all(&alice, &bob));

    // Bob transfers token 1 from Alice to Charlie
    client.transfer_from(&bob, &alice, &charlie, &1u32);
    assert_eq!(client.owner_of(&1u32), charlie);
}

// 4. Marketplace Fixed Price Listing and Buy
#[test]
fn test_marketplace_fixed_price_listing_and_buy() {
    let env = Env::default();
    env.mock_all_auths();

    let mkt_admin = Address::generate(&env);
    let nft_admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let royalty_rec = Address::generate(&env);

    let nft_id = env.register_contract(None, basic_nft::BasicNftContract);
    let nft_client = basic_nft::BasicNftContractClient::new(&env, &nft_id);
    nft_client.initialize(
        &nft_admin,
        &soroban_sdk::String::from_str(&env, "Test NFT"),
        &soroban_sdk::String::from_str(&env, "TNFT"),
    );
    nft_client.mint(&nft_admin, &seller, &1u32);

    let mkt_id = env.register_contract(None, nft_marketplace::NftMarketplaceContract);
    let mkt_client = nft_marketplace::NftMarketplaceContractClient::new(&env, &mkt_id);
    mkt_client.initialize(&mkt_admin);

    // Seller approves marketplace for token 1
    nft_client.approve(&seller, &mkt_id, &1u32);

    // List item on marketplace
    let items = Vec::from_array(
        &env,
        [nft_marketplace::ListingItem {
            nft_contract: nft_id.clone(),
            token_id: 1u32,
        }],
    );

    let listing_id =
        mkt_client.create_fixed_price_listing(&seller, &items, &1000i128, &royalty_rec, &500u32);

    let listing = mkt_client.get_listing(&listing_id);
    assert_eq!(listing.seller, seller);
    assert_eq!(listing.price, 1000);
    assert!(!listing.sold);

    // Buy item
    mkt_client.buy(&buyer, &listing_id, &1000i128);

    // Complete the transfer (simulated coordinator step matching cross-contract marketplace flow)
    nft_client.transfer_from(&mkt_id, &seller, &buyer, &1u32);

    let updated_listing = mkt_client.get_listing(&listing_id);
    assert!(updated_listing.sold);
    assert_eq!(nft_client.owner_of(&1u32), buyer);

    let trade = mkt_client.get_trade(&0);
    assert_eq!(trade.buyer, buyer);
    assert_eq!(trade.seller, seller);
    assert_eq!(trade.amount, 1000);
    assert_eq!(trade.royalty_paid, 50); // 1000 * 500 / 10000 = 50
}

// 5. Marketplace Auction Bidding and Finalization
#[test]
fn test_marketplace_auction_bidding_and_finalization() {
    let env = Env::default();
    env.mock_all_auths();

    let mkt_admin = Address::generate(&env);
    let nft_admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    let nft_id = env.register_contract(None, basic_nft::BasicNftContract);
    let nft_client = basic_nft::BasicNftContractClient::new(&env, &nft_id);
    nft_client.initialize(
        &nft_admin,
        &soroban_sdk::String::from_str(&env, "Test NFT"),
        &soroban_sdk::String::from_str(&env, "TNFT"),
    );
    nft_client.mint(&nft_admin, &seller, &1u32);

    let mkt_id = env.register_contract(None, nft_marketplace::NftMarketplaceContract);
    let mkt_client = nft_marketplace::NftMarketplaceContractClient::new(&env, &mkt_id);
    mkt_client.initialize(&mkt_admin);

    nft_client.approve(&seller, &mkt_id, &1u32);

    let items = Vec::from_array(
        &env,
        [nft_marketplace::ListingItem {
            nft_contract: nft_id.clone(),
            token_id: 1u32,
        }],
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    let listing_id =
        mkt_client.create_auction_listing(&seller, &items, &500i128, &10u32, &seller, &0u32);

    // Bidding
    mkt_client.place_bid(&bidder1, &listing_id, &550i128);
    mkt_client.place_bid(&bidder2, &listing_id, &600i128);

    // Close auction
    env.ledger().with_mut(|l| l.sequence_number = 111);

    mkt_client.finalize_auction(&bidder1, &listing_id);
    nft_client.transfer_from(&mkt_id, &seller, &bidder2, &1u32);

    let listing = mkt_client.get_listing(&listing_id);
    assert!(listing.sold);
    assert_eq!(nft_client.owner_of(&1u32), bidder2);
}

// 6. Marketplace Invalid Bids and Early Finalization
#[test]
fn test_marketplace_invalid_bids_and_early_finalization() {
    let env = Env::default();
    env.mock_all_auths();

    let mkt_admin = Address::generate(&env);
    let nft_admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);

    let nft_id = env.register_contract(None, basic_nft::BasicNftContract);
    let nft_client = basic_nft::BasicNftContractClient::new(&env, &nft_id);
    nft_client.initialize(
        &nft_admin,
        &soroban_sdk::String::from_str(&env, "Test NFT"),
        &soroban_sdk::String::from_str(&env, "TNFT"),
    );
    nft_client.mint(&nft_admin, &seller, &1u32);

    let mkt_id = env.register_contract(None, nft_marketplace::NftMarketplaceContract);
    let mkt_client = nft_marketplace::NftMarketplaceContractClient::new(&env, &mkt_id);
    mkt_client.initialize(&mkt_admin);

    let items = Vec::from_array(
        &env,
        [nft_marketplace::ListingItem {
            nft_contract: nft_id.clone(),
            token_id: 1u32,
        }],
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    let listing_id =
        mkt_client.create_auction_listing(&seller, &items, &500i128, &10u32, &seller, &0u32);

    // Bid too low should error
    let res = mkt_client.try_place_bid(&bidder, &listing_id, &499i128);
    assert_eq!(res, Err(Ok(nft_marketplace::MarketplaceError::BidTooLow)));

    // Finalize too early should error
    let res2 = mkt_client.try_finalize_auction(&bidder, &listing_id);
    assert_eq!(
        res2,
        Err(Ok(nft_marketplace::MarketplaceError::AuctionNotActive))
    );
}

// 7. Fractional NFT Initialization and Transfers
#[test]
fn test_fractional_nft_initialization_and_transfers() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let alice = Address::generate(&env);

    let nft_id = env.register_contract(None, basic_nft::BasicNftContract);
    let nft_client = basic_nft::BasicNftContractClient::new(&env, &nft_id);
    nft_client.initialize(
        &admin,
        &soroban_sdk::String::from_str(&env, "Test NFT"),
        &soroban_sdk::String::from_str(&env, "TNFT"),
    );
    nft_client.mint(&admin, &owner, &42u32);

    let frac_id = env.register_contract(None, FractionalNftContract);
    let frac_client = FractionalNftContractClient::new(&env, &frac_id);

    // Owner approves fractional contract for token 42
    nft_client.approve(&owner, &frac_id, &42u32);

    // Initialize fractional NFT (locks NFT, mints 1000 shares to owner)
    frac_client.initialize(&nft_id, &42u32, &1000i128, &owner);

    assert_eq!(nft_client.owner_of(&42u32), frac_id);
    assert_eq!(frac_client.balance_of(&owner), 1000);

    // Owner transfers 250 shares to Alice
    frac_client.transfer_shares(&owner, &alice, &250i128);
    assert_eq!(frac_client.balance_of(&owner), 750);
    assert_eq!(frac_client.balance_of(&alice), 250);
}

// 8. Fractional NFT Redemption
#[test]
fn test_fractional_nft_redemption() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let alice = Address::generate(&env);

    let nft_id = env.register_contract(None, basic_nft::BasicNftContract);
    let nft_client = basic_nft::BasicNftContractClient::new(&env, &nft_id);
    nft_client.initialize(
        &admin,
        &soroban_sdk::String::from_str(&env, "Test NFT"),
        &soroban_sdk::String::from_str(&env, "TNFT"),
    );
    nft_client.mint(&admin, &owner, &42u32);

    let frac_id = env.register_contract(None, FractionalNftContract);
    let frac_client = FractionalNftContractClient::new(&env, &frac_id);

    nft_client.approve(&owner, &frac_id, &42u32);
    frac_client.initialize(&nft_id, &42u32, &1000i128, &owner);

    frac_client.transfer_shares(&owner, &alice, &300i128);

    // Try to redeem without all shares (fails)
    let res = frac_client.try_redeem(&owner);
    assert!(res.is_err());

    // Alice transfers shares back to owner
    frac_client.transfer_shares(&alice, &owner, &300i128);

    // Now owner can redeem (unlocks NFT)
    frac_client.redeem(&owner);

    assert_eq!(nft_client.owner_of(&42u32), owner);
    assert_eq!(frac_client.balance_of(&owner), 0);
}

// 9. Governance Proposal Lifecycle Full Flow
#[test]
fn test_governance_proposal_lifecycle_full_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let gov_client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    gov_client.initialize(&admin, &100i128);

    let dummy_id = env.register_contract(None, GovDummyContract);
    let _dummy_client = GovDummyContractClient::new(&env, &dummy_id);

    let description = soroban_sdk::String::from_str(&env, "Set value to 99");
    let action_args = Vec::from_array(&env, [99u32.into_val(&env)]);

    let proposal_id = gov_client.create_proposal(
        &proposer,
        &description,
        &dummy_id,
        &Symbol::new(&env, "execute_action"),
        &action_args,
    );

    env.ledger().with_mut(|l| l.sequence_number = 100);
    gov_client.submit_proposal(&proposer, &proposal_id, &50u32, &100u32); // vote end = 150, exec end = 250

    // Voting
    gov_client.vote(&voter1, &proposal_id, &true, &70i128);
    gov_client.vote(&voter2, &proposal_id, &true, &40i128); // total = 110 (quorum met)

    // Close voting
    env.ledger().with_mut(|l| l.sequence_number = 155);

    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Passed
    );

    // Execute proposal
    gov_client.execute_proposal(&voter1, &proposal_id);

    assert_eq!(
        gov_client.get_proposal_state(&proposal_id),
        proposal_lifecycle::ProposalState::Executed
    );

    // Verify target executed
    let val: u32 = env.as_contract(&dummy_id, || {
        env.storage()
            .instance()
            .get(&symbol_short!("executed"))
            .unwrap()
    });
    assert_eq!(val, 99);
}

// 10. Governance Proposal Expired and Cancelled
#[test]
fn test_governance_proposal_expired_and_cancelled() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let dummy_id = Address::generate(&env);

    let gov_id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let gov_client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov_id);
    gov_client.initialize(&admin, &100i128);

    let description = soroban_sdk::String::from_str(&env, "Mock Proposal");
    let proposal_id_1 = gov_client.create_proposal(
        &proposer,
        &description,
        &dummy_id,
        &Symbol::new(&env, "mock_action"),
        &Vec::new(&env),
    );

    // Proposal 1: Proposer cancels
    gov_client.cancel_proposal(&proposer, &proposal_id_1);
    assert_eq!(
        gov_client.get_proposal_state(&proposal_id_1),
        proposal_lifecycle::ProposalState::Cancelled
    );

    // Proposal 2: Expired
    let proposal_id_2 = gov_client.create_proposal(
        &proposer,
        &description,
        &dummy_id,
        &Symbol::new(&env, "mock_action"),
        &Vec::new(&env),
    );

    env.ledger().with_mut(|l| l.sequence_number = 200);
    gov_client.submit_proposal(&proposer, &proposal_id_2, &50u32, &100u32); // vote end = 250, exec end = 350
    gov_client.vote(&voter, &proposal_id_2, &true, &150i128); // quorum met

    // Advance sequence beyond execution end
    env.ledger().with_mut(|l| l.sequence_number = 351);

    assert_eq!(
        gov_client.get_proposal_state(&proposal_id_2),
        proposal_lifecycle::ProposalState::Expired
    );

    let res = gov_client.try_execute_proposal(&voter, &proposal_id_2);
    assert_eq!(
        res,
        Err(Ok(proposal_lifecycle::ProposalError::ExecutionEnded))
    );
}

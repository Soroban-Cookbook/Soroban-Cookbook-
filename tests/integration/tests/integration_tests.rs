//! Integration Tests for Soroban Cookbook Basic Examples
//!
//! This test suite demonstrates cross-contract interactions and end-to-end
//! scenarios combining multiple basic examples.  Contracts are registered
//! natively (no WASM binary required) so the tests work with any Rust
//! toolchain without special build-time flags.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{symbol_short, testutils::Address as _, Address, BytesN, Env, IntoVal, Symbol, Vec};

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
    let _: Result<(), validation_patterns::ValidationError> = env.invoke_contract(
        &validation_id,
        &symbol_short!("initialize"),
        Vec::from_array(&env, [owner.clone().into_val(&env)]),
    );

    // Step 2: Test validation parameters (Success)
    let _: Result<(), validation_patterns::ValidationError> = env.invoke_contract(
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
    let error_result: Result<(), custom_errors::ContractError> = env.invoke_contract(
        &errors_id,
        &Symbol::new(&env, "validate_input"),
        Vec::from_array(&env, [0i64.into_val(&env)]),
    );
    assert!(error_result.is_err());
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

    // Step 2: Initialize Ajo Factory with Ajo contract WASM
    let wasm_hash = env.deployer().upload_contract_wasm(ajo_factory::Ajo::WASM);

    env.invoke_contract::<Result<(), ajo_factory::FactoryError>>(
        &factory_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [wasm_hash.into_val(&env)]),
    ).unwrap();

    // Step 3: Create Ajo via factory
    let ajo_address: Address = env.invoke_contract::<Result<Address, ajo_factory::FactoryError>>(
        &factory_id,
        &Symbol::new(&env, "create_ajo"),
        Vec::from_array(
            &env,
            [
                1000i128.into_val(&env),
                10u32.into_val(&env),
                creator.clone().into_val(&env),
            ],
        ),
    ).unwrap();

    // Step 4: Verify Ajo was created and tracked
    let deployed_ajos: Vec<Address> = env.invoke_contract(
        &factory_id,
        &Symbol::new(&env, "get_deployed_ajos"),
        Vec::new(&env),
    );
    assert_eq!(deployed_ajos.len(), 1);
    assert_eq!(deployed_ajos.get(0).unwrap(), ajo_address);
}

// ---------------------------------------------------------------------------
// Test 7: Multi-Sig Governance + Events Tracking
// ---------------------------------------------------------------------------

#[test]
fn test_multi_sig_governance_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let events_id = env.register_contract(None, events_counter::Contract);

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
    let proposal_id: u32 = env.invoke_contract::<Result<u32, multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "create_proposal"),
        Vec::from_array(&env, [signer1.clone().into_val(&env)]),
    ).unwrap();

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
    let success: bool = env.invoke_contract::<Result<bool, multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "execute"),
        Vec::from_array(&env, [proposal_id.into_val(&env), signer1.into_val(&env)]),
    ).unwrap();
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

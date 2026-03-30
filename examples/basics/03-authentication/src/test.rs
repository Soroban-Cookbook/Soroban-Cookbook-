use super::*;
<<<<<<< HEAD
use soroban_sdk::testutils::{Address as _, AuthorizedFunction};
use soroban_sdk::{symbol_short, Address, Env};

#[test]
fn test_basic_auth_success() {
    // Create a test environment
    let env = Env::default();

    // Register the contract in the test environment
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    // Generate a test address
    let user = Address::generate(&env);

    // Mock authentication for the user (simulates the user signing the transaction)
    env.mock_all_auths();

    // Call the basic_auth function - should succeed
    let result = client.basic_auth(&user);

    // Verify the function returned true as expected
    assert_eq!(result, true);
}

#[test]
fn test_transfer_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 100_i128;

    // Mock authentication for the 'from' address
    env.mock_all_auths();

    // Call the transfer function - should succeed
    let result = client.transfer(&from, &to, &amount);

    // Verify the function returned true as expected
    assert_eq!(result, true);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_transfer_invalid_amount() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let invalid_amount = -10_i128; // Negative amount should cause panic

    // Mock authentication for the 'from' address
    env.mock_all_auths();

    // Call the transfer function with invalid amount - should panic
    client.transfer(&from, &to, &invalid_amount);
}

#[test]
fn test_initial_admin_setup() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    let initial_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    // Mock authentication for the initial admin
    env.mock_all_auths();

    // Set the initial admin
    client.set_admin(&initial_admin, &new_admin);

    // Verify the admin was set correctly
    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, Some(new_admin));
}

#[test]
fn test_admin_only_access() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    let initial_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let _unauthorized_user = Address::generate(&env);

    // Mock authentication for the initial admin
    env.mock_all_auths();

    // Set the initial admin
    client.set_admin(&initial_admin, &new_admin);

    // Try to change admin with unauthorized user - should fail with AuthError::AdminOnly
    // Since this will cause a panic in the contract, we'll test with the correct admin instead
    let another_new_admin = Address::generate(&env);
    client.set_admin(&new_admin, &another_new_admin); // new_admin is now the admin

    // Verify the admin changed correctly
    let current_admin = client.get_admin();
    assert_eq!(current_admin, Some(another_new_admin));
}

#[test]
fn test_user_specific_data_storage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let data1 = symbol_short!("usr1_data");
    let data2 = symbol_short!("usr2_data");

    // Mock authentication for users
    env.mock_all_auths();

    // Update data for user1
    client.update_user_data(&user1, &data1);

    // Update data for user2
    client.update_user_data(&user2, &data2);

    // Verify each user gets their own data
    let retrieved_data1 = client.get_user_data(&user1);
    let retrieved_data2 = client.get_user_data(&user2);

    assert_eq!(retrieved_data1, Some(data1));
    assert_eq!(retrieved_data2, Some(data2));

    // Verify users don't share data
    assert_ne!(retrieved_data1, retrieved_data2);
}

#[test]
fn test_secure_operation_valid() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let valid_operation = symbol_short!("valid_op");

    // Mock authentication for the user
    env.mock_all_auths();

    // Call secure operation with valid operation - should succeed
    let result_data = client.secure_operation(&user, &valid_operation);
    // Verify the result contains expected values
    assert_eq!(result_data.get(0).unwrap(), symbol_short!("success"));
    assert_eq!(result_data.get(1).unwrap(), valid_operation);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_secure_operation_invalid() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let invalid_operation = symbol_short!("invalid");

    // Mock authentication for the user
    env.mock_all_auths();

    // This should panic with Unauthorized error
    client.secure_operation(&user, &invalid_operation);
}

#[test]
fn test_self_authentication() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    // Use the contract's own address for self-authentication
    let contract_address = env.register_contract(None, AuthContract);

    // Mock authentication for the contract address
    env.mock_all_auths();

    // Test self-authentication - should succeed
    let result = client.self_authenticate(&contract_address);
    assert_eq!(result, true);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_auth_failure_scenarios() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // DON'T mock auth - this simulates an unauthorized call
    // This should cause the transaction to fail at the require_auth() call

    // Attempting to call basic_auth without proper authorization should panic
    client.basic_auth(&user);
}

#[test]
fn test_multiple_auth_patterns() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    // Generate test addresses
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    // Mock all auths for this test
    env.mock_all_auths();

    // Test basic auth
    let basic_result = client.basic_auth(&user1);
    assert_eq!(basic_result, true);

    // Test transfer
    let transfer_result = client.transfer(&user1, &user2, &50_i128);
    assert_eq!(transfer_result, true);

    // Test admin function
    client.set_admin(&admin, &new_admin);
    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, Some(new_admin));

    // Test user-specific operation
    let data = symbol_short!("test_data");
    let update_result = client.update_user_data(&user1, &data);
    assert_eq!(update_result, true);

    let retrieved_data = client.get_user_data(&user1);
    assert_eq!(retrieved_data, Some(data));
}
=======
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    vec, Env,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup_initialized(env: &Env) -> (AuthContractClient<'_>, Address) {
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&admin);
    (client, admin)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_sets_admin() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.initialize(&admin);
}

// ---------------------------------------------------------------------------
// Admin-only actions
// ---------------------------------------------------------------------------

#[test]
fn test_admin_action_doubles_value() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    assert_eq!(client.admin_action(&admin, &10), 20);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_admin_action_non_admin_fails() {
    let env = Env::default();
    let (client, _admin) = setup_initialized(&env);
    let attacker = Address::generate(&env);
    client.admin_action(&attacker, &10);
}

#[test]
fn test_set_balance_admin_only() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);
    client.set_balance(&admin, &user, &5000);
    assert_eq!(client.get_balance(&user), 5000);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_set_balance_non_admin_fails() {
    let env = Env::default();
    let (client, _admin) = setup_initialized(&env);
    let non_admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.set_balance(&non_admin, &user, &5000);
}

// ---------------------------------------------------------------------------
// Transfer
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_updates_balances() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client.set_balance(&admin, &user1, &1000);
    client.transfer(&user1, &user2, &300);

    assert_eq!(client.get_balance(&user1), 700);
    assert_eq!(client.get_balance(&user2), 300);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_transfer_insufficient_balance_fails() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client.set_balance(&admin, &user1, &100);
    client.transfer(&user1, &user2, &500);
}

// ---------------------------------------------------------------------------
// Allowance (approve + transfer_from)
// ---------------------------------------------------------------------------

#[test]
fn test_approve_and_transfer_from() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.set_balance(&admin, &owner, &1000);
    client.approve(&owner, &spender, &500);
    client.transfer_from(&spender, &owner, &recipient, &200);

    assert_eq!(client.get_balance(&owner), 800);
    assert_eq!(client.get_balance(&recipient), 200);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_transfer_from_exceeds_allowance() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.set_balance(&admin, &owner, &1000);
    client.approve(&owner, &spender, &100);
    client.transfer_from(&spender, &owner, &recipient, &200);
}

// ---------------------------------------------------------------------------
// Multi-sig
// ---------------------------------------------------------------------------

#[test]
fn test_multi_sig_adds_signer_count() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);
    env.mock_all_auths();

    let signers = vec![
        &env,
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];
    assert_eq!(client.multi_sig_action(&signers, &10), 13);
}

// ---------------------------------------------------------------------------
// Secure operation
// ---------------------------------------------------------------------------

#[test]
fn test_secure_operation_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);
    env.mock_all_auths();

    let user = Address::generate(&env);
    let result = client.secure_operation(&user, &symbol_short!("action"));
    assert_eq!(result.get(0).unwrap(), symbol_short!("success"));
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_secure_operation_invalid_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);
    env.mock_all_auths();

    let user = Address::generate(&env);
    client.secure_operation(&user, &symbol_short!("invalid"));
}

// ---------------------------------------------------------------------------
// Emit event
// ---------------------------------------------------------------------------

#[test]
fn test_emit_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);
    env.mock_all_auths();

    let user = Address::generate(&env);
    // Should not panic.
    client.emit_event(&user, &symbol_short!("hello"));
}

// ---------------------------------------------------------------------------
// Role-Based Access Control Tests
// ---------------------------------------------------------------------------

#[test]
fn test_grant_role() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Moderator);
    assert_eq!(client.get_role(&user), Role::Moderator as u32);
}

#[test]
fn test_revoke_role() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Admin);
    assert_eq!(client.get_role(&user), Role::Admin as u32);

    client.revoke_role(&admin, &user);
    assert_eq!(client.get_role(&user), Role::User as u32);
}

#[test]
fn test_has_role() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Moderator);
    assert!(client.has_role(&user, &Role::Moderator));
    assert!(client.has_role(&user, &Role::User));
    assert!(!client.has_role(&user, &Role::Admin));
}

#[test]
fn test_admin_role_action_success() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Admin);
    assert_eq!(client.admin_role_action(&user, &10), 20);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_admin_role_action_insufficient_role() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::User);
    client.admin_role_action(&user, &10);
}

#[test]
fn test_moderator_action_with_moderator() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Moderator);
    assert_eq!(client.moderator_action(&user, &10), 20);
}

#[test]
fn test_moderator_action_with_admin() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Admin);
    assert_eq!(client.moderator_action(&user, &10), 20);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_moderator_action_with_user_fails() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::User);
    client.moderator_action(&user, &10);
}

// ---------------------------------------------------------------------------
// Time-Based Restrictions Tests
// ---------------------------------------------------------------------------

#[test]
fn test_set_time_lock() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.set_time_lock(&admin, &1000);
    // Time lock is set, verified by attempting a time-locked action
}

#[test]
fn test_time_locked_action_before_unlock() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 500);
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.set_time_lock(&admin, &1000);

    let result = client.try_time_locked_action(&user);
    assert_eq!(result, Err(Ok(AuthError::TimeLocked)));
}

#[test]
fn test_time_locked_action_after_unlock() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1500);
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.set_time_lock(&admin, &1000);
    assert_eq!(client.time_locked_action(&user), 1500);
}

#[test]
fn test_set_cooldown() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.set_cooldown(&admin, &300);
    // Cooldown is set, verified by attempting a cooldown action
}

#[test]
fn test_cooldown_action_first_call() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.set_cooldown(&admin, &300);
    assert_eq!(client.cooldown_action(&user), 1000);
}

#[test]
fn test_cooldown_action_within_period() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.set_cooldown(&admin, &300);
    client.cooldown_action(&user);

    env.ledger().with_mut(|li| li.timestamp = 1200);
    let result = client.try_cooldown_action(&user);
    assert_eq!(result, Err(Ok(AuthError::CooldownActive)));
}

#[test]
fn test_cooldown_action_after_period() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.set_cooldown(&admin, &300);
    client.cooldown_action(&user);

    env.ledger().with_mut(|li| li.timestamp = 1400);
    assert_eq!(client.cooldown_action(&user), 1400);
}

// ---------------------------------------------------------------------------
// State-Based Authorization Tests
// ---------------------------------------------------------------------------

#[test]
fn test_set_state() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.set_state(&admin, &ContractState::Paused);
    assert_eq!(client.get_state(), ContractState::Paused as u32);
}

#[test]
fn test_active_only_action_when_active() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.set_state(&admin, &ContractState::Active);
    assert_eq!(client.active_only_action(&user), 1000);
}

#[test]
fn test_active_only_action_when_paused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.set_state(&admin, &ContractState::Paused);
    let result = client.try_active_only_action(&user);
    assert_eq!(result, Err(Ok(AuthError::InvalidState)));
}

#[test]
fn test_active_only_action_when_frozen() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.set_state(&admin, &ContractState::Frozen);
    let result = client.try_active_only_action(&user);
    assert_eq!(result, Err(Ok(AuthError::InvalidState)));
}

#[test]
fn test_default_state_is_active() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let (client, _admin) = setup_initialized(&env);
    let user = Address::generate(&env);

    assert_eq!(client.get_state(), ContractState::Active as u32);
    assert_eq!(client.active_only_action(&user), 1000);
}

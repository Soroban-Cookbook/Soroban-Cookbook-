use super::*;
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

/// Benchmark the transfer function with authentication.
#[test]
fn test_transfer_benchmark() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client.set_balance(&admin, &user1, &1000);
    
    println!("--- Transfer with Auth Benchmark ---");
    env.budget().reset_default();
    client.transfer(&user1, &user2, &100);
    env.budget().print();
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

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, vec, Env};

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
// Negative Authentication Tests
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "HostError")]
fn test_transfer_missing_auth() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.set_balance(&admin, &from, &1000);

    // Disable all-auth-mocking and provide nothing, forcing an auth failure
    env.mock_auths(&[]);
    
    client.transfer(&from, &to, &300);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_approve_missing_auth() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);

    client.set_balance(&admin, &owner, &1000);

    // Disable all-auth-mocking
    env.mock_auths(&[]);
    
    client.approve(&owner, &spender, &500);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_transfer_from_missing_spender_auth() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.set_balance(&admin, &owner, &1000);
    client.approve(&owner, &spender, &500);

    // Disable all-auth-mocking
    env.mock_auths(&[]);

    // This requires `spender.require_auth()`
    client.transfer_from(&spender, &owner, &recipient, &200);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_admin_action_missing_auth() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    // Disable all-auth-mocking
    env.mock_auths(&[]);

    // This requires `admin.require_auth()`
    client.admin_action(&admin, &10);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_multi_sig_missing_one_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);
    
    let signers = vec![
        &env,
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];

    // Setting mock auths to empty causes any require_auth to panic
    env.mock_auths(&[]);

    client.multi_sig_action(&signers, &10);
}

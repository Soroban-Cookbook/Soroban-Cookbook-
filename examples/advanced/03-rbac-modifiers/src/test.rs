extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup() -> (Env, Address, RbacContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RbacContract);
    let client = RbacContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_grants_admin() {
    let (env, admin, client) = setup();
    assert!(client.has_role(&ROLE_ADMIN, &admin));
    assert!(!client.is_paused());
    let _ = env;
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_initialize_twice_panics() {
    let (env, _admin, client) = setup();
    let second = Address::generate(&env);
    client.initialize(&second);
}

// ---------------------------------------------------------------------------
// grant_role
// ---------------------------------------------------------------------------

#[test]
fn test_grant_role_by_admin() {
    let (env, admin, client) = setup();
    let minter = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MINTER, &minter);

    assert!(client.has_role(&ROLE_MINTER, &minter));
}

#[test]
fn test_grant_role_idempotent() {
    let (env, admin, client) = setup();
    let minter = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MINTER, &minter);
    client.grant_role(&admin, &ROLE_MINTER, &minter); // second grant is a no-op

    let members = client.get_role_members(&ROLE_MINTER);
    assert_eq!(members.len(), 1);
}

#[test]
#[should_panic(expected = "Caller does not have required role")]
fn test_grant_role_non_admin_panics() {
    let (env, _admin, client) = setup();
    let non_admin = Address::generate(&env);
    let target = Address::generate(&env);

    client.grant_role(&non_admin, &ROLE_MINTER, &target);
}

// ---------------------------------------------------------------------------
// revoke_role
// ---------------------------------------------------------------------------

#[test]
fn test_revoke_role_by_admin() {
    let (env, admin, client) = setup();
    let minter = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MINTER, &minter);
    assert!(client.has_role(&ROLE_MINTER, &minter));

    client.revoke_role(&admin, &ROLE_MINTER, &minter);
    assert!(!client.has_role(&ROLE_MINTER, &minter));
}

#[test]
fn test_revoke_role_not_held_is_noop() {
    let (env, admin, client) = setup();
    let nobody = Address::generate(&env);

    // Revoking a role that was never granted should not panic.
    client.revoke_role(&admin, &ROLE_MINTER, &nobody);
    assert!(!client.has_role(&ROLE_MINTER, &nobody));
}

#[test]
#[should_panic(expected = "Caller does not have required role")]
fn test_revoke_role_non_admin_panics() {
    let (env, admin, client) = setup();
    let minter = Address::generate(&env);
    let non_admin = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MINTER, &minter);
    client.revoke_role(&non_admin, &ROLE_MINTER, &minter);
}

// ---------------------------------------------------------------------------
// renounce_role
// ---------------------------------------------------------------------------

#[test]
fn test_renounce_role() {
    let (env, admin, client) = setup();
    let minter = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MINTER, &minter);
    assert!(client.has_role(&ROLE_MINTER, &minter));

    client.renounce_role(&minter, &ROLE_MINTER);
    assert!(!client.has_role(&ROLE_MINTER, &minter));
}

#[test]
fn test_renounce_role_not_held_is_noop() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);

    // Renouncing a role never held should not panic.
    client.renounce_role(&nobody, &ROLE_MINTER);
}

// ---------------------------------------------------------------------------
// get_role_members
// ---------------------------------------------------------------------------

#[test]
fn test_get_role_members_empty() {
    let (_env, _admin, client) = setup();
    let members = client.get_role_members(&ROLE_MINTER);
    assert_eq!(members.len(), 0);
}

#[test]
fn test_get_role_members_multiple() {
    let (env, admin, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MINTER, &a);
    client.grant_role(&admin, &ROLE_MINTER, &b);

    let members = client.get_role_members(&ROLE_MINTER);
    assert_eq!(members.len(), 2);
    assert!(members.contains(&a));
    assert!(members.contains(&b));
}

// ---------------------------------------------------------------------------
// Protected operations — only_role guard
// ---------------------------------------------------------------------------

#[test]
fn test_protected_mint_with_minter_role() {
    let (env, admin, client) = setup();
    let minter = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MINTER, &minter);
    client.protected_mint(&minter, &recipient, &1000i128);
}

#[test]
#[should_panic(expected = "Caller does not have required role")]
fn test_protected_mint_without_role_panics() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.protected_mint(&nobody, &recipient, &1000i128);
}

#[test]
fn test_admin_action_with_admin_role() {
    let (_env, admin, client) = setup();
    client.admin_action(&admin);
}

#[test]
#[should_panic(expected = "Caller does not have required role")]
fn test_admin_action_without_role_panics() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    client.admin_action(&nobody);
}

// ---------------------------------------------------------------------------
// Protected operations — any_role guard
// ---------------------------------------------------------------------------

#[test]
fn test_any_role_admin_passes() {
    let (_env, admin, client) = setup();
    client.admin_or_minter_action(&admin);
}

#[test]
fn test_any_role_minter_passes() {
    let (env, admin, client) = setup();
    let minter = Address::generate(&env);
    client.grant_role(&admin, &ROLE_MINTER, &minter);
    client.admin_or_minter_action(&minter);
}

#[test]
#[should_panic(expected = "Caller does not have any required role")]
fn test_any_role_neither_panics() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    client.admin_or_minter_action(&nobody);
}

// ---------------------------------------------------------------------------
// Pause / unpause
// ---------------------------------------------------------------------------

#[test]
fn test_pause_and_unpause() {
    let (env, admin, client) = setup();
    let pauser = Address::generate(&env);

    client.grant_role(&admin, &ROLE_PAUSER, &pauser);

    assert!(!client.is_paused());
    client.pause(&pauser);
    assert!(client.is_paused());
    client.unpause(&pauser);
    assert!(!client.is_paused());
}

#[test]
#[should_panic(expected = "Caller does not have required role")]
fn test_pause_without_pauser_role_panics() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    client.pause(&nobody);
}

#[test]
#[should_panic(expected = "Caller does not have required role")]
fn test_unpause_without_pauser_role_panics() {
    let (env, admin, client) = setup();
    let pauser = Address::generate(&env);
    client.grant_role(&admin, &ROLE_PAUSER, &pauser);
    client.pause(&pauser);

    let nobody = Address::generate(&env);
    client.unpause(&nobody);
}

// ---------------------------------------------------------------------------
// Auth guard — unauthorized caller
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_grant_role_unauthorized_caller() {
    let env = Env::default();
    // No mock_all_auths
    let contract_id = env.register_contract(None, RbacContract);
    let client = RbacContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);

    env.set_auths(&[]); // strip auths
    let target = Address::generate(&env);
    client.grant_role(&admin, &ROLE_MINTER, &target);
}

// ---------------------------------------------------------------------------
// Role lifecycle: grant → revoke → re-grant
// ---------------------------------------------------------------------------

#[test]
fn test_role_lifecycle() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    // Grant
    client.grant_role(&admin, &ROLE_MINTER, &user);
    assert!(client.has_role(&ROLE_MINTER, &user));

    // Revoke
    client.revoke_role(&admin, &ROLE_MINTER, &user);
    assert!(!client.has_role(&ROLE_MINTER, &user));

    // Re-grant
    client.grant_role(&admin, &ROLE_MINTER, &user);
    assert!(client.has_role(&ROLE_MINTER, &user));
}

// ---------------------------------------------------------------------------
// Multiple roles on same account
// ---------------------------------------------------------------------------

#[test]
fn test_account_can_hold_multiple_roles() {
    let (env, admin, client) = setup();
    let power_user = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MINTER, &power_user);
    client.grant_role(&admin, &ROLE_PAUSER, &power_user);

    assert!(client.has_role(&ROLE_MINTER, &power_user));
    assert!(client.has_role(&ROLE_PAUSER, &power_user));

    // Revoking one role does not affect the other.
    client.revoke_role(&admin, &ROLE_MINTER, &power_user);
    assert!(!client.has_role(&ROLE_MINTER, &power_user));
    assert!(client.has_role(&ROLE_PAUSER, &power_user));
}

// ---------------------------------------------------------------------------
// Custom role symbol
// ---------------------------------------------------------------------------

#[test]
fn test_custom_role() {
    let (env, admin, client) = setup();
    let custom_role = Symbol::new(&env, "BURNER");
    let burner = Address::generate(&env);

    client.grant_role(&admin, &custom_role, &burner);
    assert!(client.has_role(&custom_role, &burner));

    client.revoke_role(&admin, &custom_role, &burner);
    assert!(!client.has_role(&custom_role, &burner));
}

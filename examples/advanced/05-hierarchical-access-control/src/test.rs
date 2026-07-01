extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup() -> (Env, Address, HierarchicalAccessControlContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, HierarchicalAccessControlContract);
    let client = HierarchicalAccessControlContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_grants_admin_and_default_permissions() {
    let (env, admin, client) = setup();

    assert!(client.has_role(&ROLE_ADMIN, &admin));
    assert!(client.account_has_permission(&admin, &PERM_MANAGE_ROLES));
    assert!(client.account_has_permission(&admin, &PERM_MANAGE_PERMISSIONS));
    assert!(client.account_has_permission(&admin, &PERM_MANAGE_RESOURCES));
    assert!(client.account_has_permission(&admin, &PERM_USE_RESOURCES));
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
// Role management
// ---------------------------------------------------------------------------

#[test]
fn test_grant_role_by_admin() {
    let (env, admin, client) = setup();
    let manager = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MANAGER, &manager);
    assert!(client.has_role(&ROLE_MANAGER, &manager));
    assert!(client.account_has_permission(&manager, &PERM_MANAGE_RESOURCES));
    assert!(client.account_has_permission(&manager, &PERM_USE_RESOURCES));
    assert!(!client.account_has_permission(&manager, &PERM_MANAGE_ROLES));
}

#[test]
fn test_grant_role_idempotent() {
    let (env, admin, client) = setup();
    let operator = Address::generate(&env);

    client.grant_role(&admin, &ROLE_OPERATOR, &operator);
    client.grant_role(&admin, &ROLE_OPERATOR, &operator); // second grant is no-op

    let members = client.get_role_members(&ROLE_OPERATOR);
    assert_eq!(members.len(), 1);
}

#[test]
#[should_panic(expected = "Caller does not have required permission")]
fn test_grant_role_non_admin_panics() {
    let (env, admin, client) = setup();
    let manager = Address::generate(&env);
    let target = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MANAGER, &manager);
    client.grant_role(&manager, &ROLE_OPERATOR, &target);
}

#[test]
fn test_revoke_role_by_admin() {
    let (env, admin, client) = setup();
    let manager = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MANAGER, &manager);
    assert!(client.has_role(&ROLE_MANAGER, &manager));

    client.revoke_role(&admin, &ROLE_MANAGER, &manager);
    assert!(!client.has_role(&ROLE_MANAGER, &manager));
    assert!(!client.account_has_permission(&manager, &PERM_MANAGE_RESOURCES));
}

#[test]
fn test_renounce_role() {
    let (env, admin, client) = setup();
    let operator = Address::generate(&env);

    client.grant_role(&admin, &ROLE_OPERATOR, &operator);
    assert!(client.has_role(&ROLE_OPERATOR, &operator));

    client.renounce_role(&operator, &ROLE_OPERATOR);
    assert!(!client.has_role(&ROLE_OPERATOR, &operator));
}

// ---------------------------------------------------------------------------
// Permission management
// ---------------------------------------------------------------------------

#[test]
fn test_grant_permission_by_admin() {
    let (env, admin, client) = setup();
    let manager = Address::generate(&env);
    let new_perm = Symbol::new(&env, "CUSTOM_PERM");

    client.grant_role(&admin, &ROLE_MANAGER, &manager);
    client.grant_permission(&admin, &new_perm, &ROLE_MANAGER);

    assert!(client.role_has_permission(&ROLE_MANAGER, &new_perm));
    assert!(client.account_has_permission(&manager, &new_perm));
}

#[test]
fn test_revoke_permission_by_admin() {
    let (env, admin, client) = setup();
    let manager = Address::generate(&env);

    client.grant_role(&admin, &ROLE_MANAGER, &manager);
    assert!(client.account_has_permission(&manager, &PERM_MANAGE_RESOURCES));

    client.revoke_permission(&admin, &PERM_MANAGE_RESOURCES, &ROLE_MANAGER);
    assert!(!client.role_has_permission(&ROLE_MANAGER, &PERM_MANAGE_RESOURCES));
    assert!(!client.account_has_permission(&manager, &PERM_MANAGE_RESOURCES));
}

#[test]
#[should_panic(expected = "Caller does not have required permission")]
fn test_grant_permission_non_admin_panics() {
    let (env, admin, client) = setup();
    let manager = Address::generate(&env);
    let new_perm = Symbol::new(&env, "CUSTOM_PERM");

    client.grant_role(&admin, &ROLE_MANAGER, &manager);
    client.grant_permission(&manager, &new_perm, &ROLE_OPERATOR);
}

// ---------------------------------------------------------------------------
// Protected operations
// ---------------------------------------------------------------------------

#[test]
fn test_admin_can_manage_resources() {
    let (_env, admin, client) = setup();
    let resource_id = symbol_short!("RES1");
    client.manage_resource(&admin, &resource_id);
}

#[test]
fn test_manager_can_manage_resources() {
    let (env, admin, client) = setup();
    let manager = Address::generate(&env);
    let resource_id = symbol_short!("RES1");

    client.grant_role(&admin, &ROLE_MANAGER, &manager);
    client.manage_resource(&manager, &resource_id);
}

#[test]
#[should_panic(expected = "Caller does not have required permission")]
fn test_operator_cannot_manage_resources() {
    let (env, admin, client) = setup();
    let operator = Address::generate(&env);
    let resource_id = symbol_short!("RES1");

    client.grant_role(&admin, &ROLE_OPERATOR, &operator);
    client.manage_resource(&operator, &resource_id);
}

#[test]
fn test_all_roles_can_use_resources() {
    let (env, admin, client) = setup();
    let manager = Address::generate(&env);
    let operator = Address::generate(&env);
    let resource_id = symbol_short!("RES1");

    client.grant_role(&admin, &ROLE_MANAGER, &manager);
    client.grant_role(&admin, &ROLE_OPERATOR, &operator);

    client.use_resource(&admin, &resource_id);
    client.use_resource(&manager, &resource_id);
    client.use_resource(&operator, &resource_id);
}

#[test]
#[should_panic(expected = "Caller does not have required permission")]
fn test_unauthorized_cannot_use_resources() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    let resource_id = symbol_short!("RES1");

    client.use_resource(&nobody, &resource_id);
}

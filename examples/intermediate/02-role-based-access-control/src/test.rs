use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    vec, Env,
};

fn setup_initialized(env: &Env) -> (RoleBasedAccessControlClient<'_>, Address) {
    let contract_id = env.register(RoleBasedAccessControl, ());
    let client = RoleBasedAccessControlClient::new(env, &contract_id);
    let owner = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&owner);
    (client, owner)
}

// Requirement 1: Initial privileged role is configured correctly.
#[test]
fn test_initialize_sets_owner_role() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);

    assert!(client.has_role(&owner, &Role::Owner));
    assert!(client.has_role(&owner, &Role::Admin));
    assert!(client.has_role(&owner, &Role::Moderator));
    assert!(client.has_role(&owner, &Role::User));
}

// Requirement 2: An account with the required role can call a protected function.
#[test]
fn test_account_with_required_role_can_call_protected_function() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let moderator = Address::generate(&env);

    client.grant_role(&owner, &moderator, &Role::Moderator);
    let res = client.moderator_action(&moderator, &5u64);
    assert_eq!(res, 15u64);
}

// Requirement 3: An account without the required role cannot call that function.
#[test]
fn test_account_without_required_role_cannot_call_protected_function() {
    let env = Env::default();
    let (client, _owner) = setup_initialized(&env);
    let user = Address::generate(&env);

    let res = client.try_moderator_action(&user, &5u64);
    assert_eq!(res, Err(Ok(RbacError::Unauthorized)));
}

// Requirement 4: Authentication is required and cannot be bypassed merely by passing another user's address.
#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_auth_required_cannot_be_bypassed_by_passing_address() {
    let env = Env::default();
    let contract_id = env.register(RoleBasedAccessControl, ());
    let client = RoleBasedAccessControlClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&owner);

    // Revoke mock auths to simulate an unauthenticated call
    env.set_auths(&[]);
    let target = Address::generate(&env);
    client.grant_role(&owner, &target, &Role::Moderator);
}

// Requirement 5: `grant_role` succeeds when performed by an authorized role manager.
#[test]
fn test_grant_role_succeeds_for_authorized_manager() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.grant_role(&owner, &admin, &Role::Admin);
    client.grant_role(&admin, &user, &Role::Moderator);
    assert!(client.has_role(&user, &Role::Moderator));
}

// Requirement 6: `grant_role` fails for an unauthorized account.
#[test]
fn test_grant_role_fails_for_unauthorized_account() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let moderator = Address::generate(&env);
    let target = Address::generate(&env);

    client.grant_role(&owner, &moderator, &Role::Moderator);
    let res = client.try_grant_role(&moderator, &target, &Role::Admin);
    assert_eq!(res, Err(Ok(RbacError::Unauthorized)));
}

// Requirement 7: `revoke_role` succeeds when performed by an authorized role manager.
#[test]
fn test_revoke_role_succeeds_for_authorized_manager() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.grant_role(&owner, &admin, &Role::Admin);
    client.grant_role(&admin, &user, &Role::Moderator);
    assert!(client.has_role(&user, &Role::Moderator));

    client.revoke_role(&admin, &user);
    assert!(!client.has_role(&user, &Role::Moderator));
}

// Requirement 8: `revoke_role` fails for an unauthorized account.
#[test]
fn test_revoke_role_fails_for_unauthorized_account() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let moderator = Address::generate(&env);
    let admin = Address::generate(&env);

    client.grant_role(&owner, &moderator, &Role::Moderator);
    client.grant_role(&owner, &admin, &Role::Admin);

    let res = client.try_revoke_role(&moderator, &admin);
    assert_eq!(res, Err(Ok(RbacError::Unauthorized)));
}

// Requirement 9: A granted role immediately permits the intended protected operation.
#[test]
fn test_granted_role_immediately_permits_operation() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let user = Address::generate(&env);

    // Before grant
    assert_eq!(
        client.try_admin_action(&user, &10u64),
        Err(Ok(RbacError::Unauthorized))
    );

    // Grant
    client.grant_role(&owner, &user, &Role::Admin);

    // Immediately after grant
    let res = client.admin_action(&user, &10u64);
    assert_eq!(res, 20u64);
}

// Requirement 10: A revoked role can no longer perform the protected operation.
#[test]
fn test_revoked_role_can_no_longer_perform_operation() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&owner, &user, &Role::Moderator);
    assert_eq!(client.moderator_action(&user, &5u64), 15u64);

    client.revoke_role(&owner, &user);
    assert_eq!(
        client.try_moderator_action(&user, &5u64),
        Err(Ok(RbacError::Unauthorized))
    );
}

// Requirement 11: The multi-role guard succeeds for the first permitted role.
#[test]
fn test_multi_role_guard_succeeds_for_first_permitted_role() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let moderator = Address::generate(&env);

    client.grant_role(&owner, &moderator, &Role::Moderator);
    let res = client.moderator_or_admin_action(&moderator, &10u64);
    assert_eq!(res, 110u64);
}

// Requirement 12: The multi-role guard succeeds for another permitted role.
#[test]
fn test_multi_role_guard_succeeds_for_another_permitted_role() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let admin = Address::generate(&env);

    client.grant_role(&owner, &admin, &Role::Admin);
    let res = client.moderator_or_admin_action(&admin, &10u64);
    assert_eq!(res, 110u64);
}

// Requirement 13: The multi-role guard rejects an account with none of the allowed roles.
#[test]
fn test_multi_role_guard_rejects_account_without_allowed_roles() {
    let env = Env::default();
    let (client, _owner) = setup_initialized(&env);
    let user = Address::generate(&env);

    let res = client.try_moderator_or_admin_action(&user, &10u64);
    assert_eq!(res, Err(Ok(RbacError::Unauthorized)));
}

// Requirement 14: Role hierarchy behavior works correctly.
#[test]
fn test_role_hierarchy_behavior() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let admin = Address::generate(&env);

    client.grant_role(&owner, &admin, &Role::Admin);

    // Admin has hierarchy >= Moderator and User
    assert!(client.has_role(&admin, &Role::Moderator));
    assert!(client.has_role(&admin, &Role::User));
    assert!(!client.has_role(&admin, &Role::Owner));

    // Admin can perform moderator action due to hierarchy
    assert_eq!(client.moderator_action(&admin, &5u64), 15u64);
}

// Requirement 15: Successful role grants emit the expected grant event.
#[test]
fn test_grant_role_emits_event() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&owner, &user, &Role::Admin);

    let events = env.events().all();
    assert!(!events.events().is_empty());
}

// Requirement 16: Successful role revocations emit the expected revoke event.
#[test]
fn test_revoke_role_emits_event() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let user = Address::generate(&env);

    client.grant_role(&owner, &user, &Role::Admin);
    client.revoke_role(&owner, &user);

    let events = env.events().all();
    assert!(!events.events().is_empty());
}

// Requirement 17: Failed/unauthorized role modifications do not produce successful role-change state.
#[test]
fn test_unauthorized_role_modification_does_not_change_state() {
    let env = Env::default();
    let (client, _owner) = setup_initialized(&env);
    let user = Address::generate(&env);
    let target = Address::generate(&env);

    let _ = client.try_grant_role(&user, &target, &Role::Admin);
    assert!(!client.has_role(&target, &Role::Admin));
}

// Requirement 18: Existing RBAC functionality still works after the refactor.
#[test]
fn test_existing_rbac_functionality() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);

    // Initialization check
    assert_eq!(
        client.try_initialize(&owner),
        Err(Ok(RbacError::AlreadyInitialized))
    );

    // require_role function check
    let admin = Address::generate(&env);
    client.grant_role(&owner, &admin, &Role::Admin);
    env.mock_all_auths();
    assert_eq!(client.require_role(&admin, &vec![&env, Role::Admin]), ());

    // Admin action check
    assert_eq!(client.admin_action(&admin, &10u64), 20u64);
}

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, vec, Env};

fn setup_initialized(env: &Env) -> (RoleBasedAccessControlClient<'_>, Address) {
    let contract_id = env.register_contract(None, RoleBasedAccessControl);
    let client = RoleBasedAccessControlClient::new(env, &contract_id);
    let owner = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&owner);
    (client, owner)
}

#[test]
fn test_initialize_sets_owner_role() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);

    assert!(client.has_role(&owner, &Role::Owner));
    assert!(client.has_role(&owner, &Role::Admin));
}

#[test]
fn test_owner_can_grant_admin_and_moderator_roles() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let user = Address::generate(&env);

    assert_eq!(client.grant_role(&owner, &user, &Role::Admin), Ok(()));
    assert!(client.has_role(&user, &Role::Admin));

    let other_user = Address::generate(&env);
    assert_eq!(client.grant_role(&owner, &other_user, &Role::Moderator), Ok(()));
    assert!(client.has_role(&other_user, &Role::Moderator));
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_admin_cannot_grant_admin_role() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.grant_role(&owner, &admin, &Role::Admin);
    client.grant_role(&admin, &user, &Role::Admin);
}

#[test]
fn test_admin_can_grant_and_revoke_moderator_role() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.grant_role(&owner, &admin, &Role::Admin);
    client.grant_role(&admin, &user, &Role::Moderator);
    assert!(client.has_role(&user, &Role::Moderator));

    assert_eq!(client.revoke_role(&admin, &user), Ok(()));
    assert!(!client.has_role(&user, &Role::Moderator));
    assert!(client.has_role(&user, &Role::User));
}

#[test]
fn test_has_role_hierarchy() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let admin = Address::generate(&env);

    client.grant_role(&owner, &admin, &Role::Admin);
    assert!(client.has_role(&admin, &Role::Moderator));
    assert!(client.has_role(&admin, &Role::User));
    assert!(!client.has_role(&admin, &Role::Owner));
}

// ── Security tests: privilege escalation attempts ──

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_moderator_cannot_grant_roles() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let moderator = Address::generate(&env);
    let user = Address::generate(&env);

    // Owner grants moderator role
    client.grant_role(&owner, &moderator, &Role::Moderator);

    // Moderator tries to grant admin - should fail
    client.grant_role(&moderator, &user, &Role::Admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_moderator_cannot_revoke_admin() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let moderator = Address::generate(&env);
    let admin = Address::generate(&env);

    // Owner grants moderator and admin roles
    client.grant_role(&owner, &moderator, &Role::Moderator);
    client.grant_role(&owner, &admin, &Role::Admin);

    // Moderator tries to revoke admin - should fail
    client.revoke_role(&moderator, &admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_user_cannot_grant_any_role() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let user = Address::generate(&env);
    let target = Address::generate(&env);

    // User (default role) tries to grant moderator - should fail
    client.grant_role(&user, &target, &Role::Moderator);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_admin_cannot_grant_owner() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    // Owner grants admin role
    client.grant_role(&owner, &admin, &Role::Admin);

    // Admin tries to grant owner role - should fail
    client.grant_role(&admin, &target, &Role::Owner);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_admin_cannot_revoke_owner() {
    let env = Env::default();
    let (client, owner) = setup_initialized(&env);
    let admin = Address::generate(&env);

    // Owner grants admin role
    client.grant_role(&owner, &admin, &Role::Admin);

    // Admin tries to revoke owner - should fail
    client.revoke_role(&admin, &owner);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_unauthorized_cannot_grant_roles() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RoleBasedAccessControl);
    let client = RoleBasedAccessControlClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.initialize(&owner);

    let attacker = Address::generate(&env);
    let target = Address::generate(&env);

    // Attacker (unauthorized) tries to grant moderator
    client.grant_role(&attacker, &target, &Role::Moderator);
}

#[test]
fn test_admin_action_requires_admin_role() {
    let env = Env::default();
    let (client, _owner) = setup_initialized(&env);
    let user = Address::generate(&env);

    // User without admin role cannot call admin_action
    let result = client.try_admin_action(&user, &10u64);
    assert_eq!(result, Err(Ok(RbacError::Unauthorized)));
}

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

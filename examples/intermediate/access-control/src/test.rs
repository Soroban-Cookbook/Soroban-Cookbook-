#![allow(deprecated)]
use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, testutils::Ledger, Env};

fn setup_initialized(env: &Env) -> (AccessControlClient<'_>, Address) {
    let contract_id = env.register_contract(None, AccessControl);
    let client = AccessControlClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.initialize(
        &admin,
        &2,
        &Vec::from_array(env, [Address::generate(env), Address::generate(env)]),
        &100,
    );
    (client, admin)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, _) = setup_initialized(&env);
    assert_eq!(client.get_timelock_delay(), 100);
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccessControl);
    let client = AccessControlClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let signers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);
    client.initialize(&admin, &2, &signers, &100);
    assert_eq!(
        client.try_initialize(&admin, &2, &signers, &100),
        Err(Ok(AccessControlError::AlreadyInitialized))
    );
}

#[test]
fn test_initialize_invalid_threshold_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccessControl);
    let client = AccessControlClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let signers = Vec::from_array(&env, [Address::generate(&env)]);
    assert_eq!(
        client.try_initialize(&admin, &2, &signers, &100),
        Err(Ok(AccessControlError::InvalidThreshold))
    );
}

#[test]
fn test_grant_and_revoke_role() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let alice = Address::generate(&env);

    client.grant_role(&admin, &alice, &Role::Operator);
    assert!(client.has_role(&alice, &Role::Operator));
    assert!(client.has_role(&alice, &Role::User));

    client.revoke_role(&admin, &alice);
    assert!(!client.has_role(&alice, &Role::Operator));
}

#[test]
fn test_revoke_admin_fails() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    assert_eq!(
        client.try_revoke_role(&admin, &admin),
        Err(Ok(AccessControlError::InvalidRole))
    );
}

#[test]
fn test_unauthorized_grant_fails() {
    let env = Env::default();
    let (client, _) = setup_initialized(&env);
    let bob = Address::generate(&env);

    assert_eq!(
        client.try_grant_role(&bob, &bob, &Role::Operator),
        Err(Ok(AccessControlError::Unauthorized))
    );
}

#[test]
fn test_timelock_delay_update() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.set_timelock_delay(&admin, &200);
    assert_eq!(client.get_timelock_delay(), 200);
}

#[test]
fn test_pause_and_unpause() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    assert!(!client.is_paused());

    client.set_pause(&admin, &true);
    assert!(client.is_paused());

    client.set_pause(&admin, &false);
    assert!(!client.is_paused());
}

#[test]
fn test_create_and_get_proposal() {
    let env = Env::default();
    let (client, _) = setup_initialized(&env);
    let proposer = Address::generate(&env);

    env.mock_all_auths();

    env.ledger().set_timestamp(0);

    let proposal_id = client.create_proposal(&proposer, &symbol_short!("transfer"));
    assert_eq!(proposal_id, 1);

    env.ledger().set_timestamp(10);

    let proposal = client.get_proposal(&1).unwrap();
    assert_eq!(proposal.id, 1);
    assert_eq!(proposal.action, symbol_short!("transfer"));
    assert!(!proposal.executed);
}

#[test]
fn test_paused_blocks_proposals() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let proposer = Address::generate(&env);

    env.mock_all_auths();
    client.set_pause(&admin, &true);

    assert_eq!(
        client.try_create_proposal(&proposer, &symbol_short!("transfer")),
        Err(Ok(AccessControlError::TimelockActive))
    );
}

#[test]
fn test_proposal_execution_requires_approvals() {
    let env = Env::default();
    let (client, _) = setup_initialized(&env);
    let proposer = Address::generate(&env);

    env.mock_all_auths();

    env.ledger().set_timestamp(0);

    let proposal_id = client.create_proposal(&proposer, &symbol_short!("upgrade"));
    env.ledger().set_timestamp(101);

    assert_eq!(
        client.try_execute(&proposer, &proposal_id),
        Err(Ok(AccessControlError::InsufficientApprovals))
    );
}

#[test]
fn test_add_and_remove_signer() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let signer = Address::generate(&env);

    env.mock_all_auths();

    assert!(!client.is_signer(&signer));

    client.add_signer(&admin, &signer);
    assert!(client.is_signer(&signer));

    client.remove_signer(&admin, &signer);
    assert!(!client.is_signer(&signer));
}

#[test]
fn test_unauthorized_signer_operation_fails() {
    let env = Env::default();
    let (client, _) = setup_initialized(&env);
    let bob = Address::generate(&env);
    let signer = Address::generate(&env);

    env.mock_all_auths();

    assert_eq!(
        client.try_add_signer(&bob, &signer),
        Err(Ok(AccessControlError::Unauthorized))
    );
}

#[test]
fn test_has_role() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let alice = Address::generate(&env);

    env.mock_all_auths();
    client.grant_role(&admin, &alice, &Role::Auditor);

    assert!(client.has_role(&alice, &Role::User));
    assert!(client.has_role(&alice, &Role::Auditor));
    assert!(!client.has_role(&alice, &Role::Admin));
}

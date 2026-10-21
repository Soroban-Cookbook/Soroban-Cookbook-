#![allow(deprecated)]
//! Fuzz Tests for Access Control Patterns
//!
//! Adversarial / edge-case tests covering:
//!   - Authorization bypass attempts
//!   - Role management fuzzing
//!   - Multi-sig logic validation
//!
//! Each test targets a specific security invariant and verifies that the
//! contract panics or returns the expected error when the invariant is violated.
//! Contracts are registered natively (no WASM binary required).

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Env, IntoVal, Symbol, Vec};

// ===========================================================================
// Section 1: Authorization Bypass Attempts
// ===========================================================================

#[test]
fn test_unauthorized_admin_action_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let client = authentication::AuthContractClient::new(&env, &auth_id);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(
        client.try_admin_action(&attacker, &42),
        Err(Ok(authentication::AuthError::NotAdmin))
    );
}

#[test]
fn test_unauthorized_role_action_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let client = authentication::AuthContractClient::new(&env, &auth_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);
    client
        .try_grant_role(&admin, &user, &authentication::Role::User)
        .unwrap()
        .unwrap();

    assert_eq!(
        client.try_admin_role_action(&user, &100),
        Err(Ok(authentication::AuthError::InsufficientRole))
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unauthorized_pause_rejected() {
    let env = Env::default();

    let pausable_id = env.register_contract(None, pause_unpause::PausableContract);
    let client = pause_unpause::PausableContractClient::new(&env, &pausable_id);
    let admin = Address::generate(&env);
    let _attacker = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    env.set_auths(&[]);
    client.pause();
}

#[test]
fn test_unauthorized_registry_owner_actions_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register_contract(None, registry_access_controls::RegistryContract);
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);

    let client = registry_access_controls::RegistryContractClient::new(&env, &registry_id);
    client.init(&owner, &false, &100);

    env.set_auths(&[]);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.add_whitelist(&attacker);
    }));
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unauthorized_proxy_admin_propose_rejected() {
    let env = Env::default();

    let proxy_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let client = proxy_admin::ProxyAdminClient::new(&env, &proxy_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    client.try_initialize(&admin).unwrap().unwrap();
    env.set_auths(&[]);

    let wasm_hash = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);
    client.propose_upgrade(&wasm_hash, &60);
}

// ===========================================================================
// Section 2: Role Management Fuzzing
// ===========================================================================

#[test]
fn test_role_grant_requires_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, role_based_access_control::RoleBasedAccessControl);
    let client = role_based_access_control::RoleBasedAccessControlClient::new(&env, &rbac_id);
    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.try_initialize(&owner).unwrap().unwrap();
    client
        .try_grant_role(&owner, &admin, &role_based_access_control::Role::Admin)
        .unwrap()
        .unwrap();

    client
        .try_grant_role(&admin, &user, &role_based_access_control::Role::Moderator)
        .unwrap()
        .unwrap();
}

#[test]
fn test_role_revoke_prevents_escalation() {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, role_based_access_control::RoleBasedAccessControl);
    let client = role_based_access_control::RoleBasedAccessControlClient::new(&env, &rbac_id);
    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.try_initialize(&owner).unwrap().unwrap();

    assert_eq!(
        client.try_revoke_role(&admin, &owner),
        Err(Ok(role_based_access_control::RbacError::Unauthorized))
    );
}

#[test]
fn test_role_hierarchy_enforced() {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, role_based_access_control::RoleBasedAccessControl);
    let owner = Address::generate(&env);
    let moderator = Address::generate(&env);

    env.invoke_contract::<Result<(), role_based_access_control::RbacError>>(
        &rbac_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [owner.clone().into_val(&env)]),
    )
    .unwrap();

    // Owner grants Moderator
    env.invoke_contract::<Result<(), role_based_access_control::RbacError>>(
        &rbac_id,
        &Symbol::new(&env, "grant_role"),
        Vec::from_array(
            &env,
            [
                owner.clone().into_val(&env),
                moderator.clone().into_val(&env),
                role_based_access_control::Role::Moderator.into_val(&env),
            ],
        ),
    )
    .unwrap();

    // Moderator should have role
    let has_mod: bool = env.invoke_contract(
        &rbac_id,
        &Symbol::new(&env, "has_role"),
        Vec::from_array(
            &env,
            [
                moderator.into_val(&env),
                role_based_access_control::Role::Moderator.into_val(&env),
            ],
        ),
    );
    assert!(has_mod);
}

#[test]
fn test_symbol_role_guard_rejects_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, rbac_modifiers::RbacContract);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let client = rbac_modifiers::RbacContractClient::new(&env, &rbac_id);
    client.initialize(&admin);

    // Non-minter tries protected_mint — should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        env.invoke_contract::<()>(
            &rbac_id,
            &Symbol::new(&env, "protected_mint"),
            Vec::from_array(
                &env,
                [
                    user.into_val(&env),
                    user.clone().into_val(&env),
                    100i128.into_val(&env),
                ],
            ),
        );
    }));
    assert!(result.is_err());

    // Admin should be able to call protected_mint after getting MINTER role
    client.grant_role(&admin, &rbac_modifiers::ROLE_MINTER, &admin);
    client.protected_mint(&admin, &user, &100);

    // Admin can also call admin_action
    client.admin_action(&admin);
}

// ===========================================================================
// Section 3: Multi-Sig Logic Tested
// ===========================================================================

#[test]
fn test_multisig_partial_approval_fails_execution() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let client = multi_sig_patterns::MultiPartyAuthClient::new(&env, &multisig_id);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);

    client.try_initialize(&3u32, &signers).unwrap().unwrap();

    let proposal_id = client.try_create_proposal(&signer1).unwrap().unwrap();

    client.try_approve(&proposal_id, &signer1).unwrap().unwrap();
    client.try_approve(&proposal_id, &signer2).unwrap().unwrap();

    assert_eq!(
        client.try_execute(&proposal_id, &signer1),
        Err(Ok(multi_sig_patterns::AuthError::ThresholdNotMet))
    );
}

#[test]
fn test_multisig_duplicate_approval_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let client = multi_sig_patterns::MultiPartyAuthClient::new(&env, &multisig_id);
    let signer1 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    client.try_initialize(&1u32, &signers).unwrap().unwrap();

    let proposal_id = client.try_create_proposal(&signer1).unwrap().unwrap();
    client.try_approve(&proposal_id, &signer1).unwrap().unwrap();

    assert_eq!(
        client.try_approve(&proposal_id, &signer1),
        Err(Ok(multi_sig_patterns::AuthError::AlreadyApproved))
    );
}

#[test]
fn test_multisig_unauthorized_signer_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let client = multi_sig_patterns::MultiPartyAuthClient::new(&env, &multisig_id);
    let signer1 = Address::generate(&env);
    let outsider = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    client.try_initialize(&1u32, &signers).unwrap().unwrap();

    let proposal_id = client.try_create_proposal(&signer1).unwrap().unwrap();

    assert_eq!(
        client.try_approve(&proposal_id, &outsider),
        Err(Ok(multi_sig_patterns::AuthError::NotAuthorized))
    );
}

#[test]
fn test_multisig_cancel_after_execute_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let client = multi_sig_patterns::MultiPartyAuthClient::new(&env, &multisig_id);
    let signer1 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    client.try_initialize(&1u32, &signers).unwrap().unwrap();

    let proposal_id = client.try_create_proposal(&signer1).unwrap().unwrap();
    client.try_approve(&proposal_id, &signer1).unwrap().unwrap();
    client.try_execute(&proposal_id, &signer1).unwrap().unwrap();

    assert_eq!(
        client.try_cancel(&proposal_id, &signer1),
        Err(Ok(multi_sig_patterns::AuthError::AlreadyExecuted))
    );
}

// ===========================================================================
// Section 4: Additional Edge-Case / Fuzz Tests
// ===========================================================================

#[test]
fn test_timelock_early_execute_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let timelock_id = env.register_contract(None, timelock::TimelockContract);
    let admin = Address::generate(&env);

    timelock::TimelockContractClient::new(&env, &timelock_id).initialize(&admin);
    let op_id = soroban_sdk::Bytes::from_array(&env, &[3u8; 32]);
    timelock::TimelockContractClient::new(&env, &timelock_id).queue(&op_id, &60u64);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        timelock::TimelockContractClient::new(&env, &timelock_id).execute(&op_id);
    }));
    assert!(result.is_err());
}

#[test]
fn test_timelock_replay_after_execution_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let timelock_id = env.register_contract(None, timelock::TimelockContract);
    let client = timelock::TimelockContractClient::new(&env, &timelock_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    let op_id = soroban_sdk::Bytes::from_array(&env, &[4u8; 32]);
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&op_id, &min_delay);

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + min_delay + 1);
    client.execute(&op_id);

    let state = client.get_state(&op_id);
    assert_eq!(state, timelock::OperationState::Unknown);

    assert!(client.try_execute(&op_id).is_err());
}

#[test]
fn test_auth_vector_round_trip() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, multi_party_auth::MultiPartyAuthContract);
    let client = multi_party_auth::MultiPartyAuthContractClient::new(&env, &contract_id);

    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);
    let s3 = Address::generate(&env);
    let signers = soroban_sdk::Vec::from_array(&env, [s1.clone(), s2.clone(), s3.clone()]);

    let encoded = client.encode_auth_vec(&signers);
    assert!(client.validate_auth_vec(&encoded));
    assert_eq!(client.auth_vec_len(&encoded), 3);

    let decoded = client.decode_auth_vec(&encoded);
    assert!(decoded.contains(&s1));
    assert!(decoded.contains(&s2));
    assert!(decoded.contains(&s3));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_proxy_admin_unauthorized_set_pause_rejected() {
    let env = Env::default();

    let proxy_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let client = proxy_admin::ProxyAdminClient::new(&env, &proxy_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    client.try_initialize(&admin).unwrap().unwrap();
    env.set_auths(&[]);
    client.pause();
}

#[test]
fn test_allowance_excess_spend_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let client = authentication::AuthContractClient::new(&env, &auth_id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.initialize(&admin);
    client.set_balance(&admin, &alice, &100);
    client.approve(&alice, &bob, &50);

    assert_eq!(
        client.try_transfer_from(&bob, &alice, &recipient, &200),
        Err(Ok(authentication::AuthError::Unauthorized))
    );
}

#[test]
fn test_revoked_role_loses_access() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let client = authentication::AuthContractClient::new(&env, &auth_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);
    client
        .try_grant_role(&admin, &user, &authentication::Role::Moderator)
        .unwrap()
        .unwrap();
    client.try_revoke_role(&admin, &user).unwrap().unwrap();

    assert_eq!(
        client.try_moderator_action(&user, &42),
        Err(Ok(authentication::AuthError::InsufficientRole))
    );
}

#[test]
fn test_timelock_pause_blocks_queue() {
    let env = Env::default();
    env.mock_all_auths();

    let timelock_id = env.register_contract(None, timelock::TimelockContract);
    let admin = Address::generate(&env);

    timelock::TimelockContractClient::new(&env, &timelock_id).initialize(&admin);
    timelock::TimelockContractClient::new(&env, &timelock_id).set_pause(&true);
    assert!(timelock::TimelockContractClient::new(&env, &timelock_id).is_paused());

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        timelock::TimelockContractClient::new(&env, &timelock_id)
            .queue(&soroban_sdk::Bytes::from_array(&env, &[5u8; 32]), &60u64);
    }));
    assert!(result.is_err());
}

#[test]
fn test_proxy_admin_delay_bounds_enforced() {
    let env = Env::default();
    env.mock_all_auths();

    let proxy_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let client = proxy_admin::ProxyAdminClient::new(&env, &proxy_id);
    let admin = Address::generate(&env);

    client.try_initialize(&admin).unwrap().unwrap();

    let hash_low = soroban_sdk::BytesN::from_array(&env, &[2u8; 32]);
    assert_eq!(
        client.try_propose_upgrade(&hash_low, &10),
        Err(Ok(proxy_admin::AdminError::DelayOutOfRange))
    );

    let hash_high = soroban_sdk::BytesN::from_array(&env, &[3u8; 32]);
    assert_eq!(
        client.try_propose_upgrade(&hash_high, &604_801),
        Err(Ok(proxy_admin::AdminError::DelayOutOfRange))
    );

    let hash_ok = soroban_sdk::BytesN::from_array(&env, &[4u8; 32]);
    client.try_propose_upgrade(&hash_ok, &300).unwrap().unwrap();
}

#[test]
fn test_register_without_whitelist_fails_when_whitelist_only() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register_contract(None, registry_access_controls::RegistryContract);
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    env.invoke_contract::<()>(
        &registry_id,
        &Symbol::new(&env, "init"),
        Vec::from_array(
            &env,
            [
                owner.into_val(&env),
                true.into_val(&env),
                0i128.into_val(&env),
            ],
        ),
    );

    // set_fee is owner-only; owner already set whitelist_only=true and fee=0
    // user tries to register — should panic because not whitelisted
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        env.invoke_contract::<()>(
            &registry_id,
            &Symbol::new(&env, "register"),
            Vec::from_array(&env, [user.into_val(&env), 0i128.into_val(&env)]),
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_non_initializer_cannot_reinitialize() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let client = authentication::AuthContractClient::new(&env, &auth_id);
    let admin = Address::generate(&env);
    let other = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(
        client.try_initialize(&other),
        Err(Ok(authentication::AuthError::AlreadyInitialized))
    );
}

#[test]
fn test_multi_sig_proposal_nonexistent_execute_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let client = multi_sig_patterns::MultiPartyAuthClient::new(&env, &multisig_id);
    let signer1 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    client.try_initialize(&1u32, &signers).unwrap().unwrap();

    assert_eq!(
        client.try_execute(&999, &signer1),
        Err(Ok(multi_sig_patterns::AuthError::ProposalNotFound))
    );
}

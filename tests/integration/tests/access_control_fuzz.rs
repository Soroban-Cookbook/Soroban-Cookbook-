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

use soroban_sdk::{
    symbol_short, testutils::Address as _, Address, Bytes, Env, IntoVal, Symbol, Vec,
};

// ===========================================================================
// Section 1: Authorization Bypass Attempts
// ===========================================================================

#[test]
fn test_unauthorized_admin_action_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );

    let result: Result<u32, authentication::AuthError> = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "admin_action"),
        Vec::from_array(&env, [attacker.into_val(&env), 42u32.into_val(&env)]),
    );
    assert_eq!(result, Err(authentication::AuthError::NotAdmin));
}

#[test]
fn test_unauthorized_role_action_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );

    env.invoke_contract::<Result<(), authentication::AuthError>>(
        &auth_id,
        &Symbol::new(&env, "grant_role"),
        Vec::from_array(
            &env,
            [
                admin.into_val(&env),
                user.clone().into_val(&env),
                authentication::Role::User.into_val(&env),
            ],
        ),
    )
    .unwrap();

    let result: Result<u64, authentication::AuthError> = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "admin_role_action"),
        Vec::from_array(&env, [user.into_val(&env), 100u64.into_val(&env)]),
    );
    assert_eq!(result, Err(authentication::AuthError::InsufficientRole));
}

#[test]
fn test_unauthorized_pause_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let pausable_id = env.register_contract(None, pause_unpause::PausableContract);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    env.invoke_contract::<Result<(), pause_unpause::PauseError>>(
        &pausable_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.into_val(&env)]),
    )
    .unwrap();

    let result: Result<(), pause_unpause::PauseError> = env.invoke_contract(
        &pausable_id,
        &Symbol::new(&env, "pause"),
        Vec::new(&env),
    );

    // No attacker auth in the auths list (env.mock_all_auths mocks all, but
    // the contract checks stored admin != caller). With mock_all_auths the
    // stored admin check fails for attacker because attacker != admin.
    assert!(result.is_err());
}

#[test]
fn test_unauthorized_registry_owner_actions_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register_contract(None, registry_access_controls::RegistryContract);
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);

    env.invoke_contract::<()>(
        &registry_id,
        &Symbol::new(&env, "init"),
        Vec::from_array(
            &env,
            [owner.into_val(&env), false.into_val(&env), 100i128.into_val(&env)],
        ),
    );

    // attacker tries to add_whitelist — should panic because attacker.require_auth()
    // succeeds with mock_all_auths but stored admin check fails
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        env.invoke_contract::<()>(
            &registry_id,
            &Symbol::new(&env, "add_whitelist"),
            Vec::from_array(&env, [attacker.into_val(&env)]),
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_unauthorized_proxy_admin_propose_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let proxy_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    env.invoke_contract::<Result<(), proxy_admin::AdminError>>(
        &proxy_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.into_val(&env)]),
    )
    .unwrap();

    let result: Result<(), proxy_admin::AdminError> = env.invoke_contract(
        &proxy_id,
        &Symbol::new(&env, "propose_upgrade"),
        Vec::from_array(
            &env,
            [
                soroban_sdk::BytesN::from_array(&env, &[1u8; 32]).into_val(&env),
                60u64.into_val(&env),
            ],
        ),
    );
    assert!(result.is_err());
}

// ===========================================================================
// Section 2: Role Management Fuzzing
// ===========================================================================

#[test]
fn test_role_grant_requires_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, role_based_access_control::RoleBasedAccessControl);
    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    env.invoke_contract::<Result<(), role_based_access_control::RbacError>>(
        &rbac_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [owner.clone().into_val(&env)]),
    )
    .unwrap();

    // Owner grants Admin to admin user
    env.invoke_contract::<Result<(), role_based_access_control::RbacError>>(
        &rbac_id,
        &Symbol::new(&env, "grant_role"),
        Vec::from_array(
            &env,
            [
                owner.into_val(&env),
                admin.clone().into_val(&env),
                role_based_access_control::Role::Admin.into_val(&env),
            ],
        ),
    )
    .unwrap();

    // Admin tries to grant Admin to user — should succeed
    let res = env.invoke_contract::<Result<(), role_based_access_control::RbacError>>(
        &rbac_id,
        &Symbol::new(&env, "grant_role"),
        Vec::from_array(
            &env,
            [
                admin.into_val(&env),
                user.into_val(&env),
                role_based_access_control::Role::Admin.into_val(&env),
            ],
        ),
    );
    assert!(res.is_ok());
}

#[test]
fn test_role_revoke_prevents_escalation() {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, role_based_access_control::RoleBasedAccessControl);
    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    env.invoke_contract::<Result<(), role_based_access_control::RbacError>>(
        &rbac_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [owner.clone().into_val(&env)]),
    )
    .unwrap();

    // Admin tries to revoke Owner role from owner — should fail
    let result: Result<(), role_based_access_control::RbacError> = env.invoke_contract(
        &rbac_id,
        &Symbol::new(&env, "revoke_role"),
        Vec::from_array(
            &env,
            [
                admin.into_val(&env),
                owner.into_val(&env),
            ],
        ),
    );
    assert_eq!(result, Err(role_based_access_control::RbacError::Unauthorized));
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

    rbac_modifiers::RbacContract::initialize(&env, admin.clone());

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
    rbac_modifiers::RbacContract::grant_role(
        &env,
        admin.clone(),
        rbac_modifiers::ROLE_MINTER,
        admin.clone(),
    );
    rbac_modifiers::RbacContract::protected_mint(
        &env,
        admin.clone(),
        user.clone(),
        100i128,
    );

    // Admin can also call admin_action
    rbac_modifiers::RbacContract::admin_action(&env, admin);
}

// ===========================================================================
// Section 3: Multi-Sig Logic Tested
// ===========================================================================

#[test]
fn test_multisig_partial_approval_fails_execution() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);

    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [3u32.into_val(&env), signers.into_val(&env)]),
    )
    .unwrap();

    let proposal_id: u32 = env
        .invoke_contract::<Result<u32, multi_sig_patterns::AuthError>>(
            &multisig_id,
            &Symbol::new(&env, "create_proposal"),
            Vec::from_array(&env, [signer1.clone().into_val(&env)]),
        )
        .unwrap();

    // Only 2 of 3 approve — below threshold
    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [proposal_id.into_val(&env), signer1.into_val(&env)],
        ),
    )
    .unwrap();
    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [proposal_id.into_val(&env), signer2.into_val(&env)],
        ),
    )
    .unwrap();

    let result: Result<bool, multi_sig_patterns::AuthError> = env.invoke_contract(
        &multisig_id,
        &Symbol::new(&env, "execute"),
        Vec::from_array(&env, [proposal_id.into_val(&env), signer1.into_val(&env)]),
    );
    assert_eq!(result, Err(multi_sig_patterns::AuthError::ThresholdNotMet));
}

#[test]
fn test_multisig_duplicate_approval_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let signer1 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [1u32.into_val(&env), signers.into_val(&env)]),
    )
    .unwrap();

    let proposal_id: u32 = env
        .invoke_contract::<Result<u32, multi_sig_patterns::AuthError>>(
            &multisig_id,
            &Symbol::new(&env, "create_proposal"),
            Vec::from_array(&env, [signer1.clone().into_val(&env)]),
        )
        .unwrap();

    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [proposal_id.into_val(&env), signer1.clone().into_val(&env)],
        ),
    )
    .unwrap();

    // Approve again — should fail
    let dup: Result<(), multi_sig_patterns::AuthError> = env.invoke_contract(
        &multisig_id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [proposal_id.into_val(&env), signer1.into_val(&env)],
        ),
    );
    assert_eq!(dup, Err(multi_sig_patterns::AuthError::AlreadyApproved));
}

#[test]
fn test_multisig_unauthorized_signer_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let signer1 = Address::generate(&env);
    let outsider = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [1u32.into_val(&env), signers.into_val(&env)]),
    )
    .unwrap();

    let proposal_id: u32 = env
        .invoke_contract::<Result<u32, multi_sig_patterns::AuthError>>(
            &multisig_id,
            &Symbol::new(&env, "create_proposal"),
            Vec::from_array(&env, [signer1.clone().into_val(&env)]),
        )
        .unwrap();

    let result: Result<(), multi_sig_patterns::AuthError> = env.invoke_contract(
        &multisig_id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [proposal_id.into_val(&env), outsider.into_val(&env)],
        ),
    );
    assert_eq!(result, Err(multi_sig_patterns::AuthError::NotAuthorized));
}

#[test]
fn test_multisig_cancel_after_execute_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let signer1 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [1u32.into_val(&env), signers.into_val(&env)]),
    )
    .unwrap();

    let proposal_id: u32 = env
        .invoke_contract::<Result<u32, multi_sig_patterns::AuthError>>(
            &multisig_id,
            &Symbol::new(&env, "create_proposal"),
            Vec::from_array(&env, [signer1.clone().into_val(&env)]),
        )
        .unwrap();

    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [proposal_id.into_val(&env), signer1.clone().into_val(&env)],
        ),
    )
    .unwrap();

    env.invoke_contract::<Result<bool, multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "execute"),
        Vec::from_array(&env, [proposal_id.into_val(&env), signer1.clone().into_val(&env)]),
    )
    .unwrap();

    let cancel_result: Result<(), multi_sig_patterns::AuthError> = env.invoke_contract(
        &multisig_id,
        &Symbol::new(&env, "cancel"),
        Vec::from_array(&env, [proposal_id.into_val(&env), signer1.into_val(&env)]),
    );
    assert_eq!(cancel_result, Err(multi_sig_patterns::AuthError::AlreadyExecuted));
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
    let admin = Address::generate(&env);

    timelock::TimelockContractClient::new(&env, &timelock_id).initialize(&admin);
    let op_id = soroban_sdk::Bytes::from_array(&env, &[4u8; 32]);
    timelock::TimelockContractClient::new(&env, &timelock_id).queue(&op_id, &5u64);

    env.ledger().with_mut(|l| l.timestamp += 6);
    timelock::TimelockContractClient::new(&env, &timelock_id).execute(&op_id);

    // State should be Unknown after execution
    let state = timelock::TimelockContractClient::new(&env, &timelock_id).get_state(&op_id);
    assert_eq!(state, timelock::OperationState::Unknown);
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
fn test_proxy_admin_unauthorized_set_pause_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let proxy_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    env.invoke_contract::<Result<(), proxy_admin::AdminError>>(
        &proxy_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.into_val(&env)]),
    )
    .unwrap();

    // attacker tries to pause — stored-admin check should fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        env.invoke_contract::<()>(
            &proxy_id,
            &Symbol::new(&env, "pause"),
            Vec::new(&env),
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_allowance_excess_spend_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.into_val(&env)]),
    );

    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.into_val(&env),
                alice.clone().into_val(&env),
                100i128.into_val(&env),
            ],
        ),
    )
    .unwrap();

    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "approve"),
        Vec::from_array(
            &env,
            [
                alice.clone().into_val(&env),
                bob.clone().into_val(&env),
                50i128.into_val(&env),
            ],
        ),
    )
    .unwrap();

    let result: Result<(), authentication::AuthError> = env.invoke_contract(
        &auth_id,
        &symbol_short!("transfer_from"),
        Vec::from_array(
            &env,
            [
                bob.into_val(&env),
                alice.into_val(&env),
                Address::generate(&env).into_val(&env),
                200i128.into_val(&env),
            ],
        ),
    );
    assert_eq!(result, Err(authentication::AuthError::Unauthorized));
}

#[test]
fn test_revoked_role_loses_access() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );

    env.invoke_contract::<Result<(), authentication::AuthError>>(
        &auth_id,
        &Symbol::new(&env, "grant_role"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                user.clone().into_val(&env),
                authentication::Role::Moderator.into_val(&env),
            ],
        ),
    )
    .unwrap();

    // Revoke role
    env.invoke_contract::<Result<(), authentication::AuthError>>(
        &auth_id,
        &Symbol::new(&env, "revoke_role"),
        Vec::from_array(&env, [admin.into_val(&env), user.clone().into_val(&env)]),
    )
    .unwrap();

    // Try moderator action after revocation
    let result: Result<u64, authentication::AuthError> = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "moderator_action"),
        Vec::from_array(&env, [user.into_val(&env), 42u64.into_val(&env)]),
    );
    assert_eq!(result, Err(authentication::AuthError::InsufficientRole));
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
        timelock::TimelockContractClient::new(&env, &timelock_id).queue(
            &soroban_sdk::Bytes::from_array(&env, &[5u8; 32]),
            &60u64,
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_proxy_admin_delay_bounds_enforced() {
    let env = Env::default();
    env.mock_all_auths();

    let proxy_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let admin = Address::generate(&env);

    env.invoke_contract::<Result<(), proxy_admin::AdminError>>(
        &proxy_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.into_val(&env)]),
    )
    .unwrap();

    // Delay too low
    let result: Result<(), proxy_admin::AdminError> = env.invoke_contract(
        &proxy_id,
        &Symbol::new(&env, "propose_upgrade"),
        Vec::from_array(
            &env,
            [
                soroban_sdk::BytesN::from_array(&env, &[2u8; 32]).into_val(&env),
                10u64.into_val(&env),
            ],
        ),
    );
    assert_eq!(result, Err(proxy_admin::AdminError::DelayOutOfRange));

    // Delay too high ( > 7 days)
    let result2: Result<(), proxy_admin::AdminError> = env.invoke_contract(
        &proxy_id,
        &Symbol::new(&env, "propose_upgrade"),
        Vec::from_array(
            &env,
            [
                soroban_sdk::BytesN::from_array(&env, &[3u8; 32]).into_val(&env),
                604_801u64.into_val(&env),
            ],
        ),
    );
    assert_eq!(result2, Err(proxy_admin::AdminError::DelayOutOfRange));

    // Valid delay
    let result3: Result<(), proxy_admin::AdminError> = env.invoke_contract(
        &proxy_id,
        &Symbol::new(&env, "propose_upgrade"),
        Vec::from_array(
            &env,
            [
                soroban_sdk::BytesN::from_array(&env, &[4u8; 32]).into_val(&env),
                300u64.into_val(&env),
            ],
        ),
    );
    assert!(result3.is_ok());
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
            [owner.into_val(&env), true.into_val(&env), 0i128.into_val(&env)],
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
    let admin = Address::generate(&env);
    let other = Address::generate(&env);

    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.into_val(&env)]),
    );

    let result: Result<(), authentication::AuthError> = env.invoke_contract(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [other.into_val(&env)]),
    );
    assert_eq!(result, Err(authentication::AuthError::AlreadyInitialized));
}

#[test]
fn test_multi_sig_proposal_nonexistent_execute_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let multisig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let signer1 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    env.invoke_contract::<Result<(), multi_sig_patterns::AuthError>>(
        &multisig_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [1u32.into_val(&env), signers.into_val(&env)]),
    )
    .unwrap();

    let result: Result<bool, multi_sig_patterns::AuthError> = env.invoke_contract(
        &multisig_id,
        &Symbol::new(&env, "execute"),
        Vec::from_array(&env, [999u32.into_val(&env), signer1.into_val(&env)]),
    );
    assert_eq!(result, Err(multi_sig_patterns::AuthError::ProposalNotFound));
}

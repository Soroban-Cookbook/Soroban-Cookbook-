#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Bytes, Env, Vec,
};

use multi_sig_patterns::{MultiPartyAuth, MultiPartyAuthClient};
use role_based_access_control::{Role, RoleBasedAccessControl, RoleBasedAccessControlClient};
use timelock::{OperationState, TimelockContract, TimelockContractClient};

fn setup_env<'a>() -> (
    Env,
    Address,
    MultiPartyAuthClient<'a>,
    TimelockContractClient<'a>,
    RoleBasedAccessControlClient<'a>,
    Vec<Address>, // signers
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    // Deploy Multisig
    let multisig_id = env.register_contract(None, MultiPartyAuth);
    let multisig_client = MultiPartyAuthClient::new(&env, &multisig_id);

    // Deploy Timelock
    let timelock_id = env.register_contract(None, TimelockContract);
    let timelock_client = TimelockContractClient::new(&env, &timelock_id);

    // Deploy RBAC
    let rbac_id = env.register_contract(None, RoleBasedAccessControl);
    let rbac_client = RoleBasedAccessControlClient::new(&env, &rbac_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);

    // Initialize Multisig (threshold 2 out of 3)
    multisig_client.initialize(&2, &signers);

    // Initialize Timelock with multisig as admin
    timelock_client.initialize(&multisig_id);

    // Initialize RBAC with timelock as owner
    rbac_client.initialize(&timelock_id);

    (
        env,
        admin,
        multisig_client,
        timelock_client,
        rbac_client,
        signers,
    )
}

#[test]
fn test_flow_1_happy_path_full_pipeline() {
    let (env, _admin, multisig_client, timelock_client, rbac_client, signers) = setup_env();
    let operation_id = Bytes::from_slice(&env, b"grant_admin_to_user");
    let delay = 60;

    let proposal_id = multisig_client.create_proposal(&signers.get_unchecked(0));
    multisig_client.approve(&proposal_id, &signers.get_unchecked(0));
    multisig_client.approve(&proposal_id, &signers.get_unchecked(1));
    let executed = multisig_client.execute(&proposal_id, &signers.get_unchecked(0));
    assert!(executed);

    timelock_client.queue(&operation_id, &delay);
    assert_eq!(
        timelock_client.get_state(&operation_id),
        OperationState::Pending
    );

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + delay + 1);
    assert_eq!(
        timelock_client.get_state(&operation_id),
        OperationState::Ready
    );

    timelock_client.execute(&operation_id);
    assert_eq!(
        timelock_client.get_state(&operation_id),
        OperationState::Unknown
    );

    let target_user = Address::generate(&env);
    rbac_client.grant_role(&timelock_client.address, &target_user, &Role::Admin);

    assert!(rbac_client.has_role(&target_user, &Role::Admin));
}

#[test]
fn test_flow_2_multisig_threshold_not_met() {
    let (_env, _admin, multisig_client, _timelock_client, _rbac_client, signers) = setup_env();

    let proposal_id = multisig_client.create_proposal(&signers.get_unchecked(0));

    // Only 1 approval, threshold is 2
    multisig_client.approve(&proposal_id, &signers.get_unchecked(0));

    // Attempt execution, it should fail. We catch the panic.
    let res = multisig_client.try_execute(&proposal_id, &signers.get_unchecked(0));
    assert!(
        res.is_err(),
        "Expected execution to fail due to threshold not met"
    );
}

#[test]
fn test_flow_3_timelock_executed_too_early() {
    let (env, _admin, multisig_client, timelock_client, _rbac_client, signers) = setup_env();
    let operation_id = Bytes::from_slice(&env, b"op_too_early");
    let delay = 60;

    let proposal_id = multisig_client.create_proposal(&signers.get_unchecked(0));
    multisig_client.approve(&proposal_id, &signers.get_unchecked(0));
    multisig_client.approve(&proposal_id, &signers.get_unchecked(1));
    let executed = multisig_client.execute(&proposal_id, &signers.get_unchecked(0));
    assert!(executed);

    timelock_client.queue(&operation_id, &delay);

    // Advance time, but not enough
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + delay - 10);
    assert_eq!(
        timelock_client.get_state(&operation_id),
        OperationState::Pending
    );

    let res = timelock_client.try_execute(&operation_id);
    assert!(
        res.is_err(),
        "Expected execution to fail because delay hasn't passed"
    );
}

#[test]
fn test_flow_4_unauthorized_timelock_queueing() {
    let (_env, _admin, _multisig_client, _timelock_client, _rbac_client, _signers) = setup_env();
    let delay = 60;

    let env2 = Env::default();
    let operation_id = Bytes::from_slice(&env2, b"unauth_op");
    let multisig_id = env2.register_contract(None, MultiPartyAuth);
    let timelock_id = env2.register_contract(None, TimelockContract);
    let timelock_client2 = TimelockContractClient::new(&env2, &timelock_id);

    timelock_client2.initialize(&multisig_id);

    let res = timelock_client2.try_queue(&operation_id, &delay);
    assert!(res.is_err(), "Expected queue to fail due to lack of auth");
}

#[test]
fn test_flow_5_cancellation_workflow() {
    let (env, _admin, multisig_client, timelock_client, _rbac_client, signers) = setup_env();
    let operation_id = Bytes::from_slice(&env, b"op_to_cancel");
    let delay = 60;

    let proposal_id1 = multisig_client.create_proposal(&signers.get_unchecked(0));
    multisig_client.approve(&proposal_id1, &signers.get_unchecked(0));
    multisig_client.approve(&proposal_id1, &signers.get_unchecked(1));
    multisig_client.execute(&proposal_id1, &signers.get_unchecked(0));

    timelock_client.queue(&operation_id, &delay);
    assert_eq!(
        timelock_client.get_state(&operation_id),
        OperationState::Pending
    );

    let proposal_id2 = multisig_client.create_proposal(&signers.get_unchecked(0));
    multisig_client.approve(&proposal_id2, &signers.get_unchecked(0));
    multisig_client.approve(&proposal_id2, &signers.get_unchecked(1));
    multisig_client.execute(&proposal_id2, &signers.get_unchecked(0));

    timelock_client.cancel(&operation_id);
    assert_eq!(
        timelock_client.get_state(&operation_id),
        OperationState::Unknown
    );

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + delay + 1);
    let res = timelock_client.try_execute(&operation_id);
    assert!(
        res.is_err(),
        "Expected execution to fail because operation was cancelled"
    );
}

#[test]
fn test_flow_6_rbac_validation_failure() {
    let (env, _admin, multisig_client, timelock_client, rbac_client, signers) = setup_env();
    let operation_id = Bytes::from_slice(&env, b"grant_moderator");
    let delay = 60;

    let proposal_id = multisig_client.create_proposal(&signers.get_unchecked(0));
    multisig_client.approve(&proposal_id, &signers.get_unchecked(0));
    multisig_client.approve(&proposal_id, &signers.get_unchecked(1));
    multisig_client.execute(&proposal_id, &signers.get_unchecked(0));

    timelock_client.queue(&operation_id, &delay);

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + delay + 1);
    timelock_client.execute(&operation_id);

    let target_user = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let res = rbac_client.try_grant_role(&unauthorized, &target_user, &Role::Moderator);
    assert!(
        res.is_err(),
        "Expected role grant to fail due to unauthorized caller"
    );
}

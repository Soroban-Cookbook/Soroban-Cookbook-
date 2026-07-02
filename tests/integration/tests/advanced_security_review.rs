#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use batch_operations::{BatchError, BatchOperation};
use proposal_validation::ProposalError;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, Env, Symbol, Vec,
};

#[test]
fn test_atomic_batch_rollback_security_invariant() {
    let env = Env::default();
    let contract_id = env.register_contract(None, batch_operations::BatchOperations);
    let client = batch_operations::BatchOperationsClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.set_balance(&alice, &100).unwrap();
    client.set_balance(&bob, &0).unwrap();

    let ops = Vec::from_array(
        &env,
        [
            BatchOperation::Transfer(alice.clone(), bob.clone(), 30),
            BatchOperation::Debit(bob.clone(), 999),
        ],
    );

    assert_eq!(
        client.execute_batch_atomic(&ops),
        Err(Ok(BatchError::InsufficientBalance))
    );
    assert_eq!(client.get_balance(&alice), 100);
    assert_eq!(client.get_balance(&bob), 0);
}

#[test]
fn test_timelock_replay_protection_security_invariant() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 10_000);

    let contract_id = env.register_contract(None, timelock::TimelockContract);
    let client = timelock::TimelockContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let operation_id = Bytes::from_slice(&env, b"upgrade-op");
    client.queue(&operation_id, &60);

    env.ledger().with_mut(|l| l.timestamp += 61);
    client.execute(&operation_id);

    let replay = client.try_execute(&operation_id);
    assert!(replay.is_err());
}

#[test]
fn test_governance_topic_conflict_detection_security_invariant() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 2_000);

    let contract_id = env.register_contract(None, proposal_validation::ProposalValidation);
    let client = proposal_validation::ProposalValidationClient::new(&env, &contract_id);

    let topic = Symbol::new(&env, "bridge");
    let first = client
        .create_proposal(
            &topic,
            &2_020,
            &2_120,
            &6_000,
            &Bytes::from_slice(&env, b"hash-a"),
        )
        .unwrap();

    assert_eq!(first, 1);

    let conflict = client.create_proposal(
        &topic,
        &2_100,
        &2_200,
        &6_000,
        &Bytes::from_slice(&env, b"hash-b"),
    );
    assert_eq!(conflict, Err(Ok(ProposalError::TopicConflict)));
}

extern crate std;

use super::*;
use soroban_sdk::{testutils::Ledger, Bytes, Env, Symbol};

fn setup() -> (Env, ProposalValidationClient<'static>) {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProposalValidation);
    let client = ProposalValidationClient::new(&env, &contract_id);
    env.ledger().with_mut(|ledger| ledger.timestamp = 1_000);
    (env, client)
}

fn bytes(env: &Env, input: &[u8]) -> Bytes {
    Bytes::from_slice(env, input)
}

#[test]
fn test_create_proposal_success() {
    let (env, client) = setup();
    let topic = Symbol::new(&env, "treasury");

    let id = client
        .create_proposal(&topic, &1_020, &1_120, &6_000, &bytes(&env, b"hash-1"))
        .unwrap();

    assert_eq!(id, 1);
    let proposal = client.get_proposal(&id).unwrap();
    assert_eq!(proposal.topic, topic);
    assert!(proposal.active);
}

#[test]
fn test_reject_start_too_soon() {
    let (env, client) = setup();

    let result = client.create_proposal(
        &Symbol::new(&env, "ops"),
        &1_009,
        &1_100,
        &5_000,
        &bytes(&env, b"hash"),
    );
    assert_eq!(result, Err(Ok(ProposalError::InvalidWindow)));
}

#[test]
fn test_reject_end_before_start() {
    let (env, client) = setup();

    let result = client.create_proposal(
        &Symbol::new(&env, "ops"),
        &1_020,
        &1_019,
        &5_000,
        &bytes(&env, b"hash"),
    );
    assert_eq!(result, Err(Ok(ProposalError::InvalidWindow)));
}

#[test]
fn test_reject_duration_too_short() {
    let (env, client) = setup();

    let result = client.create_proposal(
        &Symbol::new(&env, "ops"),
        &1_020,
        &1_030,
        &5_000,
        &bytes(&env, b"hash"),
    );
    assert_eq!(result, Err(Ok(ProposalError::InvalidDuration)));
}

#[test]
fn test_reject_invalid_quorum() {
    let (env, client) = setup();

    let low = client.create_proposal(
        &Symbol::new(&env, "ops"),
        &1_020,
        &1_120,
        &50,
        &bytes(&env, b"hash"),
    );
    assert_eq!(low, Err(Ok(ProposalError::InvalidQuorum)));

    let high = client.create_proposal(
        &Symbol::new(&env, "ops"),
        &1_020,
        &1_120,
        &20_000,
        &bytes(&env, b"hash"),
    );
    assert_eq!(high, Err(Ok(ProposalError::InvalidQuorum)));
}

#[test]
fn test_reject_empty_metadata() {
    let (env, client) = setup();

    let result = client.create_proposal(
        &Symbol::new(&env, "ops"),
        &1_020,
        &1_120,
        &5_000,
        &Bytes::new(&env),
    );
    assert_eq!(result, Err(Ok(ProposalError::InvalidMetadata)));
}

#[test]
fn test_detect_topic_conflict() {
    let (env, client) = setup();
    let topic = Symbol::new(&env, "bridge");

    client
        .create_proposal(&topic, &1_020, &1_120, &5_000, &bytes(&env, b"hash-a"))
        .unwrap();

    let conflict = client.create_proposal(&topic, &1_100, &1_200, &5_000, &bytes(&env, b"hash-b"));
    assert_eq!(conflict, Err(Ok(ProposalError::TopicConflict)));
}

#[test]
fn test_allow_same_topic_after_close() {
    let (env, client) = setup();
    let topic = Symbol::new(&env, "bridge");

    let first = client
        .create_proposal(&topic, &1_020, &1_120, &5_000, &bytes(&env, b"hash-a"))
        .unwrap();
    client.close_proposal(&first).unwrap();

    let second = client
        .create_proposal(&topic, &1_130, &1_230, &5_000, &bytes(&env, b"hash-b"))
        .unwrap();
    assert_eq!(second, 2);
}

#[test]
fn test_non_conflicting_topics_are_allowed() {
    let (env, client) = setup();

    client
        .create_proposal(
            &Symbol::new(&env, "treasury"),
            &1_020,
            &1_120,
            &5_000,
            &bytes(&env, b"hash-a"),
        )
        .unwrap();

    let other = client
        .create_proposal(
            &Symbol::new(&env, "validator"),
            &1_030,
            &1_130,
            &6_500,
            &bytes(&env, b"hash-b"),
        )
        .unwrap();

    assert_eq!(other, 2);
}

#[test]
fn test_close_errors_for_missing_or_closed_proposal() {
    let (env, client) = setup();

    assert_eq!(client.close_proposal(&404), Err(Ok(ProposalError::ProposalNotFound)));

    let id = client
        .create_proposal(
            &Symbol::new(&env, "ops"),
            &1_020,
            &1_120,
            &5_000,
            &bytes(&env, b"hash"),
        )
        .unwrap();
    client.close_proposal(&id).unwrap();

    assert_eq!(
        client.close_proposal(&id),
        Err(Ok(ProposalError::ProposalAlreadyClosed))
    );
}

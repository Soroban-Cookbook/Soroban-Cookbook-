extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, Env,
};

const RATE_LIMIT: i128 = 1_000;
const RATE_WINDOW: u64 = 60;
const CHALLENGE_PERIOD: u64 = 120;

struct Fixture {
    env: Env,
    client: BridgeSecurityContractClient<'static>,
    admin: Address,
    operator: Address,
    reviewer: Address,
    challenger: Address,
    recipient: Address,
}

fn proof(env: &Env, bytes: &[u8]) -> Bytes {
    Bytes::from_slice(env, bytes)
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, BridgeSecurityContract);
    let client = BridgeSecurityContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let reviewer = Address::generate(&env);
    let challenger = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.initialize(&admin, &RATE_LIMIT, &RATE_WINDOW, &CHALLENGE_PERIOD);

    Fixture {
        env,
        client,
        admin,
        operator,
        reviewer,
        challenger,
        recipient,
    }
}

fn submit_default_transfer(f: &Fixture, amount: i128) -> u64 {
    f.client.submit_transfer(
        &f.operator,
        &f.recipient,
        &amount,
        &1u32,
        &proof(&f.env, b"bridge-proof"),
    )
}

#[test]
fn initialize_sets_config() {
    let f = setup();
    let rate_limit = f.client.get_rate_limit_state();

    assert_eq!(rate_limit.amount_limit, RATE_LIMIT);
    assert_eq!(rate_limit.window_seconds, RATE_WINDOW);
    assert_eq!(f.client.get_challenge_period(), CHALLENGE_PERIOD);
    assert!(!f.client.is_paused());
}

#[test]
fn submit_transfer_records_pending_transfer() {
    let f = setup();
    let transfer_id = submit_default_transfer(&f, 400);
    let transfer = f.client.get_transfer(&transfer_id);

    assert_eq!(transfer.operator, f.operator);
    assert_eq!(transfer.recipient, f.recipient);
    assert_eq!(transfer.amount, 400);
    assert_eq!(transfer.status, TransferStatus::Pending);
}

#[test]
fn submit_transfer_updates_rate_limit_usage() {
    let f = setup();
    submit_default_transfer(&f, 400);

    let state = f.client.get_rate_limit_state();
    assert_eq!(state.used_in_window, 400);
}

#[test]
fn rate_limit_rejects_excess_value_in_same_window() {
    let f = setup();
    submit_default_transfer(&f, 700);

    assert_eq!(
        f.client.try_submit_transfer(
            &f.operator,
            &f.recipient,
            &400,
            &1u32,
            &proof(&f.env, b"second-proof"),
        ),
        Err(Ok(BridgeError::RateLimitExceeded))
    );
}

#[test]
fn rate_limit_resets_after_window_rollover() {
    let f = setup();
    submit_default_transfer(&f, 900);

    f.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp += RATE_WINDOW + 1);

    let transfer_id = submit_default_transfer(&f, 800);
    let transfer = f.client.get_transfer(&transfer_id);
    let state = f.client.get_rate_limit_state();

    assert_eq!(transfer.amount, 800);
    assert_eq!(state.used_in_window, 800);
}

#[test]
fn pause_blocks_new_submissions() {
    let f = setup();
    f.client.pause(&f.admin);

    assert_eq!(
        f.client.try_submit_transfer(
            &f.operator,
            &f.recipient,
            &100,
            &1u32,
            &proof(&f.env, b"paused-proof"),
        ),
        Err(Ok(BridgeError::ContractPaused))
    );
}

#[test]
fn unpause_restores_submissions() {
    let f = setup();
    f.client.pause(&f.admin);
    f.client.unpause(&f.admin);

    let transfer_id = submit_default_transfer(&f, 100);
    assert_eq!(f.client.get_transfer(&transfer_id).amount, 100);
}

#[test]
fn finalize_rejects_before_challenge_period_expires() {
    let f = setup();
    let transfer_id = submit_default_transfer(&f, 200);

    assert_eq!(
        f.client.try_finalize_transfer(&f.operator, &transfer_id),
        Err(Ok(BridgeError::ChallengeWindowOpen))
    );
}

#[test]
fn finalize_succeeds_after_challenge_period() {
    let f = setup();
    let transfer_id = submit_default_transfer(&f, 200);

    f.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp += CHALLENGE_PERIOD);

    f.client.finalize_transfer(&f.operator, &transfer_id);
    assert_eq!(
        f.client.get_transfer(&transfer_id).status,
        TransferStatus::Finalized
    );
}

#[test]
fn challenge_within_period_blocks_finalization() {
    let f = setup();
    let transfer_id = submit_default_transfer(&f, 200);
    f.client.challenge_transfer(&f.challenger, &transfer_id);

    f.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp += CHALLENGE_PERIOD);

    assert_eq!(
        f.client.try_finalize_transfer(&f.operator, &transfer_id),
        Err(Ok(BridgeError::TransferChallenged))
    );
}

#[test]
fn challenge_rejects_after_window_closes() {
    let f = setup();
    let transfer_id = submit_default_transfer(&f, 200);

    f.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp += CHALLENGE_PERIOD + 1);

    assert_eq!(
        f.client.try_challenge_transfer(&f.challenger, &transfer_id),
        Err(Ok(BridgeError::ChallengeWindowClosed))
    );
}

#[test]
fn fraud_proof_marks_transfer_as_fraudulent() {
    let f = setup();
    let transfer_id = submit_default_transfer(&f, 300);

    f.client
        .submit_fraud_proof(&f.reviewer, &transfer_id, &proof(&f.env, b"fraud-proof"));

    assert_eq!(
        f.client.get_transfer(&transfer_id).status,
        TransferStatus::Fraudulent
    );
    assert_eq!(
        f.client.get_fraud_proof(&transfer_id),
        Some(proof(&f.env, b"fraud-proof"))
    );
}

#[test]
fn finalized_transfer_cannot_be_resolved_again() {
    let f = setup();
    let transfer_id = submit_default_transfer(&f, 200);

    f.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp += CHALLENGE_PERIOD);
    f.client.finalize_transfer(&f.operator, &transfer_id);

    assert_eq!(
        f.client.try_finalize_transfer(&f.operator, &transfer_id),
        Err(Ok(BridgeError::TransferAlreadyResolved))
    );
    assert_eq!(
        f.client
            .try_submit_fraud_proof(&f.reviewer, &transfer_id, &proof(&f.env, b"late-proof")),
        Err(Ok(BridgeError::TransferAlreadyResolved))
    );
}

#[test]
fn finalize_rejects_wrong_operator() {
    let f = setup();
    let transfer_id = submit_default_transfer(&f, 100);
    let other_operator = Address::generate(&f.env);

    f.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp += CHALLENGE_PERIOD);

    assert_eq!(
        f.client
            .try_finalize_transfer(&other_operator, &transfer_id),
        Err(Ok(BridgeError::Unauthorized))
    );
}

#[test]
fn initialize_rejects_invalid_config() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BridgeSecurityContract);
    let client = BridgeSecurityContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    assert_eq!(
        client.try_initialize(&admin, &0, &RATE_WINDOW, &CHALLENGE_PERIOD),
        Err(Ok(BridgeError::InvalidConfig))
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn submit_transfer_requires_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BridgeSecurityContract);
    let client = BridgeSecurityContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &RATE_LIMIT, &RATE_WINDOW, &CHALLENGE_PERIOD);
    env.set_auths(&[]);

    client.submit_transfer(&operator, &recipient, &100, &1u32, &proof(&env, b"proof"));
}

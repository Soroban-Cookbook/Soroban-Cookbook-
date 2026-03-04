#![cfg(test)]

extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, Env,
};

fn setup() -> (Env, Address, TimelockContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TimelockContract);
    let client = TimelockContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

fn op_id(env: &Env, s: &[u8]) -> Bytes {
    Bytes::from_slice(env, s)
}

// ── queue ────────────────────────────────────────────────────────────────────

#[test]
fn test_queue_success() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"op1");
    client.queue(&id, &MIN_DELAY);
    // should be in Pending state immediately after queuing
    assert_eq!(client.get_state(&id), OperationState::Pending);
}

#[test]
#[should_panic(expected = "Delay out of range")]
fn test_queue_delay_too_short() {
    let (env, _admin, client) = setup();
    client.queue(&op_id(&env, b"op2"), &(MIN_DELAY - 1));
}

#[test]
#[should_panic(expected = "Delay out of range")]
fn test_queue_delay_too_long() {
    let (env, _admin, client) = setup();
    client.queue(&op_id(&env, b"op3"), &(MAX_DELAY + 1));
}

#[test]
#[should_panic(expected = "Operation already queued")]
fn test_queue_duplicate() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"op4");
    client.queue(&id, &MIN_DELAY);
    client.queue(&id, &MIN_DELAY); // second call should panic
}

// ── execute ──────────────────────────────────────────────────────────────────

#[test]
fn test_execute_after_delay() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"exec1");
    client.queue(&id, &MIN_DELAY);

    // advance ledger time past the delay
    env.ledger().with_mut(|l| l.timestamp += MIN_DELAY + 1);

    assert_eq!(client.get_state(&id), OperationState::Ready);
    client.execute(&id);
    // after execution the operation is gone
    assert_eq!(client.get_state(&id), OperationState::Unknown);
}

#[test]
#[should_panic(expected = "Too early")]
fn test_execute_too_early() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"early1");
    client.queue(&id, &MIN_DELAY);
    // do NOT advance time
    client.execute(&id);
}

#[test]
#[should_panic(expected = "Operation not found")]
fn test_execute_nonexistent() {
    let (env, _admin, client) = setup();
    client.execute(&op_id(&env, b"ghost"));
}

#[test]
#[should_panic(expected = "Operation not found")]
fn test_execute_replay() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"replay1");
    client.queue(&id, &MIN_DELAY);
    env.ledger().with_mut(|l| l.timestamp += MIN_DELAY + 1);
    client.execute(&id);
    client.execute(&id); // replay — must panic
}

// ── cancel ───────────────────────────────────────────────────────────────────

#[test]
fn test_cancel_success() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"cancel1");
    client.queue(&id, &MIN_DELAY);
    client.cancel(&id);
    assert_eq!(client.get_state(&id), OperationState::Unknown);
}

#[test]
#[should_panic(expected = "Operation not found")]
fn test_cancel_nonexistent() {
    let (env, _admin, client) = setup();
    client.cancel(&op_id(&env, b"ghost2"));
}

// ── auth guards ──────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_queue_unauthorized() {
    let env = Env::default();
    // no mock_all_auths
    let contract_id = env.register_contract(None, TimelockContract);
    let client = TimelockContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);
    env.set_auths(&[]); // strip auths
    client.queue(&op_id(&env, b"unauth"), &MIN_DELAY);
}

// ── state helpers ─────────────────────────────────────────────────────────────

#[test]
fn test_get_execute_at() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"ts1");
    let before = env.ledger().timestamp();
    client.queue(&id, &MIN_DELAY);
    let execute_at = client.get_execute_at(&id);
    assert_eq!(execute_at, before + MIN_DELAY);
}

#[test]
fn test_get_state_unknown() {
    let (env, _admin, client) = setup();
    assert_eq!(
        client.get_state(&op_id(&env, b"nope")),
        OperationState::Unknown
    );
}

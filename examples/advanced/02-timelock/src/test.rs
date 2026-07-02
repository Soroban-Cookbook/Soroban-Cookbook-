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
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);
    // should be in Pending state immediately after queuing
    assert_eq!(client.get_state(&id), OperationState::Pending);
}

#[test]
#[should_panic(expected = "Delay out of range")]
fn test_queue_delay_too_short() {
    let (env, _admin, client) = setup();
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&op_id(&env, b"op2"), &(min_delay - 1));
}

#[test]
#[should_panic(expected = "Delay out of range")]
fn test_queue_delay_too_long() {
    let (env, _admin, client) = setup();
    let (_, max_delay) = client.get_delay_bounds();
    client.queue(&op_id(&env, b"op3"), &(max_delay + 1));
}

#[test]
#[should_panic(expected = "Operation already queued")]
fn test_queue_duplicate() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"op4");
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);
    client.queue(&id, &min_delay); // second call should panic
}

// ── execute ──────────────────────────────────────────────────────────────────

#[test]
fn test_execute_after_delay() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"exec1");
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);

    // advance ledger time past the delay
    env.ledger().with_mut(|l| l.timestamp += min_delay + 1);

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
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);
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
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);
    env.ledger().with_mut(|l| l.timestamp += min_delay + 1);
    client.execute(&id);
    client.execute(&id); // replay — must panic
}

// ── cancel ───────────────────────────────────────────────────────────────────

#[test]
fn test_cancel_success() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"cancel1");
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);
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
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&op_id(&env, b"unauth"), &min_delay);
}

// ── state helpers ─────────────────────────────────────────────────────────────

#[test]
fn test_get_execute_at() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"ts1");
    let before = env.ledger().timestamp();
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);
    let execute_at = client.get_execute_at(&id);
    assert_eq!(execute_at, before + min_delay);
}

#[test]
fn test_get_state_unknown() {
    let (env, _admin, client) = setup();
    assert_eq!(
        client.get_state(&op_id(&env, b"nope")),
        OperationState::Unknown
    );
}

// ── admin controls ───────────────────────────────────────────────────────────

#[test]
fn test_update_delay_bounds() {
    let (_env, _admin, client) = setup();
    client.update_delay_bounds(&120, &172800); // 2 min to 2 days
    let (min_delay, max_delay) = client.get_delay_bounds();
    assert_eq!(min_delay, 120);
    assert_eq!(max_delay, 172800);
}

#[test]
#[should_panic(expected = "Invalid delay bounds")]
fn test_update_delay_bounds_invalid() {
    let (_env, _admin, client) = setup();
    client.update_delay_bounds(&10, &86400); // below absolute minimum
}

#[test]
fn test_emergency_pause() {
    let (_env, _admin, client) = setup();
    assert!(!client.is_paused());

    client.set_pause(&true);
    assert!(client.is_paused());

    client.set_pause(&false);
    assert!(!client.is_paused());
}

#[test]
#[should_panic(expected = "Contract is paused")]
fn test_queue_when_paused() {
    let (env, _admin, client) = setup();
    client.set_pause(&true);
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&op_id(&env, b"paused"), &min_delay);
}

#[test]
#[should_panic(expected = "Contract is paused")]
fn test_execute_when_paused() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"pause_exec");
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);
    env.ledger().with_mut(|l| l.timestamp += min_delay + 1);

    client.set_pause(&true);
    client.execute(&id);
}

// ── Security tests: timelock bypass attempts ──

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_execute_unauthorized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimelockContract);
    let client = TimelockContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    let id = op_id(&env, b"unauth_exec");
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);
    env.ledger().with_mut(|l| l.timestamp += min_delay + 1);

    // Strip auths and try to execute
    env.set_auths(&[]);
    client.execute(&id);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_cancel_unauthorized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimelockContract);
    let client = TimelockContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    let id = op_id(&env, b"unauth_cancel");
    let (min_delay, _) = client.get_delay_bounds();
    client.queue(&id, &min_delay);

    // Strip auths and try to cancel
    env.set_auths(&[]);
    client.cancel(&id);
}

#[test]
fn test_timelock_skip_prevented() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"skip_test");
    let (min_delay, _) = client.get_delay_bounds();

    // Queue operation with minimum delay
    client.queue(&id, &min_delay);

    // Try to execute before delay - should fail
    let result = client.try_execute(&id);
    assert!(result.is_err());

    // Advance time
    env.ledger().with_mut(|l| l.timestamp += min_delay + 1);

    // Now execute should succeed
    assert!(client.execute(&id));
}

#[test]
fn test_paused_state_blocks_execute_after_threshold() {
    let (env, _admin, client) = setup();
    let id = op_id(&env, b"pause_block");
    let (min_delay, _) = client.get_delay_bounds();

    client.queue(&id, &min_delay);
    // Simulate approvals would have happened (in threshold contract)
    // For this simple timelock, we just test pause blocks execution
    client.set_pause(&true);

    env.ledger().with_mut(|l| l.timestamp += min_delay + 1);

    // Execute should fail because contract is paused
    let result = client.try_execute(&id);
    assert!(result.is_err());
    client.set_pause(&false); // unpause for cleanup
}

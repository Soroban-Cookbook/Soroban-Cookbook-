#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, Env};

/// Standard topology: Alice ↔ Ingrid ↔ Bob.
struct Topology {
    #[allow(dead_code)]
    env: Env,
    alice: Address,
    bob: Address,
    ingrid: Address,
    client: VirtualChannelContractClient<'static>,
    ledger_a: u64,
    ledger_b: u64,
}

const DEPOSIT: i128 = 100;
/// Challenge window used by the tests, in ledgers.
const WINDOW: u32 = 100;

fn new_client(env: &Env) -> VirtualChannelContractClient<'static> {
    let contract_id = env.register_contract(None, VirtualChannelContract);
    let client = VirtualChannelContractClient::new(env, &contract_id);
    client.initialize(&Address::generate(env), &WINDOW);
    client
}

fn setup() -> Topology {
    let env = Env::default();
    env.mock_all_auths();
    let client = new_client(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let ingrid = Address::generate(&env);

    let ledger_a = client.open_ledger(&alice, &ingrid, &DEPOSIT, &DEPOSIT);
    let ledger_b = client.open_ledger(&bob, &ingrid, &DEPOSIT, &DEPOSIT);

    Topology {
        env,
        alice,
        bob,
        ingrid,
        client,
        ledger_a,
        ledger_b,
    }
}

// ── creation ────────────────────────────────────────────────────────────────

#[test]
fn test_open_ledger_and_virtual_channel() {
    let t = setup();
    let vc_id = t
        .client
        .open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &50);
    let vc = t.client.get_virtual(&vc_id);
    assert_eq!(vc.amount, 50);
    assert_eq!(vc.bal_a, 50);
    assert_eq!(vc.bal_b, 0);
    assert_eq!(vc.seq, 0);
    assert_eq!(vc.status, ChannelStatus::Open);
    assert_eq!(vc.challenge_window, WINDOW);
    assert_eq!(t.client.get_ledger(&t.ledger_a).active_virtual, 1);
}

#[test]
#[should_panic(expected = "Insufficient collateral")]
fn test_open_virtual_insufficient_collateral() {
    let t = setup();
    t.client
        .open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &201);
}

#[test]
#[should_panic(expected = "Ledger channels do not match virtual topology")]
fn test_open_virtual_wrong_topology() {
    let env = Env::default();
    env.mock_all_auths();
    let client = new_client(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let ingrid = Address::generate(&env);
    let mallory = Address::generate(&env);

    // ledger_a is Alice↔Ingrid but ledger_b is Bob↔Mallory: topology mismatch.
    let la = client.open_ledger(&alice, &ingrid, &100, &100);
    let lb = client.open_ledger(&bob, &mallory, &100, &100);
    client.open_virtual(&alice, &bob, &ingrid, &la, &lb, &50);
}

// ── updates (off-chain) + settlement (materialize) ──────────────────────────

#[test]
fn test_routing_update_and_settlement() {
    let t = setup();
    let vc_id = t
        .client
        .open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &100);

    // Off-chain: Alice pays Bob 30 (bal 70/30), then Bob pays back 10 (80/20).
    // Each update is signed by both endpoints; only the final state is
    // materialized on-chain — that is the whole point of the virtual channel.
    t.client.materialize(&vc_id, &2, &80, &20);

    let vc = t.client.get_virtual(&vc_id);
    assert_eq!(vc.seq, 2);
    assert_eq!(vc.bal_a, 80);
    assert_eq!(vc.bal_b, 20);
    assert_eq!(vc.status, ChannelStatus::Closed);

    // Settlement: backing ledger channels rebalanced to the virtual balances.
    let la = t.client.get_ledger(&t.ledger_a);
    assert_eq!(la.endpoint_deposit, 80);
    assert_eq!(la.intermediary_deposit, 20);
    assert_eq!(la.settled_seq, 2);

    let lb = t.client.get_ledger(&t.ledger_b);
    assert_eq!(lb.endpoint_deposit, 20);
    assert_eq!(lb.intermediary_deposit, 80);
    assert_eq!(lb.settled_seq, 2);

    // Collateral conservation across the whole topology: after settlement,
    // each backing ledger channel holds exactly `amount` of collateral
    // (endpoint balance + intermediary top-up), so the topology holds 2x.
    assert_eq!(
        la.endpoint_deposit
            + la.intermediary_deposit
            + lb.endpoint_deposit
            + lb.intermediary_deposit,
        2 * 100
    );
}

#[test]
fn test_materialize_rejects_stale_sequence() {
    let t = setup();
    let vc_id = t
        .client
        .open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &100);
    t.client.materialize(&vc_id, &3, &50, &50);
    // stale state (seq 2 < 3) must be rejected — replay protection.
    // try_materialize surfaces the contract panic as an Err instead of aborting.
    let res = t.client.try_materialize(&vc_id, &2, &50, &50);
    assert!(res.is_err(), "stale state (seq 2 < 3) must be rejected");
}

#[test]
#[should_panic(expected = "Balances must be non-negative and conserve amount")]
fn test_materialize_rejects_amount_mismatch() {
    let t = setup();
    let vc_id = t
        .client
        .open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &100);
    t.client.materialize(&vc_id, &1, &60, &30); // 60+30 != 100
}

#[test]
#[should_panic(expected = "Already materialized")]
fn test_materialize_twice_fails() {
    let t = setup();
    let vc_id = t
        .client
        .open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &100);
    t.client.materialize(&vc_id, &1, &50, &50);
    t.client.materialize(&vc_id, &2, &60, &40);
}

// ── cooperative close ───────────────────────────────────────────────────────

#[test]
fn test_close_ledger_cooperative() {
    let t = setup();
    t.client.close_ledger(&t.ledger_a);
    let la = t.client.get_ledger(&t.ledger_a);
    assert!(!la.open);
}

#[test]
#[should_panic(expected = "Already closed")]
fn test_close_ledger_twice_fails() {
    let t = setup();
    t.client.close_ledger(&t.ledger_a);
    t.client.close_ledger(&t.ledger_a);
}

#[test]
#[should_panic(expected = "Ledger channels must be open")]
fn test_open_virtual_on_closed_ledger_fails() {
    let t = setup();
    t.client.close_ledger(&t.ledger_b);
    t.client
        .open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &50);
}

// ── configuration ───────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "Already initialized")]
fn test_initialize_twice_fails() {
    let t = setup();
    t.client.initialize(&t.alice, &10);
}

#[test]
#[should_panic(expected = "Invalid challenge window")]
fn test_initialize_zero_window_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VirtualChannelContract);
    let client = VirtualChannelContractClient::new(&env, &contract_id);
    client.initialize(&Address::generate(&env), &0);
}

#[test]
#[should_panic(expected = "Not initialized")]
fn test_open_virtual_requires_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VirtualChannelContract);
    let client = VirtualChannelContractClient::new(&env, &contract_id);
    let (a, b, i) = (
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    );
    let la = client.open_ledger(&a, &i, &100, &100);
    let lb = client.open_ledger(&b, &i, &100, &100);
    client.open_virtual(&a, &b, &i, &la, &lb, &50);
}

#[test]
#[should_panic(expected = "Ledger channels must differ")]
fn test_open_virtual_same_ledger_fails() {
    let t = setup();
    t.client
        .open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_a, &50);
}

// ── two-phase close ─────────────────────────────────────────────────────────

fn open_vc(t: &Topology) -> u64 {
    t.client
        .open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &100)
}

#[test]
fn test_honest_close_after_timeout() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.env.ledger().set_sequence_number(1_000);

    let deadline = t.client.initiate_close(&vc_id, &4, &30, &70);
    assert_eq!(deadline, 1_000 + WINDOW);

    let vc = t.client.get_virtual(&vc_id);
    assert_eq!(vc.status, ChannelStatus::PendingClose);
    assert_eq!(vc.close_deadline, deadline);

    // Nothing is settled while the window is open.
    assert_eq!(t.client.get_ledger(&t.ledger_a).settled_seq, 0);

    t.env.ledger().set_sequence_number(deadline + 1);
    t.client.finalize_close(&vc_id);

    let vc = t.client.get_virtual(&vc_id);
    assert_eq!(vc.status, ChannelStatus::Closed);
    assert_eq!((vc.seq, vc.bal_a, vc.bal_b), (4, 30, 70));
    let la = t.client.get_ledger(&t.ledger_a);
    let lb = t.client.get_ledger(&t.ledger_b);
    assert_eq!((la.endpoint_deposit, la.settled_seq), (30, 4));
    assert_eq!((lb.endpoint_deposit, lb.settled_seq), (70, 4));
    assert_eq!(la.active_virtual, 0);
}

#[test]
fn test_initiate_close_requires_both_signatures() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.client.initiate_close(&vc_id, &2, &50, &50);
    let auths = t.env.auths();
    assert!(auths.iter().any(|(a, _)| *a == t.alice));
    assert!(auths.iter().any(|(a, _)| *a == t.bob));
}

#[test]
fn test_stale_state_overridden_by_challenge() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.env.ledger().set_sequence_number(10);

    // Alice submits an old state (seq 1) that favours her while Bob is away.
    let deadline = t.client.initiate_close(&vc_id, &1, &90, &10);

    // Bob returns before the deadline with the latest state (seq 5).
    t.env.ledger().set_sequence_number(deadline);
    t.client.challenge_close(&vc_id, &5, &40, &60);

    let vc = t.client.get_virtual(&vc_id);
    assert_eq!((vc.seq, vc.bal_a, vc.bal_b), (5, 40, 60));
    // The challenge does not extend the deadline.
    assert_eq!(vc.close_deadline, deadline);

    t.env.ledger().set_sequence_number(deadline + 1);
    t.client.finalize_close(&vc_id);
    let la = t.client.get_ledger(&t.ledger_a);
    let lb = t.client.get_ledger(&t.ledger_b);
    assert_eq!(la.endpoint_deposit, 40);
    assert_eq!(lb.endpoint_deposit, 60);
    assert_eq!(la.settled_seq, 5);
}

#[test]
fn test_multiple_challenges_keep_highest_seq() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.client.initiate_close(&vc_id, &1, &90, &10);
    t.client.challenge_close(&vc_id, &3, &70, &30);
    t.client.challenge_close(&vc_id, &7, &20, &80);
    let res = t.client.try_challenge_close(&vc_id, &6, &50, &50);
    assert!(res.is_err(), "lower seq must not replace higher seq");
    assert_eq!(t.client.get_virtual(&vc_id).seq, 7);
}

#[test]
#[should_panic(expected = "Challenge window still open")]
fn test_finalize_before_deadline_fails() {
    let t = setup();
    let vc_id = open_vc(&t);
    let deadline = t.client.initiate_close(&vc_id, &1, &50, &50);
    t.env.ledger().set_sequence_number(deadline);
    t.client.finalize_close(&vc_id);
}

#[test]
#[should_panic(expected = "Challenge window elapsed")]
fn test_challenge_after_deadline_fails() {
    let t = setup();
    let vc_id = open_vc(&t);
    let deadline = t.client.initiate_close(&vc_id, &1, &50, &50);
    t.env.ledger().set_sequence_number(deadline + 1);
    t.client.challenge_close(&vc_id, &2, &40, &60);
}

#[test]
#[should_panic(expected = "Sequence must increase")]
fn test_challenge_with_same_seq_fails() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.client.initiate_close(&vc_id, &3, &50, &50);
    t.client.challenge_close(&vc_id, &3, &40, &60);
}

#[test]
#[should_panic(expected = "No pending close")]
fn test_challenge_without_pending_close_fails() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.client.challenge_close(&vc_id, &1, &50, &50);
}

#[test]
#[should_panic(expected = "No pending close")]
fn test_finalize_without_pending_close_fails() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.client.finalize_close(&vc_id);
}

#[test]
#[should_panic(expected = "Channel not open")]
fn test_initiate_close_twice_fails() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.client.initiate_close(&vc_id, &1, &50, &50);
    t.client.initiate_close(&vc_id, &2, &40, &60);
}

#[test]
fn test_initiate_close_with_opening_state() {
    let t = setup();
    let vc_id = open_vc(&t);
    let deadline = t.client.initiate_close(&vc_id, &0, &100, &0);
    t.env.ledger().set_sequence_number(deadline + 1);
    t.client.finalize_close(&vc_id);
    let vc = t.client.get_virtual(&vc_id);
    assert_eq!(
        (vc.status, vc.bal_a, vc.bal_b),
        (ChannelStatus::Closed, 100, 0)
    );
}

#[test]
#[should_panic(expected = "Initial state must match opening balances")]
fn test_initiate_close_seq_zero_wrong_balances_fails() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.client.initiate_close(&vc_id, &0, &0, &100);
}

#[test]
#[should_panic(expected = "Balances must be non-negative and conserve amount")]
fn test_initiate_close_bad_balances_fails() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.client.initiate_close(&vc_id, &1, &-1, &101);
}

#[test]
fn test_cooperative_final_state_overrides_pending_close() {
    let t = setup();
    let vc_id = open_vc(&t);
    t.client.initiate_close(&vc_id, &1, &90, &10);
    // Both endpoints agree on a final state: settle now, no waiting.
    t.client.materialize(&vc_id, &2, &60, &40);
    let vc = t.client.get_virtual(&vc_id);
    assert_eq!(vc.status, ChannelStatus::Closed);
    assert_eq!(vc.close_deadline, 0);
    assert!(t.client.try_finalize_close(&vc_id).is_err());
}

#[test]
#[should_panic(expected = "Virtual channel still active")]
fn test_close_ledger_blocked_while_virtual_active() {
    let t = setup();
    open_vc(&t);
    t.client.close_ledger(&t.ledger_a);
}

#[test]
fn test_close_ledger_after_finalize() {
    let t = setup();
    let vc_id = open_vc(&t);
    let deadline = t.client.initiate_close(&vc_id, &1, &50, &50);
    t.env.ledger().set_sequence_number(deadline + 1);
    t.client.finalize_close(&vc_id);
    t.client.close_ledger(&t.ledger_a);
    t.client.close_ledger(&t.ledger_b);
    assert!(!t.client.get_ledger(&t.ledger_a).open);
}

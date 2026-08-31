#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
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

fn setup() -> Topology {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VirtualChannelContract);
    let client = VirtualChannelContractClient::new(&env, &contract_id);
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
    let vc_id = t.client.open_virtual(
        &t.alice,
        &t.bob,
        &t.ingrid,
        &t.ledger_a,
        &t.ledger_b,
        &50,
    );
    let vc = t.client.get_virtual(&vc_id);
    assert_eq!(vc.amount, 50);
    assert_eq!(vc.bal_a, 50);
    assert_eq!(vc.bal_b, 0);
    assert_eq!(vc.seq, 0);
    assert!(!vc.materialized);
}

#[test]
#[should_panic(expected = "Insufficient collateral")]
fn test_open_virtual_insufficient_collateral() {
    let t = setup();
    t.client.open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &201);
}

#[test]
#[should_panic(expected = "Ledger channels do not match virtual topology")]
fn test_open_virtual_wrong_topology() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VirtualChannelContract);
    let client = VirtualChannelContractClient::new(&env, &contract_id);
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
    let vc_id = t.client.open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &100);

    // Off-chain: Alice pays Bob 30 (bal 70/30), then Bob pays back 10 (80/20).
    // Each update is signed by both endpoints; only the final state is
    // materialized on-chain — that is the whole point of the virtual channel.
    t.client.materialize(&vc_id, &2, &80, &20);

    let vc = t.client.get_virtual(&vc_id);
    assert_eq!(vc.seq, 2);
    assert_eq!(vc.bal_a, 80);
    assert_eq!(vc.bal_b, 20);
    assert!(vc.materialized);

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
        la.endpoint_deposit + la.intermediary_deposit + lb.endpoint_deposit + lb.intermediary_deposit,
        2 * 100
    );
}

#[test]
fn test_materialize_rejects_stale_sequence() {
    let t = setup();
    let vc_id = t.client.open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &100);
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
    let vc_id = t.client.open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &100);
    t.client.materialize(&vc_id, &1, &60, &30); // 60+30 != 100
}

#[test]
#[should_panic(expected = "Already materialized")]
fn test_materialize_twice_fails() {
    let t = setup();
    let vc_id = t.client.open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &100);
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
    t.client.open_virtual(&t.alice, &t.bob, &t.ingrid, &t.ledger_a, &t.ledger_b, &50);
}

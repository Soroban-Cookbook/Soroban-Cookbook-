//! Tests for event filtering patterns.
//!
//! Each test validates:
//! - Correct number of events emitted
//! - Topic count and topic values at each slot
//! - Data payload deserialization

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env, Symbol, TryFromVal,
};

fn setup() -> (Env, Address, EventFilteringContractClient<'static>) {
    let env = Env::default();
    let id = env.register_contract(None, EventFilteringContract);
    let client = EventFilteringContractClient::new(&env, &id);
    (env, id, client)
}

// ---------------------------------------------------------------------------
// transfer_simple — 2 topics
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_simple_topics() {
    let (env, _, client) = setup();
    client.transfer_simple(&500);

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let (_, topics, _) = events.get(0).unwrap();
    assert_eq!(topics.len(), 2);

    let t0 = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1 = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, NS);
    assert_eq!(t1, ACT_TRANSFER);
}

#[test]
fn test_transfer_simple_data() {
    let (env, _, client) = setup();
    client.transfer_simple(&999);

    let (_, _, data) = env.events().all().get(0).unwrap();
    let payload = TransferData::try_from_val(&env, &data).unwrap();
    assert_eq!(payload.amount, 999);
}

// ---------------------------------------------------------------------------
// transfer_from — 3 topics
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_from_topics() {
    let (env, _, client) = setup();
    let alice = Address::generate(&env);
    client.transfer_from(&alice, &100);

    let (_, topics, _) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 3);

    let t2 = Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    assert_eq!(t2, alice);
}

// ---------------------------------------------------------------------------
// transfer_full — 4 topics
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_full_four_topics() {
    let (env, _, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.transfer_full(&alice, &bob, &250);

    let (_, topics, _) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 4);

    let t2 = Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    let t3 = Address::try_from_val(&env, &topics.get(3).unwrap()).unwrap();
    assert_eq!(t2, alice);
    assert_eq!(t3, bob);
}

#[test]
fn test_transfer_full_data() {
    let (env, _, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.transfer_full(&alice, &bob, &777);

    let (_, _, data) = env.events().all().get(0).unwrap();
    let payload = TransferData::try_from_val(&env, &data).unwrap();
    assert_eq!(payload.amount, 777);
}

// ---------------------------------------------------------------------------
// record_sale — 4 topics, SaleData payload
// ---------------------------------------------------------------------------

#[test]
fn test_record_sale_topics_and_data() {
    let (env, _, client) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    client.record_sale(&seller, &buyer, &1000, &42);

    let (_, topics, data) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 4);

    let t1 = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t1, ACT_SALE);

    let t2 = Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    let t3 = Address::try_from_val(&env, &topics.get(3).unwrap()).unwrap();
    assert_eq!(t2, seller);
    assert_eq!(t3, buyer);

    let payload = SaleData::try_from_val(&env, &data).unwrap();
    assert_eq!(payload.price, 1000);
    assert_eq!(payload.token_id, 42);
}

// ---------------------------------------------------------------------------
// update_status — 3 topics, StatusData payload
// ---------------------------------------------------------------------------

#[test]
fn test_update_status_topics_and_data() {
    let (env, _, client) = setup();
    let entity = Address::generate(&env);
    let old = symbol_short!("pending");
    let new_s = symbol_short!("active");
    client.update_status(&entity, &old, &new_s);

    let (_, topics, data) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 3);

    let t1 = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t1, ACT_STATUS);

    let t2 = Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    assert_eq!(t2, entity);

    let payload = StatusData::try_from_val(&env, &data).unwrap();
    assert_eq!(payload.old_status, old);
    assert_eq!(payload.new_status, new_s);
}

// ---------------------------------------------------------------------------
// Multiple events — simulates off-chain filtering by topic slot
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_events_share_namespace() {
    let (env, _, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.transfer_simple(&1);
    client.transfer_from(&alice, &2);
    client.transfer_full(&alice, &bob, &3);
    client.record_sale(&alice, &bob, &100, &1);
    client.update_status(&alice, &symbol_short!("off"), &symbol_short!("on"));

    let events = env.events().all();
    assert_eq!(events.len(), 5);

    // All events share the same namespace in topic_0 — simulates a
    // "filter by contract namespace" query.
    for i in 0..5u32 {
        let (_, topics, _) = events.get(i).unwrap();
        let t0 = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
        assert_eq!(t0, NS, "event {i} must have namespace topic");
    }
}

#[test]
fn test_filter_by_action_topic() {
    let (env, _, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.transfer_simple(&1);
    client.record_sale(&alice, &bob, &50, &7);
    client.transfer_full(&alice, &bob, &2);

    // Simulate: filter topic_1 == "transfer" → should match events 0 and 2
    let mut transfer_count = 0u32;
    for (_, topics, _) in env.events().all().iter() {
        if let Some(raw) = topics.get(1) {
            if let Ok(s) = Symbol::try_from_val(&env, &raw) {
                if s == ACT_TRANSFER {
                    transfer_count += 1;
                }
            }
        }
    }
    assert_eq!(transfer_count, 2);
}

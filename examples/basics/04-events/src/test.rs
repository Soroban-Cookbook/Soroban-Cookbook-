//! Unit tests for Events contract - event emission, data validation, topic verification.

#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Events as _, Env, Symbol, TryFromVal};

#[test]
fn test_event_emission_exists() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    client.emit_simple(&100);

    let events = env.events().all();
    assert!(!events.is_empty(), "At least one event must be emitted");
}

#[test]
fn test_event_count_single() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    client.emit_simple(&42);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "emit_simple must emit exactly one event");
}

#[test]
fn test_event_count_multiple() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    client.emit_multiple(&3);

    let events = env.events().all();
    assert_eq!(
        events.len(),
        3,
        "emit_multiple(3) must emit exactly 3 events"
    );
}

#[test]
fn test_emit_multiple_topics_match_indices() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    client.emit_multiple(&3);

    let events = env.events().all();
    assert_eq!(events.len(), 3);

    for i in 0..3 {
        let event = events.get(i).unwrap();
        let (_contract_id, topics, _data) = event;
        assert_eq!(topics.len(), 2, "multi event must have 2 topics");

        let name: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
        let index: u32 = u32::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
        assert_eq!(name, symbol_short!("multi"));
        assert_eq!(index, i);
    }
}

#[test]
fn test_topic_structure_simple() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    client.emit_simple(&99);

    let events = env.events().all();
    let event = events.get(0).unwrap();
    let (_contract_id, topics, _data) = event;
    assert_eq!(topics.len(), 1, "Simple event must have 1 topic");
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("simple"));
}

#[test]
fn test_topic_structure_tagged() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    let tag = symbol_short!("mytag");
    client.emit_tagged(&tag, &50);

    let events = env.events().all();
    let event = events.get(0).unwrap();
    let (_contract_id, topics, _data) = event;
    assert_eq!(topics.len(), 2, "Tagged event must have 2 topics");
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("tagged"));
    assert_eq!(t1, tag);
}

#[test]
fn test_payload_values() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    let value = 12345u64;
    client.emit_simple(&value);

    let events = env.events().all();
    let event = events.get(0).unwrap();
    let (_contract_id, _topics, data) = event;
    let payload: u64 = u64::try_from_val(&env, &data).unwrap();
    assert_eq!(payload, value, "Event data must match emitted value");
}

#[test]
fn test_action_differentiation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    client.emit_simple(&1);
    client.emit_tagged(&symbol_short!("x"), &2);

    let events = env.events().all();
    assert_eq!(events.len(), 2);

    let (_id0, topics0, _) = events.get(0).unwrap();
    let (_id1, topics1, _) = events.get(1).unwrap();

    let t0: Symbol = Symbol::try_from_val(&env, &topics0.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics1.get(0).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("simple"));
    assert_eq!(t1, symbol_short!("tagged"));
}

#[test]
fn test_no_extra_events() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    client.emit_simple(&10);
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Must not emit extra events");
}

#[test]
fn test_zero_events_on_empty_emit() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    client.emit_multiple(&0);
    let events = env.events().all();
    assert_eq!(events.len(), 0, "emit_multiple(0) must emit zero events");
}

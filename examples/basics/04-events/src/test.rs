//! Unit tests for the structured event patterns contract.
//!
//! Tests verify:
//! - Correct number of events emitted
//! - Correct topic count and topic values (including indexed parameters)
//! - Correct data payload deserialization for custom types

#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env, Symbol, TryFromVal,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_env_and_client() -> (Env, Address, EventsContractClient<'static>) {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);
    (env, contract_id, client)
}

// ---------------------------------------------------------------------------
// Structured event 1: transfer (4 topics)
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_emits_one_event() {
    let (env, _, client) = make_env_and_client();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.transfer(&sender, &recipient, &1000, &0);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "transfer must emit exactly one event");
}

#[test]
fn test_transfer_event_has_four_topics() {
    let (env, _, client) = make_env_and_client();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.transfer(&sender, &recipient, &500, &42);

    let events = env.events().all();
    let (_id, topics, _data) = events.get(0).unwrap();

    assert_eq!(topics.len(), 4, "transfer event must have 4 topics");
}

#[test]
fn test_transfer_topic_namespace_and_action() {
    let (env, _, client) = make_env_and_client();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.transfer(&sender, &recipient, &1, &0);

    let (_id, topics, _data) = env.events().all().get(0).unwrap();

    // Topic 0: contract namespace
    let ns: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(ns, symbol_short!("events"));

    // Topic 1: action name
    let action: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(action, symbol_short!("transfer"));
}

#[test]
fn test_transfer_indexed_addresses_in_topics() {
    let (env, _, client) = make_env_and_client();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.transfer(&sender, &recipient, &999, &0);

    let (_id, topics, _data) = env.events().all().get(0).unwrap();

    // Topic 2: sender (indexed for off-chain search)
    let t_sender = Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    assert_eq!(t_sender, sender);

    // Topic 3: recipient (indexed for off-chain search)
    let t_recipient = Address::try_from_val(&env, &topics.get(3).unwrap()).unwrap();
    assert_eq!(t_recipient, recipient);
}

#[test]
fn test_transfer_structured_data_payload() {
    let (env, _, client) = make_env_and_client();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let amount: i128 = 12_345;
    let memo: u64 = 99;

    client.transfer(&sender, &recipient, &amount, &memo);

    let (_id, _topics, data) = env.events().all().get(0).unwrap();
    let payload = TransferEventData::try_from_val(&env, &data).unwrap();

    assert_eq!(payload.amount, amount);
    assert_eq!(payload.memo, memo);
}

// ---------------------------------------------------------------------------
// Structured event 2: config update (3 topics)
// ---------------------------------------------------------------------------

#[test]
fn test_config_update_emits_one_event() {
    let (env, _, client) = make_env_and_client();

    client.update_config(&symbol_short!("max_sup"), &100, &200);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "update_config must emit exactly one event");
}

#[test]
fn test_config_update_event_has_three_topics() {
    let (env, _, client) = make_env_and_client();

    client.update_config(&symbol_short!("fee"), &5, &10);

    let (_id, topics, _data) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 3, "cfg_update event must have 3 topics");
}

#[test]
fn test_config_update_topic_namespace_and_action() {
    let (env, _, client) = make_env_and_client();

    client.update_config(&symbol_short!("fee"), &5, &10);

    let (_id, topics, _data) = env.events().all().get(0).unwrap();

    let ns: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(ns, symbol_short!("events"));

    let action: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(action, symbol_short!("cfg_upd"));
}

#[test]
fn test_config_update_indexed_key_in_topic() {
    let (env, _, client) = make_env_and_client();

    let key = symbol_short!("max_sup");
    client.update_config(&key, &50, &100);

    let (_id, topics, _data) = env.events().all().get(0).unwrap();

    // Topic 2: the config key — indexed so consumers can filter by key name
    let t_key: Symbol = Symbol::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    assert_eq!(t_key, key);
}

#[test]
fn test_config_update_structured_data_payload() {
    let (env, _, client) = make_env_and_client();

    client.update_config(&symbol_short!("rate"), &10, &20);

    let (_id, _topics, data) = env.events().all().get(0).unwrap();
    let payload = ConfigUpdateEventData::try_from_val(&env, &data).unwrap();

    assert_eq!(payload.old_value, 10);
    assert_eq!(payload.new_value, 20);
}

// ---------------------------------------------------------------------------
// Backward-compatible simple/tagged/multiple helpers
// ---------------------------------------------------------------------------

#[test]
fn test_event_emission_exists() {
    let (env, _, client) = make_env_and_client();
    client.emit_simple(&100);
    assert!(!env.events().all().is_empty());
}

#[test]
fn test_event_count_single() {
    let (env, _, client) = make_env_and_client();
    client.emit_simple(&42);
    assert_eq!(env.events().all().len(), 1);
}

#[test]
fn test_event_count_multiple() {
    let (env, _, client) = make_env_and_client();
    client.emit_multiple(&3);
    assert_eq!(env.events().all().len(), 3);
}

#[test]
fn test_topic_structure_simple() {
    let (env, _, client) = make_env_and_client();
    client.emit_simple(&99);
    let (_id, topics, _data) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 1);
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("simple"));
}

#[test]
fn test_topic_structure_tagged() {
    let (env, _, client) = make_env_and_client();
    let tag = symbol_short!("mytag");
    client.emit_tagged(&tag, &50);
    let (_id, topics, _data) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 2);
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("tagged"));
    assert_eq!(t1, tag);
}

#[test]
fn test_payload_values() {
    let (env, _, client) = make_env_and_client();
    let value = 12345u64;
    client.emit_simple(&value);
    let (_id, _topics, data) = env.events().all().get(0).unwrap();
    let payload: u64 = u64::try_from_val(&env, &data).unwrap();
    assert_eq!(payload, value);
}

#[test]
fn test_zero_events_on_empty_emit() {
    let (env, _, client) = make_env_and_client();
    client.emit_multiple(&0);
    assert_eq!(env.events().all().len(), 0);
}

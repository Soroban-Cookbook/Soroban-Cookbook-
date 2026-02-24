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

// ==================== QUERY-FRIENDLY PATTERN TESTS ====================

#[test]
fn test_emit_transfer_topic_layout() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.emit_transfer(&from, &to, &500);

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let (_id, topics, data) = events.get(0).unwrap();

    // topic[0] must always be the action name for event-type filtering
    let action: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(action, symbol_short!("transfer"));

    // topic[1] = from-address; topic[2] = to-address
    // These positions enable per-address history queries off-chain.
    let t_from: Address = Address::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    let t_to: Address = Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    assert_eq!(t_from, from);
    assert_eq!(t_to, to);

    // amount lives in data — readable after filtering, but not a filter key
    let amount: u64 = u64::try_from_val(&env, &data).unwrap();
    assert_eq!(amount, 500);
}

#[test]
fn test_emit_transfer_independent_senders_queryable() {
    // Verifies that multiple transfers can be distinguished by topic[1] (sender).
    // An off-chain indexer watching topic[1] == alice sees only one event.
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.emit_transfer(&alice, &carol, &100);
    client.emit_transfer(&bob, &carol, &200);

    let events = env.events().all();
    assert_eq!(events.len(), 2);

    // Both events share the same action topic, so a "get all transfers" query works.
    for i in 0..2u32 {
        let (_id, topics, _data) = events.get(i).unwrap();
        let action: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
        assert_eq!(action, symbol_short!("transfer"));
    }

    // Sender is distinguishable via topic[1].
    let (_id0, topics0, _) = events.get(0).unwrap();
    let (_id1, topics1, _) = events.get(1).unwrap();
    let sender0: Address = Address::try_from_val(&env, &topics0.get(1).unwrap()).unwrap();
    let sender1: Address = Address::try_from_val(&env, &topics1.get(1).unwrap()).unwrap();
    assert_ne!(
        sender0, sender1,
        "Senders must be distinguishable via topic[1]"
    );
}

#[test]
fn test_emit_namespaced_three_topic_hierarchy() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    let category = symbol_short!("defi");
    let action = symbol_short!("swap");
    let pool = symbol_short!("pool1");

    client.emit_namespaced(&category, &action, &pool, &1000);

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let (_id, topics, data) = events.get(0).unwrap();
    assert_eq!(topics.len(), 3, "Namespaced event must carry 3 topics");

    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    let t2: Symbol = Symbol::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    assert_eq!(t0, category);
    assert_eq!(t1, action);
    assert_eq!(t2, pool);

    let amount: u64 = u64::try_from_val(&env, &data).unwrap();
    assert_eq!(amount, 1000);
}

#[test]
fn test_emit_status_change_four_topics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, &contract_id);

    let entity = symbol_short!("order42");
    let old_s = symbol_short!("pending");
    let new_s = symbol_short!("filled");

    client.emit_status_change(&entity, &old_s, &new_s);

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let (_id, topics, data) = events.get(0).unwrap();
    assert_eq!(
        topics.len(),
        4,
        "Status-change event must use all 4 topic slots"
    );

    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    let t2: Symbol = Symbol::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    let t3: Symbol = Symbol::try_from_val(&env, &topics.get(3).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("status"));
    assert_eq!(t1, entity);
    assert_eq!(t2, old_s);
    assert_eq!(t3, new_s);

    // data holds the ledger sequence for off-chain ordering / deduplication
    let _ledger: u32 = u32::try_from_val(&env, &data).unwrap();
}

// ---------------------------------------------------------------------------
// Admin action event tests (3 topics)
// ---------------------------------------------------------------------------

#[test]
fn test_admin_action_emits_one_event() {
    let (env, _, client) = make_env_and_client();

    let admin = Address::generate(&env);
    client.admin_action(&admin, &symbol_short!("pause"));

    let events = env.events().all();
    assert_eq!(events.len(), 1, "admin_action must emit exactly one event");
}

#[test]
fn test_admin_action_event_has_three_topics() {
    let (env, _, client) = make_env_and_client();

    let admin = Address::generate(&env);
    client.admin_action(&admin, &symbol_short!("pause"));

    let (_id, topics, _data) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 3, "admin_action event must have 3 topics");
}

#[test]
fn test_admin_action_topic_namespace_and_category() {
    let (env, _, client) = make_env_and_client();

    let admin = Address::generate(&env);
    client.admin_action(&admin, &symbol_short!("upgrade"));

    let (_id, topics, _data) = env.events().all().get(0).unwrap();

    let ns: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(ns, symbol_short!("events"));

    let category: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(category, symbol_short!("admin"));
}

#[test]
fn test_admin_action_indexed_admin_address() {
    let (env, _, client) = make_env_and_client();

    let admin = Address::generate(&env);
    client.admin_action(&admin, &symbol_short!("pause"));

    let (_id, topics, _data) = env.events().all().get(0).unwrap();

    let t_admin = Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    assert_eq!(t_admin, admin);
}

#[test]
fn test_admin_action_structured_data_payload() {
    let (env, _, client) = make_env_and_client();

    let admin = Address::generate(&env);
    let action = symbol_short!("pause");
    client.admin_action(&admin, &action);

    let (_id, _topics, data) = env.events().all().get(0).unwrap();
    let payload = AdminActionEventData::try_from_val(&env, &data).unwrap();

    assert_eq!(payload.action, action);
}

// ---------------------------------------------------------------------------
// Audit trail event tests (4 topics)
// ---------------------------------------------------------------------------

#[test]
fn test_audit_trail_emits_one_event() {
    let (env, _, client) = make_env_and_client();

    let actor = Address::generate(&env);
    client.audit_trail(&actor, &symbol_short!("delete"), &symbol_short!("rec_42"));

    let events = env.events().all();
    assert_eq!(events.len(), 1, "audit_trail must emit exactly one event");
}

#[test]
fn test_audit_trail_event_has_four_topics() {
    let (env, _, client) = make_env_and_client();

    let actor = Address::generate(&env);
    client.audit_trail(&actor, &symbol_short!("create"), &symbol_short!("item_1"));

    let (_id, topics, _data) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 4, "audit_trail event must have 4 topics");
}

#[test]
fn test_audit_trail_topic_namespace_and_category() {
    let (env, _, client) = make_env_and_client();

    let actor = Address::generate(&env);
    client.audit_trail(&actor, &symbol_short!("update"), &symbol_short!("cfg_x"));

    let (_id, topics, _data) = env.events().all().get(0).unwrap();

    let ns: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(ns, symbol_short!("events"));

    let category: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(category, symbol_short!("audit"));
}

#[test]
fn test_audit_trail_indexed_actor_and_action() {
    let (env, _, client) = make_env_and_client();

    let actor = Address::generate(&env);
    let action = symbol_short!("delete");
    client.audit_trail(&actor, &action, &symbol_short!("rec_7"));

    let (_id, topics, _data) = env.events().all().get(0).unwrap();

    let t_actor = Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap();
    assert_eq!(t_actor, actor);

    let t_action: Symbol = Symbol::try_from_val(&env, &topics.get(3).unwrap()).unwrap();
    assert_eq!(t_action, action);
}

#[test]
fn test_audit_trail_structured_data_payload() {
    let (env, _, client) = make_env_and_client();

    let actor = Address::generate(&env);
    let action = symbol_short!("create");
    let details = symbol_short!("new_usr");
    client.audit_trail(&actor, &action, &details);

    let (_id, _topics, data) = env.events().all().get(0).unwrap();
    let payload = AuditTrailEventData::try_from_val(&env, &data).unwrap();

    assert_eq!(payload.details, details);
    assert_eq!(payload.timestamp, env.ledger().timestamp());
    assert_eq!(payload.sequence, env.ledger().sequence());
}

use super::*;
use soroban_sdk::{symbol_short, testutils::Events as _, Env, Symbol, TryFromVal};

fn setup() -> (Env, EventContractClient<'static>) {
    let env = Env::default();
    let id = env.register_contract(None, EventContract);
    let client = EventContractClient::new(&env, &id);
    (env, client)
}

// ---------------------------------------------------------------------------
// env.events().publish() usage
// ---------------------------------------------------------------------------

#[test]
fn test_set_emits_event() {
    let (env, client) = setup();
    client.set(&42);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "set() should emit exactly one event");
}

#[test]
fn test_increment_emits_event() {
    let (env, client) = setup();
    client.set(&10);
    client.increment();

    let events = env.events().all();
    assert_eq!(
        events.len(),
        2,
        "increment() should emit one additional event"
    );
}

// ---------------------------------------------------------------------------
// Simple event topics
// ---------------------------------------------------------------------------

#[test]
fn test_set_event_topic() {
    let (env, client) = setup();
    client.set(&5);

    let (_, topics, _) = env.events().all().get(0).unwrap();
    let topic: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(topic, symbol_short!("set"));
}

#[test]
fn test_increment_event_topics() {
    let (env, client) = setup();
    client.set(&0);
    client.increment();

    let (_, topics, _) = env.events().all().get(1).unwrap();
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("counter"));
    assert_eq!(t1, symbol_short!("inc"));
}

// ---------------------------------------------------------------------------
// Event data payload
// ---------------------------------------------------------------------------

#[test]
fn test_set_event_data_payload() {
    let (env, client) = setup();
    client.set(&99);

    let (_, _, data) = env.events().all().get(0).unwrap();
    let payload: u32 = u32::try_from_val(&env, &data).unwrap();
    assert_eq!(payload, 99);
}

#[test]
fn test_increment_event_data_payload() {
    let (env, client) = setup();
    client.set(&10);
    client.increment();

    let (_, _, data) = env.events().all().get(1).unwrap();
    let payload: u32 = u32::try_from_val(&env, &data).unwrap();
    assert_eq!(payload, 11, "payload should reflect incremented value");
}

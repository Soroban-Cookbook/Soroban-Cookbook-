#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::{contract, contractimpl, symbol_short, testutils::Address as _, Env};

// ---------------------------------------------------------------------------
// Mock subscribers used to observe pushed notifications in tests.
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub topic: Symbol,
    pub entity: Option<Address>,
    pub payload: i128,
}

const RECORDS: Symbol = symbol_short!("recs");

#[contract]
pub struct RecordingSubscriber;

#[contractimpl]
impl RecordingSubscriber {
    pub fn notify(env: Env, topic: Symbol, entity: Option<Address>, payload: i128) {
        let mut records: Vec<Record> = env
            .storage()
            .instance()
            .get(&RECORDS)
            .unwrap_or_else(|| Vec::new(&env));
        records.push_back(Record {
            topic,
            entity,
            payload,
        });
        env.storage().instance().set(&RECORDS, &records);
    }

    pub fn records(env: Env) -> Vec<Record> {
        env.storage()
            .instance()
            .get(&RECORDS)
            .unwrap_or_else(|| Vec::new(&env))
    }
}

/// A malfunctioning subscriber that always panics on notify, used to prove
/// that one bad subscriber cannot block delivery to the rest.
#[contract]
pub struct PanickingSubscriber;

#[contractimpl]
impl PanickingSubscriber {
    pub fn notify(_env: Env, _topic: Symbol, _entity: Option<Address>, _payload: i128) {
        panic!("subscriber malfunction");
    }
}

// ---------------------------------------------------------------------------
// Test fixture
// ---------------------------------------------------------------------------

struct Fixture {
    env: Env,
    hub: EventSubscriptionHubClient<'static>,
    alice: Address,
    bob: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let hub_id = env.register(EventSubscriptionHub, ());
    let hub = EventSubscriptionHubClient::new(&env, &hub_id);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    Fixture {
        env,
        hub,
        alice,
        bob,
    }
}

fn topic(env: &Env, name: &str) -> Symbol {
    Symbol::new(env, name)
}

#[test]
fn subscribe_returns_increasing_ids() {
    let f = setup();
    let orders = topic(&f.env, "orders");
    let recorder_a = f.env.register(RecordingSubscriber, ());
    let recorder_b = f.env.register(RecordingSubscriber, ());

    let id_a = f.hub.subscribe(&recorder_a, &orders, &None);
    let id_b = f.hub.subscribe(&recorder_b, &orders, &None);

    assert_eq!(id_a, 0);
    assert_eq!(id_b, 1);
    assert_eq!(f.hub.subscriber_count(&orders), 2);
}

#[test]
fn publish_pushes_to_all_matching_subscribers() {
    let f = setup();
    let orders = topic(&f.env, "orders");
    let recorder_a = f.env.register(RecordingSubscriber, ());
    let recorder_b = f.env.register(RecordingSubscriber, ());
    f.hub.subscribe(&recorder_a, &orders, &None);
    f.hub.subscribe(&recorder_b, &orders, &None);

    let notified = f.hub.publish(&orders, &None, &42);

    assert_eq!(notified, 2);
    let client_a = RecordingSubscriberClient::new(&f.env, &recorder_a);
    let client_b = RecordingSubscriberClient::new(&f.env, &recorder_b);
    assert_eq!(client_a.records().len(), 1);
    assert_eq!(client_b.records().len(), 1);
    assert_eq!(client_a.records().get(0).unwrap().payload, 42);
}

#[test]
fn filter_excludes_non_matching_entity() {
    let f = setup();
    let orders = topic(&f.env, "orders");
    let recorder = f.env.register(RecordingSubscriber, ());
    f.hub.subscribe(&recorder, &orders, &Some(f.alice.clone()));

    // Published for a different entity: filtered subscriber gets nothing.
    let notified = f.hub.publish(&orders, &Some(f.bob.clone()), &1);
    assert_eq!(notified, 0);
    let client = RecordingSubscriberClient::new(&f.env, &recorder);
    assert_eq!(client.records().len(), 0);

    // Published for the matching entity: subscriber is notified.
    let notified = f.hub.publish(&orders, &Some(f.alice.clone()), &2);
    assert_eq!(notified, 1);
    assert_eq!(client.records().len(), 1);
    assert_eq!(client.records().get(0).unwrap().payload, 2);
}

#[test]
fn wildcard_filter_matches_any_entity() {
    let f = setup();
    let orders = topic(&f.env, "orders");
    let recorder = f.env.register(RecordingSubscriber, ());
    f.hub.subscribe(&recorder, &orders, &None);

    assert_eq!(f.hub.publish(&orders, &Some(f.alice.clone()), &1), 1);
    assert_eq!(f.hub.publish(&orders, &Some(f.bob.clone()), &2), 1);
    assert_eq!(f.hub.publish(&orders, &None, &3), 1);
}

#[test]
fn publish_with_no_subscribers_returns_zero() {
    let f = setup();
    let orders = topic(&f.env, "orders");

    assert_eq!(f.hub.publish(&orders, &None, &1), 0);
}

#[test]
fn unsubscribe_stops_further_notifications() {
    let f = setup();
    let orders = topic(&f.env, "orders");
    let recorder = f.env.register(RecordingSubscriber, ());
    let id = f.hub.subscribe(&recorder, &orders, &None);

    f.hub.publish(&orders, &None, &1);
    f.hub.unsubscribe(&recorder, &id);
    f.hub.publish(&orders, &None, &2);

    let client = RecordingSubscriberClient::new(&f.env, &recorder);
    assert_eq!(client.records().len(), 1);
    assert_eq!(client.records().get(0).unwrap().payload, 1);
    assert_eq!(f.hub.subscriber_count(&orders), 0);
}

#[test]
fn unsubscribe_unknown_id_fails() {
    let f = setup();

    assert_eq!(
        f.hub.try_unsubscribe(&f.alice, &999),
        Err(Ok(SubscriptionError::SubscriptionNotFound))
    );
}

#[test]
fn unsubscribe_by_non_owner_fails() {
    let f = setup();
    let orders = topic(&f.env, "orders");
    let recorder = f.env.register(RecordingSubscriber, ());
    let id = f.hub.subscribe(&recorder, &orders, &None);

    // `alice` never subscribed; the subscription belongs to `recorder`.
    assert_eq!(
        f.hub.try_unsubscribe(&f.alice, &id),
        Err(Ok(SubscriptionError::NotSubscriptionOwner))
    );
}

#[test]
fn double_unsubscribe_fails() {
    let f = setup();
    let orders = topic(&f.env, "orders");
    let recorder = f.env.register(RecordingSubscriber, ());
    let id = f.hub.subscribe(&recorder, &orders, &None);

    f.hub.unsubscribe(&recorder, &id);

    assert_eq!(
        f.hub.try_unsubscribe(&recorder, &id),
        Err(Ok(SubscriptionError::AlreadyUnsubscribed))
    );
}

#[test]
fn failing_subscriber_does_not_block_others() {
    let f = setup();
    let orders = topic(&f.env, "orders");
    let bad = f.env.register(PanickingSubscriber, ());
    let good = f.env.register(RecordingSubscriber, ());
    f.hub.subscribe(&bad, &orders, &None);
    f.hub.subscribe(&good, &orders, &None);

    let notified = f.hub.publish(&orders, &None, &7);

    // Only the well-behaved subscriber counts toward `notified`.
    assert_eq!(notified, 1);
    let client = RecordingSubscriberClient::new(&f.env, &good);
    assert_eq!(client.records().len(), 1);
    assert_eq!(client.records().get(0).unwrap().payload, 7);
}

#[test]
fn subscription_query_reflects_state() {
    let f = setup();
    let orders = topic(&f.env, "orders");
    let recorder = f.env.register(RecordingSubscriber, ());
    let id = f.hub.subscribe(&recorder, &orders, &Some(f.alice.clone()));

    let sub = f.hub.subscription(&id).unwrap();
    assert_eq!(sub.subscriber, recorder);
    assert_eq!(sub.topic, orders);
    assert_eq!(sub.filter, Some(f.alice.clone()));
    assert!(sub.active);

    f.hub.unsubscribe(&recorder, &id);
    let sub = f.hub.subscription(&id).unwrap();
    assert!(!sub.active);

    assert_eq!(f.hub.subscription(&999), None);
}

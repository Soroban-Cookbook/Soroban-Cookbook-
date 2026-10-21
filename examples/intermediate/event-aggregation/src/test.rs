#![allow(deprecated)]
use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env, Symbol, TryFromVal,
};
use soroban_validation::test_events::EventList;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn setup() -> (Env, EventAggregatorClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(EventAggregator, ());
    let client = EventAggregatorClient::new(&env, &id);
    (env, client)
}

fn make_actor(env: &Env) -> Address {
    Address::generate(env)
}

// Decode the BatchEvent data payload from an EventList entry.
fn decode_batch(env: &Env, events: &EventList, index: u32) -> BatchEvent {
    let (_, _, data) = events.get(index).unwrap();
    BatchEvent::try_from_val(env, &data).unwrap()
}

// ── Basic accumulation ────────────────────────────────────────────────────────

/// Queuing N actions should NOT emit any events — emission is deferred to flush.
#[test]
fn test_queue_does_not_emit_events() {
    let (env, client) = setup();
    let actor = make_actor(&env);

    client.queue_action(&actor, &symbol_short!("transfer"), &100, &0);
    client.queue_action(&actor, &symbol_short!("mint"), &50, &1);

    // No events yet.
    assert_eq!(EventList::new(&env, env.events().all()).len(), 0);
    assert_eq!(client.pending_count(), 2);
}

/// A single flush emits exactly one BatchEvent containing all queued actions.
#[test]
fn test_flush_emits_single_batch_event() {
    let (env, client) = setup();
    let actor = make_actor(&env);

    let xfer = symbol_short!("transfer");
    let burn = symbol_short!("burn");

    client.queue_action(&actor, &xfer, &200, &42);
    client.queue_action(&actor, &burn, &75, &0);

    let batch_id = client.flush();
    assert_eq!(batch_id, 0);

    let events = EventList::new(&env, env.events().all());

    // Exactly one event emitted.
    assert_eq!(events.len(), 1);

    // Unpack and verify topics.
    let (_, topics, _) = events.get(0).unwrap();

    let ns: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let action: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    let emitted_id: u32 = u32::try_from_val(&env, &topics.get(2).unwrap()).unwrap();

    assert_eq!(ns, symbol_short!("evt_agg"));
    assert_eq!(action, symbol_short!("batch"));
    assert_eq!(emitted_id, 0u32);

    // Decode data payload.
    let batch = decode_batch(&env, &events, 0);
    assert_eq!(batch.batch_id, 0);
    assert_eq!(batch.action_count, 2);
    assert_eq!(batch.actions.len(), 2);

    // Verify first action.
    let first = batch.actions.get(0).unwrap();
    assert_eq!(first.action_type, xfer);
    assert_eq!(first.amount, 200);
    assert_eq!(first.memo, 42);

    // Verify second action.
    let second = batch.actions.get(1).unwrap();
    assert_eq!(second.action_type, burn);
    assert_eq!(second.amount, 75);
}

/// Intra-batch ordering must be preserved (insertion order).
#[test]
fn test_action_order_preserved() {
    let (env, client) = setup();
    let actor = make_actor(&env);

    for i in 0u64..5 {
        client.queue_action(&actor, &symbol_short!("op"), &(i as i128), &i);
    }

    client.flush();

    let events = EventList::new(&env, env.events().all());
    let batch = decode_batch(&env, &events, 0);

    for i in 0u32..5 {
        assert_eq!(batch.actions.get(i).unwrap().memo, i as u64);
    }
}

// ── Batch id monotonicity ────────────────────────────────────────────────────

/// Each flush increments the batch_id by 1.
#[test]
fn test_batch_id_increments_across_flushes() {
    let (env, client) = setup();
    let actor = make_actor(&env);

    for round in 0u32..3 {
        client.queue_action(&actor, &symbol_short!("op"), &1, &0);
        let id = client.flush();
        assert_eq!(id, round);
    }

    assert_eq!(client.next_batch_id(), 3);
}

// ── Queue cleared after flush ────────────────────────────────────────────────

/// After a flush the pending queue is empty.
#[test]
fn test_pending_queue_cleared_after_flush() {
    let (env, client) = setup();
    let actor = make_actor(&env);

    client.queue_action(&actor, &symbol_short!("op"), &1, &0);
    assert_eq!(client.pending_count(), 1);

    client.flush();

    assert_eq!(client.pending_count(), 0);
}

// ── Large batch ───────────────────────────────────────────────────────────────

/// Verify a batch of 20 actions is emitted correctly.
#[test]
fn test_large_batch() {
    let (env, client) = setup();
    let actor = make_actor(&env);

    for i in 0i128..20 {
        client.queue_action(&actor, &symbol_short!("op"), &i, &(i as u64));
    }

    client.flush();

    let events = EventList::new(&env, env.events().all());
    let batch = decode_batch(&env, &events, 0);

    assert_eq!(batch.action_count, 20);
    assert_eq!(batch.actions.len(), 20);
}

// ── Multiple actors ───────────────────────────────────────────────────────────

/// Actions from different actors should all be included in the same batch.
#[test]
fn test_multiple_actors_in_one_batch() {
    let (env, client) = setup();
    let alice = make_actor(&env);
    let bob = make_actor(&env);

    client.queue_action(&alice, &symbol_short!("send"), &100, &0);
    client.queue_action(&bob, &symbol_short!("recv"), &100, &0);

    client.flush();

    let events = EventList::new(&env, env.events().all());
    let batch = decode_batch(&env, &events, 0);

    assert_eq!(batch.action_count, 2);
    assert_eq!(batch.actions.get(0).unwrap().actor, alice);
    assert_eq!(batch.actions.get(1).unwrap().actor, bob);
}

// ── Empty-batch edge case ─────────────────────────────────────────────────────

/// Flushing with no queued actions returns AggError::EmptyBatch.
/// Design decision: we surface this as an explicit error rather than a no-op
/// so callers can detect programming mistakes (forgot to queue anything).
#[test]
fn test_flush_with_empty_queue_returns_error() {
    let (_, client) = setup();

    let result = client.try_flush();
    assert_eq!(result, Err(Ok(AggError::EmptyBatch)));
}

/// Confirm that an empty-batch flush does NOT emit any event.
#[test]
fn test_flush_empty_does_not_emit_event() {
    let (env, client) = setup();

    let _ = client.try_flush();
    assert_eq!(EventList::new(&env, env.events().all()).len(), 0);
}

/// Confirm batch_id does NOT advance on a failed empty flush.
#[test]
fn test_batch_id_does_not_advance_on_empty_flush() {
    let (env, client) = setup();
    let actor = make_actor(&env);

    // Failed flush — id stays at 0.
    let _ = client.try_flush();
    assert_eq!(client.next_batch_id(), 0);

    // Successful flush — id becomes 0 then advances to 1.
    client.queue_action(&actor, &symbol_short!("op"), &1, &0);
    let id = client.flush();
    assert_eq!(id, 0);
    assert_eq!(client.next_batch_id(), 1);
}

// ── Timestamp captured at flush ───────────────────────────────────────────────

/// BatchEvent.ledger_timestamp should match env.ledger().timestamp() at flush time.
#[test]
fn test_batch_timestamp_matches_ledger() {
    let (env, client) = setup();
    let actor = make_actor(&env);

    let expected_ts = env.ledger().timestamp();
    client.queue_action(&actor, &symbol_short!("op"), &1, &0);
    client.flush();

    let events = EventList::new(&env, env.events().all());
    let batch = decode_batch(&env, &events, 0);

    assert_eq!(batch.ledger_timestamp, expected_ts);
}

// ── Emit-count vs action-count comparison ────────────────────────────────────

/// Core property: N actions → 1 emitted event (not N events).
#[test]
fn test_n_actions_produce_one_event() {
    let (env, client) = setup();
    let actor = make_actor(&env);
    const N: u32 = 10;

    for i in 0..N {
        client.queue_action(&actor, &symbol_short!("op"), &(i as i128), &0);
    }

    client.flush();

    let events = EventList::new(&env, env.events().all());
    assert_eq!(events.len(), 1, "Expected 1 event, got more");

    let batch = decode_batch(&env, &events, 0);
    assert_eq!(batch.action_count, N);
}

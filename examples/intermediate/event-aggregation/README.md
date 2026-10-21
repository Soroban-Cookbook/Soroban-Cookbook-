# Event Aggregation

**Category**: Intermediate  
**Issue**: [#828](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/828)

Demonstrates how to **batch multiple actions into a single emitted event** instead
of emitting one event per action, reducing per-event overhead for high-throughput
Soroban contracts.

---

## The Problem

Every `env.events().publish()` call carries a fixed cost: XDR serialisation,
ledger-write overhead, and fee metering.  A contract that processes a list of
items in one invocation and emits an individual event per item pays that cost N
times:

```
[invocation]
  queue item 1 → emit event   ← overhead × 1
  queue item 2 → emit event   ← overhead × 2
  ...
  queue item N → emit event   ← overhead × N
```

For loops of dozens or hundreds of items, the cumulative overhead is significant.

---

## The Solution — Batch Aggregation

Accumulate individual `ActionEntry` values into an in-memory `Vec` during the
invocation, then emit **exactly one** `BatchEvent` at the end:

```
[invocation]
  queue_action(item 1) → accumulate
  queue_action(item 2) → accumulate
  ...
  flush() → emit ONE BatchEvent { actions: [item 1, item 2, …, item N] }
```

This trades one large serialisation for N small ones, and pays the
ledger-write overhead only once.

---

## Contract API

### `queue_action(actor, action_type, amount, memo)`

Appends one `ActionEntry` to the pending queue stored in instance storage.
Requires `actor.require_auth()`.  **No event is emitted here.**

### `flush() -> Result<u32, AggError>`

Drains the pending queue, emits a single `BatchEvent`, clears the queue, and
returns the `batch_id`.  Returns `AggError::EmptyBatch` if there are no queued
actions (see [Empty-batch policy](#empty-batch-policy)).

### `pending_count() -> u32`

Returns the number of actions currently queued.

### `next_batch_id() -> u32`

Returns the `batch_id` that will be assigned to the next `flush()`.

---

## Data Schema

### `ActionEntry`

```rust
pub struct ActionEntry {
    pub action_type: Symbol,  // e.g. symbol_short!("transfer")
    pub actor:       Address, // auth principal
    pub amount:      i128,    // token amount, vote weight, etc.
    pub memo:        u64,     // opaque application-level correlation id
}
```

Mirrors a simplified SEP-41 transfer record.  Adapt `action_type` to your
domain's vocabulary (`"mint"`, `"burn"`, `"vote"`, …).

### `BatchEvent` (the emitted data payload)

```rust
pub struct BatchEvent {
    pub batch_id:         u32,              // monotonically increasing
    pub ledger_timestamp: u64,              // env.ledger().timestamp() at flush
    pub action_count:     u32,              // == actions.len()
    pub actions:          Vec<ActionEntry>, // ordered, intra-batch index preserved
}
```

---

## Event Topic Layout

```
env.events().publish(
    (NS, ACTION_BATCH, batch_id),  // topics — indexed, filterable
    BatchEvent { … },              // data   — decoded after topic match
);
```

| Slot | Value       | Type   | Purpose                                     |
|------|-------------|--------|---------------------------------------------|
| [0]  | `"evt_agg"` | Symbol | Namespace — catch all events from this contract |
| [1]  | `"batch"`   | Symbol | Action tag — filter batch events specifically   |
| [2]  | `batch_id`  | u32    | Correlation id — join across services           |

> **Why is `batch_id` in topics?** Downstream services often need to join a
> batch with an off-chain record created at the same time (e.g. a database row
> that stores the `batch_id`).  Putting it in topics avoids decoding the full
> payload just to correlate.  Amount totals, timestamps, and the action list
> itself are in the data slot because they are read *after* a match, not
> filtered on.

---

## Indexing Guide

Off-chain indexers (Horizon, custom event-stream processors) should:

1. **Filter** events where `topic[0] == "evt_agg"` AND `topic[1] == "batch"`.
2. **Decode** the XDR `data` field as `BatchEvent`.
3. **Iterate** `batch.actions` in array order.  The array index is the
   intra-batch sequence number — it is deterministic and preserved across
   re-indexing runs.
4. **Filter by inner action type** after decoding:
   ```python
   for i, entry in enumerate(batch.actions):
       if entry.action_type == "transfer":
           process_transfer(batch.batch_id, i, entry)
   ```
5. **Time-range filter**: use `batch.ledger_timestamp` for coarse time
   windows.  For exact ledger-level ordering combine with the ledger sequence
   number from the event envelope (available from Horizon's `/events` endpoint
   as `ledger`).
6. **Correlation**: `batch_id` is monotonically increasing per contract
   instance.  Use it as a stable foreign key when storing derived data.

### Example Horizon query (pseudo-code)

```
GET /events
  ?type=contract
  &contractId=<CONTRACT_ADDRESS>
  &topic1=evt_agg          ← namespace
  &topic2=batch            ← action
```

Horizon returns events in ledger order; iterate and decode `data` as
`BatchEvent` for each result.

---

## Empty-batch Policy

`flush()` with no queued actions returns `AggError::EmptyBatch` **instead of**
emitting an empty event.

**Why?** An empty flush almost always indicates a programming mistake in the
caller (e.g. a code path that forgot to queue anything before flushing).
Surfacing it as an explicit error makes the mistake visible immediately.

Callers that want a deliberate no-op can guard with `pending_count()`:

```rust
if contract.pending_count() > 0 {
    contract.flush();
}
```

---

## Usage Example

```rust
// Within a single invocation — accumulate then flush once.
let actor = Address::generate(&env);

contract.queue_action(&actor, &symbol_short!("transfer"), &500, &1001);
contract.queue_action(&actor, &symbol_short!("burn"),     &100, &0);
contract.queue_action(&actor, &symbol_short!("mint"),     &200, &1002);

// One event emitted, batch_id = 0
let batch_id = contract.flush();
assert_eq!(batch_id, 0);
```

---

## Build & Test

```bash
# Run all unit tests
cargo test -p event-aggregation

# Build to WASM
cargo build --target wasm32-unknown-unknown --release -p event-aggregation
```

---

## Related Examples

- [`event-history`](../event-history/) — on-chain audit trail with pagination
- [`multi-sig-patterns`](../multi-sig-patterns/) — access-control gating patterns
- [`ajo-factory`](../ajo-factory/) — factory/deployment patterns
- [Events reference](../../book/src/examples/events.md) — topic layout conventions

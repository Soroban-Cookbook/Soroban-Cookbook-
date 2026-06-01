# Structured Event Patterns

This example demonstrates how to emit well-structured, query-friendly events in Soroban smart contracts. It covers the full event anatomy — topics, data payloads, and `#[contracttype]` structs — and shows practical patterns for topic design, off-chain filtering, and gas-conscious emission.

For a quick syntax reference, see [`EVENT_QUICK_REFERENCE.md`](./EVENT_QUICK_REFERENCE.md).

## Key Concepts

- `env.events().publish()` — the single API for emitting on-ledger events
- `#[contracttype]` — derives `SCVal` serialisation for structured data payloads
- `symbol_short!` — compiles short identifiers (≤ 9 alphanumeric chars) into compact `Symbol` values at compile time
- `Symbol` — the recommended type for topic slots: compact, `Val`-serialisable, and filterable
- `Address` — an indexable topic type for sender/recipient/admin fields
- `#![no_std]` — Soroban contracts run in a `no_std` Wasm sandbox

---

## Event Model

### Anatomy of a Soroban Event

Every Soroban event has two parts:

```rust
env.events().publish(
    (topic_0, topic_1, topic_2, topic_3),  // up to 4 indexed topics
    data_payload,                           // arbitrary SCVal; not indexed
);
```

- **Topics** are indexed by off-chain indexers (e.g. Stellar Horizon). They must be `Val`-serialisable types (`Symbol`, `Address`, small integers) and are limited to **4 per event**.
- **Data payload** is not indexed. It is decoded by consumers *after* topic-based filtering. Use `#[contracttype]` structs for structured payloads.

> Events are for **observability, not storage**. They do not persist state and cannot be read back by the contract itself.

### Topic Slot Convention

| Slot | Purpose | Example (from this contract) |
|------|---------|------------------------------|
| 0 | Contract namespace | `"events"` |
| 1 | Action name | `"transfer"` |
| 2 | Primary index | `sender: Address` |
| 3 | Secondary index | `recipient: Address` |

Using a shared namespace in `topic[0]` lets indexers discover every event type from this contract with a single prefix filter.

### Structured Payload Types

`#[contracttype]` structs are the recommended way to define event data payloads. They serialise automatically as `SCVal` and can be decoded by off-chain consumers using the contract's ABI.

```rust
#[contracttype]
pub struct TransferEventData {
    pub amount: i128,
    pub memo: u64,
}
```

---

## When to Use Events

### Use events for

- Wallet and UI updates (e.g. notify a frontend that a balance changed)
- Off-chain indexing and analytics (e.g. build a transaction history)
- Monitoring and alerting (e.g. detect large transfers)
- Audit trails for governance or admin actions
- Cross-system integrations (e.g. trigger a webhook on a state change)

### Avoid events for

- Internal-only computations that no external consumer needs
- Data that is already queryable from contract storage
- Redundant or noisy state transitions (e.g. emitting on every loop iteration without a bound)

Events do not persist state and cannot be read back by the contract itself. If you need the data on-chain, store it; if you need it off-chain, emit it.

---

## Topic Design Guidelines

### Rule 1 — Use `topic[0]` as the namespace or event type

Placing a contract-wide namespace in `topic[0]` enables a single-prefix filter for all events from this contract:

```
filter: topic[0] == "events"   →  returns every event this contract emits
```

### Rule 2 — Put filterable fields in topics; put payload data in data

Frequently queried identifiers (addresses, IDs, status values) belong in topics. Large or non-filterable data (amounts, timestamps, free-form text) belongs in the data payload.

### Rule 3 — Topic order and meaning MUST remain stable

Changing topic layout across contract versions breaks existing indexer filters. Treat your topic schema as a public API.

If a breaking change is unavoidable, use a versioned action name as the migration strategy:

```rust
// v1
const ACTION_TRANSFER: Symbol = symbol_short!("transfer");

// v2 — new topic layout; old filters still work on v1 events
const ACTION_TRANSFER_V2: Symbol = symbol_short!("trnsfr_v2");
```

### Rule 4 — All topic values must be `Val`-serialisable and fit within 4 topics

Valid topic types: `Symbol`, `Address`, `u32`, `i32`, `bool`. Do not place structs, `Vec`, or `Map` in topics.

### Rule 5 — Use consistent naming: snake_case, short Symbols via `symbol_short!`

```rust
const ACTION_TRANSFER:      Symbol = symbol_short!("transfer");
const ACTION_CONFIG_UPDATE: Symbol = symbol_short!("cfg_upd");
const ACTION_ADMIN:         Symbol = symbol_short!("admin");
const ACTION_AUDIT:         Symbol = symbol_short!("audit");
```

### Topic Layout Reference

| Function | topic[0] | topic[1] | topic[2] | topic[3] | Data Payload |
|----------|----------|----------|----------|----------|--------------|
| `transfer` | `"events"` | `"transfer"` | `sender: Address` | `recipient: Address` | `TransferEventData` |
| `update_config` | `"events"` | `"cfg_upd"` | `key: Symbol` | — | `ConfigUpdateEventData` |
| `admin_action` | `"events"` | `"admin"` | `admin: Address` | — | `AdminActionEventData` |
| `audit_trail` | `"events"` | `"audit"` | `actor: Address` | `action: Symbol` | `AuditTrailEventData` |
| `emit_simple` | `"simple"` | — | — | — | `value: u64` |
| `emit_tagged` | `"tagged"` | `tag: Symbol` | — | — | `value: u64` |
| `emit_multiple` | `"multi"` | `i: u32` | — | — | `i as u64` |
| `emit_transfer` | `"transfer"` | `from: Address` | `to: Address` | — | `amount: u64` |
| `emit_namespaced` | `category: Symbol` | `action: Symbol` | `pool_id: Symbol` | — | `amount: u64` |
| `emit_status_change` | `"status"` | `entity_id: Symbol` | `old_status: Symbol` | `new_status: Symbol` | `ledger sequence: u32` |

---

## Monitoring and Filtering Tips

### Recommended filtering strategy

Apply `topic[0]` as the primary filter, then narrow by subsequent topic positions:

```
topic[0] == "events"                                    →  all events from this contract
topic[0] == "events" AND topic[1] == "transfer"         →  all transfer events
topic[0] == "events" AND topic[1] == "transfer"
  AND topic[2] == <alice>                               →  all transfers sent by Alice
topic[0] == "events" AND topic[1] == "transfer"
  AND topic[3] == <bob>                                 →  all transfers received by Bob
topic[0] == "events" AND topic[1] == "transfer"
  AND topic[2] == <alice> AND topic[3] == <bob>         →  Alice → Bob transfers only
```

The same pattern applies to `emit_transfer` (which uses `"transfer"` as `topic[0]`):

```
topic[0] == "transfer"                                  →  all emit_transfer events
topic[0] == "transfer" AND topic[1] == <alice>          →  all sends by Alice
topic[0] == "transfer" AND topic[2] == <bob>            →  all receives by Bob
topic[0] == "transfer" AND topic[1] == <alice>
  AND topic[2] == <bob>                                 →  Alice → Bob only
```

### Indexer implementation advice

Indexers SHOULD handle unknown or new event types gracefully rather than failing on unrecognised topics. A robust implementation ignores events it does not recognise and logs them for later analysis.

### Payload types reference

After filtering by topics, decode the data payload using the corresponding `#[contracttype]` struct:

| Function | Payload struct | Fields |
|----------|---------------|--------|
| `transfer` | `TransferEventData` | `amount: i128`, `memo: u64` |
| `update_config` | `ConfigUpdateEventData` | `old_value: u64`, `new_value: u64` |
| `admin_action` | `AdminActionEventData` | `action: Symbol`, `timestamp: u64` |
| `audit_trail` | `AuditTrailEventData` | `details: Symbol`, `timestamp: u64`, `sequence: u32` |
| `emit_simple` | *(primitive)* | `value: u64` |
| `emit_tagged` | *(primitive)* | `value: u64` |
| `emit_multiple` | *(primitive)* | `i as u64` |
| `emit_transfer` | *(primitive)* | `amount: u64` |
| `emit_namespaced` | *(primitive)* | `amount: u64` |
| `emit_status_change` | *(primitive)* | `ledger sequence: u32` |

---

## Gas Cost Considerations

Each `env.events().publish()` call incurs a gas cost. That cost increases with:

1. The **number of topics** — more topics = higher cost per event
2. The **size of the data payload** — larger structs cost more to serialise

### Practical guidelines

- **Keep topics compact** — use `Symbol`, `Address`, and small integers. Avoid placing structs or variable-length data in topics; they have stricter type constraints and higher serialisation cost.
- **Place large data in the payload** — amounts, timestamps, and free-form text belong in the data struct, not in topics.
- **Bound loop emission** — `emit_multiple` demonstrates sequential emission inside a loop. Always cap the iteration count to prevent unbounded gas consumption:

```rust
pub fn emit_multiple(env: Env, count: u32) {
    for i in 0..count {
        env.events().publish((symbol_short!("multi"), i), i as u64);
    }
}
```

Without a caller-enforced or contract-enforced limit, a large `count` will make the transaction prohibitively expensive for callers.

- **Avoid unnecessary emission in hot paths** — events are not free. Emitting on every state read or minor computation increases transaction fees for every caller.

---

## Event Patterns Demonstrated

### 1. Transfer event — 4 topics + structured data

```rust
pub fn transfer(env: Env, sender: Address, recipient: Address, amount: i128, memo: u64) {
    env.events().publish(
        (CONTRACT_NS, ACTION_TRANSFER, sender, recipient),
        TransferEventData { amount, memo },
    );
}
```

Both addresses are in topics so indexers can efficiently query transfers by sender or recipient without scanning payloads.

### 2. Config update event — 3 topics + structured data

```rust
pub fn update_config(env: Env, key: Symbol, old_value: u64, new_value: u64) {
    env.events().publish(
        (CONTRACT_NS, ACTION_CONFIG_UPDATE, key),
        ConfigUpdateEventData { old_value, new_value },
    );
}
```

The config `key` is indexed so consumers can subscribe to changes for a specific parameter only.

### 3. Admin action event — 3 topics + structured data

```rust
pub fn admin_action(env: Env, admin: Address, action: Symbol) {
    let timestamp = env.ledger().timestamp();
    env.events().publish(
        (CONTRACT_NS, ACTION_ADMIN, admin),
        AdminActionEventData { action, timestamp },
    );
}
```

### 4. Audit trail event — 4 topics + structured data

```rust
pub fn audit_trail(env: Env, actor: Address, action: Symbol, details: Symbol) {
    let timestamp = env.ledger().timestamp();
    let sequence = env.ledger().sequence();
    env.events().publish(
        (CONTRACT_NS, ACTION_AUDIT, actor, action),
        AuditTrailEventData { details, timestamp, sequence },
    );
}
```

Full accountability: who (`actor`), what (`action`), when (`timestamp`), and sequence for ordering.

### 5. Namespaced event — 3-topic hierarchy

```rust
pub fn emit_namespaced(env: Env, category: Symbol, action: Symbol, pool_id: Symbol, amount: u64) {
    env.events().publish((category, action, pool_id), amount);
}
```

Useful when a contract owns multiple logical sub-systems. Indexers can filter at any level of the hierarchy.

### 6. Status change event — all 4 topics

```rust
pub fn emit_status_change(env: Env, entity_id: Symbol, old_status: Symbol, new_status: Symbol) {
    let ledger = env.ledger().sequence();
    env.events().publish(
        (symbol_short!("status"), entity_id, old_status, new_status),
        ledger,
    );
}
```

All 4 topic slots used: indexers can query by entity, by old state, by new state, or by specific transitions.

---

## Build

```bash
# From this directory
cargo build --target wasm32-unknown-unknown --release

# Or from the repository root
cargo build -p events --target wasm32-unknown-unknown --release
```

---

## Test

```bash
# From this directory
cargo test

# Or from the repository root
cargo test -p events
```

| Test | What it verifies |
|------|-----------------|
| `test_naming_convention_namespace_and_action_slots_are_stable` | All structured events share `topic[0]=namespace`, `topic[1]=action` convention |
| `test_transfer_emits_one_event` | `transfer` emits exactly one event |
| `test_transfer_event_has_four_topics` | `transfer` event carries 4 topics |
| `test_transfer_topic_namespace_and_action` | `topic[0]="events"`, `topic[1]="transfer"` |
| `test_transfer_indexed_addresses_in_topics` | `topic[2]=sender`, `topic[3]=recipient` |
| `test_transfer_structured_data_payload` | Data decodes to `TransferEventData { amount, memo }` |
| `test_config_update_emits_one_event` | `update_config` emits exactly one event |
| `test_config_update_event_has_three_topics` | `update_config` event carries 3 topics |
| `test_config_update_topic_namespace_and_action` | `topic[0]="events"`, `topic[1]="cfg_upd"` |
| `test_config_update_indexed_key_in_topic` | `topic[2]=key` |
| `test_config_update_structured_data_payload` | Data decodes to `ConfigUpdateEventData { old_value, new_value }` |
| `test_event_emission_exists` | `emit_simple` produces at least one event |
| `test_event_count_single` | `emit_simple` emits exactly one event |
| `test_event_count_multiple` | `emit_multiple(3)` emits exactly 3 events |
| `test_topic_structure_simple` | `emit_simple` topic[0] is `"simple"` |
| `test_topic_structure_tagged` | `emit_tagged` topic[0]="tagged", topic[1]=tag |
| `test_payload_values` | `emit_simple` data payload equals the supplied value |
| `test_zero_events_on_empty_emit` | `emit_multiple(0)` emits zero events |
| `test_emit_transfer_topic_layout` | `emit_transfer` topic layout and data payload are correct |
| `test_emit_transfer_independent_senders_queryable` | Multiple transfers are distinguishable by topic[1] |
| `test_emit_namespaced_three_topic_hierarchy` | `emit_namespaced` carries 3 topics in correct order |
| `test_emit_status_change_four_topics` | `emit_status_change` uses all 4 topic slots correctly |
| `test_admin_action_emits_one_event` | `admin_action` emits exactly one event |
| `test_admin_action_event_has_three_topics` | `admin_action` event carries 3 topics |
| `test_admin_action_topic_namespace_and_category` | `topic[0]="events"`, `topic[1]="admin"` |
| `test_admin_action_indexed_admin_address` | `topic[2]=admin address` |
| `test_admin_action_structured_data_payload` | Data decodes to `AdminActionEventData { action }` |
| `test_audit_trail_emits_one_event` | `audit_trail` emits exactly one event |
| `test_audit_trail_event_has_four_topics` | `audit_trail` event carries 4 topics |
| `test_audit_trail_topic_namespace_and_category` | `topic[0]="events"`, `topic[1]="audit"` |
| `test_audit_trail_indexed_actor_and_action` | `topic[2]=actor`, `topic[3]=action` |
| `test_audit_trail_structured_data_payload` | Data decodes to `AuditTrailEventData { details, timestamp, sequence }` |

---

## Project Structure

```
04-events/
├── Cargo.toml               # crate manifest
├── README.md                # this file
├── EVENT_QUICK_REFERENCE.md # quick-reference syntax card
└── src/
    ├── lib.rs               # contract definition and event payload types
    └── test.rs              # unit tests (32 tests)
```

---

## Next Steps

- [03-authentication](../03-authentication/) — restrict who can call your functions
- [05-auth-context](../05-auth-context/) — invoker detection and cross-contract call chains
- [05-error-handling](../05-error-handling/) — structured error types for contract functions

---

## Further Reading

- [Soroban Events — Stellar Developer Docs](https://developers.stellar.org/docs/smart-contracts/fundamentals-and-concepts/events)
- [Soroban SDK — `Events`](https://docs.rs/soroban-sdk/latest/soroban_sdk/events/struct.Events.html)
- [Stellar Horizon — Event Filtering](https://developers.stellar.org/docs/data/horizon)
- [Soroban SDK — `#[contracttype]`](https://docs.rs/soroban-sdk/latest/soroban_sdk/attr.contracttype.html)

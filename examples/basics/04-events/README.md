# Events

Learn how to design, emit, and structure Soroban events for observability, indexing, analytics, and production integrations.

This example goes beyond basic event emission and demonstrates structured, query-friendly, and audit-ready event patterns suitable for real-world contracts.

## 📖 What You'll Learn

- Core Soroban event model: **topics + data payload**
- How to design query-friendly topic schemas
- When to emit events (and when not to)
- Structured event payloads using `#[contracttype]`
- Multi-topic indexing strategies (up to 4 topics)
- Namespacing and long-term schema stability
- Monitoring and filtering patterns for indexers
- Gas/resource trade-offs when emitting events
- Deterministic event testing patterns


## 🔔 Event Concepts

In Soroban, every event has:

- **Topics** (indexed): up to 4 values used for filtering
- **Data** (payload): structured or primitive event body (not indexed)

```rust
env.events().publish(
    (topic_0, topic_1, topic_2, topic_3),
    data_payload,
);
```
## Key Rules
- Maximum of 4 topics

- Topics are indexed and filterable

- Data is not indexed but can be decoded

- Topic ordering is part of the schema contract

Think of topics as your query keys and payload as your event body.

## 🧭 When To Use Events

Use events when contract state changes matter to systems outside the contract:

- Wallet and UI updates

- Indexers and analytics pipelines

- Monitoring/alerting systems

- Audit trails for governance or admin actions

- Cross-system integrations

Avoid events for:

- Internal-only computations

- Data that no external consumer needs

- Redundant or noisy state transitions

- Events are for observability, not storage.

## 🔍 Event Patterns Demonstrated in This Contract

This example includes both minimal and production-grade structured patterns.

### Minimal Event
```rust
pub fn emit_simple(env: Env, value: u64)
```
### Topics:
```code
("simple")
```
### Data:
```code
value
```
Use this for the simplest event case.

### Tagged Event
```rust
pub fn emit_tagged(env: Env, tag: Symbol, value: u64)
```
### Topics:
```code
("tagged", tag)
```
Useful when grouping events by dynamic category.

### Transfer Event (4 Topics + Structured Payload)
```rust
pub fn transfer(env: Env, sender: Address, recipient: Address, amount: i128, memo: u64)
```
### Topics:
```code
("events", "transfer", sender, recipient)
```
### Data:
```rust
TransferEventData { amount, memo }
```
This enables efficient filtering:

- All transfers

- Transfers from a specific address

- Transfers to a specific address

- Transfers between two specific addresses

### Configuration Update Event
```rust
pub fn update_config(env: Env, key: Symbol, old_value: u64, new_value: u64)
```
### Topics:
```code
("events", "cfg_upd", key)
```
### Data:
```rust
ConfigUpdateEventData { old_value, new_value }
```
Allows targeted monitoring of specific configuration keys.

### Admin Action Event
```rust
pub fn admin_action(env: Env, admin: Address, action: Symbol)
```
### Topics:
```code
("events", "admin", admin)
```
### Data:
```rust
AdminActionEventData { action, timestamp }
```
Tracks privileged operations in a filterable way.

### Audit Trail Event (Full Accountability Pattern)
```rust
pub fn audit_trail(env: Env, actor: Address, action: Symbol, details: Symbol)
```
### Topics:
```code
("events", "audit", actor, action)
```
### Data:
```rust
AuditTrailEventData { details, timestamp, sequence }
```
Provides:

- Who performed the action

- What action was performed

- When it occurred

- Ledger ordering information

This pattern is ideal for compliance, governance, and high-trust systems.

### Multi-Emission Pattern
```rust
pub fn emit_multiple(env: Env, count: u32)
```
Emits sequential indexed events inside a loop.

⚠ Production contracts should enforce sensible limits to avoid excessive gas consumption.

### Query-Optimized Transfer Pattern
```rust
(topic[0] = "transfer", topic[1] = from, topic[2] = to)
```
Off-chain filtering examples:

- All transfers
`topic[0] == "transfer"`

- Transfers from Alice
`topic[0] == "transfer" AND topic[1] == Alice`

- Transfers to Bob
`topic[0] == "transfer" AND topic[2] == Bob`

- Alice → Bob transfers
All three topics fixed

Design topics intentionally for filtering efficiency.


## 🏷️ Topic Design Guidelines

### 1. Keep Topic 0 as the Event Type or Namespace

This contract uses:
```rust
const CONTRACT_NS: Symbol = symbol_short!("events");
```
This allows indexers to retrieve all contract events using a shared prefix.

### 2. Index What You Filter

Put frequently queried identifiers in topics:

- Addresses

- IDs

- Symbols

- Status values

Keep larger data in the payload.

### 3. Keep Topic Shape Stable

Changing topic order/meaning breaks indexers. Prefer additive changes and versioned event names when needed:

- `transfer_v1`
- `transfer_v2`

### 4. Be Consistent Across Functions
### Structured Event Payloads

Use `#[contracttype]` to define rich data payloads that are stored in the event's data slot:

```rust
#[contracttype]
pub struct TransferEventData {
    pub amount: i128,
    pub memo: u64,
}
```

### Multiple Topics & Indexing

- **Topics** (up to 4) are indexed and searchable off-chain.
- **Data** is the rich payload, not indexed but decodable.
- **Naming Convention**: Use a consistent `(namespace, action, [key...])` layout.

```rust
// Publishing 4 topics (contract name, action, sender, recipient)
env.events().publish(
    (symbol_short!("events"), symbol_short!("transfer"), sender, recipient),
    TransferEventData { amount, memo }
);
```

### State Change Tracking

Use structured events to create an on-chain audit log that off-chain systems can replay:

- **Admin actions** — Track privileged operations with a 3-topic layout `(namespace, "admin", admin_address)`. The data payload carries the action symbol and ledger timestamp, giving indexers a filterable record of every admin operation.
- **Audit trails** — Full accountability tracking with a 4-topic layout `(namespace, "audit", actor, action)`. The data payload includes human-readable details, a timestamp, and the ledger sequence number for deterministic ordering.

```rust
#[contracttype]
pub struct AdminActionEventData {
    pub action: Symbol,
    pub timestamp: u64,
}

#[contracttype]
pub struct AuditTrailEventData {
    pub details: Symbol,
    pub timestamp: u64,
    pub sequence: u32,
}
```

Choose admin action events when you need a simple record of who did what. Choose audit trail events when you also need to capture why (details) and guarantee ordering (sequence).

### Topics and Indexing

Use one naming convention for all event types (`snake_case`, short symbols, deterministic order).

## 📡 Monitoring and Filtering Tips

### Off-chain Consumers Should

- Filter by `topic 0` first (event type)
- Apply secondary filters by topic position (`topic[1]`, `topic[2]`, ...)
- Treat payload as schema-bound data for downstream parsing
- Handle unknown/new event types gracefully

### Practical Pattern

- Use topics for fast selection (`("tagged", tag)`)
- Use payload for business values (`amount`, struct-like tuples)

This keeps index queries efficient and reduces parsing overhead for unrelated events.

## ⛽ Gas and Resource Considerations

Event emission consumes resources. Keep event design intentional:

- More events per call => higher cost
- More/larger topic values => higher cost
- Larger payloads => higher cost

Recommendations:

- Emit only meaningful events
- Prefer compact topic keys
- Avoid duplicate/noise events
- Batch only when downstream consumers need each item event

In this example, `emit_multiple` is useful for demonstrating patterns, but production usage should enforce sensible limits on `count`.

## 🧪 Testing Strategy

Run tests:

```bash
cargo test
```

The test suite validates:

- Event emission exists
- Correct event counts (single/multiple/zero)
- Topic structure and ordering
- Payload correctness
- Distinct actions emit distinct event types
- No unexpected extra events
- **Event emission** — At least one event is emitted
- **Event count** — Correct number of events per action
- **Topic structure** — Topics match expected shape and values
- **Payload values** — Event data matches emitted values
- **Action differentiation** — Different actions emit distinct topics
- **No extra events** — Only expected events are emitted
- **Admin action events** — Correct topic structure and payload for admin operations
- **Audit trail events** — Full accountability tracking with actor, action, and details

## 🚀 Build and Deploy

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/events.wasm \
  --source alice \
  --network testnet
```

## ✅ Event Best Practices Checklist

- Event type in topic 0
- Topics reserved for filterable identifiers
- Payload reserved for non-indexed business data
- Stable schema and topic ordering
- Event tests for count, structure, and payload
- Cost-aware emission strategy

## 🎓 Next Steps

- [Basics Index](../README.md) - Continue the fundamentals track
- [Storage Patterns](../02-storage-patterns/) - Pair state changes with events
- [Intermediate Examples](../../intermediate/) - Explore multi-contract systems

## 📚 References

- [Soroban Events Docs](https://developers.stellar.org/docs/smart-contracts/fundamentals-and-concepts/logging-events)
- [Soroban SDK `Events`](https://docs.rs/soroban-sdk/latest/soroban_sdk/struct.Events.html)

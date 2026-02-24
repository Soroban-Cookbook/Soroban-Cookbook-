# Events

Learn how to design and emit Soroban events for observability, indexing, analytics, and integrations.

This example focuses on practical event patterns you can reuse in production contracts.

## 📖 What You'll Learn

- Core Soroban event model: **topics + data payload**
- When to emit events (and when not to)
- Topic schema design for long-term compatibility
- Monitoring and filtering patterns for indexers
- Gas/resource trade-offs when emitting events
- How to test event behavior deterministically

## 🔔 Event Concepts

In Soroban, each event has:

- **Topics** (indexed): up to 4 values used for filtering
- **Data** (payload): associated event value/body

```rust
env.events().publish((symbol_short!("transfer"), from, to), amount);
```

Think of topics as your query keys and payload as your event body.

## 🧭 When To Use Events

Use events when state changes matter to systems outside the contract:

- Wallet and UI updates
- Indexers and analytics pipelines
- Alerting/monitoring workflows
- Audit trails for important actions

Avoid events for internal-only computations that no external system needs.

## 🔍 Example Contract API

This contract demonstrates three event patterns:

```rust
// Single-topic event
pub fn emit_simple(env: Env, value: u64)

// Type + tag in topics
pub fn emit_tagged(env: Env, tag: Symbol, value: u64)

// Repeated event emission with index topic
This example demonstrates both simple and structured event patterns:

### Structured Events (Recommended)

```rust
// Transfer event: 4 topics (ns, action, sender, recipient) + custom payload
pub fn transfer(env: Env, sender: Address, recipient: Address, amount: i128, memo: u64)

// Config update event: 3 topics (ns, action, key) + custom payload
pub fn update_config(env: Env, key: Symbol, old_value: u64, new_value: u64)
```

```rust
// Admin action event: 3 topics (ns, category, admin) + action data
pub fn admin_action(env: Env, admin: Address, action: Symbol)

// Audit trail event: 4 topics (ns, category, actor, action) + detailed data
pub fn audit_trail(env: Env, actor: Address, action: Symbol, details: Symbol)
```

### Simple Helpers

```rust
// Simple event: single topic
pub fn emit_simple(env: Env, value: u64)

// Tagged event: two topics (name + tag)
pub fn emit_tagged(env: Env, tag: Symbol, value: u64)

// Multiple events: emits N indexed events in a loop
pub fn emit_multiple(env: Env, count: u32)
```

## 🏷️ Topic Design Guidelines

### 1. Keep Topic 0 as the Event Type

Use the first topic as a stable event name:

```rust
env.events().publish((symbol_short!("simple"),), value);
env.events().publish((symbol_short!("tagged"), tag), value);
```

### 2. Use Remaining Topics for Filter Keys

Put high-value filter fields in topics (tags, IDs, addresses, indices).  
Keep larger or less frequently queried data in the payload.

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

- Filter by **topic 0** first (event type)
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

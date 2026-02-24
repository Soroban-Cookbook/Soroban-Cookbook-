# Events

Learn how to emit and handle events in Soroban smart contracts for off-chain monitoring and indexing.

## 📖 What You'll Learn

- Event structure: topics (up to 4) + data payload
- Publishing events with `env.events().publish()`
- Event topics for indexing and filtering
- Testing emitted events with the Soroban test environment

## 🔔 Event Basics

Soroban events consist of:

- **Topics** — Up to 4 identifiers (e.g. event name, indexed parameters)
- **Data** — The event payload (e.g. amount, address)

```rust
env.events().publish((symbol_short!("transfer"), from, to), amount);
```

## 🔍 Contract Overview

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

## 💡 Key Concepts

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

- Topics are indexed and can be used for off-chain filtering
- First topic typically identifies the event type
- Additional topics can carry indexed parameters (addresses, IDs)

## 🧪 Testing

```bash
cargo test
```

Tests cover:

- **Event emission** — At least one event is emitted
- **Event count** — Correct number of events per action
- **Topic structure** — Topics match expected shape and values
- **Payload values** — Event data matches emitted values
- **Action differentiation** — Different actions emit distinct topics
- **No extra events** — Only expected events are emitted
- **Admin action events** — Correct topic structure and payload for admin operations
- **Audit trail events** — Full accountability tracking with actor, action, and details

## 🚀 Building & Deployment

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/events.wasm \
  --source alice \
  --network testnet
```

## 🎓 Next Steps

- [Basics Index](../README.md) - Browse the full basics learning path
- [Storage Patterns](../02-storage-patterns/) - Combine events with state changes
- [Intermediate Examples](../../intermediate/) - Multi-contract patterns

## 📚 References

- [Events Documentation](https://developers.stellar.org/docs/smart-contracts/fundamentals-and-concepts/logging-events)
- [Soroban SDK Events](https://docs.rs/soroban-sdk/latest/soroban_sdk/struct.Events.html)

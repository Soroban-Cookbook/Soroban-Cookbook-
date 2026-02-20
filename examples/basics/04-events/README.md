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

This example demonstrates:

```rust
// Simple event: single topic, u64 data
pub fn emit_simple(env: Env, value: u64)

// Tagged event: two topics (name + tag), u64 data
pub fn emit_tagged(env: Env, tag: Symbol, value: u64)

// Multiple events: emits N events with sequential indices
pub fn emit_multiple(env: Env, count: u32)
```

## 💡 Key Concepts

### Publishing Events

```rust
// Single-topic event
env.events().publish((symbol_short!("simple"),), value);

// Multi-topic event (topics are indexed for filtering)
env.events().publish((symbol_short!("tagged"), tag), value);
```

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

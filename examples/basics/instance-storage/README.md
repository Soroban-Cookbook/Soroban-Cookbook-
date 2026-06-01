# Instance Storage Pattern

A focused demonstration of Soroban instance storage, the storage map attached
to a deployed contract instance.

## What You'll Learn

- How to read and write values with `env.storage().instance()`
- Why instance storage is useful for small contract-wide state
- How shared instance TTL differs from persistent storage's per-key TTL
- When to choose instance storage and when to use persistent storage instead
- How to test counters, runtime config, and TTL keep-alive behavior

## Overview

Instance storage is physically attached to the contract instance ledger entry.
It shares the contract instance TTL, so a live contract instance keeps its
instance storage live too.

This makes instance storage a good fit for small, bounded state that is useful
across most calls:

- Admin or owner address
- Protocol configuration, such as fee basis points or limits
- Contract metadata and version flags
- Counters or aggregate totals
- Token pair metadata for a pool contract

Do not use instance storage for unbounded per-user or per-entity data. Instance
data is loaded with the contract instance, so large instance storage can make
every invocation more expensive.

## Instance vs Persistent Storage

| Feature | Instance Storage | Persistent Storage |
| --- | --- | --- |
| SDK API | `env.storage().instance()` | `env.storage().persistent()` |
| TTL scope | Shared with the contract instance | Independent per key |
| Size model | Limited by instance ledger-entry size | Supports unbounded keys |
| Best for | Small shared config/state | User, asset, proposal, or record data |
| Invocation footprint | Loaded with the contract instance | Loaded only when accessed |
| Expiration behavior | Archived with the instance | Archived per entry |

Use instance storage when the data is small, shared, and has a known upper
bound. Use persistent storage when the data grows with users, tokens, proposals,
orders, or other entities.

## Contract API

This example exposes six functions:

| Function | Purpose |
| --- | --- |
| `set_instance(key, value)` | Store a `u64` under a named config key |
| `get_instance(key)` | Read a named config key as `Option<u64>` |
| `increment_counter()` | Increment a contract-wide transaction counter |
| `get_counter()` | Read the counter, defaulting to `0` |
| `set_config(key, value)` | Semantic wrapper for runtime config |
| `get_config(key)` | Read runtime config as `Option<u64>` |
| `extend_ttl()` | Explicitly refresh the instance TTL |

## Key Pattern

The contract uses a typed key enum instead of raw symbols:

```rust
#[contracttype]
#[derive(Clone)]
pub enum InstanceKey {
    TxCounter,
    Config(Symbol),
}
```

Typed keys make the storage layout explicit and reduce accidental collisions.

## Writing Instance Data

```rust
pub fn set_instance(env: Env, key: Symbol, value: u64) {
    let storage_key = InstanceKey::Config(key);
    env.storage().instance().set(&storage_key, &value);
    env.storage().instance().extend_ttl(1_000, 10_000);
}
```

The important difference from persistent storage is that the TTL extension does
not name a specific key. A single instance TTL extension applies to the contract
instance and all instance storage entries.

## Use Case: Runtime Configuration

Runtime config is one of the safest instance-storage use cases because it is
small and contract-wide:

```rust
client.set_config(&symbol_short!("fee_bps"), &30);
assert_eq!(client.get_config(&symbol_short!("fee_bps")), Some(30));
```

Examples include protocol fees, caps, cooldown durations, feature flags, and
small version markers.

## Use Case: Contract Counter

The example also includes a counter:

```rust
assert_eq!(client.increment_counter(), 1);
assert_eq!(client.increment_counter(), 2);
assert_eq!(client.get_counter(), 2);
```

A counter is appropriate here because it is one small piece of shared state. A
counter per user would belong in persistent storage instead.

## Best Practices

1. Keep instance storage small and bounded.
2. Store user-specific and entity-specific records in persistent storage.
3. Refresh instance TTL from read/write paths that indicate active use.
4. Use typed key enums with `#[contracttype]`.
5. Return `Option<T>` for keys that may be unset.

## Running Tests

From the repository root:
# Instance Storage

A focused demonstration of Soroban's instance storage — the middle tier between persistent and temporary storage.

## 📖 What You'll Learn

- When and why to use instance storage over persistent or temporary storage
- How all instance keys share a single TTL (unlike persistent's per-key TTL)
- Two real-world use cases: transaction counters and runtime configuration
- How to extend instance TTL efficiently with a single call

## 🎯 Overview

Instance storage is scoped to the deployed contract address. All keys share one TTL, so a single `extend_ttl` call protects every key at once — making it cheaper to manage than persistent storage for contract-wide state.

| Property                  | Persistent     | Instance       | Temporary      |
|---------------------------|----------------|----------------|----------------|
| Survives contract upgrade | ✅ Yes         | ❌ No          | ❌ No          |
| TTL management            | Per-key        | Per-instance   | Per-key        |
| Relative cost             | Highest        | Medium         | Lowest         |
| Best for                  | User balances  | Contract config| Short-lived    |

## 🔑 Key Concepts

### Shared TTL

```rust
// One call refreshes ALL instance keys — no per-key bookkeeping needed
env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
```

Compare this to persistent storage, where you must call `extend_ttl` once per key.

### Use Case 1 — Transaction Counter

```rust
pub fn increment_counter(env: Env) -> u64 {
    let count: u64 = env.storage().instance()
        .get(&InstanceKey::TxCounter)
        .unwrap_or(0) + 1;
    env.storage().instance().set(&InstanceKey::TxCounter, &count);
    env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
    count
}
```

### Use Case 2 — Runtime Configuration

```rust
pub fn set_config(env: Env, key: Symbol, value: u64) {
    env.storage().instance().set(&InstanceKey::Config(key), &value);
    env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}
```

## ✅ When to Use Instance Storage

**Use instance storage when:**
- Data is contract-wide (not per-user)
- Data does not need to survive a contract upgrade
- You want simpler TTL management than persistent storage

**Don't use instance storage when:**
- Data must survive `upgrade()` calls → use persistent
- Data is only needed for one invocation → use temporary
- Data is per-user (e.g. balances) → use persistent

## 🚀 Running Tests

```bash
cargo test -p instance-storage
```

Build the contract as Wasm:

```bash
cargo build -p instance-storage --target wasm32-unknown-unknown --release
```

Output:

```text
target/wasm32-unknown-unknown/release/instance_storage.wasm
```

## Related Examples

- [Storage Patterns](../02-storage-patterns/) - compares persistent, instance, and temporary storage
- [Persistent Storage](../persistent-storage/) - focused per-key durable storage example
- [Temporary Storage](../temporary_storage/) - short-lived storage example

## Further Reading

- [Use instance storage in a contract](https://developers.stellar.org/docs/build/guides/storage/use-instance)
- [Choosing the right storage type](https://developers.stellar.org/docs/build/guides/storage/choosing-the-right-storage)
- [State archival](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival)
## 📚 Related Examples

- [persistent-storage](../persistent-storage/) — Per-key TTL, user balances
- [temporary_storage](../temporary_storage/) — Single-ledger caches
- [02-storage-patterns](../02-storage-patterns/) — All three types compared

# Storage Patterns

Learn the fundamentals of storing and retrieving data in Soroban contracts across all three storage types: **Persistent**, **Instance**, and **Temporary**.

## Project Structure

```text
examples/basics/02-storage-patterns/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    └── test.rs
```

## What This Example Shows

- **Three storage tiers**: Persistent, Instance, and Temporary
- **TTL (Time-To-Live) Management**: How to keep your data alive and avoid archival
- **CRUD Operations**: Reading, writing, checking, and deleting entries
- **Cost Optimization**: Choosing the right storage type for different data lifetimes

## Core Concepts

| Type           | Lifetime       | Cost    | Best For                             |
| :------------- | :------------- | :------ | :----------------------------------- |
| **Persistent** | Permanent      | Highest | User balances, critical state        |
| **Instance**   | Instance-bound | Medium  | Admin config, counters               |
| **Temporary**  | Single ledger  | Lowest  | Intermediate data, short-lived flags |

## Implementation Preview

```rust
// Persistent: Per-key TTL
env.storage().persistent().set(&key, &value);
env.storage().persistent().extend_ttl(&key, 100, 100);

// Instance: Shared TTL for all instance data
env.storage().instance().set(&key, &value);
env.storage().instance().extend_ttl(100, 100);

// Temporary: Ephemeral, no rent
env.storage().temporary().set(&key, &value);
```

## Build

```bash
cargo build -p storage-patterns
```

## Test

```bash
cargo test -p storage-patterns
```

## Further Reading

- [Storage Types Reference](../../../docs/storage-types.md)
- [State Archival & TTL](https://developers.stellar.org/docs/smart-contracts/data/state-archival)
- [Detailed Instance Storage](../instance-storage/)
- [Detailed Persistent Storage](../persistent-storage/)
- [Detailed Temporary Storage](../temporary_storage/)

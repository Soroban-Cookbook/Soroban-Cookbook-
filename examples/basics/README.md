# Basic Examples

This category contains beginner-friendly examples that introduce the core concepts of Soroban smart contract development, one at a time. Each example is designed to be minimal, focused, and easy to understand.

## What's Inside?

- **Fundamental Concepts**: Learn about contract structure, storage types, authentication, custom errors, and event emission.
- **Core Data Types**: Understand how to work with Soroban's built-in types like `Address`, `Symbol`, `Vec`, `Map`, and primitive types.
- **Best Practices**: See simple, effective patterns for validation, error handling, and writing clean, testable code.

## Planned Examples

## 📋 All Examples

### Core Contract Structure

#### [01-hello-world](./01-hello-world/) — 🟢 Beginner
The simplest possible Soroban contract — a single `hello` function that returns a greeting vector.
- **Concepts:** `#[contract]`, `#[contractimpl]`, `Env`, `Symbol`, `Vec`, `#![no_std]`
- **Best for:** First contract, understanding the minimal contract skeleton

---

### Storage

#### [02-storage-patterns](./02-storage-patterns/) — 🟢 Beginner
All three Soroban storage layers (persistent, instance, temporary) side-by-side with TTL management.
- **Concepts:** `persistent`, `instance`, `temporary` storage; TTL extension; `DataKey` enums; storage isolation
- **Best for:** Understanding when to use each storage tier

#### [instance-storage](./instance-storage/) — 🟢 Beginner
Focused deep dive into instance storage — the contract-wide, shared-TTL tier.
- **Concepts:** Shared TTL, contract configuration, counters, `extend_ttl` on instance
- **Best for:** Storing admin addresses, protocol config, aggregate counters

#### [persistent-storage](./persistent-storage/) — 🟢 Beginner
Focused deep dive into persistent storage — the highest-durability tier with per-key TTL.
- **Concepts:** Per-key TTL, `extend_ttl` strategies, `DataKey` enum, `checked_add`
- **Best for:** User balances, ownership records, permissions

#### [temporary_storage](./temporary_storage/) — 🟢 Beginner
Focused deep dive into temporary storage — the cheapest, ephemeral tier.
- **Concepts:** Short-lived TTL, reentrancy guards, intra-transaction caching, gas cost trade-offs
- **Best for:** Flags, intermediate computation results, short-lived caches

---

### Error Handling

#### [03-custom-errors](./03-custom-errors/) — 🟡 Intermediate
Custom error enums with structured error codes for frontend integration.
- **Concepts:** `#[contracterror]`, `#[repr(u32)]`, error codes 1–8, `Result<T, E>`, event logging on error
- **Best for:** Any contract that needs typed, actionable errors

#### [05-error-handling](./05-error-handling/) — 🟡 Intermediate
Result-based error handling and propagation using `try_*` client methods.
- **Concepts:** `#[contracterror]`, `Result<T, Error>`, `try_*` test methods, `LimitExceeded`
- **Best for:** Learning the test-side error assertion pattern

---

### Authentication & Authorization

#### [03-authentication](./03-authentication/) — 🟡 Intermediate
Address-based authorization with layered access control: admin roles, RBAC, time-locks, cooldowns, and state gating.
- **Concepts:** `require_auth()`, admin pattern, role-based access, allowances, `transfer_from`, multi-sig, time-lock, circuit-breaker
- **Best for:** Any contract with privileged operations or user-level permissions

#### [05-auth-context](./05-auth-context/) — 🟡 Intermediate
Understanding execution context and authorization across cross-contract call chains.
- **Concepts:** `env.current_contract_address()`, `env.auths()`, invoker vs. current contract, proxy patterns
- **Best for:** Proxy contracts, factory patterns, inter-contract communication

---

### Events

#### [basic-event-emission](./basic-event-emission/) — 🟢 Beginner
The simplest possible event emission — single and two-topic events with a data payload.
- **Concepts:** `env.events().publish()`, topic tuples, data payload, `symbol_short!`
- **Best for:** First event, understanding the basic event API

#### [04-events](./04-events/) — 🟡 Intermediate
Production-grade structured event design with typed payloads, multi-topic indexing, and audit trails.
- **Concepts:** `#[contracttype]` payloads, 4-topic layout, namespace convention, `TransferEventData`, `AuditTrailEventData`, indexer-friendly schemas
- **Best for:** Contracts that need off-chain observability, analytics, or compliance trails

#### [events](./events/) — 🟡 Intermediate
A minimal counter contract that emits events on every state change — used in integration test scenarios.
- **Concepts:** Instance storage + events, `set_number`, `increment`, `decrement`, multi-contract test helpers
- **Best for:** Understanding how events and storage interact; integration testing patterns

#### [11-event-filtering](./11-event-filtering/) — 🟠 Advanced
Designing Soroban events specifically for efficient off-chain filtering.
- **Concepts:** Topic slot strategy, namespace in topic[0], primary/secondary entity indexing, query-optimized layouts, `record_sale`, `update_status`
- **Best for:** Indexer authors, contracts with high event volume, marketplace/DeFi contracts

---

### Types


### [12-error-handling](./12-error-handling/)

Foundational error handling patterns using Result and panic.

**Concepts:** `#[contracterror]`, `Result<T, E>`, error codes, `try_*` client methods, invariant panics

---

## Supporting Packages

### [11-collection-types](./11-collection-types/)
Working with `Vec` and `Map` collections in Soroban.
- **Concepts:** Collection operations, iteration, storage efficiency.


---

#### [09-primitive-types](./09-primitive-types/) — 🟠 Advanced
Integer types, overflow behaviour, boolean logic, and financial arithmetic.
- **Concepts:** `u32`, `u64`, `i128`, `checked_*`, `saturating_*`, `wrapping_*`, fixed-point arithmetic, bitmasks
- **Best for:** Financial contracts, counters, any arithmetic-heavy logic

#### [10-data-types](./10-data-types/) — 🟠 Advanced
Comprehensive reference for every Soroban data type with gas cost comparisons.
- **Concepts:** Full type system overview, `Symbol` vs `String` gas trade-off, `BytesN` vs `Bytes`, `Vec` vs `Map`, type conversion patterns
- **Best for:** Optimizing an existing contract's type choices

#### [11-collection-types](./11-collection-types/) — 🟠 Advanced
`Vec<T>` and `Map<K, V>` operations, iteration patterns, and performance trade-offs.
- **Concepts:** `push_back`, `pop_back`, `get`, `Map::keys()`, `Map::values()`, O(1) vs O(log n) access, zip pattern
- **Best for:** Contracts with lists, leaderboards, balance maps, or batch operations

---

### Validation & Data Modeling

### [instance-storage](./instance-storage/)
Focused demonstration of the Instance storage layer for small contract-wide state.
- **Concepts:** Shared instance TTL, bounded configuration, counters, persistent-storage trade-offs.

#### [07-enum-types](./07-enum-types/) — 🟠 Advanced
Contract-level enumerations for type-safe state, roles, and operation dispatch.
- **Concepts:** `#[contracttype]` enums, data enums with associated fields, `#[contracterror]`, exhaustive pattern matching, state machines
- **Best for:** Contracts with lifecycle states, role hierarchies, or polymorphic operations

#### [08-custom-structs](./08-custom-structs/) — 🟠 Advanced
Complex on-chain data structures with nested types, storage patterns, and serialization.
- **Concepts:** `#[contracttype]` structs, nested structs, `Vec<Struct>`, composite storage keys, `Option<T>` fields, portfolio modeling
- **Best for:** Contracts that store rich per-user or per-entity data

---

## 🏷️ Difficulty Key

| Badge | Level | Description |
|-------|-------|-------------|
| 🟢 Beginner | No prior Soroban knowledge needed | Basic Rust + blockchain concepts sufficient |
| 🟡 Intermediate | Assumes hello-world and storage basics | Introduces auth, errors, and events |
| 🟠 Advanced | Assumes intermediate examples | Deep type system, validation, data modeling |

---

## 🎯 Prerequisites

Before diving in, make sure you have:
- [Set up your development environment](../../guides/getting-started.md)
- [Read the Testing Guide](../../guides/testing.md)
- A basic understanding of Rust programming

---

## 🧪 Running Tests

```bash
# Run a single example
cargo test -p hello-world
cargo test -p storage-patterns
cargo test -p authentication

# Run all basic examples at once
cargo test --workspace
```

---

## 📋 Planned Examples

- **Iterative Mappings** — Efficient iteration over large data sets
- **Batch Processing** — Handling multiple operations in a single call
- **State Machine Patterns** — Structured state transitions for complex logic

---

## ➡️ What's Next

Once you're comfortable with the basics, move on to:
- [Intermediate Examples](../intermediate/) — Tokens, NFTs, governance patterns
- [Advanced Examples](../advanced/) — Multi-party auth, timelocks, complex DeFi

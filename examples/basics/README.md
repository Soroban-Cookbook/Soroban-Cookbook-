# Basic Examples

Core Soroban fundamentals, one concept per example. Perfect for beginners starting their journey with Soroban smart contract development.

## 🎯 Learning Path

Follow this recommended sequence to build your understanding progressively:

```
Beginner
  │
  ├── 01-hello-world          ← Start here: contract structure basics
  ├── basic-event-emission    ← Emit your first event
  ├── 02-storage-patterns     ← Understand all three storage tiers
  ├── instance-storage        ← Deep dive: instance storage
  ├── persistent-storage      ← Deep dive: persistent storage
  ├── temporary_storage       ← Deep dive: temporary storage
  │
Intermediate
  │
  ├── 03-custom-errors        ← Structured error handling
  ├── 05-error-handling       ← Error propagation patterns
  ├── 03-authentication       ← Auth with require_auth()
  ├── 05-auth-context         ← Cross-contract auth context
  ├── 04-events               ← Structured, production-grade events
  ├── events                  ← Counter contract with events
  ├── 11-event-filtering      ← Design events for off-chain filtering
  │
Advanced
  │
  ├── 06-soroban-types        ← Full type system overview
  ├── 06-type-conversions     ← Safe type conversion patterns
  ├── 09-primitive-types      ← Integer types and overflow safety
  ├── 10-data-types           ← Comprehensive data type reference
  ├── 11-collection-types     ← Vec and Map in depth
  ├── 06-validation-patterns  ← Input, state, and auth validation
  ├── 07-enum-types           ← Enums, state machines, dispatch
  └── 08-custom-structs       ← Complex on-chain data structures
```

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

| # | Example | Difficulty | Description | Key Concepts |
|:---:|:---|:---:|:---|:---|
| 1 | [01-hello-world](./01-hello-world/) | ⭐ | The "Hello World" of Soroban. | `#[contract]`, `Symbol`, Tests |
| 2 | [09-primitive-types](./09-primitive-types/) | ⭐ | Integer types and overflow safety. | `u32`, `i128`, Arithmetic safety |
| 3 | [06-soroban-types](./06-soroban-types/) | ⭐ | Built-in Soroban types. | `Address`, `Symbol`, `Bytes` |
| 4 | [10-data-types](./10-data-types/) | ⭐ | Comprehensive type exploration. | Data representation |
| 5 | [06-type-conversions](./06-type-conversions/) | ⭐⭐ | Secure type casting and conversion. | `Into`, `From`, `TryInto` |
| 6 | [07-enum-types](./07-enum-types/) | ⭐ | Using enums in contract logic. | Enums, Pattern matching |
| 7 | [08-custom-structs](./08-custom-structs/) | ⭐ | Complex data structures. | Structs, `#[contracttype]` |
| 8 | [11-collection-types](./11-collection-types/) | ⭐⭐ | Working with `Vec` and `Map`. | Collections, Iteration |
| 9 | [instance-storage](./instance-storage/) | ⭐⭐ | Shared contract-instance storage. | Instance storage, TTL |
| 10 | [persistent-storage](./persistent-storage/) | ⭐⭐ | Long-term data persistence. | Persistent storage, Keys |
| 11 | [temporary_storage](./temporary_storage/) | ⭐⭐ | Cost-optimized transient data. | Temporary storage, TTL mgmt |
| 12 | [02-storage-patterns](./02-storage-patterns/) | ⭐⭐⭐ | Advanced storage management. | Combined storage layers |
| 13 | [basic-event-emission](./basic-event-emission/) | ⭐ | Simple event publishing. | `env.events().publish()` |
| 14 | [events](./events/) | ⭐ | General event counter example. | State changes, Events |
| 15 | [04-events](./04-events/) | ⭐⭐ | Structured event topics and design. | Topic indexing, Layouts |
| 16 | [11-event-filtering](./11-event-filtering/) | ⭐⭐⭐ | Complex multi-topic filters. | Advanced event queries |
| 17 | [03-custom-errors](./03-custom-errors/) | ⭐⭐ | Custom contract error enums. | `#[contracterror]` |
| 18 | [05-error-handling](./05-error-handling/) | ⭐⭐⭐ | Propagation and validation patterns. | Result, Panic vs Return |
| 19 | [03-authentication](./03-authentication/) | ⭐⭐ | Authorization with `require_auth()`. | Auth, Addresses, Roles |
| 20 | [05-auth-context](./05-auth-context/) | ⭐⭐⭐ | Cross-contract execution context. | Invoker, Contract address |
| 21 | [06-validation-patterns](./06-validation-patterns/) | ⭐⭐⭐ | Security and validation best practices. | Preconditions, State gating |

<<<<<<< HEAD

## 📋 Planned Examples

- **Iterative Mappings** - Efficient iteration over large data sets.
- **Batch Processing** - Handling multiple operations in a single call.
- **State Machine Patterns** - Structured state transitions for complex logic.
=======
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

#### [06-soroban-types](./06-soroban-types/) — 🟠 Advanced
Working with all of Soroban's built-in types in one place.
- **Concepts:** `Address`, `Bytes`, `BytesN<N>`, `Symbol`, `String`, `Vec`, `Map`, type selection guidelines, gas trade-offs
- **Best for:** Reference when choosing the right type for a use case

#### [06-type-conversions](./06-type-conversions/) — 🟠 Advanced
Safe and unsafe conversions between Soroban and Rust types.
- **Concepts:** `TryFrom`/`TryInto`, `Val` roundtrips, numeric conversions, overflow-safe casting
- **Best for:** Contracts that bridge external data or perform complex type coercions

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

#### [06-validation-patterns](./06-validation-patterns/) — 🟠 Advanced
Comprehensive input, state, and authorization validation with structured error codes.
- **Concepts:** Parameter validation (100–199), state validation (200–299), auth validation (300–399), fail-fast ordering, cooldowns, blacklists
- **Best for:** Production contracts that need defense-in-depth validation

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
# Basic Examples

Core Soroban fundamentals, one concept per example. Perfect for beginners starting their journey with Soroban smart contracts.

## 🎯 Learning Path

Follow this recommended sequence to build your understanding progressively:

```
Beginner
  │
  ├── 01-hello-world          ← Start here: contract structure basics
  ├── basic-event-emission    ← Emit your first event
  ├── 02-storage-patterns     ← Understand all three storage tiers
  ├── instance-storage        ← Deep dive: instance storage
  ├── persistent-storage      ← Deep dive: persistent storage
  ├── temporary_storage       ← Deep dive: temporary storage
  │
Intermediate
  │
  ├── 03-custom-errors        ← Structured error handling
  ├── 05-error-handling       ← Error propagation patterns
  ├── 03-authentication       ← Auth with require_auth()
  ├── 05-auth-context         ← Cross-contract auth context
  ├── 04-events               ← Structured, production-grade events
  ├── events                  ← Counter contract with events
  ├── 11-event-filtering      ← Design events for off-chain filtering
  │
Advanced
  │
  ├── 06-soroban-types        ← Full type system overview
  ├── 06-type-conversions     ← Safe type conversion patterns
  ├── 09-primitive-types      ← Integer types and overflow safety
  ├── 10-data-types           ← Comprehensive data type reference
  ├── 11-collection-types     ← Vec and Map in depth
  ├── 06-validation-patterns  ← Input, state, and auth validation
  ├── 07-enum-types           ← Enums, state machines, dispatch
  └── 08-custom-structs       ← Complex on-chain data structures
```

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
=======
- `10-cross-contract-calls`: A simple example of one contract calling another.
- `11-data-structures`: Demonstrations of `Vec` and `Map` usage.
- `12-testing-with-mocks`: How to use `mock_all_auths` and other test utilities.
>>>>>>> c564f7b (readme for each examples category)

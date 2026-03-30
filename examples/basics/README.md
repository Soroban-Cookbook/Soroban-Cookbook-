# Basic Examples

Core Soroban fundamentals, one concept per example. Perfect for beginners starting their journey with Soroban smart contracts.

## 📋 Examples

### [01-hello-world](./01-hello-world/)
The simplest possible Soroban contract — a single `hello` function.
- **Concepts:** `#[contract]`, `#[contractimpl]`, `Symbol`, `Vec`, unit tests.

---

### [02-storage-patterns](./02-storage-patterns/)
All three Soroban storage layers (persistent, instance, temporary) + TTL management.
- **Concepts:** `persistent`, `instance`, `temporary` storage; TTL extension; data keys.

---

### [03-authentication](./03-authentication/)
Address-based authorization using `require_auth()` and role management.
- **Concepts:** `require_auth()`, admin roles, balances, allowances.

---

### [03-custom-errors](./03-custom-errors/)
Custom error enums and structured error handling.
- **Concepts:** `#[contracterror]`, error codes, panic vs. graceful errors.

---

### [04-events](./04-events/)
Structured event emission with query-friendly topic layouts.
- **Concepts:** `env.events().publish()`, topic design, indexing.

---

### [05-auth-context](./05-auth-context/)
Understanding execution context in cross-contract calls.
- **Concepts:** `env.current_contract_address()`, invoker detection, proxy calls.

---

### [05-error-handling](./05-error-handling/)
Comprehensive error handling patterns and error propagation.
- **Concepts:** Error enums, contract errors, validation, event logging.

---

### [06-soroban-types](./06-soroban-types/)
Working with Soroban's built-in type system.
- **Concepts:** `Address`, `Symbol`, `Bytes`, `Map`, `Vec`, type conversions.

---

### [06-type-conversions](./06-type-conversions/)
Converting between different Soroban and Rust types.
- **Concepts:** `Into`, `From`, `TryInto`, type safety.

---

### [06-validation-patterns](./06-validation-patterns/)
Input validation, range checks, and state machine gating.
- **Concepts:** Precondition checks, overflow-safe arithmetic, state validation.

---

### [07-enum-types](./07-enum-types/)
Contract-level enumerations and their use in storage and logic.
- **Concepts:** `#[contracttype]` enums, matching, operation dispatch.

---

### [08-custom-structs](./08-custom-structs/)
Complex data structures stored on-chain.
- **Concepts:** `#[contracttype]` structs, nested types, data modeling.

---

### [09-primitive-types](./09-primitive-types/)
Integer types, overflow behaviour, and type conversions.
- **Concepts:** `u32`, `u64`, `i128`, arithmetic safety, type casting.

---

### [10-data-types](./10-data-types/)
In-depth exploration of Soroban data types.
- **Concepts:** Comprehensive type coverage and use cases.

---

### [11-collection-types](./11-collection-types/)
Working with `Vec` and `Map` collections in Soroban.
- **Concepts:** Collection operations, iteration, storage efficiency.

---

### [11-event-filtering](./11-event-filtering/)
Advanced event filtering and indexing patterns.
- **Concepts:** Multi-topic filters, efficient event retrieval.

---

### [basic-event-emission](./basic-event-emission/)
Simplified event emission for beginners.
- **Concepts:** Basic `env.events().publish()` usage.

---

### [events](./events/)
General event examples and patterns.
- **Concepts:** Event structure, naming conventions, audit logs.

---

### [instance-storage](./instance-storage/)
Deep dive into Instance storage layer.
- **Concepts:** Shared TTL, contract configuration, state management.

---

### [persistent-storage](./persistent-storage/)
Deep dive into Persistent storage layer.
- **Concepts:** Per-key TTL, user balances, long-term data.

---

### [temporary_storage](./temporary_storage/)
Deep dive into Temporary storage layer.
- **Concepts:** Short-lived caches, cost optimization, TTL management.

## 📋 Planned Examples

- **Iterative Mappings** - Efficient iteration over large data sets.
- **Batch Processing** - Handling multiple operations in a single call.
- **State Machine Patterns** - Structured state transitions for complex logic.

## 🎯 Prerequisites

Before diving into these examples, ensure you have:
- [Set up your development environment](../../guides/getting-started.md)
- [Read the Testing Guide](../../guides/testing.md)
- A basic understanding of Rust programming.

## 🧪 Running Tests

```bash
# From the root directory
cargo test -p [package-name]

# Example:
cargo test -p hello-world
```

# Basic Examples

Beginner-friendly examples that introduce core Soroban concepts one at a time.

## Examples

### [01-hello-world](./01-hello-world/)

The simplest possible Soroban contract — a single `hello` function.

**Concepts:** Contract struct, `#[contract]` / `#[contractimpl]`, symbol types, unit tests

---

### [02-storage-patterns](./02-storage-patterns/)

All three Soroban storage layers in one contract.

**Concepts:** `persistent`, `instance`, and `temporary` storage; TTL management; key types

---

### [03-authentication](./03-authentication/)

Address-based authorization using `require_auth()` with role management and balance tracking.

**Concepts:** `require_auth()`, admin roles, balances, allowances, transfer logic

---

### [03-custom-errors](./03-custom-errors/)

Custom error enums and structured error handling.

**Concepts:** `#[contracterror]`, error codes, panic vs. graceful errors, rate limiting

---

### [04-events](./04-events/)

Structured event emission with query-friendly topic layouts.

**Concepts:** `env.events().publish()`, topic design, indexed vs. non-indexed data, naming conventions

---

### [05-auth-context](./05-auth-context/)

Understanding the execution context in cross-contract calls.

**Concepts:** `env.current_contract_address()`, invoker detection, admin-only operations, proxy calls

---

### [05-error-handling](./05-error-handling/)

Comprehensive error handling patterns and error propagation.

**Concepts:** Error enums, contract errors, validation, event logging on errors

---

### [06-soroban-types](./06-soroban-types/)

Working with Soroban's built-in type system.

**Concepts:** `Address`, `Symbol`, `Bytes`, `Map`, `Vec`, type conversions

---

### [06-validation-patterns](./06-validation-patterns/)

Input validation, range checks, and state machine gating.

**Concepts:** Precondition checks, overflow-safe arithmetic, state validation

---

### [07-enum-types](./07-enum-types/)

Contract-level enumerations and their use in storage and logic.

**Concepts:** `#[contracttype]` enums, matching, role enums, operation dispatch

---

### [08-custom-structs](./08-custom-structs/)

Complex data structures stored on-chain.

**Concepts:** `#[contracttype]` structs, nested types, portfolio/user-profile patterns

---

### [09-primitive-types](./09-primitive-types/)

Integer types, overflow behaviour, and type conversions.

**Concepts:** `u32`, `u64`, `i128`, arithmetic safety, type casting

---

### [12-error-handling](./12-error-handling/)

Foundational error handling patterns using Result and panic.

**Concepts:** `#[contracterror]`, `Result<T, E>`, error codes, `try_*` client methods, invariant panics

---

## Supporting Packages

| Package | Path | Purpose |
| --- | --- | --- |
| `events_example` | [`events/`](./events/) | Minimal counter used by the integration test suite |
| `instance-storage` | [`instance-storage/`](./instance-storage/) | Focused instance storage demo |
| `persistent-storage` | [`persistent-storage/`](./persistent-storage/) | Focused persistent storage demo |
| `temporary_storage` | [`temporary_storage/`](./temporary_storage/) | Focused temporary storage demo |

## Learning Path

Follow the numbered examples in order:

1. **Hello World** — understand contract structure
2. **Storage Patterns** — persist data on-chain
3. **Authentication** — secure your contract
4. **Events** — make your contract observable
5. **Auth Context** — handle cross-contract execution safely
6. **Error Handling** — communicate failures clearly
7. **Types** — master Soroban's type system

## Running Examples

```bash
# Test a single example
cargo test -p hello-world

# Test all basic examples
cargo test --workspace

# Build WASM for deployment
cargo build -p hello-world --target wasm32-unknown-unknown --release
```

## Next Steps

- [Intermediate Examples](../intermediate/) — token interactions, multi-contract patterns
- [Advanced Examples](../advanced/) — protocols, timelocks, multi-party auth
- [DeFi / NFT / Governance](../defi/) — real-world use-case examples

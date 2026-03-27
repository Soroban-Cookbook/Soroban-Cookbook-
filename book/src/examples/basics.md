# Basic Examples

Core Soroban fundamentals, one concept per example. Perfect for beginners.

## 📋 Examples (11 total)

### [01-hello-world](../examples/basics/01-hello-world/)
**Basic contract structure.** Simplest possible contract - a `hello` function returning greeting.

**Key Concepts:**
- `#[contract]` / `#[contractimpl]` macros
- Contract client generation for testing
- Symbol types & Vec return

**Quick Code:**
```rust
pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
    vec![&env, symbol_short!(\"Hello\"), to]
}
```
**Test:** `cargo test -p hello-world`

---

### [02-storage-patterns](../examples/basics/02-storage-patterns/)
**All 3 storage types:** persistent, instance, temporary + TTL basics.

**Key Concepts:**
- Persistent (user data), Instance (config), Temporary (ephemeral)
- `has()`, `get().unwrap_or()`, `extend_ttl()`
- Typed storage keys with `#[contracttype]`

---

### [03-authentication](../examples/basics/03-authentication/)
**Authorization & roles:** `require_auth()`, admin checks, allowances.

**Key Concepts:**
- `address.require_auth()`
- Stored admin verification
- Balance transfer with validation
- Role-based access control (RBAC)

---

### [04-events](../examples/basics/04-events/)
**Event emission** with query-optimized topics.

**Key Concepts:**
- `env.events().publish((topics..), data)`
- Topic layout: `(namespace, action, [entities...])`
- Off-chain filtering patterns

---

### [05-auth-context](../examples/basics/05-auth-context/)
**Invocation context:** Who called the contract?

**Key Concepts:**
- `env.invoker().clone()` vs `env.current_contract_address()`
- Cross-contract auth propagation

---

### [06-soroban-types](../examples/basics/06-soroban-types/)
**Core types:** Address, Symbol, Map, Vec, bytes.

**Key Concepts:**
- Type conversions (`to_val()`, `try_from_val()`)
- Collection operations (push/pop)

---

### [06-validation-patterns](../examples/basics/06-validation-patterns/)
**Input validation** with checked math.

**Key Concepts:**
- `checked_add/sub/mul/div`
- Range bounds, zero checks

---

### [07-enum-types](../examples/basics/07-enum-types/)
**`#[contracttype]` enums** for state machines.

**Key Concepts:**
- Enum storage & matching
- Operation dispatch

---

### [08-custom-structs](../examples/basics/08-custom-structs/)
**Complex data** with nested `#[contracttype]` structs.

**Key Concepts:**
- User profiles, portfolios
- Tuple keys `(Status, Address)`

---

### [09-primitive-types](../examples/basics/09-primitive-types/)
**Integer handling** & overflow safety.

**Key Concepts:**
- u32/u64/i128 behaviors
- Safe casting

---

**Supporting:** events/, instance-storage/, persistent-storage/, temporary_storage/

## 🚀 Quick Start
```bash
cd examples/basics/01-hello-world
cargo test && cargo build --target wasm32-unknown-unknown --release
```

## Next: [Intermediate](../intermediate.md)

# Basic Examples

Core Soroban fundamentals, one concept per example. Perfect for beginners.

## Examples

### [01-hello-world](../examples/basics/01-hello-world/)
Basic contract structure — `#[contract]`, `#[contractimpl]`, `Env`, `Symbol`, `Vec`.

### [02-storage-patterns](../examples/basics/02-storage-patterns/)
All three storage tiers (persistent, instance, temporary) with TTL management.

**[Full Guide →](../storage-patterns.md)**

### [03-custom-errors](../examples/basics/03-custom-errors/)
Custom error enums with structured codes.

**[Full Guide →](../error-handling.md)**

### [03-authentication](../examples/basics/03-authentication/)
Authorization with `require_auth()`, admin checks, and RBAC.

### [04-events](../examples/basics/04-events/)
Structured events with multi-topic indexing.

**[Full Guide →](../events.md)**

### [05-error-handling](../examples/basics/05-error-handling/)
Result-based error handling and propagation.

### [05-auth-context](../examples/basics/05-auth-context/)
Invocation context across cross-contract call chains.

### [06-validation-patterns](../examples/basics/06-validation-patterns/)
Input validation with checked arithmetic.

### [07-type-conversions](../examples/basics/07-type-conversions/)
Safe type conversions with `TryFromVal` and `IntoVal`.

### [08-soroban-types](../examples/basics/08-soroban-types/)
Core SDK types: Address, Symbol, Map, Vec, bytes.

### [09-enum-types](../examples/basics/09-enum-types/)
Contract enums for state machines and dispatch.

### [10-custom-structs](../examples/basics/10-custom-structs/)
Nested `#[contracttype]` structs and composite storage keys.

### [11-primitive-types](../examples/basics/11-primitive-types/)
Integer handling and overflow safety.

### [12-data-types](../examples/basics/12-data-types/)
Comprehensive type system reference.

### [13-collection-types](../examples/basics/13-collection-types/)
Vec and Map collection patterns.

### [14-event-filtering](../examples/basics/14-event-filtering/)
Indexer-friendly event topic design.

## Quick Start

```bash
cd examples/basics/01-hello-world
cargo test && cargo build --target wasm32-unknown-unknown --release
```

## Interactive Playground

Want to explore every example with runnable code snippets, edit hints, and
deployment instructions all in one place?

**[→ Open the Interactive Playground](./playground.md)**

The playground page covers all 14 examples with:
- Editable code blocks showing the key contract patterns
- `cargo test` commands for each example
- What to change to trigger intentional failures (great for learning)
- Full suite test and build commands
- Testnet deployment walk-through
- Troubleshooting for the most common errors

## Next: [Intermediate](../intermediate.md)

# Basic Examples

Core Soroban fundamentals, one concept per example. Perfect for beginners.

## Examples

| # | Example | Focus |
|---|---------|-------|
| 01 | [hello-world](./01-hello-world/) | Minimal contract skeleton |
| 02 | [storage-patterns](./02-storage-patterns/) | Persistent, instance, temporary storage |
| 03 | [authentication](./03-authentication/) | `require_auth()`, RBAC |
| 03 | [custom-errors](./03-custom-errors/) | `#[contracterror]` enums |
| 04 | [events](./04-events/) | Structured event emission |
| 05 | [error-handling](./05-error-handling/) | `Result` vs panic patterns |
| 05 | [auth-context](./05-auth-context/) | Cross-contract auth context |
| 06 | [validation-patterns](./06-validation-patterns/) | Input validation, checked math |
| 07 | [type-conversions](./07-type-conversions/) | `TryFromVal`, `IntoVal` |
| 08 | [soroban-types](./08-soroban-types/) | Address, Symbol, Map, Vec |
| 09 | [enum-types](./09-enum-types/) | Contract enums, state machines |
| 10 | [custom-structs](./10-custom-structs/) | Nested `#[contracttype]` structs |
| 11 | [primitive-types](./11-primitive-types/) | Integer safety, overflow |
| 12 | [data-types](./12-data-types/) | Full type system reference |
| 13 | [collection-types](./13-collection-types/) | Vec and Map patterns |
| 14 | [event-filtering](./14-event-filtering/) | Indexer-friendly topic design |

## Quick Start

```bash
cd examples/basics/01-hello-world
cargo test && cargo build --target wasm32-unknown-unknown --release
```

## Next

- [Intermediate Examples](../intermediate/)
- [Advanced Examples](../advanced/)

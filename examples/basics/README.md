# Basic Examples

This directory contains beginner-friendly examples that introduce core Soroban concepts.

## Examples

### [01-hello-world](./01-hello-world/)

Your first Soroban contract - learn the basic structure and deployment process.

**Concepts:** Contract structure, functions, symbol types

### [02-storage-patterns](./02-storage-patterns/)

Learn how to store and retrieve data in Soroban contracts.

**Concepts:** Persistent storage, temporary storage, instance storage

### [03-authentication](./03-authentication/)

Build custom authorization logic beyond basic `require_auth()`.

**Concepts:** Role-based access control, time-locks, cooldowns, contract state gating

### [04-events](./04-events/)

Emit and handle events for off-chain monitoring.

**Concepts:** Event emission, indexing, event topics

### [05-auth-context](./05-auth-context/)

Understand the execution context, invoker vs current address, and proxy patterns.

**Concepts:** `env.invoker()`, `env.current_contract_address()`, proxy calls

### [05-error-handling](./05-error-handling/)

Proper error handling and custom error types.

**Concepts:** Error enums, panic vs graceful errors, error propagation

### [06-data-types](./06-data-types/)

Working with Soroban data types and conversions.

**Concepts:** Addresses, symbols, bytes, maps, vectors

## Getting Started

Each example includes:

- Complete source code with inline documentation
- Comprehensive unit tests
- README with deployment instructions
- Usage examples

To run any example:

```bash
cd examples/basics/[example-name]
cargo test
cargo build --target wasm32-unknown-unknown --release
```

## Learning Path

We recommend following the examples in order:

1. Start with Hello World to understand basic structure
2. Learn storage patterns for data persistence
3. Master authentication for security
4. Add events for observability
5. Learn execution context to write secure proxy and cross-contract calls
6. Handle errors gracefully
6. Explore all available data types

## Next Steps

Once comfortable with basics, explore:

- [Intermediate Examples](../intermediate/) - Token interactions, multi-contract patterns
- [Advanced Examples](../advanced/) - Complex protocols and systems
- [Use-Case Examples](../defi/) - Real-world DeFi, NFT, and governance implementations

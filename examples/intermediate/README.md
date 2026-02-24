# Intermediate Examples

Intermediate-level examples demonstrating common patterns and real-world use cases.

## Examples

### Token Interactions

- **Custom Token** - Create a custom token with minting and burning
- **Token Wrapper** - Wrap existing tokens with additional functionality
- **Multi-Token** - Handle multiple token types in a single contract

### Cross-Contract Patterns

- **Contract Factory** - Deploy contracts from within a contract
- **Proxy Pattern** - Upgradeable contract pattern
- **Registry** - Central registry for contract discovery

### Access Control

- **[Multi-Sig Patterns](./multi-sig-patterns/)** - Threshold signatures and multi-party authorization
- **Role-Based Access** - Implement RBAC (Role-Based Access Control)
- **Multi-Sig** - Multi-signature authorization
- **Timelock** - Time-delayed execution for security

### Data Structures

- **Iterables** - Implement iterable mappings
- **Queues** - FIFO queue implementation
- **Priority Queue** - Heap-based priority queue

## Prerequisites

Before diving into intermediate examples, ensure you understand:

- [Basic Examples](../basics/) - Core Soroban concepts
- Rust ownership and borrowing
- Basic blockchain concepts (addresses, signatures, transactions)

## Learning Path

1. Start with **Token Interactions** to understand asset handling
2. Explore **Cross-Contract Patterns** for complex architectures
3. Master **Access Control** for secure applications
4. Study **Data Structures** for efficient storage patterns

## Building and Testing

```bash
# Navigate to any example
cd examples/intermediate/[example-name]

# Run tests
cargo test

# Build the contract
cargo build --target wasm32-unknown-unknown --release
```

## Next Steps

Once comfortable with intermediate patterns:

- [Advanced Examples](../advanced/) - Complex systems and protocols
- [DeFi Examples](../defi/) - Decentralized finance applications
- [Governance Examples](../governance/) - DAO and voting systems

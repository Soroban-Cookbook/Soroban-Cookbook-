# Intermediate Examples

Intermediate-level examples demonstrating common patterns and real-world use cases for developers who have mastered the Soroban basics.

## 📂 Example Categories

### Access Control
Patterns for securing contracts and managing permissions.
- **[Multi-Sig Patterns](./multi-sig-patterns/)** - Threshold signatures and multi-party authorization.
- **Planned:** Role-Based Access Control (RBAC), Timelocks, Admin management.

### Token Interactions
Working with fungible and non-fungible tokens.
- **Planned:** Custom token implementation, Token wrappers, Multi-token handling.

### Cross-Contract Patterns
How contracts interact with each other.
- **Planned:** Contract Factory, Proxy patterns, Registry discovery.

### Data Structures
Efficient ways to store and manage data on-chain.
- **Planned:** Iterable mappings, Queues, Priority queues, Linked lists.

## 📋 Planned Examples

- **Custom Token** - Create a custom token with minting and burning.
- **Token Wrapper** - Wrap existing tokens with additional functionality.
- **Contract Factory** - Deploy contracts from within a contract.
- **Proxy Pattern** - Upgradeable contract pattern.
- **Registry** - Central registry for contract discovery.
- **Role-Based Access** - Implement RBAC (Role-Based Access Control).
- **Iterables** - Implement iterable mappings.

## 🎯 Prerequisites

Before diving into intermediate examples, ensure you understand:
- [Basic Examples](../basics/) - Core Soroban concepts.
- Rust ownership and borrowing.
- Basic blockchain concepts (addresses, signatures, transactions).

## 🚀 Building and Testing

```bash
# Navigate to an example
cd examples/intermediate/multi-sig-patterns

# Run tests
cargo test

# Build the contract
cargo build --target wasm32-unknown-unknown --release
```

## 📚 Learning Path

1. Start with **Access Control** to understand security patterns.
2. Explore **Token Interactions** for asset handling.
3. Master **Cross-Contract Patterns** for complex architectures.
4. Study **Data Structures** for efficient storage patterns.

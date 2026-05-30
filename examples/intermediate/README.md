# Intermediate Examples

This category contains examples that demonstrate common, real-world design patterns and use cases for Soroban smart contracts. These examples combine multiple basic concepts to solve practical problems and prepare you for production-grade contract development.

## 🎯 Prerequisites

Before diving into intermediate examples, make sure you've completed:
- [Basic Examples](../basics/) — Core Soroban concepts (storage, auth, errors, events)
- [Getting Started Guide](../../book/src/guides/getting-started.md) — Development environment setup
- [Testing Guide](../../book/src/guides/testing.md) — Unit and integration testing patterns

You should be comfortable with:
- Contract structure (`#[contract]`, `#[contractimpl]`)
- Storage types (persistent, instance, temporary)
- Authentication (`require_auth()`)
- Error handling (`#[contracterror]`, `Result<T, E>`)
- Event emission (`env.events().publish()`)

---

## 📋 All Examples

### Access Control & Authorization

#### [multi-sig-patterns](./multi-sig-patterns/) — 🟡 Intermediate
Multi-party authorization patterns including threshold signatures, proposal-based approvals, and authorization vectors.
- **Concepts:** Threshold signatures (N-of-M), proposal workflow, sequential approvals, authorization vectors, multi-sig wallets
- **Prerequisites:** [03-authentication](../basics/03-authentication/), [05-auth-context](../basics/05-auth-context/)
- **Best for:** Multi-sig wallets, DAO treasury management, joint accounts, high-value transactions
- **Related:** [Advanced Multi-Party Auth](../advanced/01-multi-party-auth/), [Governance Examples](../governance/)

---

### Cross-Contract Patterns

#### [ajo-factory](./ajo-factory/) — 🟡 Intermediate
Factory pattern for deploying versioned contract instances with template-based parameter validation.
- **Concepts:** Factory pattern, contract deployment, template registry, versioned metadata, instance tracking, parameter validation
- **Prerequisites:** [02-storage-patterns](../basics/02-storage-patterns/), [08-custom-structs](../basics/08-custom-structs/)
- **Best for:** Multi-contract systems, versioned deployments, template-based contract creation
- **Related:** [05-auth-context](../basics/05-auth-context/), [DeFi Examples](../defi/)

---

## 🏷️ Difficulty Key

| Badge | Level | Description |
|-------|-------|-------------|
| 🟡 Intermediate | Assumes basic Soroban knowledge | Combines multiple concepts, real-world patterns |
| 🟠 Advanced | Complex multi-contract systems | Production-grade patterns, optimization |

---

## 🛤️ Learning Path

### Recommended Sequence

1. **Start with Access Control**
   - [multi-sig-patterns](./multi-sig-patterns/) — Learn multi-party authorization before building complex systems

2. **Move to Cross-Contract Patterns**
   - [ajo-factory](./ajo-factory/) — Understand contract deployment and factory patterns

3. **Explore Use-Case Categories**
   - [Token Examples](../tokens/) — Token standards and wrappers
   - [DeFi Examples](../defi/) — Financial protocols and patterns
   - [NFT Examples](../nfts/) — NFT minting and marketplaces
   - [Governance Examples](../governance/) — DAO and voting systems

4. **Advance to Complex Systems**
   - [Advanced Examples](../advanced/) — Multi-party auth, timelocks, complex protocols

---

## 🧪 Running Tests

```bash
# Run a single example
cargo test -p multi-sig-patterns
cargo test -p ajo-factory

# Run all intermediate examples
cargo test --workspace

# Build contracts
cargo build --target wasm32-unknown-unknown --release
```

---

## 📚 Key Concepts by Example

| Concept | Examples |
|---------|----------|
| **Multi-Party Authorization** | [multi-sig-patterns](./multi-sig-patterns/) |
| **Factory Pattern** | [ajo-factory](./ajo-factory/) |
| **Contract Deployment** | [ajo-factory](./ajo-factory/) |
| **Template Registry** | [ajo-factory](./ajo-factory/) |
| **Threshold Signatures** | [multi-sig-patterns](./multi-sig-patterns/) |
| **Proposal Workflow** | [multi-sig-patterns](./multi-sig-patterns/) |

---

## 📋 Planned Examples

- **Role-Based Access Control (RBAC)** — Hierarchical permission management
- **Token Wrapper** — Add functionality to existing tokens
- **Upgradable Proxy** — Contract upgradability patterns
- **Registry Pattern** — Contract discovery and lookup
- **Iterable Data Structures** — Efficient iteration over large datasets
- **Queue & Priority Queue** — FIFO and priority-based processing

---

## 🔗 Related Resources

### Documentation
- [Best Practices](../../docs/best-practices.md) — Security and code quality guidelines
- [Style Guide](../../docs/style-guide.md) — Naming and documentation standards
- [Deployment Guide](../../book/src/guides/deployment.md) — Deploy to testnet and mainnet
- [Ethereum to Soroban](../../book/src/guides/ethereum-to-soroban.md) — Solidity pattern translation

### External Links
- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Soroban Authorization](https://developers.stellar.org/docs/smart-contracts/fundamentals-and-concepts/authorization)
- [Multi-Signature Wallets](https://developers.stellar.org/docs/smart-contracts/example-contracts/multi-sig)
- [Soroban SDK](https://docs.rs/soroban-sdk/latest/soroban_sdk/)

---

## ➡️ What's Next

Once you're comfortable with intermediate patterns:
- [Advanced Examples](../advanced/) — Complex multi-party systems, timelocks, advanced DeFi
- [DeFi Examples](../defi/) — AMMs, lending pools, vaults, yield protocols
- [Governance Examples](../governance/) — DAO governance, voting, proposals
- [Contributing Guide](../../CONTRIBUTING.md) — Add your own examples to the cookbook

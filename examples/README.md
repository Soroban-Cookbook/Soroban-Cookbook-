
# Soroban Cookbook Examples

A comprehensive collection of Soroban smart contract examples, ranging from basic fundamentals to complex DeFi protocols and governance systems.

## 📂 Example Categories

### [Basics](./basics/)
**Beginner-friendly examples** that introduce core Soroban concepts one at a time. Essential for anyone starting with Soroban development.
- **Key Topics:** Storage, Authentication, Events, Error Handling, Data Types.
- **Status:** Active development with many completed examples.

### [Intermediate](./intermediate/)
**Common patterns and real-world use cases** for developers who have mastered the basics.
- **Key Topics:** Multi-sig patterns, Token interactions, Access Control, Data Structures.
- **Status:** Several patterns implemented, more planned.

### [Advanced](./advanced/)
**Complex architectural designs and optimizations** for production-grade systems.
- **Key Topics:** Multi-party auth, Timelocks, Factories, Bonding Curves.
- **Status:** Advanced patterns being added incrementally.

### [DeFi](./defi/)
**Decentralized Finance protocol implementations** including AMMs, lending, and yield vaults.
- **Key Topics:** Constant product pools, collateralized loans, oracle integration.
- **Status:** 📋 Planned and coming soon.

### [Tokens](./tokens/)
**Fungible and semi-fungible token standards** and utilities.
- **Key Topics:** SEP-41 compliance, Mintable/Burnable tokens, Wrappers.
- **Status:** 📋 Planned and coming soon.

### [NFTs](./nfts/)
**Non-Fungible Token patterns** and marketplace implementations.
- **Key Topics:** Minting, metadata standards, royalty enforcement, auctions.
- **Status:** 📋 Planned and coming soon.

### [Governance](./governance/)
**Decentralized governance systems** and DAO frameworks.
- **Key Topics:** Voting systems, treasury management, proposal lifecycles.
- **Status:** 📋 Planned and coming soon.

## 🚀 How to Use These Examples

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Stellar-Cookbook/Soroban-Cookbook.git
   ```

2. **Navigate to an example:**
   ```bash
   cd examples/basics/01-hello-world
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Build the contract:**
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

## 🤝 Contributing

We welcome new examples! If you have a pattern or use case you'd like to share, please check our [CONTRIBUTING.md](../CONTRIBUTING.md) guide.

# Soroban Examples Index

A collection of Soroban smart contract examples organized by complexity and use case.

## 📂 Category Overview

### [Basics](./basics/)
Beginner-friendly examples that introduce core Soroban concepts one at a time.
- Hello World, Storage, Auth, Events, Errors, Types, Validation

### [Intermediate](./intermediate/)
Common patterns and more complex contract interactions.
- Multi-sig, Cross-contract calls, Custom types, Advanced storage

### [Advanced](./advanced/)
Complex systems and architectural patterns.
- Timelocks, Multi-party auth, Proxy patterns, Upgradeability

### [DeFi](./defi/)
Financial application examples and protocols.
- Automated Market Makers (AMMs), Lending, Staking, Yield farming

### [NFTs](./nfts/)
Non-fungible token implementations and patterns.
- Minting, Transfers, Metadata, Royalties

### [Governance](./governance/)
DAO and voting system examples.
- Voting, Proposals, Multi-sig governance

### [Tokens](./tokens/)
Token standards and custom token implementations.
- SEP-41, Custom tokens, Wrapper tokens

---

#### 🚀 Getting Started

To run any of these examples:

```bash
# Navigate to an example (relative to repo root)
cd examples/basics/01-hello-world

# Run tests
cargo test
```

# Build WASM
cargo build --target wasm32-unknown-unknown --release
```

## 🛠️ Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable, 1.74+)
- `wasm32-unknown-unknown` target
- [Soroban / Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli)


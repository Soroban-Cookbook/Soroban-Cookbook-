# Soroban Cookbook

**A comprehensive guide to building smart contracts on Stellar with Soroban**

## 📖 About

The Soroban Cookbook is a developer's guide to building smart contracts on the Stellar network using Soroban. This repository provides clear, well-documented examples and practical patterns for developers at every level—from your first "Hello World" contract to complex DeFi protocols.

## 🎯 What You'll Find Here

### 📚 Examples by Difficulty

- **[Basics](./examples/basics/)** - Core concepts: storage, auth, events, and data types
- **[Intermediate](./examples/intermediate/)** - Tokens, NFTs, multi-contract interactions
- **[Advanced](./examples/advanced/)** - DeFi protocols, governance systems, cross-chain patterns

### 🏗️ Examples by Use Case

- **[DeFi](./examples/defi/)** - AMMs, lending, vaults, escrow, and yield protocols
- **[NFTs](./examples/nfts/)** - Minting, marketplaces, and metadata standards
- **[Governance](./examples/governance/)** - DAOs, voting systems, and proposals
- **[Tokens](./examples/tokens/)** - Custom tokens, wrappers, and token standards

### 📝 Comprehensive Guides

- **[Getting Started](./guides/getting-started.md)** - Set up your development environment
- **[Your First Contract](./guides/first-contract.md)** - Write, test, and deploy
- **[Testing Guide](./guides/testing.md)** - Unit tests, integration tests, and best practices
- **[Deployment Guide](./guides/deployment.md)** - Deploy to testnet and mainnet
- **[Migrating from Ethereum](./guides/ethereum-to-soroban.md)** - Solidity → Rust patterns

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook.git
cd Soroban-Cookbook

# Install Rust and Soroban CLI (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --locked soroban-cli

# Try a basic example
cd examples/basics/01-hello-world
cargo test
soroban contract build

# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
  --network testnet
```

## 📂 Repository Structure

```
Soroban-Cookbook/
├── examples/           # Smart contract examples
│   ├── basics/        # Beginner-friendly fundamentals
│   ├── intermediate/  # Common patterns and use cases
│   ├── advanced/      # Complex systems and protocols
│   ├── defi/          # DeFi-specific examples
│   ├── nfts/          # NFT implementations
│   ├── governance/    # DAO and voting systems
│   └── tokens/        # Token standards and patterns
├── guides/            # Step-by-step tutorials
├── docs/              # Reference documentation
├── scripts/           # Deployment and testing utilities
└── .github/           # CI/CD and templates
```

## 🛠️ Technical Standards

Every example in this cookbook:

- ✅ Compiles with the latest stable Soroban SDK
- ✅ Includes comprehensive unit and integration tests
- ✅ Features inline documentation explaining key concepts
- ✅ Provides deployment scripts for testnet/mainnet
- ✅ Follows Rust and Soroban best practices
- ✅ Passes automated CI/CD checks

## 🤝 Contributing

We welcome contributions from the community! Whether you're fixing a typo, improving documentation, or adding a new example, your help makes this resource better for everyone.

**Ways to contribute:**

- 📝 Add new contract examples or patterns
- 📖 Improve documentation and guides
- 🐛 Report bugs or suggest improvements
- ✅ Review pull requests
- 🌍 Translate content to other languages

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines.

## 📚 Additional Resources

- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Stellar Developer Portal](https://developers.stellar.org)
- [Soroban Rust SDK](https://github.com/stellar/rs-soroban-sdk)
- [Stellar Community Discord](https://discord.gg/stellardev)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

---

**Built by the community • Powered by Stellar • Written in Rust**

_Have a suggestion or found an issue? [Open an issue](https://github.com/Soroban-Cookbook/Soroban-Cookbook/issues/new)_

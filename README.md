# Soroban Cookbook

**A comprehensive guide to building smart contracts on Stellar with Soroban**

[![CI](https://github.com/Soroban-Cookbook/Soroban-Cookbook/actions/workflows/ci.yml/badge.svg)](https://github.com/Soroban-Cookbook/Soroban-Cookbook/actions/workflows/ci.yml)
[![Test and Lint](https://github.com/Soroban-Cookbook/Soroban-Cookbook/actions/workflows/test.yml/badge.svg)](https://github.com/Soroban-Cookbook/Soroban-Cookbook/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/Soroban-Cookbook/Soroban-Cookbook/branch/main/graph/badge.svg)](https://codecov.io/gh/Soroban-Cookbook/Soroban-Cookbook)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Table of Contents

- [About](#about)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Repository Structure](#repository-structure)
- [Examples](#examples)
- [Guides](#guides)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [Additional Resources](#additional-resources)
- [License](#license)

## About

The Soroban Cookbook is a community-driven developer resource for building smart contracts on the [Stellar](https://stellar.org) network using [Soroban](https://developers.stellar.org/docs/smart-contracts). It provides clear, well-documented examples and practical patterns for developers at every level — from a first "Hello World" contract to production-grade DeFi protocols.

Every example in this cookbook:

- Compiles with the latest stable Soroban SDK
- Includes comprehensive unit and integration tests
- Features inline documentation explaining key concepts
- Follows Rust and Soroban best practices
- Passes all automated CI/CD checks

## Quick Start

```bash
# Clone the repository
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git
cd Soroban-Cookbook

# Run a basic example
cd examples/basics/01-hello-world
cargo test

# Build the contract as WASM
cargo build --target wasm32-unknown-unknown --release
```

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, 1.74+)
- [Soroban / Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli)

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version
```

### 2. Add the WASM target

Soroban contracts compile to WebAssembly:

```bash
rustup target add wasm32-unknown-unknown
```

### 3. Install Stellar CLI

```bash
cargo install --locked stellar-cli --version 22.1.0
stellar --version
```

### 4. Clone and verify

```bash
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook.git
cd Soroban-Cookbook
cargo test --workspace
```

## Repository Structure

```
Soroban-Cookbook/
├── examples/               # Smart contract examples
│   ├── basics/             # Beginner-friendly fundamentals
│   ├── intermediate/       # Common patterns and use cases
│   ├── advanced/           # Complex systems and protocols
│   ├── defi/               # DeFi-specific examples
│   ├── nfts/               # NFT implementations
│   ├── governance/         # DAO and voting systems
│   └── tokens/             # Token standards and patterns
├── book/                   # mdBook documentation source
│   └── src/
│       ├── guides/         # Step-by-step tutorials
│       ├── examples/       # Example write-ups
│       └── docs/           # Reference documentation
├── docs/                   # Supplementary reference docs
└── .github/                # CI/CD workflows and templates
```

## Examples

### By Difficulty

#### [Basics](./examples/basics/)

Core Soroban concepts, one at a time.

| Example | Concepts |
| --- | --- |
| [01-hello-world](./examples/basics/01-hello-world/) | Contract struct, `#[contract]` / `#[contractimpl]`, unit tests |
| [02-storage-patterns](./examples/basics/02-storage-patterns/) | `persistent`, `instance`, `temporary` storage, TTL |
| [03-authentication](./examples/basics/03-authentication/) | `require_auth()`, admin roles, balances |
| [03-custom-errors](./examples/basics/03-custom-errors/) | `#[contracterror]`, error codes, rate limiting |
| [04-events](./examples/basics/04-events/) | `env.events().publish()`, topic design |
| [05-auth-context](./examples/basics/05-auth-context/) | Cross-contract execution context |
| [05-error-handling](./examples/basics/05-error-handling/) | Error enums, validation, propagation |
| [06-soroban-types](./examples/basics/06-soroban-types/) | `Address`, `Symbol`, `Bytes`, `Map`, `Vec` |
| [06-validation-patterns](./examples/basics/06-validation-patterns/) | Precondition checks, overflow-safe arithmetic |
| [07-enum-types](./examples/basics/07-enum-types/) | `#[contracttype]` enums, role dispatch |
| [08-custom-structs](./examples/basics/08-custom-structs/) | `#[contracttype]` structs, nested types |
| [09-primitive-types](./examples/basics/09-primitive-types/) | `u32`, `u64`, `i128`, arithmetic safety |

#### [Intermediate](./examples/intermediate/)

Common patterns and real-world use cases.

- Token interactions and wrappers
- Cross-contract patterns (factory, proxy, registry)
- Access control: [multi-sig patterns](./examples/intermediate/multi-sig-patterns/), RBAC, timelocks
- Data structures: iterables, queues, priority queues

#### [Advanced](./examples/advanced/)

Complex systems for experienced developers.

| Example | Concepts |
| --- | --- |
| [01-multi-party-auth](./examples/advanced/01-multi-party-auth/) | Threshold signatures, multi-party authorization |
| [02-timelock](./examples/advanced/02-timelock/) | Time-delayed execution, queue/cancel/execute |

### By Use Case

| Category | Description |
| --- | --- |
| [DeFi](./examples/defi/) | AMMs, lending pools, vaults, escrow, yield protocols |
| [NFTs](./examples/nfts/) | Minting, marketplaces, metadata standards |
| [Governance](./examples/governance/) | DAOs, voting systems, proposals |
| [Tokens](./examples/tokens/) | SEP-41 tokens, wrappers, vesting, airdrops |

## Guides

Step-by-step tutorials in the [book](./book/src/guides/):

| Guide | Description |
| --- | --- |
| [Getting Started](./book/src/guides/getting-started.md) | Set up your development environment |
| [Testing](./book/src/guides/testing.md) | Unit tests, integration tests, best practices |
| [Deployment](./book/src/guides/deployment.md) | Deploy to testnet and mainnet |
| [Ethereum to Soroban](./book/src/guides/ethereum-to-soroban.md) | Solidity → Rust pattern translation |

## Documentation

Reference docs in [docs/](./docs/):

- [Best Practices](./docs/best-practices.md) — Security, storage, and code quality guidelines
- [Performance Benchmarks](./docs/benchmarks.md) — Resource usage comparison and optimization tips
- [Quick Reference](./docs/quick-reference.md) — Cheat sheet for common patterns
- [Glossary](./docs/glossary.md) — Key terms and concepts

The full documentation site is built with [mdBook](https://rust-lang.github.io/mdBook/) and deployed to GitHub Pages on every push to `main`.

## Contributing

Contributions are welcome. Whether you're fixing a typo, improving docs, or adding a new example — see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines. Please also read our [Code of Conduct](./CODE_OF_CONDUCT.md).

Ways to contribute:

- Add new contract examples or patterns
- Improve documentation and guides
- Report bugs or suggest improvements
- Review pull requests

Before submitting a PR, make sure your changes pass the local checks:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --target wasm32-unknown-unknown --release
```

## Additional Resources

- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Stellar Developer Portal](https://developers.stellar.org)
- [Soroban Rust SDK](https://github.com/stellar/rs-soroban-sdk)
- [Stellar Community Discord](https://discord.gg/stellardev)
- [Project Roadmap](./ROADMAP.md) - Planned phases, milestones, and KPIs

## License

This project is licensed under the MIT License — see the [LICENSE](./LICENSE) file for details.

---

Built by the community · Powered by Stellar · Written in Rust

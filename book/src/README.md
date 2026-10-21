# Soroban Cookbook

**A comprehensive guide to building smart contracts on Stellar with Soroban**

## About

The Soroban Cookbook is a developer's guide to building smart contracts on the Stellar network using Soroban. This documentation provides clear, well-documented examples and practical patterns for developers at every level — from your first "Hello World" contract to complex DeFi protocols.

## Quick Start

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --version 22.1.0

# Clone the repository
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git
cd Soroban-Cookbook-

# Run a basic example
cd examples/basics/01-hello-world
cargo test
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
├── tests/                  # Integration and security tests
└── scripts/                # Build and deployment scripts
```

## Examples by Difficulty

### Basics

Core Soroban concepts, one at a time.

| Example | Concepts |
| --- | --- |
| [01-hello-world](../examples/basics/01-hello-world/) | Contract struct, `#[contract]` / `#[contractimpl]`, unit tests |
| [02-storage-patterns](../examples/basics/02-storage-patterns/) | `persistent`, `instance`, `temporary` storage, TTL |
| [03-authentication](../examples/basics/03-authentication/) | `require_auth()`, admin roles, balances |
| [03-custom-errors](../examples/basics/03-custom-errors/) | `#[contracterror]`, error codes |
| [04-events](../examples/basics/04-events/) | `env.events().publish()`, topic design |
| [05-error-handling](../examples/basics/05-error-handling/) | Error enums, validation, propagation |
| [06-validation-patterns](../examples/basics/06-validation-patterns/) | Precondition checks, overflow-safe arithmetic |
| [07-type-conversions](../examples/basics/07-type-conversions/) | `TryFromVal`, `IntoVal`, safe narrowing |
| [08-soroban-types](../examples/basics/08-soroban-types/) | `Address`, `Symbol`, `Bytes`, `Map`, `Vec` |
| [09-enum-types](../examples/basics/09-enum-types/) | `#[contracttype]` enums, role dispatch |
| [10-custom-structs](../examples/basics/10-custom-structs/) | `#[contracttype]` structs, nested types |
| [11-primitive-types](../examples/basics/11-primitive-types/) | `u32`, `u64`, `i128`, arithmetic safety |
| [12-data-types](../examples/basics/12-data-types/) | Full type system reference |
| [13-collection-types](../examples/basics/13-collection-types/) | `Vec`, `Map` collection patterns |

### Intermediate

Common patterns and real-world use cases.

- Cross-contract patterns: factory, proxy, registry
- Access control: multi-sig patterns, RBAC, timelocks
- Token interactions and wrappers

### Advanced

Complex systems for experienced developers.

| Example | Concepts |
| --- | --- |
| [01-multi-party-auth](../examples/advanced/01-multi-party-auth/) | Threshold signatures, multi-party authorization |
| [02-timelock](../examples/advanced/02-timelock/) | Time-delayed execution, queue/cancel/execute |

## Examples by Use Case

| Category | Description |
| --- | --- |
| DeFi | AMMs, lending pools, vaults, escrow, yield protocols |
| NFTs | Minting, marketplaces, metadata standards |
| Governance | DAOs, voting systems, proposals |
| Tokens | SEP-41 tokens, wrappers, vesting, airdrops |

## Guides

| Guide | Description |
| --- | --- |
| [Getting Started](./guides/getting-started.md) | Set up your development environment |
| [Testing](./guides/testing.md) | Unit tests, integration tests, best practices |
| [Deployment](./guides/deployment.md) | Deploy to testnet and mainnet |
| [Local Simulation](./guides/local-simulation.md) | Run contracts locally with the Stellar CLI |
| [Ethereum to Soroban](./guides/ethereum-to-soroban.md) | Solidity → Rust pattern translation |

## API Reference

| Document | Description |
| --- | --- |
| [Quick Reference](./docs/quick-reference.md) | Cheat sheet for common patterns |
| [Best Practices](./docs/best-practices.md) | Security, storage, and code quality guidelines |
| [Common Pitfalls](./docs/common-pitfalls.md) | Mistakes to avoid |
| [Glossary](./docs/glossary.md) | Key terms and concepts |
| [Troubleshooting](./docs/troubleshooting.md) | Common build, test, and deployment issues |

## Code Quality Standards

Every example in this cookbook:

- Compiles successfully using the latest stable Soroban SDK
- Contains comprehensive unit and integration tests
- Enforces strict security boundaries and Rust/Soroban best practices
- Passes all CI/CD pipelines including formatting, Clippy, and test suites

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines and [Community Guidelines](./community-guidelines.md) for participation norms. Please also read our [Code of Conduct](./CODE_OF_CONDUCT.md).

## Additional Resources

- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Stellar Developer Portal](https://developers.stellar.org)
- [Soroban Rust SDK](https://github.com/stellar/rs-soroban-sdk)
- [Stellar Community Discord](https://discord.gg/stellardev)

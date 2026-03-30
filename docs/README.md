# Documentation Index

Welcome to the Soroban Cookbook documentation. Use this index to find what you need quickly.

## Quick Links

| I want to…                  | Go to                                                         |
| --------------------------- | ------------------------------------------------------------- |
| Set up my environment       | [Getting Started](../book/src/guides/getting-started.md)     |
| Write my first contract     | [Hello World](../examples/basics/01-hello-world/)            |
| Learn to test contracts     | [Testing Guide](../book/src/guides/testing.md)               |
| Deploy to testnet           | [Deployment Guide](../book/src/guides/deployment.md)         |
| Migrate from Ethereum       | [Ethereum → Soroban](../book/src/guides/ethereum-to-soroban.md) |
| Look up a term              | [Glossary](./glossary.md)                                    |
| See common patterns         | [Common Patterns](./common-patterns.md)                      |
| Check best practices        | [Best Practices](./best-practices.md)                        |
| Get a cheat sheet           | [Quick Reference](./quick-reference.md)                      |

## Reference Documentation

- [Best Practices](./best-practices.md) — Security, storage, and code quality guidelines
- [Quick Reference](./quick-reference.md) — Cheat sheet for common Soroban patterns
- [Common Patterns](./common-patterns.md) — Reusable patterns with when-to-use guidance
- [Glossary](./glossary.md) — Key terms and concepts

### Architecture Decision Records

- [ADR-001: Record Architecture Decisions](./adr/001-record-architecture-decisions.md)
- [ADR Template](./adr/template.md)

## Guides

Step-by-step tutorials in [`book/src/guides/`](../book/src/guides/):

1. [Getting Started](../book/src/guides/getting-started.md) — Environment setup
2. [Testing](../book/src/guides/testing.md) — Unit and integration tests
3. [Deployment](../book/src/guides/deployment.md) — Testnet and mainnet deployment
4. [Ethereum to Soroban](../book/src/guides/ethereum-to-soroban.md) — Solidity → Rust translation

## Examples by Category

### By Difficulty

| Level        | Directory                          | Description                  |
| ------------ | ---------------------------------- | ---------------------------- |
| Basics       | [examples/basics/](../examples/basics/)           | Core concepts, one at a time |
| Intermediate | [examples/intermediate/](../examples/intermediate/) | Common patterns and use cases |
| Advanced     | [examples/advanced/](../examples/advanced/)       | Complex systems              |

### By Use Case

| Category   | Directory                              | Description                  |
| ---------- | -------------------------------------- | ---------------------------- |
| DeFi       | [examples/defi/](../examples/defi/)           | AMMs, lending, vaults        |
| NFTs       | [examples/nfts/](../examples/nfts/)           | Minting, marketplaces        |
| Governance | [examples/governance/](../examples/governance/) | DAOs, voting, proposals      |
| Tokens     | [examples/tokens/](../examples/tokens/)       | SEP-41, vesting, airdrops    |

### Basics Examples

| Example | Concepts |
| ------- | -------- |
| [01-hello-world](../examples/basics/01-hello-world/) | Contract struct, `#[contract]`, `#[contractimpl]`, unit tests |
| [02-storage-patterns](../examples/basics/02-storage-patterns/) | `persistent`, `instance`, `temporary` storage, TTL |
| [03-authentication](../examples/basics/03-authentication/) | `require_auth()`, admin roles |
| [03-custom-errors](../examples/basics/03-custom-errors/) | `#[contracterror]`, error codes |
| [04-events](../examples/basics/04-events/) | `env.events().publish()`, topic design |
| [05-auth-context](../examples/basics/05-auth-context/) | Cross-contract execution context |
| [05-error-handling](../examples/basics/05-error-handling/) | Error enums, validation, propagation |
| [06-soroban-types](../examples/basics/06-soroban-types/) | `Address`, `Symbol`, `Bytes`, `Map`, `Vec` |
| [06-validation-patterns](../examples/basics/06-validation-patterns/) | Precondition checks, overflow-safe arithmetic |
| [07-enum-types](../examples/basics/07-enum-types/) | `#[contracttype]` enums, role dispatch |
| [08-custom-structs](../examples/basics/08-custom-structs/) | `#[contracttype]` structs, nested types |
| [09-primitive-types](../examples/basics/09-primitive-types/) | `u32`, `u64`, `i128`, arithmetic safety |

## External Resources

- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Soroban Rust SDK](https://docs.rs/soroban-sdk/)
- [Stellar Developer Portal](https://developers.stellar.org/)
- [Stellar Discord](https://discord.gg/stellardev)
- [Soroban Quest](https://quest.stellar.org/)

## Search Tips

- **Looking for a pattern?** Check [Common Patterns](./common-patterns.md) or browse [`examples/`](../examples/) by difficulty.
- **Unfamiliar term?** See the [Glossary](./glossary.md).
- **Migrating from Solidity?** The [Ethereum to Soroban guide](../book/src/guides/ethereum-to-soroban.md) maps common patterns directly.
- **Can't find it here?** Search the repository or ask in [Stellar Discord](https://discord.gg/stellardev).

---

Missing something? [Open an issue](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/new) or [submit a PR](../CONTRIBUTING.md).

# Documentation Index

Welcome to the Soroban Cookbook documentation. This page gathers quick links, reference material, and pointers to guides and examples.

## Quick Links.

| I want to…                  | Go to                                                         |
| --------------------------- | ------------------------------------------------------------- |
| Set up my environment       | [Getting Started](../book/src/guides/getting-started.md)     |
| Write my first contract     | [Hello World](../examples/basics/01-hello-world/)            |
| Learn to test contracts     | [Testing Guide](../book/src/guides/testing.md)               |
| Deploy to testnet           | [Deployment Guide](../book/src/guides/deployment.md)         |
| Migrate from Ethereum       | [Ethereum → Soroban](../book/src/guides/ethereum-to-soroban.md) |
| Fix a build or test error   | [Troubleshooting](./troubleshooting.md)                      |
| Look up a term              | [Glossary](./glossary.md)                                    |
| See common patterns         | [Common Patterns](./common-patterns.md)                      |
| Design multi-contract apps  | [Factory, Proxy, and Registry Patterns](./cross-contract-patterns.md) |
| Check best practices        | [Best Practices](./best-practices.md)                        |
| Follow style guide          | [Style Guide](./style-guide.md)                              |
| Get a cheat sheet           | [Quick Reference](./quick-reference.md)                      |
| Understand token design     | [Token Patterns](./token-patterns.md)                        |
| Compare gas costs           | [Gas Benchmarks](./gas-benchmarks.md)                        |
| Compare example gas costs   | [Gas Benchmarks](./gas-benchmarks.md)                      |

## Reference Documentation

- [Best Practices](./best-practices.md) — Security, storage, and code quality guidelines
- [Style Guide](./style-guide.md) — Naming, documentation, and testing standards
- [Quick Reference](./quick-reference.md) — Cheat sheet for common Soroban patterns
- [Common Patterns](./common-patterns.md) — Reusable patterns with when-to-use guidance
- [Token Patterns](./token-patterns.md) — Metadata, mint/burn, wrapping, and access control for tokens
- [Gas Benchmarks](./gas-benchmarks.md) — CPU and memory cost comparison across examples
- [Governance & Authorization Patterns](./governance-rbac-multisig-timelock.md) — RBAC, multisig, and timelock guidance for secure deployments
- [Glossary](./glossary.md) — Key terms and concepts
- [Troubleshooting](./troubleshooting.md) — Build errors, test failures, deployment issues, and workarounds
- [Dependabot Setup](./dependabot-setup.md) — Automated dependency update configuration
- [Performance Benchmarks](./benchmarks.md) — Resource usage comparison and optimization tips
- [Security Audit Preparation](./security-audit/README.md) — Audit scope, prep checklist, and known-issues log for the intermediate examples

### 🎬 Video Walkthrough

> **Getting Started — Examples 01–03** *(coming soon)*
> A 10–15 minute video covering Hello World, Storage Patterns, and Custom Errors.
> Once published it will be linked here and in each example's README.
> See [`docs/video-script-getting-started.md`](./video-script-getting-started.md) for the full script and YouTube metadata.

## 🎓 Learning Paths

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

- **Token Operations**
  - [Token Basics](../examples/tokens/)
  - [Token Wrapper](../examples/intermediate/)
  - [Custom Token](../examples/tokens/)

- **Storage & Data**
  - [Storage Types](./storage-types.md) - Comparison and usage
  - [Basic Storage Patterns](../examples/basics/02-storage-patterns/)
  - [Detailed Instance Storage](../examples/basics/instance-storage/)
  - [Data Structures](../examples/intermediate/)
  - [Efficient Storage](../examples/advanced/)

- **Financial Operations**
  - [Simple Swap](../examples/defi/)
  - [Lending Pool](../examples/defi/)
  - [AMM](../examples/defi/)
  - [Yield Vault](../examples/defi/)

## 🛠️ Tools & Utilities

### Development Scripts

- **[build.sh](../scripts/build.sh)** - Build contracts
- **[test.sh](../scripts/test.sh)** - Run tests with options
- **[deploy.sh](../scripts/deploy.sh)** - Deploy to networks

### Testing Tools

- Mock environments
- Authorization mocking
- Time manipulation
- Event inspection

### Deployment Tools

- Network configuration
- Identity management
- Fee estimation
- Contract verification

## 📖 Guides & Tutorials

### Step-by-Step Guides

1. [Getting Started](../guides/getting-started.md)
2. [Testing Guide](../guides/testing.md)
3. [Deployment Guide](../guides/deployment.md)
4. [Ethereum to Soroban](../guides/ethereum-to-soroban.md)

### Topic-Specific Guides

- Storage optimization
- Gas efficiency
- Security auditing
- Upgrade patterns

## 🤝 Community Resources

### Getting Help

- [Stellar Discord](https://discord.gg/stellardev) - Live chat
- [Stack Exchange](https://stellar.stackexchange.com/) - Q&A
- [GitHub Discussions](https://github.com/Soroban-Cookbook/Soroban-Cookbook/discussions)
- [Forum](https://stellar.org/community)

### Contributing

- [Contributing Guide](../CONTRIBUTING.md) - How to contribute
- [Code of Conduct](../CODE_OF_CONDUCT.md) - Community guidelines
- [Issue Templates](../.github/ISSUE_TEMPLATE/) - Report issues
- [Pull Request Template](../.github/PULL_REQUEST_TEMPLATE.md)

## 🔗 External Resources

### Official Documentation

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

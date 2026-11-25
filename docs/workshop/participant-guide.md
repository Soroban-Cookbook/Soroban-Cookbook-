# Participant Guide: Soroban Development Workshop

This guide helps you prepare for and work through the Soroban Development Workshop.

## Before You Arrive

Complete these steps before the workshop starts to maximize hands-on time.

### Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Verify:

```bash
rustc --version    # Should show 1.74 or later
cargo --version
```

If your version is older:

```bash
rustup update stable
```

### Step 2: Add the WebAssembly Target

```bash
rustup target add wasm32-unknown-unknown
```

Verify:

```bash
rustup target list --installed | grep wasm32
# Should output: wasm32-unknown-unknown
```

### Step 3: Install the Stellar CLI

```bash
cargo install --locked stellar-cli --features opt
```

Verify:

```bash
stellar --version
stellar contract --help
```

> If you have the older `soroban-cli` installed, remove it first: `cargo uninstall soroban-cli`

### Step 4: Install a Code Editor

**VS Code (recommended):**

1. Install [VS Code](https://code.visualstudio.com/)
2. Install the [rust-analyzer extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
3. Optionally install the [Marp extension](https://marketplace.visualstudio.com/items?itemName=marp-team.marp-vscode) for viewing slides

**JetBrains (IntelliJ / CLion / RustRover):**

1. Install the [Rust plugin](https://plugins.jetbrains.com/plugin/8182-rust)

### Step 5: Clone the Repository

```bash
git clone https://github.com/Stellar-Cookbook/Soroban-Cookbook.git
cd Soroban-Cookbook
```

### Step 6: Verify the Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

This may take several minutes the first time. If it succeeds, you're ready.

---

## Workshop Agenda

### Day 1

| Time | Module | Description |
|------|--------|-------------|
| Morning | Module 1 | Introduction & Environment Setup |
| Morning | Module 2 | Storage & Data Types |
| Afternoon | Module 3 | Authentication & Access Control |

### Day 2

| Time | Module | Description |
|------|--------|-------------|
| Morning | Module 4 | Tokens & SEP-41 Standard |
| Afternoon | Module 5 | Advanced Patterns & Deployment |

---

## Module Walk-Through

### Module 1: Introduction & Environment Setup

**Slides:** [`slides/module-1-introduction.md`](slides/module-1-introduction.md)
**Exercise:** [`exercises/module-1-exercise.md`](exercises/module-1-exercise.md)

You will:
- Verify your development environment
- Run the Hello World contract tests
- Build the contract to WASM
- Modify the contract to add a new function

Key concepts:
- `#[contract]` and `#[contractimpl]` macros
- `Env` type for accessing storage and events
- `cargo test` for native testing
- `cargo build --target wasm32-unknown-unknown` for WASM compilation

### Module 2: Storage & Data Types

**Slides:** [`slides/module-2-storage.md`](slides/module-2-storage.md)
**Exercise:** [`exercises/module-2-exercise.md`](exercises/module-2-exercise.md)

You will:
- Study the three storage types (persistent, temporary, instance)
- Learn `#[contracttype]` for custom structs and enums
- Build a counter contract with events and error handling

Key concepts:
- `env.storage().persistent()` — permanent data with TTL
- `env.storage().temporary()` — auto-expiring data
- `env.storage().instance()` — contract-level config
- `#[contracterror]` for typed errors
- `env.events().publish()` for state-change notifications

### Module 3: Authentication & Access Control

**Slides:** [`slides/module-3-auth.md`](slides/module-3-auth.md)
**Exercise:** [`exercises/module-3-exercise.md`](exercises/module-3-exercise.md)

You will:
- Learn the `require_auth()` pattern
- Implement role-based access control (RBAC)
- Add pause/unpause functionality
- Test authorized and unauthorized paths

Key concepts:
- `address.require_auth()` — host-enforced authorization
- Role hierarchy (Owner > Admin > Operator > User)
- Pausable pattern with `require_not_paused()` guard
- `env.mock_auths()` for testing auth in unit tests

### Module 4: Tokens & SEP-41 Standard

**Slides:** [`slides/module-4-tokens.md`](slides/module-4-tokens.md)
**Exercise:** [`exercises/module-4-exercise.md`](exercises/module-4-exercise.md)

You will:
- Study the SEP-41 token interface
- Implement transfer, approve, and transfer_from
- Add mint and burn functionality
- Make the token pausable

Key concepts:
- SEP-41 interface functions (transfer, approve, transfer_from, balance, allowance)
- `i128` for token amounts with `checked_add`/`checked_sub`
- Allowance pattern for delegated spending
- Event publishing for transfers and approvals
- Balance storage in persistent, metadata in instance

### Module 5: Advanced Patterns & Deployment

**Slides:** [`slides/module-5-advanced.md`](slides/module-5-advanced.md)
**Exercise:** [`exercises/module-5-exercise.md`](exercises/module-5-exercise.md)

You will:
- Study proxy, oracle, and timelock patterns
- Build a simple price oracle with staleness checks
- Deploy a contract to Stellar testnet
- Interact with the deployed contract via CLI

Key concepts:
- Upgradeable proxy pattern
- Oracle with `max_age` freshness checks
- Timelock for delayed execution
- `stellar contract deploy` and `stellar contract invoke`
- Security checklist for production deployment

---

## Reference Materials

### Cookbook Examples by Module

| Module | Key Examples |
|--------|-------------|
| 1 | [`examples/basics/01-hello-world/`](../../examples/basics/01-hello-world/) |
| 2 | [`examples/basics/02-storage-patterns/`](../../examples/basics/02-storage-patterns/), [`examples/basics/03-custom-errors/`](../../examples/basics/03-custom-errors/), [`examples/basics/04-events/`](../../examples/basics/04-events/) |
| 3 | [`examples/basics/03-authentication/`](../../examples/basics/03-authentication/), [`examples/intermediate/02-role-based-access-control/`](../../examples/intermediate/02-role-based-access-control/), [`examples/intermediate/03-pause-unpause/`](../../examples/intermediate/03-pause-unpause/) |
| 4 | [`examples/tokens/01-sep41-token/`](../../examples/tokens/01-sep41-token/), [`examples/tokens/03-pausable-token/`](../../examples/tokens/03-pausable-token/), [`examples/tokens/04-mint-burn/`](../../examples/tokens/04-mint-burn/) |
| 5 | [`examples/advanced/04-upgradeable-proxy/`](../../examples/advanced/04-upgradeable-proxy/), [`examples/advanced/03-oracle-pattern/`](../../examples/advanced/03-oracle-pattern/), [`examples/advanced/02-timelock/`](../../examples/advanced/02-timelock/) |

### Guides

- [Getting Started](../../guides/getting-started.md) — Full environment setup
- [Testing Guide](../../guides/testing.md) — Unit and integration testing
- [Deployment Guide](../../guides/deployment.md) — Testnet and mainnet deployment
- [Ethereum to Soroban](../../guides/ethereum-to-soroban.md) — Solidity → Rust translation
- [Best Practices](../../docs/best-practices.md) — Security and code quality guidelines
- [Quick Reference](../../docs/quick-reference.md) — Cheat sheet for common patterns

### External Resources

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Developers](https://developers.stellar.org)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Stellar Discord](https://discord.gg/stellardev)
- [Stellar Friendbot](https://friendbot.stellar.org) — Testnet XLM faucet

---

## Quick Reference: Common Commands

```bash
# Build a contract to WASM
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test

# Run a specific test
cargo test test_function_name

# Check code formatting
cargo fmt -- --check

# Run linter
cargo clippy

# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/contract.wasm \
  --source testnet \
  --network testnet

# Invoke a contract function
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network testnet \
  -- function_name --arg1 value1 --arg2 value2

# Generate a testnet account
stellar keys generate --global testnet

# Check account balance
stellar account balance --source testnet --network testnet
```

---

## Troubleshooting

### Build Fails with "can't find crate for `core`"

You're missing the WASM target:

```bash
rustup target add wasm32-unknown-unknown
```

### `cargo test` Fails with "no_std" Errors

Make sure your contract has the `#![cfg_attr(target_family = "wasm", no_std)]` attribute. This keeps `std` available for native tests but removes it for WASM builds.

### `stellar` Command Not Found

The CLI may not be in your PATH. Try:

```bash
source "$HOME/.cargo/env"
```

Or check `~/.cargo/bin/stellar` directly.

### Deployment Fails with "insufficient funds"

Fund your testnet account at [friendbot.stellar.org](https://friendbot.stellar.org).

### `require_auth()` Panics in Tests

You need to mock authentication. Use `env.mock_auths()` in your test:

```rust
env.mock_auths(&[MockAuth {
    address: &caller,
    invoke: &MockAuthInvoke {
        contract: &contract_id,
        fn_name: "function_name",
        args: (),
        sub_invokes: &[],
    },
}]);
```

### Type Mismatch on `storage().get()`

`get()` returns `Option<T>`. Make sure the type matches what was stored:

```rust
// If you stored an i128...
let value: Option<i128> = env.storage().persistent().get(&key);
```

---

## After the Workshop

### Continue Learning

1. **Explore the Cookbook** — Browse [`examples/`](../../examples/) by category. Start with the basics you haven't seen yet.
2. **Read the Guides** — [`guides/`](../../guides/) has in-depth tutorials for testing, deployment, and Ethereum migration.
3. **Study Advanced Patterns** — [`examples/advanced/`](../../examples/advanced/) has 30+ patterns including diamond, bridge, and state channels.
4. **Try the Templates** — [`templates/`](../../templates/) has full-stack dApp starters for tokens, NFT marketplaces, and DAOs.

### Build a Project

Pick a project to solidify your learning:

- **Beginner:** Build a vesting contract using [`examples/tokens/05-vesting/`](../../examples/tokens/05-vesting/) as a reference.
- **Intermediate:** Build a simple AMM using [`examples/defi/01-simple-swap/`](../../examples/defi/01-simple-swap/) as a reference.
- **Advanced:** Build a governance system using [`examples/governance/`](../../examples/governance/) examples.

### Join the Community

- [Stellar Discord](https://discord.gg/stellardev) — Chat with developers in the `#soroban` channel
- [GitHub Discussions](https://github.com/gloriaibrahim2002-blip/Soroban-Cookbook-/discussions) — Ask questions and share projects
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — Contribute your own examples to the cookbook

### Provide Feedback

Help us improve the workshop:

1. Fill out the feedback form (provided by your facilitator)
2. Open a GitHub issue with suggestions
3. Contribute improvements to the workshop materials via pull request

---

## Glossary

| Term | Definition |
|------|-----------|
| **Soroban** | Stellar's smart contract platform |
| **SEP-41** | Stellar's fungible token standard (like ERC-20) |
| **WASM** | WebAssembly — the compilation target for Soroban contracts |
| **`Env`** | Soroban's execution environment object (storage, events, ledger) |
| **`require_auth()`** | Soroban's built-in authorization check |
| **Instance Storage** | Storage tied to the contract instance (singleton config) |
| **Persistent Storage** | Long-lived storage requiring TTL management |
| **Temporary Storage** | Short-lived storage that auto-expires |
| **TTL** | Time-to-live — how long storage remains active before archival |
| **Ledger** | A block on the Stellar network. Soroban executes within ledgers |
| **`Symbol`** | Short string identifier (≤9 chars via `symbol_short!`) |
| **`Address`** | Represents an account or contract on Stellar |
| **`contracttype`** | Macro that enables a type to be stored in contract storage |
| **`contracterror`** | Macro for defining typed contract errors |

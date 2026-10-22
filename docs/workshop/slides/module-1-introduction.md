---
marp: true
theme: default
paginate: true
---

# Module 1: Introduction & Environment Setup

## Soroban Development Workshop

---

# What Is Soroban?

- Smart contract platform on the Stellar network
- Contracts written in Rust, compiled to WebAssembly
- Designed for low-cost, high-throughput execution
- Built-in resource metering (gas/CPU/instruction budget)

---

# Why Soroban?

- **Stellar's native smart contracts** — no separate L2
- **Rust safety** — memory-safe contracts with strong typing
- **Built-in auth** — `require_auth()` pattern for authorization
- **Three storage types** — persistent, temporary, instance
- **Rich SDK** — `soroban-sdk` crate with Vec, Map, String, Symbol, Address

---

# The Soroban Cookbook

This workshop uses the **Soroban Cookbook** — a collection of 80+ smart contract examples:

| Category | Examples | Focus |
|----------|----------|-------|
| Basics | 14 | Storage, auth, events, types, errors |
| Intermediate | 12 | RBAC, multi-sig, pagination, pausable |
| Advanced | 30+ | Proxies, oracles, bridges, diamond pattern |
| DeFi | 15 | AMMs, lending, flash loans, vaults |
| Tokens | 12 | SEP-41, minting, wrapping, vesting |
| NFTs | 5 | Minting, metadata, marketplace |
| Governance | 8 | Voting, proposals, treasury, delegation |

---

# Development Environment

What we need:

1. **Rust** — compiler and toolchain
2. **wasm32-unknown-unknown** — WebAssembly compilation target
3. **Stellar CLI** — build, deploy, and invoke contracts
4. **Code editor** — VS Code with rust-analyzer recommended

---

# Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Verify
rustc --version
cargo --version
```

Soroban requires Rust 1.74+. Run `rustup update stable` if older.

---

# Step 2: Add WebAssembly Target

```bash
rustup target add wasm32-unknown-unknown

# Verify
rustup target list --installed | grep wasm32
```

Soroban contracts compile to WASM — this target is required.

---

# Step 3: Install Stellar CLI

```bash
cargo install --locked stellar-cli --features opt

# Verify
stellar --version
stellar contract --help
```

The `stellar` CLI replaces the older `soroban-cli` package. It handles:
- Building contracts to WASM
- Deploying to testnet/mainnet
- Invoking contract functions
- Generating client bindings

---

# Step 4: Clone the Cookbook

```bash
git clone https://github.com/Stellar-Cookbook/Soroban-Cookbook.git
cd Soroban-Cookbook
```

The cookbook is a Cargo workspace — all examples build together.

---

# Your First Contract: Hello World

```rust
#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol_short!("Hello"), to]
    }
}
```

Source: [`examples/basics/01-hello-world/`](../../examples/basics/01-hello-world/)

---

# Anatomy of a Contract

```rust
#![cfg_attr(target_family = "wasm", no_std)]  // No std lib in WASM
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]           // Macro: marks this as a contract type
pub struct HelloContract;

#[contractimpl]       // Macro: generates client + dispatch logic
impl HelloContract {
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        // env = execution environment (storage, events, ledger)
        // to  = input parameter (Symbol type)
        vec![&env, symbol_short!("Hello"), to]
    }
}
```

---

# Key Soroban SDK Types

| Type | Description | Example |
|------|-------------|---------|
| `Env` | Execution environment | `env.storage()`, `env.events()` |
| `Address` | Account or contract address | `from.require_auth()` |
| `Symbol` | Short string identifier (≤9 chars via `symbol_short!`) | `symbol_short!("transfer")` |
| `String` | Variable-length string | `String::from_str(&env, "hello")` |
| `Vec<T>` | Ordered collection | `vec![&env, 1, 2, 3]` |
| `Map<K,V>` | Key-value mapping | `Map::new(&env)` |
| `Bytes` / `BytesN<N>` | Byte arrays | `BytesN::from_array(&env, &[...])` |

---

# Building & Testing

```bash
# Build to WASM
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test

# Run a single example's tests
cd examples/basics/01-hello-world
cargo test
```

---

# Running Tests

```rust
#[test]
fn test_hello_returns_greeting_vec() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let result = client.hello(&symbol_short!("World"));

    assert_eq!(
        result,
        vec![&env, symbol_short!("Hello"), symbol_short!("World")]
    );
}
```

Tests run natively (not WASM) via the Soroban test environment.

---

# Module 1 Summary

- Soroban = Stellar smart contracts in Rust → WASM
- `soroban-sdk` provides all types and macros
- `#[contract]` + `#[contractimpl]` define a contract
- `Env` gives access to storage, events, and ledger
- Build with `cargo build --target wasm32-unknown-unknown`
- Test with `cargo test` using the built-in test env

---

# Up Next

**Module 2: Storage & Data Types**
- Persistent, temporary, and instance storage
- Custom types with `#[contracttype]`
- Collections (Vec, Map) and their patterns

---

# Exercise Time!

See [Exercise 1](../exercises/module-1-exercise.md)

**Goal:** Set up your environment and run the Hello World contract.

⏱️ 30 minutes

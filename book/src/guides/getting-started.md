# Getting Started with Soroban

This guide walks you through everything you need to write, test, and deploy your first Soroban smart contract — from a fresh machine to a live contract on testnet.

---

## Prerequisites

Before you start, make sure you have:

- A Unix-like terminal (macOS, Linux, or WSL2 on Windows)
- Basic familiarity with the command line
- No prior Rust experience required, but the [Rust Book](https://doc.rust-lang.org/book/) is a great companion

---

## Step 1 — Install Rust

Soroban contracts are written in Rust and compiled to WebAssembly. Install the Rust toolchain via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen prompts (the default installation is fine). Then reload your shell:

```bash
source "$HOME/.cargo/env"
```

Verify the installation:

```bash
rustc --version   # e.g. rustc 1.78.0
cargo --version   # e.g. cargo 1.78.0
```

> Soroban requires **Rust 1.74 or later**. Run `rustup update stable` if your version is older.

---

## Step 2 — Add the WebAssembly Target

Soroban contracts compile to WebAssembly (WASM). Add the target:

```bash
rustup target add wasm32-unknown-unknown
```

Verify it was added:

```bash
rustup target list --installed | grep wasm32
# wasm32-unknown-unknown
```

---

## Step 3 — Install the Soroban CLI

The Soroban CLI handles building, testing, deploying, and invoking contracts:

```bash
cargo install --locked stellar-cli --features opt
```

> The package is now published as `stellar-cli` (which includes the `soroban` subcommand). If you have an older `soroban-cli` installed, uninstall it first: `cargo uninstall soroban-cli`.

Verify the installation:

```bash
stellar --version        # e.g. stellar 21.x.x
stellar contract --help  # should list contract subcommands
```

---

## Step 4 — Configure Your Editor (Recommended)

### VS Code

Install [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) for inline type hints, auto-complete, and error highlighting. It works out of the box with Soroban projects.

### JetBrains IDEs (IntelliJ / CLion / RustRover)

Install the [Rust plugin](https://plugins.jetbrains.com/plugin/8182-rust).

---

## Step 5 — Set Up a Testnet Identity

You need a funded account to deploy contracts. Create one now so it's ready when you reach the deployment step.

### Add the testnet network

```bash
stellar network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

### Generate a keypair

```bash
stellar keys generate alice --network testnet
```

### Print your public key

```bash
stellar keys address alice
# G... (56-character Stellar address)
```

### Fund the account (testnet only — free)

```bash
stellar keys fund alice --network testnet
```

This calls Friendbot, Stellar's testnet faucet, and deposits 10,000 XLM into your account.

---

## Step 6 — Your First Contract

### 6.1 Create the project

```bash
cargo new --lib my-first-contract
cd my-first-contract
```

### 6.2 Configure `Cargo.toml`

Replace the generated `Cargo.toml` with:

```toml
[package]
name = "my-first-contract"
version = "0.1.0"
edition = "2021"

[lib]
# cdylib produces the .wasm file the Soroban host loads.
# rlib lets the test harness link against the crate.
crate-type = ["cdylib", "rlib"]

[dependencies]
soroban-sdk = "21.7.0"

[dev-dependencies]
soroban-sdk = { version = "21.7.0", features = ["testutils"] }

[profile.release]
opt-level = "z"          # optimise for binary size
overflow-checks = true   # keep overflow traps in release builds
debug = 0
strip = "symbols"
debug-assertions = false
panic = "abort"
codegen-units = 1
lto = true
```

Key points:
- `crate-type = ["cdylib", "rlib"]` — `cdylib` produces the `.wasm` binary; `rlib` is needed so tests can import the crate.
- The `[profile.release]` block is the standard Soroban size-optimisation profile. Copy it into every contract you write.

### 6.3 Write the contract

Replace `src/lib.rs` with:

```rust
#![no_std]
//  ^^^^^^^ Soroban contracts run in a no_std Wasm sandbox.
//          The standard library is not available.

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};
//                ^^^^^^^   ^^^^^^^^^^^^  ^^^^^^^^^^^^   ^^^
//                |         |             |              Soroban's Vec (not std::vec::Vec)
//                |         |             Creates a Symbol from a short string literal
//                |         Marks the impl block as the contract's public interface
//                Marks this struct as a Soroban contract

/// The contract type — a plain unit struct.
/// Soroban routes invocations to the #[contractimpl] block below.
#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    /// Returns a greeting: ["Hello", <to>]
    ///
    /// `env` is the execution environment injected by the host.
    /// `to`  is the name to greet, passed as a Symbol.
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol_short!("Hello"), to]
        //   ^^^^  ^^^^^^^^^^^^^^^^^^^^^^  ^^
        //   |     A compile-time Symbol   The caller-supplied name
        //   Required by Soroban's vec! macro
    }
}

#[cfg(test)]
mod test;
```

Why `Vec<Symbol>` instead of a `String`? `soroban_sdk::String` is an immutable host object — there is no `format!` or string concatenation in the Wasm sandbox. Returning a `Vec<Symbol>` is idiomatic: it is cheap, composable, and easy for frontends to decode.

### 6.4 Write the tests

Create `src/test.rs`:

```rust
#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, vec, Env};

#[test]
fn test_hello_returns_greeting() {
    // 1. Create a fresh test environment.
    let env = Env::default();

    // 2. Register the contract — this gives it an on-chain address.
    let contract_id = env.register_contract(None, HelloContract);

    // 3. Create a typed client. The SDK generates HelloContractClient
    //    automatically from the #[contractimpl] block.
    let client = HelloContractClient::new(&env, &contract_id);

    // 4. Invoke the contract function through the client.
    let result = client.hello(&symbol_short!("World"));

    // 5. Assert the expected output.
    assert_eq!(
        result,
        vec![&env, symbol_short!("Hello"), symbol_short!("World")]
    );
}
```

### 6.5 Run the tests

```bash
cargo test
```

Expected output:

```
running 1 test
test test::test_hello_returns_greeting ... ok

test result: ok. 1 passed; 0 failed
```

### 6.6 Build the contract

```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled contract lands at:

```
target/wasm32-unknown-unknown/release/my_first_contract.wasm
```

You can also use the Stellar CLI, which wraps the build and applies the workspace profile automatically:

```bash
stellar contract build
```

### 6.7 Deploy to testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_first_contract.wasm \
  --source alice \
  --network testnet
```

The CLI prints a contract ID — a 56-character address starting with `C`. Save it:

```
CA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAXE
```

### 6.8 Invoke the deployed contract

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- hello \
  --to World
```

Expected output:

```json
["Hello","World"]
```

You just deployed and invoked your first Soroban smart contract.

---

## What Just Happened?

Here is a quick map of the pieces:

| Piece | Role |
|---|---|
| `#[contract]` | Registers `HelloContract` with the Soroban host |
| `#[contractimpl]` | Exposes `hello` as a callable contract function |
| `Env` | The gateway to the blockchain: storage, events, crypto, ledger info |
| `Symbol` | A gas-efficient short string (≤ 32 alphanumeric/underscore chars) |
| `Vec<Symbol>` | A host-allocated vector — the idiomatic multi-value return type |
| `HelloContractClient` | Auto-generated typed client used in tests and off-chain tooling |
| `cdylib` crate type | Produces the `.wasm` binary the Soroban VM loads |
| `rlib` crate type | Lets the test harness import the crate's types |

---

## Next Steps

1. **Explore the examples** — start with the basics:
   - [01-hello-world](../../examples/basics/01-hello-world/) — the contract you just wrote, fully annotated
   - [02-storage-patterns](../../examples/basics/02-storage-patterns/) — persistent, instance, and temporary storage
   - [03-authentication](../../examples/basics/03-authentication/) — `require_auth()`, admin checks, RBAC
   - [04-events](../../examples/basics/04-events/) — structured event emission
   - [05-error-handling](../../examples/basics/05-error-handling/) — `Result` vs `panic!`

2. **Learn to test well** — read the [Testing Guide](./testing.md)

3. **Deploy with confidence** — read the [Deployment Guide](./deployment.md)

4. **Coming from Ethereum?** — read the [Ethereum → Soroban Migration Guide](./ethereum-to-soroban.md)

---

## Troubleshooting

### `error: linker 'rust-lld' not found`

The LLVM linker component is missing.

```bash
rustup component add llvm-tools-preview
```

### `error[E0463]: can't find crate for 'std'`

The `wasm32-unknown-unknown` target is not installed.

```bash
rustup target add wasm32-unknown-unknown
```

### `error: no such command: 'soroban'`

The CLI binary is not in your `PATH`. Either reload your shell or add `~/.cargo/bin` manually:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

If you installed the old `soroban-cli` package, uninstall it and reinstall as `stellar-cli`:

```bash
cargo uninstall soroban-cli
cargo install --locked stellar-cli --features opt
```

### `error: package 'soroban-cli ...' failed to compile`

A dependency failed to build. Try:

```bash
cargo clean
cargo install --locked stellar-cli --features opt
```

If the error mentions OpenSSL, install the system headers:

```bash
# Ubuntu / Debian
sudo apt-get install pkg-config libssl-dev

# macOS (Homebrew)
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

### `Network timeout` / `RPC error` during deploy or invoke

- Check your internet connection.
- The testnet RPC may be temporarily overloaded. Wait a minute and retry.
- Try an alternate RPC endpoint with `--rpc-url https://rpc-futurenet.stellar.org:443`.
- Check the [Stellar status page](https://status.stellar.org) for outages.

### `error: account not found` during deploy

Your account has not been funded yet.

```bash
stellar keys fund alice --network testnet
```

### `error: transaction simulation failed: HostError: Error(Value, InvalidInput)`

The contract function arguments are wrong. Double-check the `--` separator and argument names:

```bash
# Correct
stellar contract invoke --id <ID> --source alice --network testnet \
  -- hello --to World

# Wrong (missing --)
stellar contract invoke --id <ID> --source alice --network testnet \
  hello --to World
```

### `wasm validation error: reference-types not supported`

The WASM binary was compiled with reference-types enabled, which the Soroban VM rejects. Add a `.cargo/config.toml` at the workspace root:

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=-reference-types"]
```

Then rebuild:

```bash
cargo clean
cargo build --target wasm32-unknown-unknown --release
```

### Tests compile but the contract client type is not found

Make sure `Cargo.toml` includes `"rlib"` in `crate-type`:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

Without `rlib`, the test harness cannot import the crate's types and the generated client will not be visible.

---

## Getting Help

- [Stellar Discord](https://discord.gg/stellardev) — `#soroban-dev` channel
- [Stack Exchange](https://stellar.stackexchange.com/) — tagged `soroban`
- [GitHub Discussions](https://github.com/Soroban-Cookbook/Soroban-Cookbook/discussions)
- [Official Soroban Docs](https://developers.stellar.org/docs/smart-contracts)
- [Soroban SDK API Reference](https://docs.rs/soroban-sdk/21.7.0/soroban_sdk/)

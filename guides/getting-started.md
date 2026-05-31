# Getting Started with Soroban

This guide walks you through everything you need to write, test, and deploy your first Soroban smart contract from a fresh machine to a live contract on testnet.

## Prerequisites

Before you start, make sure you have:

- A Unix-like terminal (macOS, Linux, or WSL2 on Windows)
- Basic familiarity with the command line
- No prior Rust experience required, but the [Rust Book](https://doc.rust-lang.org/book/) is a great companion

## Step 1: Install Rust

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
rustc --version
cargo --version
```

Soroban requires Rust 1.74 or later. Run `rustup update stable` if your version is older.

## Step 2: Add the WebAssembly Target

Soroban contracts compile to WebAssembly (WASM). Add the target:

```bash
rustup target add wasm32-unknown-unknown
```

Verify it was added:

```bash
rustup target list --installed | grep wasm32
```

## Step 3: Install the Soroban CLI

The Soroban CLI handles building, testing, deploying, and invoking contracts:

```bash
cargo install --locked stellar-cli --features opt
```

The package is now published as `stellar-cli` (which includes the `soroban` functionality). If you have an older `soroban-cli` installed, uninstall it first:

```bash
cargo uninstall soroban-cli
```

Verify the installation:

```bash
stellar --version
stellar contract --help
```

## Step 4: Configure Your Editor (Recommended)

### VS Code

Install [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) for inline type hints, auto-complete, and error highlighting.

### JetBrains IDEs (IntelliJ / CLion / RustRover)

Install the [Rust plugin](https://plugins.jetbrains.com/plugin/8182-rust).

## Step 5: Set Up a Testnet Identity

You need a funded account to deploy contracts.

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
```

### Fund the account (testnet only)

```bash
stellar keys fund alice --network testnet
```

## Step 6: Your First Contract

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
crate-type = ["cdylib", "rlib"]

[dependencies]
soroban-sdk = "21.7.0"

[dev-dependencies]
soroban-sdk = { version = "21.7.0", features = ["testutils"] }

[profile.release]
opt-level = "z"
overflow-checks = true
debug = 0
strip = "symbols"
debug-assertions = false
panic = "abort"
codegen-units = 1
lto = true
```

### 6.3 Write the contract

Replace `src/lib.rs` with:

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol_short!("Hello"), to]
    }
}

#[cfg(test)]
mod test;
```

### 6.4 Write the tests

Create `src/test.rs`:

```rust
#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, vec, Env};

#[test]
fn test_hello_returns_greeting() {
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

### 6.5 Run the tests

```bash
cargo test
```

### 6.6 Build the contract

```bash
cargo build --target wasm32-unknown-unknown --release
```

Or use the CLI:

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

Save the returned contract ID.

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

## Next Steps

1. Explore [01-hello-world](../examples/basics/01-hello-world/)
2. Explore [02-storage-patterns](../examples/basics/02-storage-patterns/)
3. Explore [03-authentication](../examples/basics/03-authentication/)
4. Read the [Testing Guide](./testing.md)
5. Read the [Deployment Guide](./deployment.md)
6. Read the [Ethereum to Soroban Guide](./ethereum-to-soroban.md)

## Troubleshooting

### `error: linker 'rust-lld' not found`

```bash
rustup component add llvm-tools-preview
```

### `error[E0463]: can't find crate for 'std'`

```bash
rustup target add wasm32-unknown-unknown
```

### `error: no such command: 'soroban'`

Reload your shell or add Cargo binaries to your path:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

If needed, reinstall CLI as `stellar-cli`:

```bash
cargo uninstall soroban-cli
cargo install --locked stellar-cli --features opt
```

### CLI install fails during compilation

```bash
cargo clean
cargo install --locked stellar-cli --features opt
```

If OpenSSL headers are missing:

```bash
# Ubuntu / Debian
sudo apt-get install pkg-config libssl-dev

# macOS (Homebrew)
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

### Network timeout or RPC error

- Check your internet connection.
- Retry after a short wait.
- Use a specific RPC URL with `--rpc-url`.
- Check [Stellar status](https://status.stellar.org).

### `error: account not found` during deploy

```bash
stellar keys fund alice --network testnet
```

### `error: transaction simulation failed: HostError: Error(Value, InvalidInput)`

Ensure you keep the `--` separator before contract function arguments:

```bash
stellar contract invoke --id <ID> --source alice --network testnet \
  -- hello --to World
```

### `wasm validation error: reference-types not supported`

Create `.cargo/config.toml` in your project:

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=-reference-types"]
```

Then rebuild:

```bash
cargo clean
cargo build --target wasm32-unknown-unknown --release
```

### Tests compile but generated client type is missing

Ensure `Cargo.toml` includes:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

## Getting Help

- [Stellar Discord](https://discord.gg/stellardev) (`#soroban-dev`)
- [Stack Exchange](https://stellar.stackexchange.com/) (tagged `soroban`)
- [GitHub Discussions](https://github.com/Soroban-Cookbook/Soroban-Cookbook/discussions)
- [Official Soroban Docs](https://developers.stellar.org/docs/smart-contracts)
- [Soroban SDK API Reference](https://docs.rs/soroban-sdk/21.7.0/soroban_sdk/)

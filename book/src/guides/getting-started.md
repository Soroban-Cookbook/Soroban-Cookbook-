# Getting Started with Soroban

Welcome to Soroban smart contract development! This guide will help you set up your development environment and deploy your first contract.

## 📋 Prerequisites

- Basic knowledge of Rust programming
- Familiarity with blockchain concepts (helpful but not required)
- Command line comfort

## 🛠️ Installation

### 1. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify installation:

```bash
rustc --version
cargo --version
```

### 2. Add WASM Target

Soroban contracts compile to WebAssembly:

```bash
rustup target add wasm32-unknown-unknown
```

### 3. Install Soroban CLI

The Soroban CLI is essential for building, testing, and deploying contracts:

```bash
cargo install --locked soroban-cli --features opt
```

Verify installation:

```bash
soroban --version
```

### 4. Configure Your Editor (Optional but Recommended)

#### VS Code

Install the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension for excellent Rust support.

#### IntelliJ IDEA / CLion

Install the [Rust plugin](https://plugins.jetbrains.com/plugin/8182-rust).

## 🌐 Network Configuration

### Testnet Setup

1. Add the testnet network:

```bash
soroban network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

2. Create an identity (wallet):

```bash
soroban keys generate alice --network testnet
```

3. Get your public key:

```bash
soroban keys address alice
```

4. Fund your account (testnet only):

```bash
soroban keys fund alice --network testnet
```

### Mainnet Setup (When Ready)

```bash
soroban network add \
  --global mainnet \
  --rpc-url https://soroban-mainnet.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

⚠️ **Warning:** Never commit your mainnet keys to version control!

## 🚀 Your First Contract

### 1. Create a New Project

```bash
cargo new --lib my-first-contract
cd my-first-contract
```

### 2. Update Cargo.toml

```toml
[package]
name = "my-first-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

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

### 3. Write Your Contract

Edit `src/lib.rs`:

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

### 4. Add Tests

Create `src/test.rs`:

```rust
#![cfg(test)]
use super::*;
use soroban_sdk::{symbol_short, Env};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let words = client.hello(&symbol_short!("World"));
    assert_eq!(
        words,
        vec![&env, symbol_short!("Hello"), symbol_short!("World")]
    );
}
```

### 5. Test Your Contract

```bash
cargo test
```

### 6. Build Your Contract

```bash
cargo build --target wasm32-unknown-unknown --release
```

Or use Soroban CLI:

```bash
soroban contract build
```

### 7. Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_first_contract.wasm \
  --source alice \
  --network testnet
```

Save the contract ID that's returned!

### 8. Invoke Your Contract

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- hello \
  --to World
```

Expected output:

```
["Hello", "World"]
```

## 🎉 Success!

You've just deployed and invoked your first Soroban smart contract!

## 📚 Next Steps

1. **Explore Examples** - Check out the [examples](../examples/) directory
   - [Hello World](../examples/basics/01-hello-world/) - Understand the basics
   - [Storage Patterns](../examples/basics/02-storage-patterns/) - Learn data persistence
   - [Authentication](../examples/basics/03-authentication/) - Secure your contracts

2. **Learn Testing** - Read the [Testing Guide](./testing.md)

3. **Master Deployment** - Study the [Deployment Guide](./deployment.md)

4. **From Ethereum?** - Check the [Migration Guide](./ethereum-to-soroban.md)

## 🔧 Troubleshooting

### Common Issues

**Error: "error: linker `rust-lld` not found"**

```bash
rustup component add llvm-tools-preview
```

**Error: "cannot find -lsoroban-env-host"**

```bash
cargo clean
cargo build --target wasm32-unknown-unknown --release
```

**Network timeout**

- Check your internet connection
- Try a different RPC endpoint
- Use `--rpc-url` flag to specify alternate RPC server

### Getting Help

- [Stellar Discord](https://discord.gg/stellardev) - Active community
- [Official Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Stack Exchange](https://stellar.stackexchange.com/) - Q&A
- [GitHub Discussions](https://github.com/Soroban-Cookbook/Soroban-Cookbook/discussions)

## 📖 Additional Resources

- [Soroban SDK Documentation](https://docs.rs/soroban-sdk)
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [Soroban by Example](https://soroban.stellar.org/docs/examples)
- [Official Soroban Docs](https://developers.stellar.org/docs/smart-contracts)

---

**Ready to build?** Start with the [Hello World example](../examples/basics/01-hello-world/)!

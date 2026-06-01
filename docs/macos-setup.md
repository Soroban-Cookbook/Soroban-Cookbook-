# macOS Environment Setup Guide

A complete, copy-paste-ready guide for setting up a Soroban development environment on macOS.

---

## 1. Prerequisites

### 1.1 Install Homebrew (if not already installed)

Homebrew is the de facto package manager for macOS and simplifies installing command-line tools.

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the on-screen instructions. After installation, Homebrew will tell you to add it to your PATH. **Make sure to do that!** The exact commands will look something like:

```bash
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"
```

Verify Homebrew is installed:

```bash
brew --version
# Homebrew 4.x.x
```

### 1.2 Install System Dependencies

Some Soroban CLI dependencies require system libraries. Install them with Homebrew:

```bash
brew install openssl pkg-config
```

---

## 2. Install Rust Toolchain

Soroban contracts are written in Rust and compiled to WebAssembly.

### 2.1 Install Rust via `rustup`

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts (the default installation is fine). Then reload your shell to add Cargo to your PATH:

```bash
source "$HOME/.cargo/env"
```

### 2.2 Verify Installation

```bash
rustc --version
# rustc 1.78.0 (or newer)

cargo --version
# cargo 1.78.0 (or newer)
```

> Soroban requires **Rust 1.74 or later**. If your version is older, update it:
>
> ```bash
> rustup update stable
> ```

### 2.3 Add the WebAssembly Target

Soroban contracts compile to WebAssembly. Add the target:

```bash
rustup target add wasm32-unknown-unknown
```

Verify it was installed:

```bash
rustup target list --installed | grep wasm32
# wasm32-unknown-unknown
```

---

## 3. Install Stellar CLI

The Stellar CLI handles building, testing, deploying, and invoking Soroban contracts.

```bash
cargo install --locked stellar-cli --features opt
```

> If you previously installed the old `soroban-cli`, uninstall it first:
>
> ```bash
> cargo uninstall soroban-cli
> ```

### 3.1 Verify Installation

```bash
stellar --version
# stellar 21.x.x

stellar contract --help
# Should list contract-related subcommands
```

---

## 4. Configure Environment Variables (Optional but Recommended)

Add these to your shell configuration file (`~/.zprofile` or `~/.zshrc`) to avoid repeating them every time:

```bash
# Add Cargo binaries to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Tell Cargo where OpenSSL is (if installed via Homebrew)
export OPENSSL_DIR=$(brew --prefix openssl)
```

Then reload your shell:

```bash
source ~/.zprofile  # or ~/.zshrc
```

---

## 5. Set Up Your Editor (Optional but Recommended)

### VS Code

1. Install [VS Code](https://code.visualstudio.com/)
2. Install the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension

### JetBrains IDEs (IntelliJ / CLion / RustRover)

1. Install your preferred JetBrains IDE
2. Install the [Rust plugin](https://plugins.jetbrains.com/plugin/8182-rust)

---

## 6. Verify Your Setup

Let's run a quick sanity check to ensure everything works:

### 6.1 Create a Test Project

```bash
cargo new --lib hello-soroban
cd hello-soroban
```

### 6.2 Configure Cargo.toml

Replace the contents of `Cargo.toml` with:

```toml
[package]
name = "hello-soroban"
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

### 6.3 Write a Simple Contract

Replace the contents of `src/lib.rs` with:

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

### 6.4 Add Tests

Create `src/test.rs`:

```rust
#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, vec, Env};

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let result = client.hello(&symbol_short!("World"));
    assert_eq!(result, vec![&env, symbol_short!("Hello"), symbol_short!("World")]);
}
```

### 6.5 Run the Tests

```bash
cargo test
```

You should see:

```
running 1 test
test test::test_hello ... ok

test result: ok. 1 passed; 0 failed
```

If you see this, congratulations! Your macOS Soroban development environment is set up correctly.

---

## 7. Troubleshooting

### Problem: `rustup` command not found

**Solution**: Restart your terminal or reload your shell configuration:

```bash
source "$HOME/.cargo/env"
```

---

### Problem: `cargo install --locked stellar-cli --features opt` fails

#### If the error mentions OpenSSL:

```bash
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
cargo install --locked stellar-cli --features opt
```

#### If the error mentions "linker 'rust-lld' not found":

```bash
rustup component add llvm-tools-preview
```

#### If you get a generic build error:

Try cleaning Cargo's cache and reinstalling:

```bash
cargo clean
cargo install --locked stellar-cli --features opt
```

---

### Problem: `stellar` command not found

**Solution**: Add Cargo's bin directory to your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this to your `~/.zprofile` or `~/.zshrc` to make it permanent, then reload your shell.

---

### Problem: `error[E0463]: can't find crate for 'std'`

**Solution**: Install the WebAssembly target:

```bash
rustup target add wasm32-unknown-unknown
```

---

### Problem: Homebrew installation fails on Apple Silicon (M1/M2/M3)

**Solution**: Make sure you're using the Apple Silicon version of Homebrew (installed in `/opt/homebrew`). If you have the Intel version installed, uninstall it and reinstall:

```bash
# Uninstall Intel Homebrew (if installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/uninstall.sh)"

# Install Apple Silicon Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Don't forget to add Homebrew to your PATH as instructed during installation.

---

## 8. Next Steps

Now that your environment is set up, check out:

- [Getting Started](./getting-started.md) — Write your first Soroban contract
- [Testing Guide](./testing-guide.md) — Learn how to test Soroban contracts effectively
- [Examples](../examples/basics.md) — Explore more contract patterns

---

## Additional Resources

- [Soroban Official Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Stellar Discord](https://discord.gg/stellardev) — #soroban-dev channel for help
- [Rust Book](https://doc.rust-lang.org/book/) — Learn Rust (if you're new to it)

# macOS Environment Setup

A complete guide to setting up a Soroban smart contract development environment
on macOS. Follow each step in order — verification commands confirm each tool is
working before you move on.

## Prerequisites

- macOS 12 (Monterey) or later
- Terminal access (Terminal.app or iTerm2)
- Admin rights to install software

---

## Step 1 — Install Homebrew

Homebrew is the package manager used to install several dependencies in this
guide. Skip this step if `brew --version` already returns a version number.

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

After installation, follow any instructions printed about adding Homebrew to
your `PATH` (required on Apple Silicon Macs).

**Verify:**

```bash
brew --version
```

Expected output (version may differ):

```
Homebrew 4.x.x
```

---

## Step 2 — Install Rust

Soroban contracts are written in Rust. Install the official toolchain via
`rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When prompted, choose option `1` (default installation). Then reload your shell
environment:

```bash
source "$HOME/.cargo/env"
```

**Verify:**

```bash
rustc --version
cargo --version
rustup --version
```

Expected output:

```
rustc 1.74.0 (or later)
cargo 1.74.0 (or later)
rustup 1.26.0 (or later)
```

---

## Step 3 — Add the WebAssembly Target

Soroban contracts compile to WebAssembly (WASM). Add the required target:

```bash
rustup target add wasm32-unknown-unknown
```

**Verify:**

```bash
rustup target list --installed | grep wasm32
```

Expected output:

```
wasm32-unknown-unknown
```

---

## Step 4 — Install the Stellar CLI

The Stellar CLI (formerly Soroban CLI) is used to build, test, and deploy
contracts.

```bash
cargo install --locked stellar-cli --features opt
```

> This step compiles from source and may take a few minutes.

**Verify:**

```bash
stellar --version
```

Expected output:

```
stellar 21.x.x (or later)
```

---

## Step 5 — Install Node.js (optional)

Required only if you plan to use JavaScript/TypeScript SDKs or run frontend
tooling alongside your contracts.

```bash
brew install node
```

**Verify:**

```bash
node --version
npm --version
```

Expected output:

```
v20.x.x (or later)
8.x.x (or later)
```

---

## Step 6 — Install Git

macOS ships with a system Git, but the Homebrew version is more up to date:

```bash
brew install git
```

**Verify:**

```bash
git --version
```

Expected output:

```
git version 2.x.x
```

---

## Step 7 — Clone the Cookbook

```bash
git clone https://github.com/Stellar-Cookbook/Soroban-Cookbook.git
cd Soroban-Cookbook
```

---

## Step 8 — Run a Smoke Test

Confirm the full toolchain works end-to-end by running the hello-world example
tests:

```bash
cargo test -p hello-world
```

Expected output:

```
running 6 tests
test test::test_hello_returns_greeting_vec ... ok
test test::test_hello_first_element_is_hello ... ok
test test::test_hello_second_element_is_name ... ok
test test::test_hello_with_different_names ... ok
test test::test_hello_with_long_symbol_input ... ok
test test::test_hello_with_single_character_name ... ok

test result: ok. 6 passed; 0 failed; 0 ignored
```

---

## Step 9 — Configure a Testnet Identity (optional)

Required only if you want to deploy contracts to the Stellar testnet.

```bash
# Add the testnet network
stellar network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Generate a key pair
stellar keys generate alice --network testnet

# Print your public key
stellar keys address alice

# Fund the account from the testnet faucet
stellar keys fund alice --network testnet
```

---

## Full Environment Checklist

Run this block to confirm every required tool is present:

```bash
brew --version
rustc --version
cargo --version
rustup target list --installed | grep wasm32
stellar --version
git --version
```

All six commands should return version strings without errors.

---

## Troubleshooting

### `brew: command not found` after installation

Homebrew was installed but not added to `PATH`. For Apple Silicon Macs, add
this to `~/.zprofile`:

```bash
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"
```

For Intel Macs the prefix is `/usr/local` instead of `/opt/homebrew`.

---

### `rustc: command not found` after Rust installation

The Cargo bin directory is not on your `PATH`. Add it:

```bash
echo 'source "$HOME/.cargo/env"' >> ~/.zshrc
source "$HOME/.cargo/env"
```

---

### `wasm32-unknown-unknown` target missing after `rustup target add`

Your shell may be using a different Rust toolchain. Confirm the active
toolchain and re-add the target:

```bash
rustup show
rustup target add wasm32-unknown-unknown
```

---

### `stellar` command not found after `cargo install`

`~/.cargo/bin` is not on your `PATH`. Add it:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

---

### `cargo install stellar-cli` fails with linker errors

Install the Xcode Command Line Tools, which provide the system linker:

```bash
xcode-select --install
```

Then retry the install:

```bash
cargo install --locked stellar-cli --features opt
```

---

### `cargo test` fails with `error: no matching package named hello-world`

You are not in the repository root. Navigate there first:

```bash
cd /path/to/Soroban-Cookbook
cargo test -p hello-world
```

---

### Network timeout when funding testnet account

The testnet faucet is occasionally rate-limited. Wait 60 seconds and retry, or
use the web faucet at [https://friendbot.stellar.org](https://friendbot.stellar.org)
with your public key as the `addr` parameter.

---

## Next Steps

- [Getting Started Guide](./getting-started.md) — write and deploy your first contract
- [Testing Guide](./testing.md) — unit tests, fixtures, and error testing
- [Hello World Example](../examples/basics.md) — the simplest possible contract

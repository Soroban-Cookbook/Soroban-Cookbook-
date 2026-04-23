# Utility Scripts

Helper scripts for building, testing, and deploying Stellar smart contracts.

## Overview

This directory contains three essential utility scripts that streamline the Stellar smart contract development workflow:

- **build.sh** - Compile contracts to optimized WASM binaries
- **test.sh** - Run tests with comprehensive validation options
- **deploy.sh** - Deploy contracts to Stellar networks (testnet/mainnet)

These scripts provide consistent, automated workflows for common development tasks and are used in CI/CD pipelines.

## Available Scripts

### 🏗️ build.sh

Build smart contracts to optimized WASM.

**Usage:**

```bash
# Build all examples
./scripts/build.sh

# Build specific example
./scripts/build.sh examples/basics/01-hello-world
```

**Arguments:**

1. `example-path` *(optional)* — Path to a specific contract directory. Omit to build all examples.

**Features:**

- Compiles to optimized WASM (release mode)
- Shows output file size for each contract
- Comprehensive error handling
- Builds all examples or specific contracts
- Summarizes build results

### 🧪 test.sh

Run tests for smart contracts with comprehensive options and coverage reporting.

**Usage:**

```bash
# Test all examples
./scripts/test.sh

# Test specific example
./scripts/test.sh examples/basics/01-hello-world

# Test specific directory
./scripts/test.sh examples/basics

# Test with verbose output
./scripts/test.sh -v examples/basics/01-hello-world

# Test with clippy
./scripts/test.sh -c examples/basics/01-hello-world

# Test with format check
./scripts/test.sh -f examples/basics/01-hello-world

# Test with all checks
./scripts/test.sh -a examples/basics/01-hello-world

# Generate coverage report
./scripts/test.sh --coverage

# Show help
./scripts/test.sh --help
```

**Options:**

- `-v, --verbose` - Show detailed test output
- `-c, --clippy` - Run clippy linter
- `-f, --format` - Check code formatting
- `-a, --all` - Run all checks (tests, clippy, format)
- `--coverage` - Generate coverage report using cargo-tarpaulin
- `-h, --help` - Show help message

**Features:**

- **Workspace Testing**: Uses `cargo test --workspace` for efficient testing
- **Coverage Reporting**: Generates XML coverage reports compatible with Codecov
- **Error Handling**: Comprehensive error checking and informative messages
- **Flexible Targeting**: Test individual contracts, directories, or entire workspace
- **Performance Optimized**: Uses workspace-level testing when possible

### 🚀 deploy.sh

Deploy smart contracts to testnet or mainnet.

**Usage:**

```bash
# Deploy to testnet
./scripts/deploy.sh examples/basics/01-hello-world testnet alice

# Deploy to mainnet
./scripts/deploy.sh examples/basics/01-hello-world mainnet my-key
```

**Parameters:**

| Parameter | Required | Description | Example |
|-----------|----------|-------------|----------|
| `contract-path` | Yes | Relative or absolute path to the contract directory containing Cargo.toml | `examples/basics/01-hello-world` |
| `network` | Yes | Target network name (must be pre-configured with Stellar CLI) | `testnet`, `mainnet` |
| `identity` | No | Identity/keypair name for signing transactions (defaults to "default") | `alice`, `my-key` |

**Features:**

- Validates network selection (`testnet` or `mainnet`)
- Builds contract with `stellar contract build`
- Optimizes WASM with `stellar contract optimize`
- Funds testnet accounts automatically via friendbot
- Deploys with `stellar contract deploy`
- Saves contract ID to `.contract-id-{network}` file inside the contract directory

## 🔧 Requirements

All scripts require:

- Rust and Cargo installed
- `wasm32-unknown-unknown` target added
- Stellar CLI installed

**Install requirements:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --version 22.1.0
```

## 📋 Common Use Cases

### Development Workflow

**1. New Contract Development**
```bash
# Create and test a new contract
cd examples/basics/my-new-contract
cargo init --lib
# ... write your contract code ...

# Test during development
../../scripts/test.sh .

# Run all checks before committing
../../scripts/test.sh -a .
```

**2. Pre-Commit Validation**
```bash
# Validate all changes before committing
./scripts/test.sh -a examples/basics/01-hello-world
./scripts/build.sh examples/basics/01-hello-world
```

**3. Testnet Deployment Workflow**
```bash
# Complete testnet deployment workflow
./scripts/test.sh -a examples/defi/liquidity-pool
./scripts/build.sh examples/defi/liquidity-pool
./scripts/deploy.sh examples/defi/liquidity-pool testnet alice

# Verify deployment
stellar contract invoke \
  --id $(cat examples/defi/liquidity-pool/.contract-id-testnet) \
  --source alice \
  --network testnet \
  -- \
  get_reserves
```

**4. Production Deployment**
```bash
# Full validation before mainnet deployment
./scripts/test.sh --coverage
./scripts/test.sh -a examples/tokens/my-token
./scripts/build.sh examples/tokens/my-token

# Deploy to mainnet (use secure identity management)
./scripts/deploy.sh examples/tokens/my-token mainnet production-key
```

**5. Batch Testing**
```bash
# Test all basic examples
for example in examples/basics/*/; do
    ./scripts/test.sh "$example"
done

# Test with all checks
for example in examples/basics/*/; do
    ./scripts/test.sh -a "$example" || echo "Failed: $example"
done
```

**6. Coverage Analysis**
```bash
# Generate coverage for entire workspace
./scripts/test.sh --coverage

# Generate coverage for specific contract
./scripts/test.sh --coverage examples/basics/03-authentication

# View coverage report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

## 📝 Examples

### Quick Start Workflow

```bash
# 1. Test the contract with all checks
./scripts/test.sh -a examples/basics/01-hello-world

# 2. Build the contract to WASM
./scripts/build.sh examples/basics/01-hello-world

# 3. Deploy to testnet
./scripts/deploy.sh examples/basics/01-hello-world testnet alice

# 4. Interact with deployed contract
stellar contract invoke \
  --id $(cat examples/basics/01-hello-world/.contract-id-testnet) \
  --source alice \
  --network testnet \
  -- \
  hello --to "World"
```

### CI/CD Integration

```bash
# GitHub Actions / GitLab CI workflow
./scripts/test.sh --coverage          # Generate coverage
./scripts/test.sh -a                  # Run all checks on workspace
./scripts/build.sh                    # Build all contracts

# Test specific category
./scripts/test.sh -a examples/basics
./scripts/test.sh -a examples/defi

# Parallel testing (GNU parallel)
find examples -name "Cargo.toml" -type f | \
  xargs -I {} dirname {} | \
  parallel -j 4 ./scripts/test.sh {}
```

## 🐛 Troubleshooting

### Common Issues

#### Script Not Executable

**Problem:** `Permission denied` when running scripts

**Solution:**
```bash
chmod +x scripts/*.sh
```

#### Network Not Configured

**Problem:** `Network 'testnet' not configured`

**Solution:**
```bash
# Add testnet network
stellar network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Verify network configuration
stellar network ls
```

#### Identity Not Found

**Problem:** `Identity 'alice' not found`

**Solution:**
```bash
# Generate new identity
stellar keys generate alice --network testnet

# Fund testnet account (testnet only)
stellar keys fund alice --network testnet

# Verify identity
stellar keys address alice
```

#### Build Fails with WASM Target Missing

**Problem:** `error: can't find crate for 'core'`

**Solution:**
```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Verify target is installed
rustup target list | grep wasm32
```

#### Coverage Generation Fails

**Problem:** `cargo-tarpaulin not found`

**Solution:**
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin --locked

# Verify installation
cargo tarpaulin --version
```

#### Deployment Fails with Insufficient Balance

**Problem:** `insufficient balance for transaction`

**Solution:**
```bash
# For testnet - fund account
stellar keys fund alice --network testnet

# For mainnet - ensure account has sufficient XLM
stellar keys address alice
# Transfer XLM to this address
```

#### Test Failures Due to Storage Issues

**Problem:** Tests fail with storage-related errors

**Solution:**
```bash
# Clean build artifacts
cargo clean

# Rebuild and test
./scripts/build.sh examples/your-contract
./scripts/test.sh examples/your-contract
```

## 🎯 Script Parameters Reference

### build.sh Parameters

| Parameter | Type | Description | Default |
|-----------|------|-------------|----------|
| `contract-path` | Optional | Path to contract directory or empty for all | All examples |

**Exit Codes:**
- `0` - Build successful
- `1` - Build failed or invalid path

### test.sh Parameters

| Parameter | Type | Description | Default |
|-----------|------|-------------|----------|
| `-v, --verbose` | Flag | Show detailed test output with stdout/stderr | Quiet mode |
| `-c, --clippy` | Flag | Run Clippy linter with `-D warnings` | Disabled |
| `-f, --format` | Flag | Check code formatting with `cargo fmt --check` | Disabled |
| `-a, --all` | Flag | Run all checks (tests + clippy + format) | Disabled |
| `--coverage` | Flag | Generate coverage report using cargo-tarpaulin | Disabled |
| `-h, --help` | Flag | Display help message | - |
| `contract-path` | Optional | Path to contract/directory or empty for workspace | Workspace |

**Exit Codes:**
- `0` - All tests and checks passed
- `1` - Tests failed, checks failed, or invalid parameters

### deploy.sh Parameters

| Parameter | Type | Description | Default |
|-----------|------|-------------|----------|
| `contract-path` | Required | Path to contract directory | - |
| `network` | Required | Network name (testnet/mainnet) | - |
| `identity` | Optional | Identity/keypair name | `default` |

**Exit Codes:**
- `0` - Deployment successful
- `1` - Build failed, deployment failed, or invalid parameters

**Output Files:**
- `.contract-id-{network}` - Contains deployed contract ID

## 💡 Tips

1. **Always test before deploying**

   ```bash
   ./scripts/test.sh -a examples/my-contract
   ```

2. **Use testnet first**

   ```bash
   ./scripts/deploy.sh examples/my-contract testnet alice
   ```

3. **Verify deployment**

   ```bash
   stellar contract invoke \
     --id $(cat examples/my-contract/.contract-id-testnet) \
     --source alice \
     --network testnet \
     -- \
     my_function
   ```

4. **Keep contract IDs safe**
   - Contract IDs are saved in `.contract-id-{network}` files
   - Add to `.gitignore` for sensitive deployments
   - Use environment variables in production

5. **Use workspace testing for speed**
   ```bash
   # Faster - tests entire workspace at once
   ./scripts/test.sh
   
   # Slower - tests each contract individually
   for example in examples/basics/*/; do
       ./scripts/test.sh "$example"
   done
   ```

6. **Optimize WASM builds**
   ```bash
   # Build with optimizations
   ./scripts/build.sh examples/my-contract
   
   # Further optimize with wasm-opt (Binaryen)
   wasm-opt -Oz target/wasm32-unknown-unknown/release/my_contract.wasm \
     -o target/wasm32-unknown-unknown/release/my_contract.opt.wasm
   ```

## 🔧 Advanced Usage

### Environment Variables

Scripts respect the following environment variables:

```bash
# Customize Rust toolchain
export RUSTUP_TOOLCHAIN=stable

# Set custom cargo target directory
export CARGO_TARGET_DIR=./target

# Enable colored output
export CARGO_TERM_COLOR=always
```

### Script Composition

Combine scripts for complex workflows:

```bash
# Test, build, and deploy in one command
./scripts/test.sh -a examples/my-contract && \
./scripts/build.sh examples/my-contract && \
./scripts/deploy.sh examples/my-contract testnet alice

# Deploy to multiple networks
for network in testnet mainnet; do
    ./scripts/deploy.sh examples/my-contract $network production-key
done
```

### Integration with Make

Create a `Makefile` for easier script access:

```makefile
.PHONY: test build deploy clean

CONTRACT ?= examples/basics/01-hello-world
NETWORK ?= testnet
IDENTITY ?= alice

test:
	./scripts/test.sh -a $(CONTRACT)

build:
	./scripts/build.sh $(CONTRACT)

deploy: test build
	./scripts/deploy.sh $(CONTRACT) $(NETWORK) $(IDENTITY)

clean:
	cargo clean

all: test build
```

Usage:
```bash
make test CONTRACT=examples/defi/amm
make deploy NETWORK=mainnet IDENTITY=prod-key
```

### Custom Script Extensions

Extend scripts for project-specific needs:

```bash
# custom-deploy.sh - Deploy with initialization
#!/bin/bash
set -e

CONTRACT=$1
NETWORK=$2
IDENTITY=$3

# Deploy using standard script
./scripts/deploy.sh "$CONTRACT" "$NETWORK" "$IDENTITY"

# Initialize contract
CONTRACT_ID=$(cat "$CONTRACT/.contract-id-$NETWORK")
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$IDENTITY" \
  --network "$NETWORK" \
  -- \
  initialize --admin "$IDENTITY"
```

## 🔗 Related Documentation

- [Getting Started Guide](../guides/getting-started.md)
- [Testing Guide](../guides/testing.md)
- [Deployment Guide](../guides/deployment.md)
- [Stellar CLI Reference](https://developers.stellar.org/docs/tools/developer-tools/cli)

---

**Automate your Stellar development workflow!** 🚀

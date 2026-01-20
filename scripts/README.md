# Utility Scripts

Helper scripts for building, testing, and deploying Soroban contracts.

## Available Scripts

### 🏗️ build.sh

Build Soroban smart contracts.

**Usage:**

```bash
# Build all examples
./scripts/build.sh

# Build specific example
./scripts/build.sh examples/basics/01-hello-world
```

**Features:**

- Runs tests before building
- Compiles to WASM
- Shows output file size
- Summarizes results

### 🧪 test.sh

Run tests for Soroban contracts with various options.

**Usage:**

```bash
# Test all examples
./scripts/test.sh

# Test specific example
./scripts/test.sh examples/basics/01-hello-world

# Test with verbose output
./scripts/test.sh -v examples/basics/01-hello-world

# Test with clippy
./scripts/test.sh -c examples/basics/01-hello-world

# Test with format check
./scripts/test.sh -f examples/basics/01-hello-world

# Test with all checks
./scripts/test.sh -a examples/basics/01-hello-world
```

**Options:**

- `-v, --verbose` - Show detailed test output
- `-c, --clippy` - Run clippy linter
- `-f, --format` - Check code formatting
- `-a, --all` - Run all checks (tests, clippy, format)

### 🚀 deploy.sh

Deploy Soroban contracts to testnet or mainnet.

**Usage:**

```bash
# Deploy to testnet
./scripts/deploy.sh examples/basics/01-hello-world testnet alice

# Deploy to mainnet
./scripts/deploy.sh examples/basics/01-hello-world mainnet my-key
```

**Arguments:**

1. `contract-path` - Path to the contract directory
2. `network` - Network name (testnet/mainnet)
3. `identity` - Identity/key name (optional, defaults to "default")

**Features:**

- Builds contract before deploying
- Verifies network configuration
- Funds testnet accounts automatically
- Saves contract ID to `.contract-id-{network}` file

## 🔧 Requirements

All scripts require:

- Rust and Cargo installed
- `wasm32-unknown-unknown` target added
- Soroban CLI installed

**Install requirements:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install --locked soroban-cli
```

## 📝 Examples

### Build and Test Workflow

```bash
# 1. Test the contract
./scripts/test.sh -a examples/basics/01-hello-world

# 2. Build the contract
./scripts/build.sh examples/basics/01-hello-world

# 3. Deploy to testnet
./scripts/deploy.sh examples/basics/01-hello-world testnet alice
```

### Continuous Integration Workflow

```bash
# Run on all examples
for example in examples/*/*/; do
    echo "Testing $example"
    ./scripts/test.sh -a "$example"
done
```

## 🐛 Troubleshooting

### Script Not Executable

```bash
chmod +x scripts/*.sh
```

### Network Not Configured

```bash
# Add testnet
soroban network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

### Identity Not Found

```bash
# Generate new identity
soroban keys generate alice --network testnet

# Fund testnet account
soroban keys fund alice --network testnet
```

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
   soroban contract invoke \
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

## 🔗 Related Documentation

- [Getting Started Guide](../guides/getting-started.md)
- [Testing Guide](../guides/testing.md)
- [Deployment Guide](../guides/deployment.md)
- [Soroban CLI Reference](https://developers.stellar.org/docs/tools/developer-tools/cli)

---

**Automate your Soroban development workflow!** 🚀

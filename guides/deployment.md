# Deployment Guide

Complete guide for deploying Soroban smart contracts to Testnet and Mainnet.

## Prerequisites

- Rust installed with `wasm32-unknown-unknown` target
- Soroban CLI installed: `cargo install --locked soroban-cli`
- A funded Stellar account (testnet: friendbot; mainnet: purchase XLM)

```bash
# Add the WASM target if not already present
rustup target add wasm32-unknown-unknown

# Verify Soroban CLI installation
soroban --version
```

---

## Network Configuration

### Testnet

```bash
# Add testnet network configuration
soroban network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Verify
soroban network ls
```

### Mainnet

```bash
# Add mainnet network configuration
soroban network add \
  --global mainnet \
  --rpc-url https://soroban-mainnet.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"

# Verify
soroban network ls
```

---

## Identity Management

### Create an Identity

```bash
# Generate a new keypair and store it locally
soroban keys generate alice --network testnet

# View the public key
soroban keys address alice

# List all stored identities
soroban keys ls
```

### Security Best Practices

> Never commit private keys to version control.

```bash
# Ensure local key storage is gitignored
echo ".soroban/" >> .gitignore
echo "*.key" >> .gitignore
```

For mainnet deployments:
- Use a hardware wallet or encrypted key storage
- Keep separate keys for deployment vs. admin operations
- Implement key rotation policies
- Consider multi-signature setups for high-value contracts

---

## Funding Your Account

### Testnet (Friendbot)

```bash
# Fund your testnet account via friendbot
soroban keys fund alice --network testnet

# Check balance
soroban keys balance alice --network testnet
```

You can also fund via the web: `https://friendbot.stellar.org?addr=<YOUR_PUBLIC_KEY>`

### Mainnet

1. Purchase XLM from an exchange
2. Send to your Stellar address (`soroban keys address alice`)
3. Maintain enough XLM for transaction fees and storage rent

---

## Building Your Contract

```bash
# From your contract directory
soroban contract build

# Or manually with cargo
cargo build --target wasm32-unknown-unknown --release
```

The compiled WASM will be at:
`target/wasm32-unknown-unknown/release/<contract_name>.wasm`

---

## Testnet Deployment Steps

### Step 1: Build

```bash
soroban contract build
```

### Step 2: Deploy

```bash
CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_contract.wasm \
  --source alice \
  --network testnet)

echo "Deployed contract ID: $CONTRACT_ID"
```

### Step 3: Verify

```bash
# Fetch contract info
soroban contract info \
  --id $CONTRACT_ID \
  --network testnet
```

### Step 4: Initialize (if required)

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --admin $(soroban keys address alice)
```

### Step 5: Invoke Functions

```bash
# Call a read function (no fee required)
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- \
  get_balance \
  --address $(soroban keys address alice)

# Call a write function
soroban contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  transfer \
  --from $(soroban keys address alice) \
  --to GDEST... \
  --amount 1000
```

---

## Mainnet Deployment Steps

> Ensure your contract is thoroughly tested on testnet before proceeding.

### Pre-deployment Checklist

- [ ] All tests passing (`cargo test`)
- [ ] Security audit completed
- [ ] Code reviewed by multiple developers
- [ ] Upgrade/emergency mechanism tested
- [ ] Sufficient XLM balance confirmed
- [ ] Monitoring plan in place

### Step 1: Build (optimized)

```bash
soroban contract build
```

### Step 2: Deploy

```bash
# CAUTION: This deploys to production
CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_contract.wasm \
  --source mainnet-deployer \
  --network mainnet)

echo "Mainnet contract ID: $CONTRACT_ID"
# Save this ID — it cannot be recovered if lost
```

### Step 3: Verify on Mainnet

```bash
soroban contract info \
  --id $CONTRACT_ID \
  --network mainnet
```

### Step 4: Initialize

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source mainnet-deployer \
  --network mainnet \
  -- \
  initialize \
  --admin $(soroban keys address mainnet-deployer)
```

### Step 5: Invoke and Validate

```bash
# Verify a read function returns expected state
soroban contract invoke \
  --id $CONTRACT_ID \
  --network mainnet \
  -- \
  get_admin
```

---

## Contract Invocation Examples

### Basic Invocation

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  <function_name> \
  --param_name value
```

### Passing Different Argument Types

```bash
# u64 / i128
soroban contract invoke ... -- my_fn --amount 1000000

# Address
soroban contract invoke ... -- my_fn --recipient GABC...XYZ

# Boolean
soroban contract invoke ... -- my_fn --enabled true

# String
soroban contract invoke ... -- my_fn --label "hello"

# Bytes (hex-encoded)
soroban contract invoke ... -- my_fn --data 0xdeadbeef
```

### Simulating Without Submitting

```bash
# Dry-run to estimate fees and check for errors
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  --send no \
  -- \
  transfer \
  --amount 500
```

### Reading Events After Invocation

```bash
soroban events \
  --start-ledger <LEDGER_NUMBER> \
  --id <CONTRACT_ID> \
  --network testnet
```

---

## Contract Upgrades

### Upgradeable Contract Pattern

```rust
pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
    let admin: Address = env.storage().instance()
        .get(&symbol_short!("admin"))
        .unwrap();
    admin.require_auth();
    env.deployer().update_current_contract_wasm(new_wasm_hash);
}
```

### Upgrade Process

```bash
# 1. Install the new WASM and capture its hash
NEW_HASH=$(soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/my_contract_v2.wasm \
  --source alice \
  --network testnet)

# 2. Call the upgrade function on the existing contract
soroban contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  upgrade \
  --new_wasm_hash $NEW_HASH
```

---

## Fee Estimation

```bash
# Simulate to see resource usage and fees
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  --send no \
  -- \
  my_function \
  --arg 123
```

Fee components:
- Network fee: base transaction fee
- Resource fee: CPU, memory, and I/O costs
- Rent: storage TTL extension costs

---

## Emergency Procedures

### Pause / Unpause Pattern

```rust
pub fn pause(env: Env) {
    require_admin(&env);
    env.storage().instance().set(&symbol_short!("paused"), &true);
}

pub fn unpause(env: Env) {
    require_admin(&env);
    env.storage().instance().remove(&symbol_short!("paused"));
}

fn check_not_paused(env: &Env) {
    if env.storage().instance().has(&symbol_short!("paused")) {
        panic!("Contract is paused");
    }
}
```

---

## Additional Resources

- [Soroban CLI Reference](https://developers.stellar.org/docs/tools/developer-tools/cli)
- [Network Configuration](https://developers.stellar.org/docs/networks)
- [Fee Documentation](https://developers.stellar.org/docs/smart-contracts/fees)
- [State Archival & TTL](https://developers.stellar.org/docs/smart-contracts/state-archival)
- [Stellar Discord](https://discord.gg/stellardev)

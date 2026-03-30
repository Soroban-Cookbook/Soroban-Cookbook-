# Deployment Guide

Complete guide to deploying Soroban smart contracts to testnet and mainnet.

## 📋 Prerequisites

- Soroban CLI installed (`cargo install soroban-cli`)
- Contract built and tested
- Funded account with XLM (for testnet: use friendbot, for mainnet: purchase)

## 🌐 Network Configuration

### Testnet Setup

```bash
# Add testnet network
soroban network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Verify network was added
soroban network ls
```

### Mainnet Setup

```bash
# Add mainnet network
soroban network add \
  --global mainnet \
  --rpc-url https://soroban-mainnet.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

## 🔑 Identity Management

### Create an Identity

```bash
# Generate a new identity
soroban keys generate alice --network testnet

# View the public key
soroban keys address alice

# List all identities
soroban keys ls
```

### Security Best Practices

⚠️ **Important:** Never commit private keys to version control!

```bash
# Add to .gitignore
echo ".soroban/" >> .gitignore
echo "*.key" >> .gitignore
```

For mainnet:

- Store keys securely (hardware wallet, encrypted storage)
- Use separate keys for different purposes
- Implement key rotation policies
- Consider multi-signature setups

## 💰 Funding Your Account

### Testnet Funding

```bash
# Fund account from friendbot (testnet only)
soroban keys fund alice --network testnet

# Verify balance
soroban keys balance alice --network testnet
```

### Mainnet Funding

For mainnet, you need to acquire XLM:

1. Purchase XLM from an exchange
2. Send to your Stellar address
3. Keep enough for transaction fees and rent

## 🏗️ Building Your Contract

### Standard Build

```bash
# Navigate to your contract directory
cd my-contract

# Build for WASM
cargo build --target wasm32-unknown-unknown --release

# Or use Soroban CLI
soroban contract build
```

### Optimized Build

```bash
# Install optimization tools
cargo install cargo-wasm-opt

# Build with optimizations
cargo build --target wasm32-unknown-unknown --release
cargo wasm-opt -- -Oz -o output.wasm target/wasm32-unknown-unknown/release/input.wasm
```

## 🚀 Deployment

### Deploy to Testnet

```bash
# Basic deployment
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_contract.wasm \
  --source alice \
  --network testnet

# Save the returned contract ID
# Example output: CA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAXE
```

### Deploy with Specific Contract ID

```bash
# Install the contract with a specific ID
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/my_contract.wasm \
  --source alice \
  --network testnet
```

### Deploy to Mainnet

```bash
# ⚠️ CAUTION: This deploys to production!

# Ensure you're ready:
# - Contract is thoroughly tested
# - Security audit completed
# - Sufficient XLM balance
# - Emergency response plan in place

soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_contract.wasm \
  --source mainnet-key \
  --network mainnet
```

## ✅ Verify Deployment

### Check Contract Info

```bash
# Get contract details
soroban contract info \
  --id <CONTRACT_ID> \
  --network testnet

# Fetch contract WASM
soroban contract fetch \
  --id <CONTRACT_ID> \
  --network testnet \
  --out-file fetched.wasm
```

### Test Contract Functions

```bash
# Invoke a contract function
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  function_name \
  --arg1 value1 \
  --arg2 value2
```

## 🔄 Contract Upgrades

### Upgradeable Contracts

Soroban supports contract upgrades through a proxy pattern:

```rust
// In your upgradeable contract
pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
    // Check authorization
    let admin: Address = env.storage().instance()
        .get(&symbol_short!("admin"))
        .unwrap();
    admin.require_auth();

    // Update contract code
    env.deployer().update_current_contract_wasm(new_wasm_hash);
}
```

### Upgrade Process

```bash
# 1. Install new WASM
NEW_WASM_HASH=$(soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/new_version.wasm \
  --source alice \
  --network testnet)

# 2. Call upgrade function
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  upgrade \
  --new_wasm_hash $NEW_WASM_HASH
```

## 📊 Cost Estimation

### Estimating Fees

```bash
# Simulate a transaction to see costs
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  my_function \
  --arg 123
```

### Fee Components

Soroban fees include:

- **Network Fee**: Base transaction fee
- **Resource Fee**: CPU, memory, storage costs
- **Rent**: Storage persistence (TTL extensions)

```bash
# View detailed fee breakdown
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  --fee 1000000 \
  -- \
  my_function
```

## 🔍 Monitoring

### View Transaction History

```bash
# Get recent transactions for an address
stellar-cli events \
  --start-ledger <LEDGER> \
  --id <CONTRACT_ID> \
  --network testnet
```

### Event Monitoring

```bash
# Subscribe to contract events
soroban events \
  --start-ledger <LEDGER> \
  --id <CONTRACT_ID> \
  --network testnet
```

## 🛡️ Security Checklist

Before mainnet deployment:

- [ ] All tests passing
- [ ] Security audit completed
- [ ] Code review by multiple developers
- [ ] Upgrade mechanism tested (if applicable)
- [ ] Emergency pause/stop mechanism (if needed)
- [ ] Documentation complete
- [ ] Monitoring and alerts configured
- [ ] Backup and recovery plan
- [ ] Legal compliance reviewed
- [ ] Bug bounty program considered

## 🚨 Emergency Procedures

### Pause Contract

Implement a pause function:

```rust
pub fn pause(env: Env) {
    require_admin(&env);
    env.storage().instance().set(&symbol_short!("paused"), &true);
}

pub fn unpause(env: Env) {
    require_admin(&env);
    env.storage().instance().remove(&symbol_short!("paused"));
}
```

### Emergency Contact

Have a plan for:

- User communication (Discord, Twitter, email)
- Incident response team
- Coordinated disclosure process

## 📚 Additional Resources

- [Soroban CLI Reference](https://developers.stellar.org/docs/tools/developer-tools/cli)
- [Network Configuration](https://developers.stellar.org/docs/networks)
- [Fee Documentation](https://developers.stellar.org/docs/smart-contracts/fees)
- [State Archival](https://developers.stellar.org/docs/smart-contracts/state-archival)

## 🤝 Getting Help

- [Stellar Discord](https://discord.gg/stellardev)
- [Developer Documentation](https://developers.stellar.org)
- [GitHub Issues](https://github.com/stellar/soroban-cli/issues)

---

**Deploy with confidence!** Test thoroughly on testnet before moving to mainnet.

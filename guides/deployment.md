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

## Monitoring & Alerting

Deploying a contract is not the end of the operational lifecycle — a mainnet
contract needs ongoing monitoring so the team hears about problems before
users do.

### What to Monitor

| Signal | Why it matters | How to check |
| --- | --- | --- |
| Invocation failures | Spikes usually mean a bad client release, an auth regression, or an upstream RPC issue | Poll `soroban events` or a Horizon/RPC operations feed for the contract ID and track failure rate over time |
| Storage TTL / rent | Persistent or instance entries that expire mid-operation cause unexpected `MissingValue` panics | Track `env.storage().*().get_ttl()` output for key entries, or query ledger entry TTL via RPC `getLedgerEntries` |
| Admin / fee-payer account balance | A drained fee-payer account blocks every invocation until refunded | Check balance with `soroban keys balance <identity> --network mainnet` on a schedule |
| Contract paused state | Confirms an emergency pause (see [Pause / Unpause Pattern](#pause--unpause-pattern)) was intentional and not accidental | Invoke a read-only status/`paused` getter and compare against the expected state |
| CI/CD pipeline health | A red pipeline on `main` means the next deploy is blocked | Watch the **Test and Lint** and **Deploy Docs** workflow runs described in `.github/workflows/README.md` |

### Dashboards

- **Stellar Expert** (`https://stellar.expert`) and **Stellar Laboratory** — inspect contract invocations, events, and balances for a given contract ID without any extra setup.
- **GitHub Actions** — the *Actions* tab on the repository is the dashboard of record for build/test/deploy health; failing runs on `main` should be treated as a P1 signal.
- **Codecov** — the coverage dashboard configured in `codecov.yml` flags coverage regressions on each PR.

For teams that need alerting rather than manual dashboard checks, a small
cron job or scheduled Action that polls `soroban events` / RPC and posts to
your existing chat/pager tooling on threshold breaches is enough to start —
there is no dependency on a specific vendor.

### Alert Response Runbook

1. **Acknowledge** — confirm the alert is real (check the dashboard/RPC query
   named in the "How to check" column above) before taking action.
2. **Assess blast radius** — is this one failing invocation type, or every
   call to the contract? Check recent events for the contract ID to see
   whether it correlates with a specific function or caller.
3. **Contain if needed** — if funds or state integrity are at risk, use the
   [Pause / Unpause Pattern](#pause--unpause-pattern) to stop further writes
   while you investigate.
4. **Diagnose** — reproduce against Testnet with the same inputs where
   possible, and check the [Fee Estimation](#fee-estimation) simulate output
   for resource/fee-related failures.
5. **Resolve** — deploy a fix via the normal [Mainnet Deployment
   Steps](#mainnet-deployment-steps), or extend TTL for entries close to
   expiry using `extend_ttl`.
6. **Unpause and verify** — once the fix is confirmed on Testnet and deployed,
   unpause and re-run the read-only checks from the monitoring table above.
7. **Record** — note the trigger, timeline, and resolution somewhere durable
   (issue tracker or incident log) so the next on-call has context.

---

## Additional Resources

- [Soroban CLI Reference](https://developers.stellar.org/docs/tools/developer-tools/cli)
- [Network Configuration](https://developers.stellar.org/docs/networks)
- [Fee Documentation](https://developers.stellar.org/docs/smart-contracts/fees)
- [State Archival & TTL](https://developers.stellar.org/docs/smart-contracts/state-archival)
- [Stellar Discord](https://discord.gg/stellardev)

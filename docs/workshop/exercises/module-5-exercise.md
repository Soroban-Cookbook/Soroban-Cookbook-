# Module 5 Exercise: Advanced Patterns & Deployment

**Objective:** Study advanced patterns and deploy a contract to Stellar testnet.

**Duration:** 60 minutes

---

## Prerequisites

- Complete Modules 1-4
- A funded Stellar testnet account (see Part 3 below)

---

## Reference Examples

- [`examples/advanced/04-upgradeable-proxy/`](../../../examples/advanced/04-upgradeable-proxy/) — Proxy pattern
- [`examples/advanced/03-oracle-pattern/`](../../../examples/advanced/03-oracle-pattern/) — Oracle
- [`examples/advanced/02-timelock/`](../../../examples/advanced/02-timelock/) — Timelock
- [`examples/advanced/06-gas-optimization/`](../../../examples/advanced/06-gas-optimization/) — Gas optimization

---

## Part 1: Study Advanced Patterns (15 min)

Read and understand these patterns from the cookbook:

### 1.1 Upgradeable Proxy

Read [`examples/advanced/04-upgradeable-proxy/src/proxy.rs`](../../../examples/advanced/04-upgradeable-proxy/src/proxy.rs).

Questions to answer:
- How does the proxy store the implementation address?
- What happens when `upgrade()` is called?
- How does `__invoke` forward calls to the implementation?

### 1.2 Oracle Pattern

Read [`examples/advanced/03-oracle-pattern/src/lib.rs`](../../../examples/advanced/03-oracle-pattern/src/lib.rs).

Questions:
- How does `get_value_strict()` handle stale data?
- What role does the `updater` address play?
- How is freshness calculated?

### 1.3 Timelock

Read [`examples/advanced/02-timelock/src/lib.rs`](../../../examples/advanced/02-timelock/src/lib.rs).

Questions:
- What is the delay range and why does it exist?
- How are operations queued and executed?
- What prevents execution before the delay expires?

---

## Part 2: Implement a Simple Oracle (20 min)

Create a simplified price oracle contract with these requirements:

1. Admin can set the price updater
2. Updater can submit prices with a timestamp
3. Anyone can read the latest price
4. `get_price_strict()` rejects data older than a configurable `max_age`

### Starter Code

```rust
#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env};

// TODO: Define DataKey (Admin, Updater, Price, Timestamp, MaxAge)
// TODO: Define OracleError enum
// TODO: Define PriceUpdatedEvent struct

#[contract]
pub struct PriceOracle;

#[contractimpl]
impl PriceOracle {
    // TODO: initialize(env, admin: Address, max_age: u64) -> Result<(), OracleError>
    // TODO: set_updater(env, admin: Address, updater: Address) -> Result<(), OracleError>
    // TODO: submit_price(env, updater: Address, price: i128) -> Result<(), OracleError>
    //   - Require auth, check updater
    //   - Store price and current ledger timestamp
    //   - Emit event
    // TODO: get_price(env) -> Option<i128>
    //   - Return latest price (or None if never set)
    // TODO: get_price_strict(env) -> Result<i128, OracleError>
    //   - Return price if timestamp is within max_age
    //   - Return StaleData error otherwise
    // TODO: get_timestamp(env) -> Option<u64>
}
```

### Test Cases

```rust
#[test]
fn test_submit_and_get_price() { /* ... */ }

#[test]
fn test_submit_unauthorized() { /* ... */ }

#[test]
fn test_strict_rejects_stale() {
    // Submit price
    // Advance ledger time beyond max_age
    // get_price_strict() should return StaleData error
}

#[test]
fn test_strict_accepts_fresh() { /* ... */ }

#[test]
fn test_get_price_none_when_empty() { /* ... */ }
```

> **Hint for advancing ledger time in tests:** `env.ledger().set_timestamp(timestamp)` or `env.ledger().with_mut(|l| l.timestamp = ...)`.

---

## Part 3: Deploy to Testnet (25 min)

### 3.1 Configure the Testnet

```bash
# Add testnet network configuration
stellar network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

### 3.2 Create a Test Account

```bash
# Generate a new keypair
stellar keys generate --global testnet
```

This prints your public address. Fund it using the [Stellar Friendbot](https://friendbot.stellar.org):

1. Go to https://friendbot.stellar.org
2. Paste your public address
3. Click "Fund"

### 3.3 Build Your Contract

Use any contract you've built during this workshop (e.g., the counter from Module 2 or the token from Module 4).

```bash
# From the contract directory
cargo build --target wasm32-unknown-unknown --release
```

### 3.4 Deploy

```bash
# Find your WASM file
ls target/wasm32-unknown-unknown/release/*.wasm

# Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source testnet \
  --network testnet

# Save the returned contract ID!
# Example output: CA3D5KRYM6CB7OWQ6TWRA4JW...
```

### 3.5 Initialize

```bash
# For a token contract:
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network testnet \
  -- initialize \
     --admin <YOUR_ADDRESS> \
     --name "WorkshopToken" \
     --symbol TST \
     --decimals 7 \
     --initial_supply 1000000000
```

### 3.6 Interact

```bash
# Check balance
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network testnet \
  -- balance --user <YOUR_ADDRESS>

# Transfer tokens
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network testnet \
  -- transfer --from <YOUR_ADDRESS> --to <RECIPIENT> --amount 1000
```

---

## Verification Checklist

- [ ] You can explain how the upgradeable proxy pattern works
- [ ] You can explain how the oracle staleness check works
- [ ] You can explain how the timelock delay works
- [ ] Your simple oracle contract compiles and passes tests
- [ ] `get_price_strict()` rejects stale data correctly
- [ ] You deployed a contract to Stellar testnet
- [ ] You initialized the deployed contract
- [ ] You successfully called a function on the deployed contract

---

## Solution for Part 2 (Simple Oracle)

<details>
<summary>Click to reveal the solution</summary>

```rust
#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Updater,
    Price,
    Timestamp,
    MaxAge,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum OracleError {
    NotInitialized = 1,
    Unauthorized = 2,
    NoData = 3,
    StaleData = 4,
    AlreadyInitialized = 5,
}

#[contracttype]
#[derive(Clone)]
pub struct PriceUpdatedEvent {
    pub price: i128,
    pub timestamp: u64,
}

#[contract]
pub struct PriceOracle;

#[contractimpl]
impl PriceOracle {
    pub fn initialize(env: Env, admin: Address, max_age: u64) -> Result<(), OracleError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(OracleError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::MaxAge, &max_age);
        Ok(())
    }

    pub fn set_updater(env: Env, admin: Address, updater: Address) -> Result<(), OracleError> {
        admin.require_auth();
        require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::Updater, &updater);
        Ok(())
    }

    pub fn submit_price(env: Env, updater: Address, price: i128) -> Result<(), OracleError> {
        updater.require_auth();
        let stored_updater: Address = env.storage()
            .instance().get(&DataKey::Updater).ok_or(OracleError::NotInitialized)?;
        if updater != stored_updater {
            return Err(OracleError::Unauthorized);
        }
        let timestamp = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::Price, &price);
        env.storage().persistent().set(&DataKey::Timestamp, &timestamp);
        env.events().publish(
            (symbol_short!("oracle"), symbol_short!("update")),
            PriceUpdatedEvent { price, timestamp },
        );
        Ok(())
    }

    pub fn get_price(env: Env) -> Option<i128> {
        env.storage().persistent().get(&DataKey::Price)
    }

    pub fn get_price_strict(env: Env) -> Result<i128, OracleError> {
        let price: i128 = env.storage().persistent().get(&DataKey::Price)
            .ok_or(OracleError::NoData)?;
        let timestamp: u64 = env.storage().persistent().get(&DataKey::Timestamp)
            .ok_or(OracleError::NoData)?;
        let max_age: u64 = env.storage().instance().get(&DataKey::MaxAge)
            .ok_or(OracleError::NotInitialized)?;
        let now = env.ledger().timestamp();
        if now.saturating_sub(timestamp) > max_age {
            return Err(OracleError::StaleData);
        }
        Ok(price)
    }

    pub fn get_timestamp(env: Env) -> Option<u64> {
        env.storage().persistent().get(&DataKey::Timestamp)
    }
}

fn require_admin(env: &Env, addr: &Address) -> Result<(), OracleError> {
    let admin: Address = env.storage()
        .instance().get(&DataKey::Admin).ok_or(OracleError::NotInitialized)?;
    if addr != &admin {
        return Err(OracleError::Unauthorized);
    }
    Ok(())
}
```

Test for stale data:

```rust
#[test]
fn test_strict_rejects_stale() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PriceOracle);
    let client = PriceOracleClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let updater = Address::generate(&env);

    env.mock_auths(&[MockAuth { address: &admin, invoke: &MockAuthInvoke {
        contract: &contract_id, fn_name: "initialize",
        args: (&admin, &100u64), sub_invokes: &[],
    }}]);
    client.initialize(&admin, &100u64);

    env.mock_auths(&[MockAuth { address: &admin, invoke: &MockAuthInvoke {
        contract: &contract_id, fn_name: "set_updater",
        args: (&admin, &updater), sub_invokes: &[],
    }}]);
    client.set_updater(&admin, &updater);

    env.mock_auths(&[MockAuth { address: &updater, invoke: &MockAuthInvoke {
        contract: &contract_id, fn_name: "submit_price",
        args: (&updater, &100i128), sub_invokes: &[],
    }}]);
    client.submit_price(&updater, &100i128);

    // Price should be fresh
    assert_eq!(client.get_price_strict(), Ok(100i128));

    // Advance time past max_age
    let now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + 200);

    // Now stale
    assert_eq!(client.get_price_strict(), Err(OracleError::StaleData));
}
```

</details>

---

## Workshop Complete!

You've now built, tested, and deployed Soroban smart contracts. Here's where to go next:

- **Explore the Cookbook:** 80+ examples at [`examples/`](../../../examples/)
- **Read the Guides:** [`guides/`](../../../guides/) for deployment, testing, and more
- **Join the Community:** [Stellar Discord](https://discord.gg/stellardev)
- **Contribute:** See [CONTRIBUTING.md](../../../CONTRIBUTING.md)
- **Build a Project:** Use [templates](../../../templates/) as starters

Congratulations on completing the Soroban Development Workshop! 🎉

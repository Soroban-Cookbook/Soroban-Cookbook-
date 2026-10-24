---
marp: true
theme: default
paginate: true
---

# Module 5: Advanced Patterns & Deployment

## Soroban Development Workshop

---

# Upgradeable Proxy Pattern

Contracts on Soroban are immutable once deployed. To upgrade logic, use a proxy that delegates to an implementation.

```
User → Proxy Contract → Implementation Contract (v1)
                      ↘ Implementation Contract (v2)
```

```rust
// Proxy stores implementation address in instance storage
pub fn upgrade(env: Env, admin: Address, new_impl: Address) {
    admin.require_auth();
    require_admin(&env, &admin)?;
    env.storage().instance().set(&DataKey::Implementation, &new_impl);
}

// Proxy forwards all calls to implementation
pub fn invoke(env: Env, args: Vec<Val>) -> Val {
    let impl_addr = env.storage().instance().get(&DataKey::Implementation).unwrap();
    // ... delegate call to implementation
}
```

Source: [`examples/advanced/04-upgradeable-proxy/`](../../examples/advanced/04-upgradeable-proxy/)

---

# Beacon Proxy Pattern

Share a single implementation across multiple proxy instances.

```
Proxy A → Beacon → Implementation v1
Proxy B → Beacon → Implementation v1
Proxy C → Beacon → Implementation v1

Upgrade beacon → all proxies now use Implementation v2
```

Source: [`examples/advanced/02-beacon-proxy/`](../../examples/advanced/02-beacon-proxy/)

---

# Factory Pattern

Deploy contracts programmatically from a factory.

```rust
pub fn create_instance(env: Env, admin: Address, params: Params) -> Address {
    admin.require_auth();

    // Deploy a new contract
    let contract_id = env.register_contract(None, MyContract);

    // Initialize it
    let client = MyContractClient::new(&env, &contract_id);
    client.initialize(&admin, &params);

    // Track it
    let mut instances = get_deployed_instances(&env);
    instances.push(contract_id.clone());
    env.storage().persistent().set(&DataKey::Instances, &instances);

    contract_id
}
```

Source: [`examples/intermediate/ajo-factory/`](../../examples/intermediate/ajo-factory/)

---

# Contract Registry

Track deployed contracts for discovery.

```rust
pub fn register(env: Env, name: Symbol, contract_addr: Address, category: Symbol) {
    let metadata = ContractMetadata {
        address: contract_addr,
        category: category.clone(),
        registered_at: env.ledger().timestamp(),
    };
    env.storage().persistent().set(
        &DataKey::Contract(name.clone()),
        &metadata,
    );
}

pub fn get_by_name(env: Env, name: Symbol) -> Option<ContractMetadata> {
    env.storage().persistent().get(&DataKey::Contract(name))
}
```

Source: [`examples/advanced/contract-registry/`](../../examples/advanced/contract-registry/)

---

# Oracle Pattern

Bring external data on-chain.

```rust
pub fn submit(env: Env, updater: Address, value: i128) -> Result<(), OracleError> {
    updater.require_auth();
    require_updater(&env, &updater)?;

    env.storage().persistent().set(&DataKey::Value, &value);
    env.storage().persistent().set(&DataKey::Timestamp, &env.ledger().timestamp());
    Ok(())
}

pub fn get_value(env: Env) -> Option<i128> {
    env.storage().persistent().get(&DataKey::Value)
}

// Strict mode: reject stale data
pub fn get_value_strict(env: Env, max_age: u64) -> Result<i128, OracleError> {
    let timestamp = env.storage().persistent().get(&DataKey::Timestamp)
        .ok_or(OracleError::NoData)?;
    let now = env.ledger().timestamp();
    if now - timestamp > max_age {
        return Err(OracleError::StaleData);
    }
    get_value(&env).ok_or(OracleError::NoData)
}
```

Source: [`examples/advanced/03-oracle-pattern/`](../../examples/advanced/03-oracle-pattern/)

---

# Reentrancy Guard

Soroban contracts are generally not reentrant, but defensive coding is still valuable.

```rust
pub fn withdraw(env: Env, caller: Address, amount: i128) {
    caller.require_auth();

    // Reentrancy check
    if env.storage().instance().get(&DataKey::Entered).unwrap_or(false) {
        panic!("Reentrancy detected");
    }
    env.storage().instance().set(&DataKey::Entered, &true);

    // ... perform withdrawal ...

    env.storage().instance().set(&DataKey::Entered, &false);
}
```

Source: [`examples/advanced/05-reentrancy-guard/`](../../examples/advanced/05-reentrancy-guard/)

---

# Timelock Pattern

Delay execution of sensitive operations for governance safety.

```rust
pub fn queue(env: Env, caller: Address, op_id: Symbol) -> Result<(), TimelockError> {
    caller.require_auth();
    require_admin(&env, &caller)?;

    let execute_at = env.ledger().timestamp() + DELAY;
    env.storage().persistent().set(&DataKey::Operation(op_id), &OperationState::Queued(execute_at));
    Ok(())
}

pub fn execute(env: Env, caller: Address, op_id: Symbol) -> Result<(), TimelockError> {
    caller.require_auth();
    let state = get_state(&env, &op_id)?;

    match state {
        OperationState::Queued(execute_at) => {
            if env.ledger().timestamp() < execute_at {
                return Err(TimelockError::TooEarly);
            }
            // ... execute the operation ...
        }
        _ => return Err(TimelockError::NotQueued),
    }
    Ok(())
}
```

Source: [`examples/advanced/02-timelock/`](../../examples/advanced/02-timelock/)

---

# Gas Optimization Techniques

1. **Batch operations** — one transaction instead of N
2. **Cache storage reads** — read once, reuse
3. **Use instance storage for config** — cheaper than persistent
4. **Short-circuit evaluation** — check cheap conditions first
5. **Symbol interning** — use `Symbol` instead of `String` for identifiers
6. **Struct packing** — combine small fields into a single struct

```rust
// Before: 3 separate storage reads
let paused = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
let fee = env.storage().instance().get(&DataKey::Fee).unwrap_or(0);
let admin = env.storage().instance().get(&DataKey::Admin).unwrap();

// After: 1 read with a packed config struct
let config: Config = env.storage().instance().get(&DataKey::Config).unwrap();
// config.paused, config.fee, config.admin all in one read
```

Source: [`examples/advanced/06-gas-optimization/`](../../examples/advanced/06-gas-optimization/)

---

# Deployment to Testnet

```bash
# 1. Build the contract
cargo build --target wasm32-unknown-unknown --release

# 2. Configure testnet
stellar network add --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# 3. Generate a test account
stellar keys generate --global testnet

# 4. Fund it from the faucet
# Visit https://friendbot.stellar.org and enter your address

# 5. Deploy the contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source testnet \
  --network testnet

# 6. Initialize the contract
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network testnet \
  -- initialize --admin <YOUR_ADDRESS> --name "MyToken" --symbol TST --decimals 7 --initial_supply 1000000000
```

---

# Deployed Contract Interaction

```bash
# Check balance
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network testnet \
  -- balance --user <USER_ADDRESS>

# Transfer tokens
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network testnet \
  -- transfer --from <FROM_ADDR> --to <TO_ADDR> --amount 1000
```

The CLI handles signing, XDR encoding, and RPC submission automatically.

---

# Testing Best Practices

```rust
// 1. Always test the happy path
#[test]
fn test_transfer_success() { /* ... */ }

// 2. Test failure paths
#[test]
fn test_insufficient_balance_fails() { /* ... */ }

// 3. Test authorization
#[test]
fn test_unauthorized_transfer_fails() { /* ... */ }

// 4. Test edge cases
#[test]
fn test_zero_amount_fails() { /* ... */ }

#[test]
fn test_overflow_rejected() { /* ... */ }

// 5. Benchmark
#[test]
fn test_transfer_benchmark() {
    env.budget().reset_default();
    let _ = client.transfer(&from, &to, &100);
    env.budget().print();
}
```

Source: [`docs/testing-guide.md`](../../docs/testing-guide.md)

---

# Security Checklist

Before deploying to production:

- [ ] All mutating functions call `require_auth()`
- [ ] Initialization is guarded against re-initialization
- [ ] Arithmetic uses `checked_add` / `checked_sub`
- [ ] Amounts are validated as positive
- [ ] Access control checks happen before state changes
- [ ] Events emitted for all state changes
- [ ] Storage TTL is managed for persistent data
- [ ] All error paths return typed errors (not panics)
- [ ] Tests cover both success and failure cases

---

# Module 5 Summary

- Proxy patterns enable contract upgradeability
- Factory pattern for deploying multiple instances
- Oracle pattern for external data with staleness checks
- Timelock for governance-safe delayed execution
- Gas optimization: batch, cache, pack structs
- Deploy with `stellar contract deploy` to testnet
- Interact with `stellar contract invoke`
- Security checklist: auth, overflow, validation, events, TTL

---

# Workshop Wrap-Up

## What You've Learned

1. ✅ Set up a complete Soroban development environment
2. ✅ Written and tested smart contracts in Rust
3. ✅ Used all three storage types effectively
4. ✅ Implemented authentication and access control
5. ✅ Created a SEP-41-compliant token
6. ✅ Applied advanced patterns (proxy, oracle, timelock)
7. ✅ Deployed contracts to Stellar testnet

---

# Where to Go Next

- **Study the Cookbook** — 80+ examples at [`examples/`](../../examples/)
- **Read the Guides** — [`guides/`](../../guides/) has deployment, testing, and Ethereum migration guides
- **Join the Community** — [Stellar Discord](https://discord.gg/stellardev), GitHub Discussions
- **Contribute** — See [CONTRIBUTING.md](../../CONTRIBUTING.md) for how to add examples
- **Build a Project** — Use the [templates](../../templates/) as starting points

---

# Exercise Time!

See [Exercise 5](../exercises/module-5-exercise.md)

**Goal:** Deploy a contract to testnet and interact with it.

⏱️ 60 minutes

---

# Thank You!

**Soroban Development Workshop**

Questions? Ask the facilitator or open a [GitHub Discussion](https://github.com/gloriaibrahim2002-blip/Soroban-Cookbook-/discussions).

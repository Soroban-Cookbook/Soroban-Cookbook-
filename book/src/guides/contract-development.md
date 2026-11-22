# Contract Development

This guide walks through building a Soroban contract from scratch: the
project layout, the contract skeleton, the four building blocks every
contract uses (storage, authentication, events and errors), how to test it,
and how to ship it.

Every snippet below is a trimmed version of a runnable example in this
repository.  See [Code Examples →](../docs/code-examples.md) for the full
index, and [Testing Guide →](./testing.md) for the test tooling.

---

## 1. Project Layout

A Soroban contract is a normal Rust crate.  The `soroban-sdk` dependency is
the only thing that makes it "a Soroban contract":

```
my-contract/
├── Cargo.toml
└── src/
    ├── lib.rs      # the contract
    └── test.rs     # the unit tests
```

```toml
# Cargo.toml
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
soroban-sdk = "29.0.0"

[dev-dependencies]
soroban-sdk = { version = "29.0.0", features = ["testutils"] }
```

`cdylib` is what the Stellar network executes; `rlib` is what lets other
crates (and your integration tests) link against the contract.

---

## 2. The Contract Skeleton

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Balance(Address),
}

#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn deposit(env: Env, from: Address, amount: i128) {
        // 1. authorize the caller
        from.require_auth();

        // 2. read the current state
        let mut balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        // 3. update the state
        balance += amount;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &balance);
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(user))
            .unwrap_or(0)
    }
}
```

Four rules hide in that skeleton:

1. `#[contract]` marks the struct as a deployable contract unit.
2. `#[contractimpl]` exposes the `pub fn`s as callable entry points.
3. `Env` is always the first parameter — it gives access to storage, events,
   crypto and the ledger.
4. Every state-changing function authorizes the address that owns the state.

---

## 3. The Four Building Blocks

### 3.1 Storage

```rust
// Persistent: survives upgrades, needs explicit TTL management.
let key = DataKey::Balance(user.clone());
env.storage().persistent().set(&key, &balance);
env.storage().persistent().extend_ttl(&key, 1000, 10000);

// Instance: tied to the contract instance (admin config, fees).
env.storage().instance().set(&symbol_short!("admin"), &admin);

// Temporary: current ledger only (reentrancy guards, caches).
env.storage().temporary().set(&symbol_short!("entered"), &true);
```

### 3.2 Authentication

```rust
// The owner of the funds signs for the transfer.
from.require_auth();

// Authorize only a specific set of arguments.
user.require_auth_for_args((&user, &amount).into_val(&env));

// Admin-gated calls: authorize the *stored* admin, not a parameter.
read_admin(&env)?.require_auth();
```

### 3.3 Events

```rust
env.events().publish(
    (symbol_short!("token"), symbol_short!("transfer"), from, to),
    amount,
);
```

Topics are indexed (maximum four per publish); everything else belongs in the
data payload.

### 3.4 Errors

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 1,
    InsufficientBalance = 2,
}

pub fn withdraw(env: Env, user: Address, amount: i128) -> Result<(), ContractError> {
    user.require_auth();
    let balance: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0);
    if balance < amount {
        return Err(ContractError::InsufficientBalance);
    }
    env.storage()
        .persistent()
        .set(&DataKey::Balance(user), &(balance - amount));
    Ok(())
}
```

---

## 4. Testing Your Contract

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn deposit_increases_balance() {
        let env = Env::default();
        env.mock_all_auths();

        let id = env.register_contract(None, MyContract);
        let client = MyContractClient::new(&env, &id);
        let alice = Address::generate(&env);

        client.deposit(&alice, &100);
        assert_eq!(client.balance(&alice), 100);

        // Typed errors are matched exactly.
        assert_eq!(
            client.try_withdraw(&alice, &1_000),
            Err(Ok(ContractError::InsufficientBalance))
        );
    }
}
```

```bash
cargo test
```

Cross-contract behaviour belongs in the integration suite:

```bash
cargo test -p integration-tests
```

See [Testing Guide →](./testing.md) for auth mocking, time travel, event
assertions and fuzzing.

---

## 5. Build and Deploy

```bash
# Build the Wasm artifact the network will run.
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet with the Stellar CLI.
soroban contract deploy \
    --network testnet \
    --source my-account \
    --wasm target/wasm32-unknown-unknown/release/my_contract.wasm
```

Full instructions: [Deployment Guide →](./deployment.md).

---

## 6. Review Before You Ship

- [Security Checklist →](./security-checklist.md)
- [Best Practices →](../docs/best-practices.md)
- [Common Pitfalls →](../docs/common-pitfalls.md)
- [Code Examples →](../docs/code-examples.md)

---

## Next Steps

- [Interactive Playground — Intermediate Examples →](../examples/playground-intermediate.md)
- [Testing Best Practices →](../docs/testing-best-practices.md)
- [Factory, Proxy, and Registry Patterns →](../docs/cross-contract-patterns.md)

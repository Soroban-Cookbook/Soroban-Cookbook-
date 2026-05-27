# Soroban Quick Reference

A quick reference for common Soroban smart contract patterns.

---

## Project Setup

```bash
# Create new contract project
cargo new --lib my-contract
cd my-contract
```

`Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
soroban-sdk = "21.7.0"

[dev-dependencies]
soroban-sdk = { version = "21.7.0", features = ["testutils"] }
```

## Basic Contract Structure

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn hello(env: Env, name: soroban_sdk::String) -> soroban_sdk::String {
        soroban_sdk::String::from_str(&env, "Hello!")
    }
}
```

---

## Common Types and Conversions

### Primitive Types

```rust
let a: u32  = 100;
let b: u64  = 1_000_000;
let c: i128 = -500;
let d: bool = true;
```

### Soroban-Specific Types

```rust
use soroban_sdk::{Address, Bytes, BytesN, Map, String, Symbol, Vec};

// Address — a Stellar account or contract address
let addr: Address = Address::from_string(&String::from_str(&env, "GABC...XYZ"));

// Symbol — short identifiers (≤9 chars use the macro)
let short = symbol_short!("transfer");          // up to 9 chars
let long  = Symbol::new(&env, "long_key_name"); // any length

// String
let s = String::from_str(&env, "Hello, Soroban!");

// Bytes (variable length)
let b = Bytes::from_slice(&env, &[0x01, 0x02, 0x03]);

// BytesN (fixed length, e.g. 32-byte hash)
let hash: BytesN<32> = BytesN::from_array(&env, &[0u8; 32]);

// Vec
let v: Vec<u64> = vec![&env, 1, 2, 3];

// Map
let mut m: Map<Symbol, u64> = Map::new(&env);
m.set(symbol_short!("key"), 42u64);
let val = m.get(symbol_short!("key")).unwrap();
```

### Type Conversions

```rust
// u32 <-> u64
let x: u64 = 42u32 as u64;
let y: u32 = 42u64 as u32;

// i128 arithmetic (common for token amounts)
let amount: i128 = 1_000_000_000; // 100 XLM in stroops

// Address to/from String
let addr_str = String::from_str(&env, "GABC...XYZ");
let addr = Address::from_string(&addr_str);

// Bytes to BytesN
let fixed: BytesN<4> = BytesN::from_array(&env, &[1, 2, 3, 4]);
```

### Math (use checked ops to avoid panics)

```rust
let sum  = a.checked_add(b).expect("overflow");
let diff = a.checked_sub(b).expect("underflow");
let prod = a.checked_mul(b).expect("overflow");
let quot = a.checked_div(b).expect("division by zero");

// Saturating (clamps instead of panicking)
let clamped = a.saturating_add(u64::MAX);
```

---

## Storage Patterns

Soroban has three storage tiers with different TTL (time-to-live) behaviors.

### Persistent Storage

Use for long-lived data (balances, ownership records).

```rust
// Write
env.storage().persistent().set(&DataKey::Balance(addr.clone()), &amount);

// Read
let balance: i128 = env.storage().persistent()
    .get(&DataKey::Balance(addr.clone()))
    .unwrap_or(0);

// Check existence
let exists = env.storage().persistent().has(&DataKey::Balance(addr.clone()));

// Delete
env.storage().persistent().remove(&DataKey::Balance(addr.clone()));

// Extend TTL (threshold, extend_to in ledgers)
env.storage().persistent().extend_ttl(&DataKey::Balance(addr.clone()), 100, 500);
```

### Instance Storage

Use for contract-level config (admin, settings). TTL is tied to the contract instance.

```rust
env.storage().instance().set(&symbol_short!("admin"), &admin_addr);

let admin: Address = env.storage().instance()
    .get(&symbol_short!("admin"))
    .unwrap();

// Extend the contract instance TTL
env.storage().instance().extend_ttl(100, 500);
```

### Temporary Storage

Use for short-lived data (nonces, session state). Automatically archived after TTL.

```rust
env.storage().temporary().set(&symbol_short!("nonce"), &nonce_val);

let nonce: u64 = env.storage().temporary()
    .get(&symbol_short!("nonce"))
    .unwrap_or(0);

env.storage().temporary().extend_ttl(&symbol_short!("nonce"), 50, 100);
```

### Common Storage Key Pattern

```rust
#[contracttype]
pub enum DataKey {
    Admin,
    Balance(Address),
    Allowance(Address, Address),
}
```

---

## Authentication Patterns

### Require Auth from an Address

```rust
// Caller must sign the transaction authorizing this call
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();
    // ... transfer logic
}
```

### Require Auth with Specific Arguments

```rust
use soroban_sdk::auth::ContractContext;

pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
    owner.require_auth_for_args((spender.clone(), amount).into_val(&env));
    // ... approval logic
}
```

### Admin-Only Functions

```rust
fn require_admin(env: &Env) -> Address {
    let admin: Address = env.storage().instance()
        .get(&symbol_short!("admin"))
        .expect("not initialized");
    admin.require_auth();
    admin
}

pub fn set_fee(env: Env, new_fee: u32) {
    require_admin(&env);
    env.storage().instance().set(&symbol_short!("fee"), &new_fee);
}
```

### Initialization Guard

```rust
pub fn initialize(env: Env, admin: Address) {
    if env.storage().instance().has(&symbol_short!("init")) {
        panic!("already initialized");
    }
    env.storage().instance().set(&symbol_short!("admin"), &admin);
    env.storage().instance().set(&symbol_short!("init"), &true);
}
```

### Testing Auth

```rust
#[test]
fn test_admin_only() {
    let env = Env::default();
    env.mock_all_auths(); // auto-approve all auth checks in tests

    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);
    client.set_fee(&admin, &50u32);
}
```

---

## Event Patterns

### Emit a Simple Event

```rust
// topic tuple + data value
env.events().publish(
    (symbol_short!("transfer"),),
    amount,
);
```

### Emit with Multiple Topics

```rust
// Up to 4 topics; topics are indexed for filtering
env.events().publish(
    (symbol_short!("transfer"), from.clone(), to.clone()),
    amount,
);
```

### Structured Event Data

```rust
#[contracttype]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

env.events().publish(
    (symbol_short!("transfer"),),
    TransferEvent { from, to, amount },
);
```

### Reading Events in Tests

```rust
#[test]
fn test_events() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    client.transfer(&from, &to, &1000i128);

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    // Inspect the event
    let (topics, data): (soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val) =
        events.first().unwrap();
}
```

### Querying Events On-Chain (CLI)

```bash
soroban events \
  --start-ledger <LEDGER> \
  --id <CONTRACT_ID> \
  --network testnet
```

---

## Error Handling

### Define Contract Errors

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized   = 1,
    AlreadyExists    = 2,
    InvalidAmount    = 3,
    Unauthorized     = 4,
    InsufficientFunds = 5,
}
```

### Return Results

```rust
pub fn transfer(env: Env, from: Address, amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    let balance = get_balance(&env, &from);
    if balance < amount {
        return Err(Error::InsufficientFunds);
    }
    // ...
    Ok(())
}
```

---

## Testing

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_transfer() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, MyContract);
        let client = MyContractClient::new(&env, &contract_id);

        let alice = Address::generate(&env);
        let bob   = Address::generate(&env);

        client.initialize(&alice);
        client.mint(&alice, &1000i128);
        client.transfer(&alice, &bob, &500i128);

        assert_eq!(client.balance(&bob), 500i128);
    }
}
```

### Manipulate Ledger Time in Tests

```rust
env.ledger().with_mut(|li| {
    li.timestamp = 1_700_000_000;
    li.sequence  = 1000;
});
```

---

## Build & Deploy

```bash
# Build
soroban contract build

# Deploy to testnet
CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_contract.wasm \
  --source alice \
  --network testnet)

# Invoke
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

## Useful Macros

```rust
symbol_short!("short")          // Symbol ≤ 9 chars
Symbol::new(&env, "long_name")  // Symbol any length
vec![&env, 1u64, 2, 3]         // Vec<u64>
bytes![&env, 0x01, 0x02]        // Bytes
map![&env, (k1, v1), (k2, v2)] // Map
```

---

## Resources

- [Soroban Developer Docs](https://developers.stellar.org/docs/smart-contracts)
- [soroban-sdk API Reference](https://docs.rs/soroban-sdk/)
- [Examples](../examples/)
- [Deployment Guide](../guides/deployment.md)
- [Stellar Discord](https://discord.gg/stellardev)

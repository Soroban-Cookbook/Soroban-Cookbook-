# Soroban Quick Reference

A quick reference guide for Soroban smart contract development.

## 📦 Project Setup

```bash
# Create new project
cargo new --lib my-contract
cd my-contract

# Add to Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
soroban-sdk = "21.7.0"

[dev-dependencies]
soroban-sdk = { version = "21.7.0", features = ["testutils"] }
```

## 🏗️ Basic Contract Structure

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn my_function(env: Env, arg: u64) -> u64 {
        arg * 2
    }
}
```

## 💾 Storage Operations

### Persistent Storage

```rust
// Set
env.storage().persistent().set(&key, &value);

// Get
let value: u64 = env.storage().persistent().get(&key).unwrap();

// Has
let exists: bool = env.storage().persistent().has(&key);

// Remove
env.storage().persistent().remove(&key);

// Extend TTL
env.storage().persistent().extend_ttl(&key, 100, 100);
```

### Temporary Storage

```rust
env.storage().temporary().set(&key, &value);
let value: u64 = env.storage().temporary().get(&key).unwrap();
```

### Instance Storage

```rust
env.storage().instance().set(&key, &value);
let value: u64 = env.storage().instance().get(&key).unwrap();
env.storage().instance().extend_ttl(100, 100);
```

## 🔐 Authorization

```rust
// Require authorization from an address
address.require_auth();

// Require authorization with specific arguments
address.require_auth_for_args(args);

// In tests - mock all auths
env.mock_all_auths();
```

## 📢 Events

```rust
// Simple event
env.events().publish((symbol_short!("event"),), value);

// Event with topics (up to 4)
env.events().publish(
    (symbol_short!("transfer"), from.clone(), to.clone()),
    amount
);

// In tests - get events
let events = env.events().all();
```

## ❌ Error Handling

### Define Errors

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    Unauthorized = 2,
    AlreadyInitialized = 3,
}
```

### Use Errors

```rust
// Return Result
pub fn transfer(env: Env, amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    Ok(())
}

// Or panic
if amount <= 0 {
    panic!("Invalid amount");
}
```

## 🧪 Testing

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, testutils::Address as _};

    #[test]
    fn test_function() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, MyContract);
        let client = MyContractClient::new(&env, &contract_id);

        let result = client.my_function(&42);
        assert_eq!(result, 84);
    }
}
```

## 📊 Common Data Types

### Primitives

```rust
let a: u32 = 100;
let b: u64 = 1000;
let c: i128 = -500;
let d: bool = true;
```

### Soroban Types

```rust
use soroban_sdk::{Address, Symbol, String, Bytes, BytesN, Vec, Map};

// Address
let addr = Address::from_string(&String::from_str(&env, "G..."));

// Symbol
let sym = symbol_short!("HELLO");  // Up to 9 chars
let sym = Symbol::new(&env, "long_symbol");  // > 9 chars

// String
let s = String::from_str(&env, "Hello, World!");

// Bytes
let b = Bytes::from_slice(&env, &[1, 2, 3]);

// BytesN (fixed size)
let hash: BytesN<32> = BytesN::from_array(&env, &[0u8; 32]);

// Vec
let v = vec![&env, 1, 2, 3];

// Map
let mut m = Map::new(&env);
m.set(key, value);
```

## 🔢 Math Operations

```rust
// Checked operations (recommended)
let sum = a.checked_add(b).expect("Overflow");
let diff = a.checked_sub(b).expect("Underflow");
let prod = a.checked_mul(b).expect("Overflow");
let quot = a.checked_div(b).expect("Division by zero");

// Saturating operations
let sum = a.saturating_add(b);  // Clamps at max value
```

## ⏰ Time & Ledger

```rust
// Current timestamp
let time = env.ledger().timestamp();

// Current ledger sequence
let seq = env.ledger().sequence();

// In tests - set time
env.ledger().with_mut(|li| {
    li.timestamp = 1640000000;
    li.sequence = 1000;
});
```

## 🔄 Contract Interactions

```rust
// Call another contract
let other_contract = OtherContractClient::new(&env, &contract_id);
let result = other_contract.some_function(&arg);

// Get current contract address
let this = env.current_contract_address();
```

## 🚀 Build & Deploy

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Or with Soroban CLI
soroban contract build

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_contract.wasm \
  --source alice \
  --network testnet

# Invoke
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  function_name \
  --arg value
```

## 📝 Common Patterns

### Initialization

```rust
pub fn initialize(env: Env, admin: Address) {
    if env.storage().instance().has(&symbol_short!("init")) {
        panic!("Already initialized");
    }
    env.storage().instance().set(&symbol_short!("admin"), &admin);
    env.storage().instance().set(&symbol_short!("init"), &true);
}
```

### Admin Check

```rust
fn require_admin(env: &Env, caller: &Address) {
    let admin: Address = env.storage().instance()
        .get(&symbol_short!("admin"))
        .unwrap();
    if caller != &admin {
        panic!("Unauthorized");
    }
    admin.require_auth();
}
```

### Balance Operations

```rust
fn get_balance(env: &Env, addr: &Address) -> i128 {
    env.storage().persistent()
        .get(&DataKey::Balance(addr.clone()))
        .unwrap_or(0)
}

fn set_balance(env: &Env, addr: &Address, amount: i128) {
    env.storage().persistent()
        .set(&DataKey::Balance(addr.clone()), &amount);
}

fn add_balance(env: &Env, addr: &Address, amount: i128) {
    let balance = get_balance(env, addr);
    set_balance(env, addr, balance + amount);
}
```

## 🔍 Debugging

```rust
#[cfg(test)]
mod test {
    #[test]
    fn debug_test() {
        let env = Env::default();

        // Enable logging
        env.logs().enable();

        // Your test code

        // Print logs
        println!("{:?}", env.logs().all());
    }
}
```

## 📦 Useful Macros

```rust
// Symbol macros
symbol_short!("short")    // ≤ 9 characters
symbol!("long_symbol")    // > 9 characters

// Vector macro
vec![&env, 1, 2, 3]

// Bytes macro
bytes![&env, 0x01, 0x02, 0x03]

// Map construction
map![&env, (key1, val1), (key2, val2)]
```

## 🔗 Resources

- [Full Documentation](https://developers.stellar.org/docs/smart-contracts)
- [SDK Reference](https://docs.rs/soroban-sdk/)
- [Examples](../examples/)
- [Discord](https://discord.gg/stellardev)

---

**Keep this handy while coding!** 📚

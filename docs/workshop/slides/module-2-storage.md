---
marp: true
theme: default
paginate: true
---

# Module 2: Storage & Data Types

## Soroban Development Workshop

---

# Soroban Storage Overview

Three storage types, each with different cost and lifetime:

| Type | Lifetime | TTL Management | Typical Use |
|------|----------|----------------|-------------|
| **Persistent** | Permanent (until deleted) | Required (extend TTL) | Token balances, user data |
| **Temporary** | Short-lived | Auto-expires | Caches, rate-limit windows |
| **Instance** | Tied to contract instance | Required (extend TTL) | Config, admin, supply |

---

# Persistent Storage

Data survives across ledgers. Must manage TTL or data gets archived.

```rust
// Write
env.storage().persistent().set(&key, &value);

// Read
let val: Option<u64> = env.storage().persistent().get(&key);

// Check existence
env.storage().persistent().has(&key);

// Remove
env.storage().persistent().remove(&key);

// Extend TTL
env.storage().persistent().extend_ttl(&key, 1000, 10000);
```

Source: [`examples/basics/02-storage-patterns/`](../../examples/basics/02-storage-patterns/)

---

# Temporary Storage

Auto-expires after a TTL. Cheaper than persistent. No `remove` needed.

```rust
// Write (auto-expires if TTL not extended)
env.storage().temporary().set(&key, &value);

// Read (returns None if expired)
let val: Option<u64> = env.storage().temporary().get(&key);
```

Use cases: rate-limit windows, caching, ephemeral state.

---

# Instance Storage

Tied to the contract instance itself. Good for singleton config.

```rust
// Write config
env.storage().instance().set(&DataKey::Admin, &admin);

// Read config
let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();

// Extend instance TTL (applies to all instance keys)
env.storage().instance().extend_ttl(1000, 10000);
```

---

# Storage Keys: The DataKey Enum

All storage uses a typed key. The convention is a `DataKey` enum:

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Persistent(Symbol),
    Temporary(Symbol),
    Instance(Symbol),
    Balance(Address),
    Allowance(Address, Address),
}
```

Each variant maps to a storage slot. Enums with data = composite keys.

---

# Custom Types with `#[contracttype]`

```rust
#[contracttype]
#[derive(Clone)]
pub struct UserProfile {
    pub name: String,
    pub age: u32,
    pub is_admin: bool,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum UserRole {
    Admin = 1,
    Moderator = 2,
    User = 3,
}
```

These can be stored in contract storage directly.

Source: [`examples/basics/10-custom-structs/`](../../examples/basics/10-custom-structs/)

---

# Collections: Vec

```rust
use soroban_sdk::{vec, Vec};

// Create
let items: Vec<u32> = vec![&env, 1, 2, 3];

// Access
let first = items.get(0).unwrap();  // Option<T>

// Length
let len = items.len();

// Push / Pop
items.push(4);
let last = items.pop().unwrap();
```

Source: [`examples/basics/13-collection-types/`](../../examples/basics/13-collection-types/)

---

# Collections: Map

```rust
use soroban_sdk::Map;

// Create
let mut map: Map<Symbol, u32> = Map::new(&env);

// Insert
map.set(symbol_short!("alice"), 100);

// Get
let val = map.get(symbol_short!("alice")).unwrap();

// Remove
map.remove(symbol_short!("alice"));

// Iterate (keys)
let keys: Vec<Symbol> = map.keys();
```

---

# Iterable Mapping Pattern

Soroban `Map` doesn't support direct iteration by value. The cookbook provides an iterable mapping pattern:

```rust
// Side-list tracks all keys for iteration
struct IterableMapping {
    keys: Vec<Symbol>,
    values: Map<Symbol, u32>,
}

// Paginated access
fn keys_page(&self, page: u32, page_size: u32) -> Vec<Symbol> {
    let start = (page * page_size) as usize;
    // ... slice the keys vec
}
```

Source: [`examples/intermediate/iterable-mappings/`](../../examples/intermediate/iterable-mappings/)

---

# Data Types You'll Use Most

| Rust Type | Soroban Usage | Notes |
|-----------|---------------|-------|
| `u32`, `u64`, `i128`, `i64` | Numbers, amounts | `i128` for token amounts |
| `bool` | Flags, state | Direct storage |
| `Address` | Accounts, contracts | `.require_auth()` for authorization |
| `Symbol` | Short identifiers | ≤9 chars via `symbol_short!` |
| `String` | Longer text | Variable length |
| `BytesN<32>` | Hashes, keys | Fixed-size byte arrays |
| `Vec<T>` | Lists, batches | Ordered, indexable |
| `Map<K,V>` | Key-value stores | No built-in iteration |

---

# Error Handling with `#[contracterror]`

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StorageError {
    KeyNotFound = 1,
    AlreadyExists = 2,
    Unauthorized = 3,
    InvalidInput = 4,
}

// Return Result<T, E> from contract functions
pub fn get_value(env: Env, key: Symbol) -> Result<u64, StorageError> {
    env.storage().persistent().get(&key)
        .ok_or(StorageError::KeyNotFound)
}
```

Source: [`examples/basics/03-custom-errors/`](../../examples/basics/03-custom-errors/)

---

# Events

Events let external systems track contract state changes.

```rust
// Publish an event with topics and data
env.events().publish(
    (symbol_short!("transfer"), from, to),  // topics (up to 4)
    TransferData { amount },                  // data payload
);

// Structured event data
#[contracttype]
#[derive(Clone)]
pub struct TransferData {
    pub amount: i128,
}
```

Source: [`examples/basics/04-events/`](../../examples/basics/04-events/)

---

# Module 2 Summary

- Three storage types: persistent (permanent), temporary (auto-expire), instance (contract config)
- Use `DataKey` enum for typed storage keys
- `#[contracttype]` enables custom structs/enums in storage
- `Vec` and `Map` for collections; iterable-mapping pattern for enumeration
- `#[contracterror]` for typed errors with `Result` returns
- `env.events().publish()` for state-change notifications

---

# Up Next

**Module 3: Authentication & Access Control**
- `require_auth()` pattern
- Role-based access control (RBAC)
- Multi-signature patterns

---

# Exercise Time!

See [Exercise 2](../exercises/module-2-exercise.md)

**Goal:** Build a storage-backed counter contract with custom types and events.

⏱️ 45 minutes

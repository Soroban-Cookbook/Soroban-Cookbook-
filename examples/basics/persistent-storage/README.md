# Persistent Storage Pattern

A focused demonstration of Soroban's persistent storage type — the most durable storage tier for data that must survive indefinitely.

## 📖 What You'll Learn

- When and why to use persistent storage over instance or temporary storage
- How to set, get, and manage persistent data with proper TTL handling
- Real-world patterns for user-specific data like balances and permissions
- Best practices for extending TTL to prevent data archival
- How to structure storage keys using enums for type safety

## 🎯 Overview

Persistent storage is Soroban's **highest durability storage tier**. Each entry has its own independent Time-To-Live (TTL), making it ideal for:

- User balances in token contracts
- Ownership records and permissions
- Per-user data that must survive contract upgrades
- Any data where loss would be unacceptable

This example demonstrates a minimal contract with two persistent storage use cases:

1. **Admin address** — a single-value persistent entry
2. **Counter** — a numeric value that increments and persists across calls

## 🔑 Key Concepts

### Persistent Storage Characteristics

| Feature               | Description                                         |
| --------------------- | --------------------------------------------------- |
| **Lifetime**          | Survives indefinitely with proper TTL management    |
| **Cost**              | Highest gas cost among the three storage types      |
| **TTL Management**    | Per-key TTL — each entry has independent lifetime   |
| **Archival**          | Archived after TTL expires, but restorable          |
| **Survives Upgrades** | Yes — data persists even when contract code changes |

### When to Use Persistent Storage

✅ **Use persistent storage when:**

- Data must survive long-term (weeks, months, years)
- Each user/entity needs independent data with its own lifecycle
- Data loss would break contract functionality or user trust
- You need to restore archived data later

❌ **Don't use persistent storage when:**

- Data is contract-wide configuration (use instance storage)
- Data is temporary or cache-like (use temporary storage)
- You're storing hundreds of small values that could be packed into one struct

## 💻 Code Walkthrough

### Storage Key Enum

We use a typed enum to define our storage keys, preventing typos and enabling compile-time checks:

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Counter,
}
```

The `#[contracttype]` macro makes this enum serializable for storage, and `Clone` allows us to reuse keys without moving them.

### Setting Persistent Data

```rust
pub fn set_admin(env: Env, address: Address) {
    let key = DataKey::Admin;
    env.storage().persistent().set(&key, &address);
    env.storage().persistent().extend_ttl(&key, 2000, 10000);
}
```

**Key points:**

1. `set(&key, &value)` writes the data to persistent storage
2. `extend_ttl(&key, threshold, extend_to)` ensures the data stays alive
   - `threshold: 2000` — only extend if TTL drops below 2000 ledgers
   - `extend_to: 10000` — extend to 10,000 ledgers from now (~14 hours on mainnet)

**Always extend TTL after writing** — forgetting this causes premature archival.

### Getting Persistent Data

```rust
pub fn get_admin(env: Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}
```

Returns `Option<Address>` because the key might not exist yet. Using `Option` is safer than `.unwrap()`, which would panic on missing keys.

### Incrementing with Persistent Storage

```rust
pub fn increment(env: Env) -> u64 {
    let key = DataKey::Counter;
    let mut count: u64 = env.storage().persistent().get(&key).unwrap_or(0);

    count = count.checked_add(1).expect("counter overflow");
    env.storage().persistent().set(&key, &count);
    env.storage().persistent().extend_ttl(&key, 2000, 10000);

    count
}
```

**Pattern breakdown:**

1. Read current value with `.unwrap_or(0)` — defaults to 0 if key doesn't exist
2. Use `checked_add` to prevent overflow (returns `None` on overflow)
3. Write the new value back to storage
4. Extend TTL to keep the data alive
5. Return the new count

### Reading the Counter

```rust
pub fn get_counter(env: Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::Counter)
        .unwrap_or(0)
}
```

Provides read-only access to the counter value, defaulting to 0 if not yet initialized.

## 🧪 Tests

### Test 1: Basic Persistent Storage Logic

```rust
#[test]
fn test_persistent_storage_logic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PersistentStorageContract);
    let client = PersistentStorageContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // Test setting and getting admin
    client.set_admin(&admin);
    assert_eq!(client.get_admin(), Some(admin.clone()));

    // Test increment logic
    assert_eq!(client.increment(), 1);
    assert_eq!(client.increment(), 2);
    assert_eq!(client.get_counter(), 2);
}
```

Verifies:

- Admin address can be stored and retrieved
- Counter increments correctly
- Data persists across multiple calls

### Test 2: Storage Durability and TTL

```rust
#[test]
fn test_storage_durability_and_ttl() {
    let env = Env::default();

    // Configure ledger with realistic TTL parameters
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 12345,
        protocol_version: 20,
        sequence_number: 10,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 6312000,
    });

    let contract_id = env.register_contract(None, PersistentStorageContract);
    let client = PersistentStorageContractClient::new(&env, &contract_id);

    // Set initial data
    client.increment();

    // Extend instance TTL (required for contract to remain accessible)
    env.as_contract(&contract_id, || {
        env.storage().instance().extend_ttl(1000, 1000);
    });

    // Jump forward 500 ledgers (~42 minutes on mainnet)
    env.ledger().with_mut(|li| {
        li.sequence_number += 500;
    });

    // Persistent storage should still be accessible
    assert_eq!(client.get_counter(), 1);
}
```

Verifies:

- Data survives across many ledgers
- TTL extension works correctly
- Persistent storage remains accessible after time passes

## ⚡ TTL Management Strategy

This contract uses the **extend-on-write** strategy:

```rust
env.storage().persistent().extend_ttl(&key, 2000, 10000);
```

**Parameters explained:**

- `threshold: 2000` — If remaining TTL > 2000 ledgers, skip extension (saves gas)
- `extend_to: 10000` — When extending, set TTL to 10,000 ledgers from now

**Alternative strategies:**

1. **Extend on read** — Extend TTL when data is accessed, keeping active data alive
2. **Dedicated maintenance** — Separate function for bulk TTL extension, called by bots
3. **Lazy extension** — Only extend when TTL drops very low (higher archival risk)

For production contracts, choose based on:

- How frequently data is accessed
- Who pays for TTL extension (users vs. protocol)
- Acceptable risk of archival

## 🎯 Real-World Use Cases

### Token Contract Balances

```rust
#[contracttype]
pub enum TokenKey {
    Balance(Address),
    Allowance(Address, Address),
}

pub fn balance(env: Env, owner: Address) -> i128 {
    env.storage()
        .persistent()
        .get(&TokenKey::Balance(owner))
        .unwrap_or(0)
}

pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    // ... auth checks ...

    let from_key = TokenKey::Balance(from.clone());
    let to_key = TokenKey::Balance(to.clone());

    let from_balance = env.storage().persistent().get(&from_key).unwrap_or(0);
    let to_balance = env.storage().persistent().get(&to_key).unwrap_or(0);

    env.storage().persistent().set(&from_key, &(from_balance - amount));
    env.storage().persistent().set(&to_key, &(to_balance + amount));

    env.storage().persistent().extend_ttl(&from_key, 5000, 20000);
    env.storage().persistent().extend_ttl(&to_key, 5000, 20000);
}
```

Each user's balance has independent TTL — active users stay cheap to maintain.

### NFT Ownership Records

```rust
#[contracttype]
pub enum NFTKey {
    Owner(u64),  // token_id -> owner
    Metadata(u64),  // token_id -> metadata URI
}

pub fn mint(env: Env, to: Address, token_id: u64, metadata: String) {
    let owner_key = NFTKey::Owner(token_id);
    let meta_key = NFTKey::Metadata(token_id);

    env.storage().persistent().set(&owner_key, &to);
    env.storage().persistent().set(&meta_key, &metadata);

    // Long TTL for NFTs — they should last years
    env.storage().persistent().extend_ttl(&owner_key, 10000, 100000);
    env.storage().persistent().extend_ttl(&meta_key, 10000, 100000);
}
```

### Permission System

```rust
#[contracttype]
pub enum PermKey {
    Role(Address),
    Permission(Address, Symbol),
}

pub fn grant_role(env: Env, user: Address, role: Symbol) {
    let key = PermKey::Role(user);
    env.storage().persistent().set(&key, &role);
    env.storage().persistent().extend_ttl(&key, 5000, 20000);
}

pub fn has_permission(env: Env, user: Address, permission: Symbol) -> bool {
    let key = PermKey::Permission(user, permission);
    env.storage().persistent().get(&key).unwrap_or(false)
}
```

## ✅ Best Practices

### 1. Always Extend TTL After Writing

```rust
// ✅ GOOD
env.storage().persistent().set(&key, &value);
env.storage().persistent().extend_ttl(&key, 2000, 10000);

// ❌ BAD — data will be archived prematurely
env.storage().persistent().set(&key, &value);
```

### 2. Use Typed Enums for Keys

```rust
// ✅ GOOD — type-safe, prevents typos
#[contracttype]
pub enum DataKey {
    Balance(Address),
    Admin,
}

// ❌ BAD — error-prone string keys
let key = symbol_short!("balance");
```

### 3. Handle Missing Keys Gracefully

```rust
// ✅ GOOD — returns default on missing key
let balance = env.storage().persistent().get(&key).unwrap_or(0);

// ❌ BAD — panics if key doesn't exist
let balance = env.storage().persistent().get(&key).unwrap();
```

### 4. Choose Appropriate TTL Values

```rust
// For frequently accessed data
env.storage().persistent().extend_ttl(&key, 1000, 5000);

// For critical long-term data (NFTs, ownership)
env.storage().persistent().extend_ttl(&key, 10000, 100000);

// For rarely accessed but important data
env.storage().persistent().extend_ttl(&key, 5000, 20000);
```

### 5. Use `checked_add` for Arithmetic

```rust
// ✅ GOOD — prevents overflow
count = count.checked_add(1).expect("overflow");

// ❌ BAD — can overflow silently
count = count + 1;
```

## 🚀 Building and Testing

### Run Tests

```bash
cargo test -p persistent-storage
```

### Build WASM

```bash
cargo build -p persistent-storage --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/persistent_storage.wasm`

### Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/persistent_storage.wasm \
  --source alice \
  --network testnet
```

### Invoke Functions

```bash
# Set admin
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- set_admin \
  --address <ADMIN_ADDRESS>

# Increment counter
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- increment

# Get counter value
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- get_counter
```

## 📊 Comparison with Other Storage Types

| Feature        | Persistent            | Instance          | Temporary        |
| -------------- | --------------------- | ----------------- | ---------------- |
| **Use Case**   | Per-user data         | Contract config   | Caches, flags    |
| **Lifetime**   | Indefinite (with TTL) | Contract lifetime | Single ledger    |
| **Cost**       | Highest               | Medium            | Lowest           |
| **TTL Scope**  | Per-key               | Shared            | Per-key          |
| **Restorable** | Yes                   | Yes               | No               |
| **Example**    | Token balances        | Admin address     | Reentrancy guard |

**When to use each:**

- **Persistent** — User balances, ownership, permissions (this example)
- **Instance** — Contract-wide config, total supply, protocol parameters
- **Temporary** — Intermediate calculations, short-lived flags

## 🎓 Next Steps

- [Storage Patterns](../02-storage-patterns/) — Compare all three storage types
- [Authentication](../03-authentication/) — Combine persistent storage with access control
- [Events](../04-events/) — Emit events when persistent data changes
- [Intermediate Examples](../../intermediate/) — Build on these patterns for real applications

## 📚 Further Reading

- [Soroban Storage Documentation](https://developers.stellar.org/docs/smart-contracts/data/storing-data)
- [State Archival Guide](https://developers.stellar.org/docs/smart-contracts/data/state-archival)
- [TTL Management Best Practices](https://developers.stellar.org/docs/smart-contracts/data/state-archival#managing-ttl)
- [Soroban SDK Storage API](https://docs.rs/soroban-sdk/latest/soroban_sdk/storage/)

---

**Built with ❤️ for the Soroban community**

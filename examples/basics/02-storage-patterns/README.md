# Storage Patterns

Learn how to persist and retrieve data in Soroban smart contracts using the three storage types: **Persistent**, **Instance**, and **Temporary**. This guide helps you understand the trade-offs and choose the right storage type for every situation.

## 📖 What You'll Learn

- How Soroban's storage model differs from other smart contract platforms
- The three storage types and their lifetime, cost, and archival characteristics
- TTL (Time-To-Live) management strategies to keep your data alive
- How to read, write, check, and delete entries in each storage type
- Performance optimization and gas cost considerations
- Real-world patterns for token balances, voting systems, caching, and more

## 🌐 Introduction to Storage in Soroban

Smart contracts need to persist state between invocations. Unlike traditional programs that write to a file system or database, blockchain smart contracts store data on-ledger, and every byte has a cost.

Soroban takes a unique approach compared to platforms like Ethereum or Solana:

- **Three distinct storage tiers** — instead of a single key-value store, Soroban gives you Persistent, Instance, and Temporary storage, each with different lifetime and cost profiles.
- **State archival** — data that isn't actively maintained (via TTL extension) can be archived off-ledger, reducing network bloat while still allowing restoration.
- **Explicit TTL management** — contracts are responsible for extending the lifetime of their own data, giving developers fine-grained control over storage costs.

This design means you must think carefully about *where* you store data. Choosing the wrong tier wastes gas or risks unexpected data loss.

## 🗄️ The Three Storage Types

### Comparison Table

| Feature | Persistent | Instance | Temporary |
|---|---|---|---|
| Lifetime | Survives upgrades | Instance lifetime | Single ledger |
| Cost | Highest | Medium | Lowest |
| Use Cases | Balances, ownership, per-user data | Admin, config, counters | Temp flags, caches |
| TTL Management | Per-key | Per-instance | Per-key |
| Archival | Archived after TTL expires; restorable | Archived with the contract instance | Deleted permanently after TTL expires |
| Key Scope | Individual keys, independent lifetimes | All keys share one TTL | Individual keys, independent lifetimes |
| Survives Upgrade | Yes | Yes | No guarantee |

### How to Access Each Type

```rust
env.storage().persistent()   // Long-lived, per-key TTL
env.storage().instance()     // Contract-scoped, shared TTL
env.storage().temporary()    // Ephemeral, cheapest
```

All three expose the same core API: `set`, `get`, `has`, and `remove`.

## 🔒 Persistent Storage Deep Dive

Persistent storage is for data that **must outlive any single transaction** and should survive contract upgrades. Each key has its own independent TTL, so you can manage lifetimes granularly.

### When to Use

- Token balances and allowances
- Ownership records
- User-specific data (profiles, permissions)
- Any data that other contracts or users rely on being available long-term

### How TTL Works

Every persistent entry has a TTL measured in ledger sequence numbers. When the remaining TTL drops below a threshold you define, you extend it. If you never extend it, the entry is eventually *archived* (moved off-ledger). Archived entries can be restored, but restoration has a cost.

```rust
// extend_ttl(key, threshold, extend_to)
// If remaining TTL < threshold, extend to extend_to ledgers from now
env.storage().persistent().extend_ttl(&key, 100, 100);
```

### CRUD Operations

```rust
pub fn set_persistent(env: Env, key: Symbol, value: u64) {
    env.storage().persistent().set(&key, &value);
    env.storage().persistent().extend_ttl(&key, 100, 100);
}

pub fn get_persistent(env: Env, key: Symbol) -> u64 {
    env.storage().persistent().get(&key).unwrap()
}

pub fn has_persistent(env: Env, key: Symbol) -> bool {
    env.storage().persistent().has(&key)
}

pub fn remove_persistent(env: Env, key: Symbol) {
    env.storage().persistent().remove(&key);
}
```

**Key detail:** Always call `extend_ttl` after writing to persistent storage. If you forget, the entry's TTL starts decaying immediately and may be archived before anyone reads it.

## 📦 Instance Storage Deep Dive

Instance storage is tied to the **contract instance itself**. All keys in instance storage share a single TTL — when you extend the instance TTL, every key gets extended together. This makes it ideal for small, contract-wide configuration that should live as long as the contract does.

### When to Use

- Contract admin address
- Feature flags and contract configuration
- Protocol parameters (fee percentages, limits)
- Counters and aggregate state (total supply, vote tally)
- Metadata (contract name, version)

### How TTL Works

Unlike persistent storage, instance storage has a **single shared TTL** for all keys. Extending the TTL extends the lifetime of the entire contract instance — the WASM code reference and all instance data together.

```rust
// extend_ttl(threshold, extend_to) — no key parameter
env.storage().instance().extend_ttl(100, 100);
```

This is simpler to manage but means you can't give different instance keys different lifetimes.

### CRUD Operations

```rust
pub fn set_instance(env: Env, key: Symbol, value: u64) {
    env.storage().instance().set(&key, &value);
    env.storage().instance().extend_ttl(100, 100);
}

pub fn get_instance(env: Env, key: Symbol) -> u64 {
    env.storage().instance().get(&key).unwrap()
}

pub fn has_instance(env: Env, key: Symbol) -> bool {
    env.storage().instance().has(&key)
}

pub fn remove_instance(env: Env, key: Symbol) {
    env.storage().instance().remove(&key);
}
```

**Key detail:** Because all instance keys share one TTL, storing large or numerous entries here increases the rent cost for the *entire* instance. Keep instance storage lean.

## ⏳ Temporary Storage Deep Dive

Temporary storage is the cheapest option. Data written here has a short TTL and is **permanently deleted** once it expires — it cannot be restored.

### When to Use

- Intermediate computation results within a multi-step operation
- Transaction-scoped flags (reentrancy guards, processing markers)
- Short-lived caches (oracle price snapshots valid for a few ledgers)
- Any data where loss is acceptable and re-computation is cheap

### Characteristics

- Lowest gas cost for reads and writes
- No rent required (the short TTL means minimal ledger burden)
- Data is **not restorable** after expiry — gone permanently
- TTL is per-key, like persistent storage

### CRUD Operations

```rust
pub fn set_temporary(env: Env, key: Symbol, value: u64) {
    env.storage().temporary().set(&key, &value);
}

pub fn get_temporary(env: Env, key: Symbol) -> u64 {
    env.storage().temporary().get(&key).unwrap()
}

pub fn has_temporary(env: Env, key: Symbol) -> bool {
    env.storage().temporary().has(&key)
}
```

**Key detail:** Temporary storage does not need `extend_ttl` in most cases. If you find yourself extending temporary TTLs frequently, you probably want persistent or instance storage instead.

## 🧭 Choosing the Right Storage Type

Use this decision guide when you're unsure which storage type fits your data:

```
Does the data need to survive beyond a few ledgers?
├── NO  → Use TEMPORARY storage
│         (cheapest, auto-expires, non-restorable)
│
└── YES
    │
    Is the data contract-wide configuration
    shared by all users/calls?
    ├── YES → Use INSTANCE storage
    │         (shared TTL, lives with the contract)
    │
    └── NO
        │
        Is the data per-user or per-entity
        with its own lifecycle?
        └── YES → Use PERSISTENT storage
                  (per-key TTL, restorable after archival)
```

### Quick Rules of Thumb

| Scenario | Storage Type |
|---|---|
| User balances in a token contract | Persistent |
| Contract admin address | Instance |
| Total supply counter | Instance |
| Reentrancy guard flag | Temporary |
| Cached oracle price (valid ~5 ledgers) | Temporary |
| Per-user vote record in a governance contract | Persistent |
| Protocol fee percentage | Instance |
| Intermediate swap calculation | Temporary |

## ⏰ TTL Management Guide

TTL (Time-To-Live) determines how many ledgers your data survives before archival or deletion. Effective TTL management balances **data availability** against **cost**.

### Core API

```rust
// Persistent — per-key TTL
env.storage().persistent().extend_ttl(&key, threshold, extend_to);

// Instance — shared TTL for all instance data
env.storage().instance().extend_ttl(threshold, extend_to);

// Temporary — per-key TTL (rarely needed)
env.storage().temporary().extend_ttl(&key, threshold, extend_to);
```

**Parameters:**

- `threshold` — Only extend if the current remaining TTL is below this value (avoids redundant extensions)
- `extend_to` — The new TTL value (in ledgers) to set when extending

### Strategy: Extend on Write

The simplest approach — extend TTL every time you write a value. This is what our example contract does:

```rust
pub fn set_persistent(env: Env, key: Symbol, value: u64) {
    env.storage().persistent().set(&key, &value);
    env.storage().persistent().extend_ttl(&key, 100, 100);
}
```

**Pros:** Simple, hard to forget.
**Cons:** Costs extra gas on every write even if the TTL was already healthy.

### Strategy: Extend on Read

Extend the TTL whenever data is accessed, ensuring actively-used data never expires:

```rust
pub fn get_persistent(env: Env, key: Symbol) -> u64 {
    let value = env.storage().persistent().get(&key).unwrap();
    env.storage().persistent().extend_ttl(&key, 50, 100);
    value
}
```

**Pros:** Actively-read data stays alive automatically.
**Cons:** Adds gas cost to reads; dormant data still expires.

### Strategy: Dedicated Maintenance Function

Expose a separate function that anyone can call to bulk-extend TTLs:

```rust
pub fn maintain_storage(env: Env, keys: Vec<Symbol>) {
    for key in keys.iter() {
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, 1000, 5000);
        }
    }
}
```

**Pros:** Separates storage maintenance from business logic; can be called by bots.
**Cons:** Requires off-chain monitoring to know when to call it.

### Choosing TTL Values

| Context | Suggested Threshold | Suggested Extend-To |
|---|---|---|
| Frequently accessed data | 100 ledgers | 500 ledgers |
| Rarely accessed critical data | 5,000 ledgers | 20,000 ledgers |
| Instance storage (keep contract alive) | 5,000 ledgers | 20,000 ledgers |
| Temporary cache | Rarely extend | 10–50 ledgers |

> **Tip:** On Stellar mainnet, one ledger is roughly 5 seconds. 100 ledgers ≈ ~8 minutes, 17,280 ledgers ≈ ~1 day.

## ⚡ Performance Considerations

### Gas Costs by Operation

| Operation | Persistent | Instance | Temporary |
|---|---|---|---|
| Write (`set`) | Highest | Medium | Lowest |
| Read (`get`) | Medium | Low | Lowest |
| Existence check (`has`) | Low | Low | Low |
| Delete (`remove`) | Low | Low | Low |
| TTL extension | Medium | Low (one call for all keys) | Low |

### Optimization Tips

1. **Batch instance reads.** All instance data is loaded together when any instance key is accessed. Reading one key has roughly the same cost as reading several, so group related config into instance storage.

2. **Minimize persistent key count.** Each persistent key has independent overhead. If you have many small related values, consider packing them into a single struct stored under one key.

3. **Use `has()` before `get()` for optional data.** Calling `get().unwrap()` on a missing key panics and wastes gas. Guard with `has()` or use `get()` with `.unwrap_or()` / `.unwrap_or_default()`.

4. **Avoid unnecessary TTL extensions.** The `threshold` parameter exists to skip extensions when the TTL is already healthy. Use it — setting threshold to 0 means "always extend," which wastes gas.

5. **Keep temporary data small.** Even though it's cheap, large temporary entries still consume resources during the ledger they exist in.

## ✅ Best Practices

### Key Design Patterns

Use descriptive, namespaced keys to avoid collisions and improve readability:

```rust
use soroban_sdk::symbol_short;

const ADMIN: Symbol = symbol_short!("admin");
const TOTAL: Symbol = symbol_short!("total");
```

For per-user keys where you need to include an address, use a tuple or a custom enum as the key:

```rust
use soroban_sdk::{Address, contracttype};

#[contracttype]
pub enum DataKey {
    Balance(Address),
    Allowance(Address, Address),
    Admin,
    TotalSupply,
}
```

### Type Safety Tips

- Define a `DataKey` enum (as above) to centralize all your storage keys — this prevents typos and makes refactoring easier.
- Use concrete types with `get::<_, YourType>()` to catch deserialization errors at compile time rather than runtime.
- Mark key enums with `#[contracttype]` so the SDK handles serialization automatically.

### TTL Management Strategies

- Pick **one consistent strategy** (extend-on-write, extend-on-read, or dedicated maintenance) and apply it across your contract.
- For critical data (balances, admin), use generous TTL values — the cost of restoration after archival is higher than preventive extension.
- For instance storage, extend the instance TTL in your most frequently called function to keep the contract alive.

### Common Mistakes to Avoid

1. **Forgetting TTL extension on persistent writes.** Your data will be archived and become inaccessible until restored.
2. **Storing per-user data in instance storage.** Instance storage costs scale with total size — hundreds of user entries will make every operation expensive.
3. **Using `unwrap()` on `get()` without checking `has()`.** This panics if the key doesn't exist, failing the entire transaction.
4. **Using the same key name across storage types unintentionally.** The three storage types are isolated, so key `"data"` in persistent and key `"data"` in temporary are independent. This is a feature, but can cause confusion.
5. **Over-extending temporary TTLs.** If you're constantly extending temporary data, it should probably be in persistent or instance storage.

## 🔍 Code Examples

### Full Contract (from `src/lib.rs`)

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct StorageContract;

#[contractimpl]
impl StorageContract {
    pub fn set_persistent(env: Env, key: Symbol, value: u64) {
        env.storage().persistent().set(&key, &value);
        env.storage().persistent().extend_ttl(&key, 100, 100);
    }

    pub fn get_persistent(env: Env, key: Symbol) -> u64 {
        env.storage().persistent().get(&key).unwrap()
    }

    pub fn has_persistent(env: Env, key: Symbol) -> bool {
        env.storage().persistent().has(&key)
    }

    pub fn remove_persistent(env: Env, key: Symbol) {
        env.storage().persistent().remove(&key);
    }

    pub fn set_temporary(env: Env, key: Symbol, value: u64) {
        env.storage().temporary().set(&key, &value);
    }

    pub fn get_temporary(env: Env, key: Symbol) -> u64 {
        env.storage().temporary().get(&key).unwrap()
    }

    pub fn has_temporary(env: Env, key: Symbol) -> bool {
        env.storage().temporary().has(&key)
    }

    pub fn set_instance(env: Env, key: Symbol, value: u64) {
        env.storage().instance().set(&key, &value);
        env.storage().instance().extend_ttl(100, 100);
    }

    pub fn get_instance(env: Env, key: Symbol) -> u64 {
        env.storage().instance().get(&key).unwrap()
    }

    pub fn has_instance(env: Env, key: Symbol) -> bool {
        env.storage().instance().has(&key)
    }

    pub fn remove_instance(env: Env, key: Symbol) {
        env.storage().instance().remove(&key);
    }
}
```

### Storage Isolation (from `src/test.rs`)

The same key can exist in all three storage types independently:

```rust
let key = symbol_short!("data");

client.set_persistent(&key, &100);
client.set_temporary(&key, &200);
client.set_instance(&key, &300);

assert_eq!(client.get_persistent(&key), 100);
assert_eq!(client.get_temporary(&key), 200);
assert_eq!(client.get_instance(&key), 300);
```

### Typed Key Enum Pattern (production recommendation)

```rust
use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Balance(Address),
    Admin,
    TotalSupply,
    TempFlag(Address),
}

#[contractimpl]
impl TokenContract {
    pub fn balance(env: Env, owner: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner))
            .unwrap_or(0)
    }

    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap()
    }
}
```

## ⚠️ Common Pitfalls

### 1. Data Silently Disappears

**Symptom:** A `get()` call panics or returns unexpected results after some time.
**Cause:** TTL expired and the data was archived (persistent) or deleted (temporary).
**Fix:** Ensure every write path includes a `extend_ttl` call with appropriate values.

### 2. Instance Storage Becomes Expensive

**Symptom:** Gas costs grow over time even though the contract logic hasn't changed.
**Cause:** Too many or too large entries in instance storage. Since all instance data is loaded together, more data means higher per-operation cost.
**Fix:** Move per-user or per-entity data to persistent storage. Reserve instance storage for small, contract-wide config.

### 3. Panics on Missing Keys

**Symptom:** Transaction fails with an unwrap error.
**Cause:** Calling `.get(&key).unwrap()` when the key doesn't exist.
**Fix:** Use `has()` to check first, or use `.unwrap_or(default)`:

```rust
let balance: u64 = env.storage()
    .persistent()
    .get(&key)
    .unwrap_or(0);
```

### 4. Key Collisions Across Contracts

**Symptom:** Two contracts using similar key names don't interfere (which is correct), but within one contract, key reuse across storage types causes logic bugs.
**Cause:** Developer assumes storage types share a namespace (they don't).
**Fix:** Use a `DataKey` enum to make every key explicit and unique.

### 5. Extending TTL with Threshold of 0

**Symptom:** Unnecessary gas spent on TTL extensions.
**Cause:** Setting `threshold` to `0` means "always extend, regardless of current TTL," wasting gas when the TTL is already healthy.
**Fix:** Set `threshold` to a meaningful value — e.g., half of `extend_to` — so you only extend when the TTL is actually low.

## 🧪 Building and Testing

### Run Tests

```bash
cargo test
```

The test suite covers 6 scenarios:

| Test | What It Verifies |
|---|---|
| `test_persistent_storage` | Set, get, has, remove for persistent storage |
| `test_temporary_storage` | Set, get, has for temporary storage |
| `test_instance_storage` | Set, get, has, remove for instance storage |
| `test_storage_isolation` | Same key in different storage types stays independent |
| `test_multiple_keys` | Multiple key-value pairs in persistent storage |
| `test_update_existing_value` | Overwriting an existing key with a new value |

### Build the WASM Binary

```bash
cargo build --target wasm32-unknown-unknown --release
```

The output will be at:

```
target/wasm32-unknown-unknown/release/storage_patterns.wasm
```

### Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/storage_patterns.wasm \
  --source alice \
  --network testnet
```

### Invoke Storage Functions

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- set_persistent \
  --key balance \
  --value 1000

soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- get_persistent \
  --key balance
```

## 🌍 Real-World Use Cases

### Token Contract (Persistent + Instance)

- **Persistent:** Individual user balances (`Balance(Address)`) — each user's balance has its own TTL, so active users stay cheap to maintain while dormant accounts naturally archive.
- **Instance:** Total supply, token name, decimals, admin address — small config that the contract always needs.

### Governance / Voting (All Three)

- **Persistent:** Each voter's record (`Vote(Address)`) — must survive until the proposal closes.
- **Instance:** Proposal metadata, vote tally, quorum threshold.
- **Temporary:** A per-user "already voted this round" flag used for reentrancy prevention within a single ledger.

### DEX / AMM (Persistent + Temporary)

- **Persistent:** Liquidity pool reserves, LP token balances per user.
- **Instance:** Pool configuration (fee rate, token pair addresses).
- **Temporary:** Mid-swap intermediate values, price oracle snapshots valid for a few ledgers.

### NFT Marketplace (Persistent + Instance)

- **Persistent:** Ownership records, listing prices, bid history.
- **Instance:** Marketplace fee percentage, admin address, contract version.

## 🎓 Next Steps

Once you understand storage patterns, explore:

- [Authentication](../03-authentication/) — Secure your contract functions with access control
- [Events](../04-events/) — Emit events when storage changes for off-chain indexing
- [Intermediate Examples](../../intermediate/) — Complex patterns combining storage, auth, and events

## 📚 Further Reading

- [Storing Data in Soroban](https://developers.stellar.org/docs/smart-contracts/data/storing-data) — Official guide on the storage API
- [Storage Types Reference](https://developers.stellar.org/docs/smart-contracts/data/storage-types) — Detailed comparison of the three types
- [State Archival & TTL](https://developers.stellar.org/docs/smart-contracts/data/state-archival) — How archival works and restoration flows
- [Soroban SDK Docs (storage module)](https://docs.rs/soroban-sdk) — API reference for `env.storage()`
- [Soroban Examples Repository](https://github.com/stellar/soroban-examples) — Official example contracts

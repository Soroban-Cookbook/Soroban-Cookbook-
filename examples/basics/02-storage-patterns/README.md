# Storage Patterns

Learn how to persist and retrieve data in Soroban smart contracts using different storage types.

## 📖 What You'll Learn

- Three types of storage: Persistent, Temporary, and Instance
- When to use each storage type
- Storage cost implications
- Best practices for data persistence
- Working with different data types in storage

## 🗄️ Storage Types

### 1. Persistent Storage

- **Lifetime:** Permanently stored until explicitly removed
- **Cost:** Highest (requires rent payments via TTL extension)
- **Use For:** Critical data that must never be lost (balances, ownership)

### 2. Temporary Storage

- **Lifetime:** Available only within the current ledger
- **Cost:** Lowest (no rent required)
- **Use For:** Intermediate calculations, temporary flags

### 3. Instance Storage

- **Lifetime:** Exists as long as the contract exists
- **Cost:** Medium (rent required, but cheaper than persistent)
- **Use For:** Contract configuration, admin settings, metadata

## 🔍 Contract Overview

This example demonstrates:

```rust
// Persistent storage
pub fn set_persistent(env: Env, key: Symbol, value: u64)
pub fn get_persistent(env: Env, key: Symbol) -> u64

// Temporary storage
pub fn set_temporary(env: Env, key: Symbol, value: u64)
pub fn get_temporary(env: Env, key: Symbol) -> u64

// Instance storage
pub fn set_instance(env: Env, key: Symbol, value: u64)
pub fn get_instance(env: Env, key: Symbol) -> u64
```

## 💡 Key Concepts

### Storage Access

```rust
env.storage().persistent()  // Permanent data
env.storage().temporary()   // Single-ledger data
env.storage().instance()    // Contract-lifetime data
```

### TTL (Time To Live)

Persistent and instance storage require TTL management:

```rust
env.storage().persistent().extend_ttl(&key, 100, 100);
```

## 🧪 Testing

```bash
cargo test
```

Tests demonstrate:

- Setting and getting values from each storage type
- Data persistence across function calls
- Storage isolation between types

## 💰 Cost Considerations

| Storage Type | Write Cost | Read Cost | Rent Required |
| ------------ | ---------- | --------- | ------------- |
| Persistent   | High       | Low       | Yes (TTL)     |
| Temporary    | Low        | Low       | No            |
| Instance     | Medium     | Low       | Yes (TTL)     |

### Best Practices

1. **Use Temporary for ephemeral data**

   ```rust
   // Good: Temporary flag for single transaction
   env.storage().temporary().set(&FLAG, &true);
   ```

2. **Use Persistent for critical data**

   ```rust
   // Good: User balances must persist
   env.storage().persistent().set(&balance_key, &amount);
   ```

3. **Use Instance for configuration**

   ```rust
   // Good: Admin address stored as instance data
   env.storage().instance().set(&ADMIN, &admin_address);
   ```

4. **Extend TTL for important data**
   ```rust
   // Extend persistent data lifetime
   env.storage().persistent().extend_ttl(&key, threshold, extend_to);
   ```

## 🚀 Building & Deployment

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/storage_patterns.wasm \
  --source alice \
  --network testnet
```

## 🎓 Next Steps

- [Authentication](../03-authentication/) - Secure your contract functions
- [Events](../04-events/) - Track storage changes with events
- [Intermediate Examples](../../intermediate/) - Complex storage patterns

## 📚 References

- [Storage Documentation](https://developers.stellar.org/docs/smart-contracts/data/storing-data)
- [Storage Types Guide](https://developers.stellar.org/docs/smart-contracts/data/storage-types)
- [TTL Management](https://developers.stellar.org/docs/smart-contracts/data/state-archival)

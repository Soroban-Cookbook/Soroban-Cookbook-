# Storage Patterns

Soroban's three storage types — **Persistent**, **Instance**, and **Temporary** — offer different trade-offs for cost, lifetime, and use cases.

## 📊 Quick Comparison

| Feature | Persistent | Instance | Temporary |
|---------|------------|----------|-----------|
| **Lifetime** | Survives upgrades, per-key TTL | Contract lifetime | Single ledger |
| **Cost** | Highest | Medium | Lowest |
| **Use Cases** | User balances, ownership | Config, admin | Caches, flags |
| **TTL** | Per-key | Shared | Per-key (short) |

## 🎯 When to Use What

```
Data > few ledgers? → Temporary
  ↓
Contract-wide config? → Instance  
  ↓
Per-user data → Persistent
```

**Rules of Thumb:**
- **Persistent**: Token balances, votes
- **Instance**: Admin address, fees  
- **Temporary**: Reentrancy guards, temp calc

## 💾 Code Example

```rust
// Persistent (extend TTL after write!)
env.storage().persistent().set(&key, &value);
env.storage().persistent().extend_ttl(&key, 100, 100);

// Instance (shared TTL)
env.storage().instance().set(&key, &value);
env.storage().instance().extend_ttl(100, 100);

// Temporary (no TTL needed)
env.storage().temporary().set(&key, &value);
```

**Always check `has()` before `get().unwrap_or(0)` to avoid panics.**

## ⚠️ Best Practices

1. **Extend TTL on writes** (persistent/instance)
2. **Use enums for keys**: `#[contracttype] enum DataKey { Balance(Address) }`
3. **Instance = small config only** (loads all keys together)
4. **Threshold != 0** in extend_ttl (avoid redundant extensions)

## 🔬 Full Example

[examples/basics/02-storage-patterns/](../examples/basics/02-storage-patterns/)

**Features:**
- All CRUD operations
- TTL management
- Isolation demo (same key in all 3 types)
- Unit tests
- Deployment instructions
- Focused follow-up examples for [instance storage](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/basics/instance-storage), [persistent storage](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/basics/persistent-storage), and [temporary storage](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/basics/temporary_storage)

```
cd examples/basics/02-storage-patterns
cargo test
cargo build --target wasm32-unknown-unknown --release
```

## 📚 Related
- [Authentication → Secure your storage ops](./authentication.md)
- [Events → Log storage changes](./events.md)


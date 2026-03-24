# Storage Types in Soroban

Soroban provides three different storage types, each with its own cost and lifetime characteristics. Understanding these is crucial for building efficient and secure smart contracts.

## Overview Comparison

| Property             | Persistent                            | Instance                                 | Temporary                                  |
| :------------------- | :------------------------------------ | :--------------------------------------- | :----------------------------------------- |
| **Lifetime**         | Permanent (until deleted)             | Lifetime of the instance                 | Current ledger                             |
| **TTL Management**   | **Per-key**: Each key has its own TTL | **Per-instance**: All keys share one TTL | **None**: Expires at the end of the ledger |
| **Survives Upgrade** | ✅ Yes                                | ❌ No                                    | ❌ No                                      |
| **Storage Cost**     | Highest                               | Medium                                   | Lowest                                     |
| **Read/Write Cost**  | Highest                               | Medium                                   | Lowest                                     |

---

## 1. Persistent Storage

`env.storage().persistent()`

Persistent storage is the most durable storage type in Soroban. Data stored here lives indefinitely as long as its TTL is maintained.

- **Best for**: User balances, critical protocol state, data that must survive contract upgrades.
- **Key Feature**: Each entry has its own independent TTL.
- **Example**: [Persistent Storage Example](../examples/basics/persistent-storage/)

```rust
env.storage().persistent().set(&key, &value);
env.storage().persistent().extend_ttl(&key, threshold, extend_to);
```

## 2. Instance Storage

`env.storage().instance()`

Instance storage is tied to the contract instance itself. It's more cost-effective than persistent storage for data that is global to the contract but doesn't need to outlive an upgrade.

- **Best for**: Contract configuration, admin addresses, transaction counters, metadata.
- **Key Feature**: All instance storage entries share a single TTL. Refreshing the instance TTL refreshes all entries.
- **Example**: [Instance Storage Example](../examples/basics/instance-storage/)

```rust
env.storage().instance().set(&key, &value);
env.storage().instance().extend_ttl(threshold, extend_to);
```

## 3. Temporary Storage

`env.storage().temporary()`

Temporary storage is the cheapest option but only lasts for the current ledger. It's ideal for transient data.

- **Best for**: Single-transaction flags, intermediate calculations, non-critical lookup tables.
- **Key Feature**: No rent is charged, and data is automatically cleared.
- **Example**: [Temporary Storage Example](../examples/basics/temporary_storage/)

```rust
env.storage().temporary().set(&key, &value);
```

---

## When to Use Which?

1.  **Does it need to survive a contract upgrade?**
    - Yes → Use **Persistent**.
    - No → Consider **Instance** or **Temporary**.

2.  **Is it needed across multiple transactions/ledgers?**
    - Yes → Use **Persistent** or **Instance**.
    - No → Use **Temporary**.

3.  **Is it shared state that most calls interact with?**
    - Yes → Use **Instance** (easier TTL management).
    - No → Use **Persistent** (independent TTLs).

## Related Examples

- [02-Storage Patterns](../examples/basics/02-storage-patterns/) - Basic overview of all three.
- [Detailed Instance Storage](../examples/basics/instance-storage/) - Deep dive into instance patterns.
- [Detailed Persistent Storage](../examples/basics/persistent-storage/) - Comprehensive persistent examples.

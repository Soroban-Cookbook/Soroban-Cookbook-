# 🗄️ Soroban Storage Patterns

Soroban introduces a unique state-archival model that differs significantly from other blockchain platforms. Understanding how to manage data on-ledger is critical for building scalable, cost-efficient, and sustainable smart contracts.

---

## 🌟 Conceptual Overview

Smart contract storage on Soroban is not "set and forget." Every byte stored on the ledger consumes network resources and has a direct cost associated with it. Soroban manages this through **State Archival** and **Storage Tiers**.

### What is State Archival?
State archival is a mechanism where ledger entries that are no longer actively maintained (via TTL extensions) are moved from **Hot Storage** (active ledger) to **Cold Storage** (archived state). This keeps the active ledger size manageable, ensuring the network remains performant over time.

- **Persistent & Instance Storage:** Can be archived and subsequently restored.
- **Temporary Storage:** Is permanently deleted upon expiration; it cannot be restored.

---

## 🏗️ Storage Tiers

Soroban provides three distinct storage tiers, each optimized for different lifecycles and cost profiles.

| Feature | **Persistent** | **Instance** | **Temporary** |
| :--- | :--- | :--- | :--- |
| **Lifetime** | Long-term (Per-key) | Per-contract instance | Short-lived (Single ledger) |
| **Cost** | Highest (Read/Write/Rent) | Medium (Shared Rent) | Lowest (No Rent) |
| **Manageability** | Per-key TTL | Shared per-instance TTL | Per-key TTL |
| **Failure Mode** | Archival (Restorable) | Archival (Restorable) | Deletion (Permanent) |
| **Typical Use** | User balances, NFTs, Roles | Admin config, Total Supply | Caching, Transient Flags |

---

## 🧭 Decision Matrix: Which Tier to Use?

Choosing the right tier is a balance between **data importance** and **gas efficiency**.

### 1. Does the data need to survive beyond the current ledger?
- **NO:** Use **Temporary Storage**. (e.g., transient state, reentrancy guards).
- **YES:** Proceed to question 2.

### 2. Is the data global to the contract or unique to a user?
- **Global Config:** Use **Instance Storage**. (e.g., Fee percentages, Admin addresses, total supply).
- **Per-User/Per-Entity Data:** Use **Persistent Storage**. (e.g., User balances, per-account settings).

---

## ⚡ Performance & Gas Considerations

### Ledger Footprint
The larger your ledger footprint, the more "rent" you pay in XLM. Rent is determined by the entry size and the desired **Time-To-Live (TTL)**. Minimizing data storage directly translates to lower protocol costs.

### Gas Costs by Operation
- **Temporary** is the most efficient for both reads and writes as it bypasses the rent mechanism.
- **Instance** data is loaded into memory at the start of every transaction that accesses *any* instance key. Avoid storing large datasets in instance storage, as it increases the cost of *every* contract interaction.
- **Persistent** entries are loaded individually. This is the correct choice for large collections of data where only a small subset is needed per transaction.

---

## 🚀 Best Practices

### 1. Namespaced Data Keys
Always use a typed `enum` for your data keys to prevent collisions and ensure type safety.

```rust
#[contracttype]
pub enum DataKey {
    Admin,           // Instance
    TotalSupply,     // Instance
    Balance(Address),// Persistent
    TempFlag(Address)// Temporary
}
```

### 2. TTL Management Strategy
Data only stays active if its TTL is healthy. You must extend the TTL to prevent archival or deletion.

- **Extend on Write:** Simple and ensures data is always active when updated.
- **Extend on Read:** Essential for data that is frequently read but rarely updated.
- **Threshold Checks:** Use the `threshold` parameter in `extend_ttl` to avoid redundant gas spending if the TTL is already sufficiently high.

### 3. Data Structure Optimization
- **Packing:** Group related small values into a single `struct` stored under a single key. This reduces the overhead of managing multiple independent ledger entries.
- **Small Keys:** Use the shortest possible representations for keys to minimize footprint.

---

## 🔍 Relevant Examples

Explore these practical implementations to see these patterns in action:

- **[All-in-One Storage Patterns](../basics/02-storage-patterns/)**: Comprehensive example covering all three tiers and TTL management.
- **[Instance Storage Deep Dive](../basics/instance-storage/)**: Focus on contract-wide configuration and shared TTLs.
- **[Persistent Storage Deep Dive](../basics/persistent-storage/)**: Focus on per-user data and independent TTLs.
- **[Temporary Storage Deep Dive](../basics/temporary_storage/)**: Focus on ephemeral state and cost optimization.

---

## 📚 Further Resources
- [Official Soroban Documentation: Storing Data](https://developers.stellar.org/docs/smart-contracts/data/storing-data)
- [State Archival Explained](https://developers.stellar.org/docs/smart-contracts/data/state-archival)

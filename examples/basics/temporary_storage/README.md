# Temporary Storage Pattern

This package demonstrates the use of **Temporary Storage** in Soroban smart contracts.

## Overview

Soroban provides three types of storage: Persistent, Instance, and Temporary. Temporary storage is the cheapest storage option, designed specifically for data that does not need to be retained long-term.

This example contract (`src/lib.rs`) illustrates two common patterns for temporary storage:
1. **Reentrancy Guard**: Storing a simple flag (`TempKey::ReentrancyGuard`) to prevent recursive calls within the same transaction.
2. **Internal Result Storage**: Storing an intermediate computation value (`TempKey::InternalResult`) that might be needed by subsequent contract calls in the same ledger sequence.

## Lifecycle Explanation

Entries in temporary storage have a highly ephemeral lifecycle:
- **Creation**: When you call `env.storage().temporary().set(&key, &value)`, an entry is created.
- **Time to Live (TTL)**: By default, temporary storage has a very short TTL, intended to last for just the current or a few ledgers.
- **TTL Extension**: You can explicitly extend the TTL using `env.storage().temporary().extend_ttl(&key, threshold, extend_to)` to ensure the data is available for a bit longer, though it should still be short-lived.
- **Eviction & Deletion**: Unlike persistent or instance storage (which require a fee to restore evicted entries), expired temporary storage entries are **physically deleted** from the state. They cannot be recovered once they expire.

## Performance Considerations

Temporary storage is extremely fast and performant. Because the data is designed to exist for a minimal timeframe, the state size is kept small, reducing I/O overhead.
- **Best Use Case**: Intra-transaction state passing, caching intermediate results inside complex cross-contract workflows, or flags that only matter during a single ledger tick.
- **Worst Use Case**: Storing user balances, global configuration, or any long-term valuable state.

## Gas Implications Documented

Temporary storage is the **cheapest** storage layer available in Soroban:
- **Write Gas**: Writing to temporary storage costs significantly less ledger state I/O and CPU gas compared to Persistent and Instance storage.
- **Archival/Rent Costs**: Because entries are physically deleted rather than evicted and archived, temporary storage does not incur ongoing rent or archival costs in the same way permanent storage does.
- **State Growth**: Since the network physically deletes temporary entries, it places less overall burden on the network's state growth, meaning it is structurally incentivized to be the lowest-cost option.

## Testing

To run the included tests:

```bash
cargo test
```

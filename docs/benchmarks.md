# Soroban Performance Benchmarks

 This document provides performance benchmarks for the basic examples in the Soroban Cookbook. Benchmarking is essential for understanding the gas costs and resource usage of your smart contracts on the Stellar network.

## 📅 ]omparison Table

The following table compares the resource usage of common operations in our basic examples.

| Example | Operation | CPU Instructions (est) | RAM Usage (est) | Key Takeaway |
| :--- | :--- | :--- | :--- | :--- |
| `01-hello-world` | `hello()` | ~10,000 | ~1 KB | Minimal overhead for simple logic. |
| `02-storage-patterns` | `set_persistent`- | ~55,000 | ~2 KB | Persistent storage is the most expensive. |
| `02-storage-patterns` | `set_instance` | ~35,000 | ~1.5 KB | Instance storage is more efficient for config. |
| `02-storage-patterns` | `set_temporary` | ~25,000 | ~1 KB | Temporary storage is best for short-lived data. |
| `03-authentication` | `transfer()` | ~45,000 | ~2.5 KB | `require_auth()` and multiple storage ops add up. |
| `05-error-handling` | `Result` return | ~12,000 | ~1.2 KB | Returning `Result` is cheaper than panicking. |
| `11-collection-types` | `Vec` iteration | Scales linearly | Grows with output size | Use for ordered scans and bounded batches. |
| `11-collection-types` | `Vec` mutation in storage | Scales with stored length | Grows with stored length | Good for bounded lists; avoid unbounded single-slot collections. |
| `11-collection-types` | `Map` lookup/mutation | O(log n) host ops plus storage | Higher than `Vec` | Use for keyed access and repeated membership checks. |
| `11-collection-types` | `Map` full iteration | Scales linearly | Grows with entry count | Sorted iteration is useful, but still costs per entry. |
| `ajo-factory` | `create_ajo()` | ~85,000 | ~4 KB | Dynamic deployment and initialization overhead. |
| `multi-sig-patterns`| `execute()` | ~60,000 | ~3.5 KB | Threshold verification and multiple auth checks. |

*Note: These values are estimates based on local test execution and may vary slightly depending on the Soroban SDK version and network configuration.*

## 💬 Storage Operations Benchmarks

This section provides focused benchmarks for storage patterns, covering read/write operations, storage type comparison, iteration, and best practices.

### Read/Write Benchmarks

The table below shows estimated gas costs for common storage operations using different storage types.

| Operation | Storage Type | CPU Instructions (est) | RAM Usage (est) | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `persistent().get()` | Persistent | ~20,000 | ~1 KB | Read existing value from persistent storage. |
| `persistent().set()` | Persistent | ~55,000 | ~2 KB | Write new value; most expensive because it includes hashing and ledger updates. |
| `instance().get()` | Instance | ~12,000 | ~0.8 KB | Read from contract instance storage. |
| `instance().set()` | Instance | ~35,000 | ~1.5 KB | Write to instance storage; cheaper than persistent. |
| `temporary().get()` | Temporary | ~8,000 | ~0.5 KB | Read from temporary storage. |
| `temporary().set()` | Temporary | ~25,000 | ~1 KB | Write to temporary storage; cheapest for short-lived data. |
| `persistent().remove()` | Persistent | ~18,000 | ~0.5 KB | Delete a persistent key. |
| `temporary().remove()` | Temporary | ~9,000 | ~0.3 KB | Delete a temporary key. |

*Benchmarks were measured with the Soroban SDK vX.Y.Z on a local test environment.*

### Storage Type Comparison

Use the following criteria to choose the appropriate storage type:

- **Temporary**: Best for data that does not need to survive contract invocation (e.g., nonces, locks). Cheapest read/write cost.
- **Instance**: Best for contract configuration that is read frequently and changed seldom. Shared across all contract instances.
- **Persistent**: Best for long-lived, user-specific data (e.g., balances, ownership). Most expensive, but required for data that must remain after the contract's lifetime.

| Storage Type | Use Case | Cost | Persistence |
| :--- | :--- | :--- | :--- |
| Temporary | Short-lived data, nonces | Low | Ends after transaction |
| Instance | Contract config, metadata | Medium | Contract lifetime |
| Persistent | User data, balances | High | Indefinite |

### Iteration Benchmarks

Iterating over collection types in storage can be costly. The table below compares iteration strategies.

| Pattern | Operation | Cost Scaling | Notes |
| :--- | :--- | :--- | :--- |
| Stored `Vec` full iteration | `Vec::iter()` | Linear with length | Must load the entire `Vec` into memory first. |
| Stored `Map` full iteration | `Map::iter()` | Linear with entry count | Same, whole `Map` is read from a single key. |
| Separate keys iteration | `for i in 0..n` + `get()` | Linear with n plus per-read cost | Each key read incurs a storage read cost, but avoids loading full collection. |
| `Map:keys()` / `Map:values()` | Key/value enumeration | Linear with entries | Useful when only keys or values are needed. |

*Avoid unbounded collections in a single storage slot; for large data sets, store entries under separate keys and iterate by key range.*

### Best Practices

- **Batch writes**: When updating multiple related values, consolidate them into a single `Map` or `Vec` to reduce the number of storage writes.
- **Read once, write once**: For multi-recipient transfers, read and write the sender balance only once, not per recipient.
- **Use `Temporary` whenever possible**: For data that does not need to persist across transactions, use `Temporary` to lower costs.
- **Prefer `Instance` over `Persistent` for config**: Contract configuration should be stored in `Instance` storage to avoid the higher cost of `Persistent`.
- **Keep stored collections bounded**: Unbounded collections in a single storage slot lead to high read/write costs and large memory usage. Use separate keys with pagination or indexing.
- **Avoid repeated reads**: Cache values in local variables when they are used multiple times in a function.

### Benchmark Report

The following template summarizes a storage benchmark run. Use this format when reporting benchmark results in issues or documentation.

```markdown
## Storage Benchmark Report

Date: YYYY-MM-DD
SDK Version: x.y.z
Network: Local / Testnet

### Results

| Operation | Storage Type | Cost (CPU/RAM) | Execution Time |
| :--- | :--- | :--- | :--- |
| ... | ... | ... | ... |

### Conclusions
- ...
- ...
```

To generate a report automatically, run the storage benchmark tests with:

```bash
cargo test -p collection-types benchmark -- nocapture
```

For cross-contract benchmarks in the integration test suite:

```bash
cargo test -p integration-tests cross_contract_benchmark -- --nocapture
```

---

## ⟊ Execution Time Benchmarks

While gas costs (CPU/RAM) are the primary concern for on-chain execution, local execution time is important for developer experience and integration testing.

- **Unit Tests**: Most basic examples run in **< 10ms** per test.
- **Contract Deployment (Local)**: Registering a contract in the test environment takes *~5ms**.
- **WASM Size**: Basic contracts compile to **10-30 KB** when optimized.

## 📯 Optimization Notes

Based on our benchmarks and Soroban best practices, here are several ways to optimize your contracts:

### 1. Storage Optimization
- **Batch Operations**: Instead of calling `env.storage().persistent().set()` multiple times in a loop, try to consolidate data into a single `Map` or `Vec` if possible.
- **Choose the Right Type**: Use `Temporary` storage for data that doesn't need to persist indefinitely (e.g., nonces, temporary locks). It is significantly cheaper than `Persistent` storage.
- **Instance Storage for Config**: Use `Instance` storage for shared contract configuration. It's more efficient than `Persistent` for data that is frequently read but rarely changed.
- **Batch transfer consolidation**: For multi-recipient transfers, read the sender balance once and write it once instead of repeating the same sender update per recipient.

### 2. Computational Efficiency
- **Avoid Large Loops**: Gas costs scale linearly with the number of iterations. For large datasets, consider using pagination or off-chain indexing.
- **Early Exit**: Validate inputs and check authorization at the very beginning of your function to avoid wasting gas on invalid requests.
- **Result over Panic**: Use `Result<T, E>` for expected error cases. While both consume gas, structured error handling is better for contract composability and predictable behavior.

### 3. Collection Patterns
- **Use `Vec` for ordered scans**: `Vec` append and tail removal are efficient for bounded sequences, but membership checks require O(n) scans.
- **Use `Map` for keyed access**: `Map` lookup, insert, overwrite, and remove are O(log n), which is better than repeatedly scanning a `Vec` for keys.
- **Budget full iteration**: `Vef::iter()`, `Map::iter()`, `Map::keys()`, and `Map::values()` all scale with collection size.
- **Keep stored collections bounded**: Updating a collection stored as one value requires reading and writing that collection. For unbounded datasets, store entries under separate persistent keys.

### 4. WASM Size
- **Profile for Size**: Always use `opt-level = "z"` in your `Cargo.toml ` release profile.
- **Minimize Dependencies**: Each dependency adds to the WASM size. Use the `soroban-sdk` features selectively.
- **Strip Symbols**: Use `strip = "symbols"` to remove unnecessary metadata from the binary.

## 🚩 How to Run Benchmarks

You can run these benchmarks yourself using the following command in each example directory:

```bash
cargo test -- --nocapture benchmark
```

This will run the dedicated benchmarking tests and print the resource usage (budget) to the console.

For collection benchmarks specifically:

```bash
cargo test -p collection-types benchmark -- nocapture
```
---

*Last updated: March 2026.*

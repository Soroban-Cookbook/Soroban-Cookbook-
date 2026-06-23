# Soroban Performance Benchmarks

This document provides performance benchmarks for the basic examples in the Soroban Cookbook. Benchmarking is essential for understanding the gas costs and resource usage of your smart contracts on the Stellar network.

## 📊 Comparison Table

The following table compares the resource usage of common operations in our basic examples.

| Example | Operation | CPU Instructions (est.) | RAM Usage (est.) | Key Takeaway |
| :--- | :--- | :--- | :--- | :--- |
| `01-hello-world` | `hello()` | ~10,000 | ~1 KB | Minimal overhead for simple logic. |
| `02-storage-patterns` | `set_persistent` | ~55,000 | ~2 KB | Persistent storage is the most expensive. |
| `02-storage-patterns` | `set_instance` | ~35,000 | ~1.5 KB | Instance storage is more efficient for config. |
| `02-storage-patterns` | `set_temporary` | ~25,000 | ~1 KB | Temporary storage is best for short-lived data. |
| `03-authentication` | `transfer()` | ~45,000 | ~2.5 KB | `require_auth()` and multiple storage ops add up. |
| `05-error-handling` | `Result` return | ~12,000 | ~1.2 KB | Returning `Result` is cheaper than panicking. |
| `11-collection-types` | `Vec` iteration | Scales linearly | Grows with output size | Use for ordered scans and bounded batches. |
| `11-collection-types` | `Vec` mutation in storage | Scales with stored length | Grows with stored length | Good for bounded lists; avoid unbounded single-slot collections. |
| `11-collection-types` | `Map` lookup/mutation | O(log n) host ops plus storage | Higher than `Vec` | Use for keyed access and repeated membership checks. |
| `11-collection-types` | `Map` full iteration | Scales linearly | Grows with entry count | Sorted iteration is useful, but still costs per entry. |

*Note: These values are estimates based on local test execution and may vary slightly depending on the Soroban SDK version and network configuration.*

## ⚡ Execution Time Benchmarks

While gas costs (CPU/RAM) are the primary concern for on-chain execution, local execution time is important for developer experience and integration testing.

- **Unit Tests**: Most basic examples run in **< 10ms** per test.
- **Contract Deployment (Local)**: Registering a contract in the test environment takes **~5ms**.
- **WASM Size**: Basic contracts compile to **~10-30 KB** when optimized.

## 💡 Optimization Notes

Based on our benchmarks and Soroban best practices, here are several ways to optimize your contracts:

### 1. Storage Optimization
- **Batch Operations**: Instead of calling `env.storage().persistent().set()` multiple times in a loop, try to consolidate data into a single `Map` or `Vec` if possible.
- **Choose the Right Type**: Use `Temporary` storage for data that doesn't need to persist indefinitely (e.g., nonces, temporary locks). It is significantly cheaper than `Persistent` storage.
- **Instance Storage for Config**: Use `Instance` storage for shared contract configuration. It's more efficient than `Persistent` for data that is frequently read but rarely changed.

### 2. Computational Efficiency
- **Avoid Large Loops**: Gas costs scale linearly with the number of iterations. For large datasets, consider using pagination or off-chain indexing.
- **Early Exit**: Validate inputs and check authorization at the very beginning of your function to avoid wasting gas on invalid requests.
- **Result over Panic**: Use `Result<T, E>` for expected error cases. While both consume gas, structured error handling is better for contract composability and predictable behavior.

### 3. Collection Patterns
- **Use `Vec` for ordered scans**: `Vec` append and tail removal are efficient for bounded sequences, but membership checks require O(n) scans.
- **Use `Map` for keyed access**: `Map` lookup, insert, overwrite, and remove are O(log n), which is better than repeatedly scanning a `Vec` for keys.
- **Budget full iteration**: `Vec::iter()`, `Map::iter()`, `Map::keys()`, and `Map::values()` all scale with collection size.
- **Keep stored collections bounded**: Updating a collection stored as one value requires reading and writing that collection. For unbounded datasets, store entries under separate persistent keys.

### 4. WASM Size
- **Profile for Size**: Always use `opt-level = "z"` in your `Cargo.toml` release profile.
- **Minimize Dependencies**: Each dependency adds to the WASM size. Use the `soroban-sdk` features selectively.
- **Strip Symbols**: Use `strip = "symbols"` to remove unnecessary metadata from the binary.

## 🧪 How to Run Benchmarks

You can run these benchmarks yourself using the following command in each example directory:

```bash
cargo test -- --nocapture benchmark
```

This will run the dedicated benchmarking tests and print the resource usage (budget) to the console.

For collection benchmarks specifically:

```bash
cargo test -p collection-types benchmark -- --nocapture
```
---

*Last updated: March 2026*

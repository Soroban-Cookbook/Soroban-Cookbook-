# Soroban Performance Benchmarks

> **Document Status**: Updated for SDK 28.0.0 (November 2026)  
> **Last Measured**: November 2026  
> **Fee Schedule**: Mainnet/Testnet (Protocol 28+)

Performance benchmarks for the basic examples in the Soroban Cookbook. Benchmarking is essential for understanding the resource costs and fee estimation of your smart contracts on the Stellar network.

## ⚠️ Important Notes on Resource Costs

**Resource fees on Soroban are multidimensional and based on actual network consumption:**
- CPU Instructions (measured by host environment)
- Ledger Entry Accesses (read/write operations)
- Ledger I/O (bytes read/written)
- Transaction Size (bytes)
- Events & Return Value Size (metadata bytes)
- Ledger Space Rent (TTL extensions and storage growth)

The numbers below are estimates for relative comparison. **For production fee estimation, always use the [simulateTransaction RPC method](https://developers.stellar.org/docs/build/guides/transactions/simulateTransaction-Deep-Dive) which provides accurate resource usage for your specific transaction.**

Current network resource rates and limits are published on [Stellar Lab's Resource Limits & Fees page](https://lab.stellar.org/account/resources).

# 📊 Comparison Table

The following table compares the relative resource usage of common operations in our basic examples. CPU instruction counts are approximate and serve as a baseline for relative comparison between operations.

| Example | Operation | CPU Instructions (est) | RAM Usage (est) | Ledger I/O (est) | Key Takeaway |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `01-hello-world` | `hello()` | ~15,000 - 20,000 | ~1-2 KB | Minimal | Minimal overhead for simple logic. Contract instantiation dominates cost. |
| `02-storage-patterns` | `set_persistent` | ~80,000 - 120,000 | ~3-4 KB | ~200-500 bytes | Persistent storage is the most expensive. Includes write amplification and rent. |
| `02-storage-patterns` | `set_instance` | ~50,000 - 70,000 | ~2-3 KB | ~150-300 bytes | Instance storage is more efficient for config. Good for contract-level data. |
| `02-storage-patterns` | `set_temporary` | ~35,000 - 50,000 | ~1.5-2 KB | ~100-200 bytes | Temporary storage is best for short-lived data. No rent costs. |
| `02-storage-patterns` | `get_persistent` | ~40,000 - 60,000 | ~2-3 KB | ~100-300 bytes | Reading persistent data is cheaper than writing but still involves I/O cost. |
| `02-storage-patterns` | `get_instance` | ~25,000 - 40,000 | ~1.5-2 KB | ~80-200 bytes | Reading instance data is efficient for config. Cached by host. |
| `02-storage-patterns` | `get_temporary` | ~20,000 - 35,000 | ~1-1.5 KB | ~50-150 bytes | Reading temporary data is the fastest. No archival overhead. |
| `03-authentication` | `transfer()` | ~70,000 - 100,000 | ~3-4 KB | ~200-400 bytes | `require_auth()` signature verification and multiple storage ops add up. |
| `05-error-handling` | `Result` return | ~18,000 - 25,000 | ~1.5-2 KB | Minimal | Returning `Result` is cheaper than panicking. Structured errors enable composability. |
| `11-collection-types` | `Vec` iteration (10 items) | ~30,000 + ~5k/item | Grows with size | Grows with size | Scales linearly. Use for ordered scans and bounded batches. |
| `11-collection-types` | `Vec` mutation in storage | Scales with length | Grows with length | Rewrite entire vec | Good for bounded lists; avoid unbounded single-slot collections. |
| `11-collection-types` | `Map` lookup/mutation | ~40,000 + O(log n) | Higher than `Vec` | Per-entry I/O | Use for keyed access and repeated membership checks. Each entry is separate. |
| `11-collection-types` | `Map` full iteration (10 items) | ~50,000 + ~8k/item | Grows with entries | Grows with entries | Sorted iteration is useful, but still costs per entry. Consider pagination. |

**Notes:**
- These values are estimates based on SDK 28.0.0 local test execution. Actual costs vary with transaction complexity, storage state, and network conditions.
- CPU instruction costs were reduced by ~20-30% in Protocol 21 (CAP-0054) due to VM instantiation optimizations.
- Ranges reflect variance from different input sizes and contract states.
- For production estimates, **always use `simulateTransaction`** before submitting.

## 📊 Cross-Contract Call Benchmarks

Cross-contract calls introduce additional overhead compared to internal calls. The following table summarizes performance metrics for factory deployment and proxy calls, which are common patterns in composable systems.

| Scenario | Operation | CPU Instructions (est) | RAM Usage (est) | Ledger I/O (est) | Key Takeaway |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Factory | Deploy contract | ~100,000 - 150,000 | ~4-5 KB | ~1-2 KB | Deployment is once per contract; amortize cost over multiple calls when possible. |
| Proxy | Cross-contract call (simple) | ~45,000 - 65,000 | ~2-3 KB | ~200-400 bytes | Overhead of external call is significant compared to internal; minimize cross-contract hops. |
| Proxy | Cross-contract call with arguments | ~55,000 - 80,000 | ~3-4 KB | ~300-600 bytes | Each additional argument increases serialization and host function call cost. |
| Proxy | Delegate call (forward) | ~60,000 - 85,000 | ~3-4 KB | ~300-500 bytes | Forwarding adds a small amount of overhead beyond a simple call. Good for upgradability. |

**Notes:**
- Cross-contract calls include contract invocation setup, argument serialization, and context switching.
- Protocol 21+ reduced VM instantiation costs significantly (CAP-0054), making cross-contract patterns more viable.
- These estimates are derived from local integration test benchmarks. Actual network costs include ledger entry access fees.
- For multi-contract systems, batch operations and minimize call depth where possible.

## ⏱️ Execution Time & Resource Benchmarks

While resource fees (CPU/RAM/I/O) are the primary concern for on-chain execution, local execution time is important for developer experience and integration testing.

### Local Test Environment Performance

- **Unit Tests**: Most basic examples run in **< 15ms** per test on modern hardware.
- **Contract Deployment (Local)**: Registering a contract in the test environment takes **~8-12ms**.
- **WASM Size**: Basic contracts compile to **~15-40 KB** when optimized with `opt-level = "z"` and `strip = "symbols"`.
- **Cross-Contract Tests**: Factory deployment adds **~8-15ms** per contract, while proxy calls add **~2-4ms** per call in local testing.

### Real Network Resource Costs

Based on current mainnet fee schedule (Protocol 28+):

**Average Transaction Costs** (from [Stellar community analysis](https://cheesecakelabs.com/blog/how-much-do-soroban-fees-cost/)):
- **Simple read operation**: ~100,000-150,000 stroops (0.01-0.015 XLM) ≈ $0.001-0.002 USD
- **Write operation with storage**: ~250,000-350,000 stroops (0.025-0.035 XLM) ≈ $0.003-0.004 USD  
- **Complex multi-step transaction**: ~500,000-800,000 stroops (0.05-0.08 XLM) ≈ $0.006-0.009 USD

**Fee Components:**
1. **Inclusion Fee**: Base fee of 100 stroops per operation (can surge during congestion)
2. **Resource Fee**: Based on actual consumption of CPU, I/O, storage rent, and events

**Note**: XLM prices fluctuate. At 0.11 USD/XLM (Nov 2026 reference), most operations cost less than $0.01.

## 🔧 Optimization Notes

Based on our benchmarks, current mainnet fee schedules, and Soroban best practices, here are several ways to optimize your contracts:

### 1. Storage Optimization
- **Batch Operations**: Instead of calling `env.storage().persistent().set()` multiple times in a loop, try to consolidate data or use separate keys. Single large writes are more efficient than many small ones.
- **Choose the Right Type**: Use `Temporary` storage for data that doesn't need to persist indefinitely (e.g., nonces, temporary locks, session data). It is significantly cheaper than `Persistent` storage and incurs no rent fees.
- **Instance Storage for Config**: Use `Instance` storage for shared contract configuration that is frequently read but rarely changed. It's more efficient than `Persistent` for contract-level metadata.
- **Minimize Storage Growth**: Storage rent fees scale with ledger size. Keep stored data compact and clean up unused entries.

### 2. Computational Efficiency
- **Avoid Large Loops**: CPU instruction costs scale linearly with iterations. For large datasets, consider pagination, early exit conditions, or off-chain indexing.
- **Early Exit**: Validate inputs and check authorization at the very beginning of your function to avoid wasting resources on invalid requests.
- **Result over Panic**: Use `Result<T, E>` for expected error cases. While both consume resources, structured error handling enables better composability and allows callers to handle failures gracefully.
- **Minimize Cross-Contract Calls**: Each cross-contract call adds ~20-40k CPU instructions. Batch operations or design contracts to minimize call depth.

### 3. Collection Patterns
- **Use `Vec` for ordered scans**: `Vec` append and tail removal are efficient for bounded sequences, but membership checks require O(n) scans.
- **Use `Map` for keyed access**: `Map` lookup, insert, overwrite, and remove are O(log n), which is better than repeatedly scanning a `Vec` for keys.
- **Budget full iteration**: `Vec::iter()`, `Map::iter()`, `Map::keys()`, and `Map::values()` all scale with collection size. Implement pagination for large datasets.
- **Keep stored collections bounded**: Updating a collection stored as a single value requires reading and writing the entire collection. For unbounded datasets, store entries under separate persistent keys to enable incremental updates.

### 4. WASM Size Optimization
- **Profile for Size**: Always use `opt-level = "z"` in your `Cargo.toml` release profile for size optimization.
- **Minimize Dependencies**: Each dependency adds to the WASM size and deployment cost. Use `soroban-sdk` features selectively.
- **Strip Symbols**: Use `strip = "symbols"` to remove unnecessary metadata from the binary.
- **LTO (Link-Time Optimization)**: Enable `lto = true` in your release profile for additional size reduction.

### 5. Cross-Contract Interaction Optimization

Cross-contract calls introduce additional overhead compared to internal calls. Here are specific recommendations to minimize the performance cost:

- **Reuse deployed contracts**: If you need to call the same contract multiple times, deploy it once and pass its address to receiving contracts. Repeated factory deployments are expensive.
- **Batch multiple calls**: If you need to call several functions on the same contract, consider designing an aggregated API on the target contract to reduce the number of cross-contract invocations.
- **Minimize data passed**: Each argument increases serialization cost and host function overhead. Pass only the necessary data, and prefer primitive types or compact structures over large nested objects.
- **Consider call patterns**: Synchronous calls are standard, but if you need results from multiple independent contracts, consider whether parallel queries from the client could reduce on-chain coordination costs.
- **Use call patterns wisely**: Delegate calls (proxies) add overhead for each invocation. Ensure the indirection is necessary and not duplicating functionality that could be handled directly.

### 6. Fee Estimation Best Practices

- **Use `simulateTransaction`**: Always use the [simulateTransaction RPC method](https://developers.stellar.org/docs/build/guides/transactions/simulateTransaction-Deep-Dive) to get accurate resource consumption estimates before submitting transactions.
- **Account for Surge Pricing**: During network congestion, inclusion fees can increase. Set reasonable fee limits based on your application's tolerance.
- **Monitor Network Conditions**: Check [Stellar Lab's resource limits](https://lab.stellar.org/account/resources) for current network capacity and fee rates.
- **Budget for Refunds**: Some resource fees (events, return values, rent) are refundable if unused. Non-refundable fees include CPU instructions, read/write bytes, and transaction size.

## 🧪 How to Run Benchmarks

You can run these benchmarks yourself using the following commands in the repository root or in each example directory:

### Basic Benchmark Commands

```bash
# Run all workspace tests (includes benchmark output with --nocapture)
cargo test --workspace -- --nocapture

# Run storage pattern benchmarks specifically
cargo test -p integration-tests storage_benchmarks -- --nocapture

# Run collection type benchmarks
cargo test -p collection-types benchmark -- --nocapture

# Run cross-contract benchmarks in the integration test suite
cargo test -p integration-tests cross_contract_benchmark -- --nocapture

# Run DeFi operation benchmarks
cargo test -p integration-tests --test defi_benchmarks -- --nocapture

# Run token operation benchmarks
cargo test -p integration-tests token_benchmarks -- --nocapture
```

### Measuring Resource Usage

Soroban's test environment provides built-in budget metering:

```rust
use soroban_sdk::Env;

#[test]
fn measure_cpu_instructions() {
    let env = Env::default();
    // ... setup your contract ...
    
    // Reset budget before measuring
    env.budget().reset_default();
    
    // Execute the operation you want to measure
    contract_client.my_function();
    
    // Get the CPU instructions consumed
    let cpu_insns = env.budget().cpu_instruction_cost();
    let mem_bytes = env.budget().memory_bytes_cost();
    
    println!("CPU: {} instructions, Memory: {} bytes", cpu_insns, mem_bytes);
}
```

### Advanced Analysis

For detailed cost analysis across multiple operations, see:
- [`tests/integration/storage_benchmarks.rs`](../../tests/integration/storage_benchmarks.rs) - Storage operation patterns
- [`tests/integration/token_benchmarks.rs`](../../tests/integration/token_benchmarks.rs) - Token operation costs
- [`tests/integration/tests/defi_benchmarks.rs`](../../tests/integration/tests/defi_benchmarks.rs) - DeFi protocol costs
- [`examples/tokens/03-optimized-operations/benches/`](../../examples/tokens/03-optimized-operations/benches/) - Optimization comparisons

## 📋 Benchmark Report Summary

We ran the benchmarks described above using the Soroban test environment (SDK 28.0.0) and the `integration-tests` crate. The following is a consolidated report of key findings:

### Storage Operation Analysis

**Read/Write Cost Comparison**: The tables above include both `set_*` and `get_*` operations for each storage type. As expected:
- Writes cost **~2-3x more** than reads due to I/O amplification and state rent
- `Persistent` storage is the most expensive but survives contract upgrades
- `Instance` storage is ideal for contract-level configuration (read-heavy workloads)
- `Temporary` storage should be used for any data that doesn't need long-term persistence (no rent fees)

**Storage Type Selection Guide**:
- **Persistent**: Use only when data must survive contract upgrades. Highest cost, includes rent.
- **Instance**: Use for contract-level configuration that changes infrequently but is read often.
- **Temporary**: Use for session data, nonces, temporary locks, or any data with short TTL. Lowest cost.

### Collection Iteration Costs

**Linear Scaling**: Iterating over `Vec` and `Map` scales linearly with the number of elements:
- `Vec` iteration: **~5,000 CPU instructions per element**
- `Map` iteration: **~8,000 CPU instructions per element** (sorted by key, includes tree traversal)

**Pagination Requirement**: For collections with more than 50-100 elements, implement pagination to avoid exceeding transaction resource limits. The network caps CPU instructions per transaction (see [Stellar Lab's limits](https://lab.stellar.org/account/resources)).

### Cross-Contract Call Overhead

**Deployment vs. Calls**: Factory deployment is **~2-3x more expensive** than a simple cross-contract call. Amortize deployment costs by reusing deployed contracts across multiple operations.

**Call Depth Matters**: Each additional layer of cross-contract calls adds **~20-40k CPU instructions**. Design your contract architecture to minimize call chains.

### Protocol 21+ Improvements (CAP-0054)

Starting with Protocol 21, VM instantiation costs were significantly reduced:
- **~20-30% reduction** in CPU instruction costs for most operations
- Cross-contract calls became more viable for composable systems
- These improvements are reflected in the SDK 28 numbers above

### Best Practices Summary

1. **Use `simulateTransaction`** for production fee estimation - local benchmarks are for relative comparison only
2. **Choose the right storage type** - Temporary < Instance < Persistent in cost
3. **Paginate large collections** - Don't iterate unbounded datasets in a single transaction
4. **Minimize cross-contract depth** - Each hop adds overhead
5. **Batch operations** where possible to reduce per-operation overhead
6. **Profile your contracts** using `env.budget()` during development to identify hotspots

### Reproducing These Results

Full benchmark outputs can be reproduced by running:
```bash
cargo test -p integration-tests benchmark -- --nocapture
```

For custom measurements in your own contracts, use the budget API as shown in the "How to Run Benchmarks" section above.


---

## 📚 Additional Resources

- **Official Soroban Documentation**: 
  - [Understanding Fees, Resource Limits, and Metering](https://developers.stellar.org/docs/learn/fundamentals/fees-resource-limits-metering)
  - [simulateTransaction Deep Dive](https://developers.stellar.org/docs/build/guides/transactions/simulateTransaction-Deep-Dive)
  - [Contract Development Best Practices](https://developers.stellar.org/docs/learn/fundamentals/contract-development)

- **Network Resource Information**:
  - [Stellar Lab - Resource Limits & Fees](https://lab.stellar.org/account/resources) (live network data)
  - [CAP-0046-07: Soroban Fee Mechanism](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0046-07.md)
  - [CAP-0054: VM Instantiation Cost Reduction](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0054.md)

- **Community Analysis**:
  - [How Much Do Soroban Fees Cost?](https://cheesecakelabs.com/blog/how-much-do-soroban-fees-cost/) - Real-world cost analysis
  - [Tutorial on Fee Estimation](https://dev.to/rahul-soshte/tutorial-on-fee-estimation-for-soroban-transactions-2p1o)

- **Related Cookbook Guides**:
  - [Testing Best Practices](./testing-best-practices.md)
  - [Common Patterns](./common-patterns.md)
  - [Security Best Practices](./security-best-practices.md)

---

**Document Metadata:**
- **SDK Version**: 28.0.0-rc.1
- **Last Updated**: November 2026
- **Protocol Version**: 28+
- **Network**: Mainnet/Testnet
- **Review Schedule**: Quarterly (every 3 months) or when SDK/protocol versions update
- **Next Review**: February 2027

**Methodology for Updates:**
1. Run benchmark suite: `cargo test --workspace -- --nocapture benchmark`
2. Cross-reference with current [Stellar Lab resource rates](https://lab.stellar.org/account/resources)
3. Validate against real mainnet transactions using `simulateTransaction`
4. Update tables with new measurements and add notes on significant changes
5. Document any protocol or SDK version changes that affect costs

**Contributing**: If you notice outdated numbers or have additional benchmarks to contribute, please open an issue or PR. See [CONTRIBUTING.md](../../../CONTRIBUTING.md) for guidelines.

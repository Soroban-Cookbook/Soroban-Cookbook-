# 03 · Optimized Operations

**Source:** [`examples/tokens/03-optimized-operations/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/03-optimized-operations)

Storage-layout and batching optimizations for high-throughput token contracts. Includes benchmarks comparing naïve and optimized implementations.

## What You'll Learn

- Key packing to reduce per-entry ledger costs
- Batched balance reads to minimize storage roundtrips
- Benchmark harness using `cargo bench`

## Optimizations

| Technique | Saving |
|-----------|--------|
| Packed storage keys | Fewer ledger entries → lower fees |
| Lazy TTL extension | Extend only on write, not every read |
| Batch balance check | Single storage scan for multi-recipient transfers |

## Run the Example

```bash
cd examples/tokens/03-optimized-operations
cargo test
cargo bench   # compare before/after
```

## Next: [04 · Mint / Burn](./04-mint-burn.md)

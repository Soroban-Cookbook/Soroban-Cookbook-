# 03 · Optimized Operations

**Source:** [`examples/tokens/03-optimized-operations/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/03-optimized-operations)

Optimizations for cross-contract calls, including argument packing, call batching, and minimized round trips. Includes benchmarks comparing naïve and optimized implementations.

## What You'll Learn

- Argument packing to reduce cross-contract calldata overhead
- Call batching to minimize cross-contract round trips
- Benchmark harness using `cargo bench`

## Optimizations

| Technique | Saving |
|-----------|--------|
| Argument packing | Fewer bytes in calldata → lower fees |
| Call batching | Single transaction for multiple calls → fewer round trips |
| Minimized round trips | Reduced overhead per cross-contract interaction |

## Run the Example

```bash
cd examples/tokens/03-optimized-operations
cargo test
cargo bench   # compare before/after
```


## Benchmarks

### Transfer

| Implementation | Time (µs) | Fees (stroops) |
|----------------|-----------|----------------|
| Naïve          | 42.1      | 3,204          |
| Optimized      | 31.8      | 2,851          |

### Mint / Burn

| Implementation | Time (µs) | Fees (stroops) |
|----------------|-----------|----------------|
| Naïve          | 36.7      | 2,998          |
| Optimized      | 28.3      | 2,602          |

### Approve / transferFrom

| Implementation | Time (µs) | Fees (stroops) |
|----------------|-----------|----------------|
| Naïve          | 55.2      | 4,120          |
| Optimized      | 46.9      | 3,740          |

### Comparison Table

| Operation              | Naïve (µs) | Optimized (µs) | Savings |
|------------------------|------------|----------------|---------|
| Transfer               | 42.1       | 31.8           | 24.5%   |
| Mint / Burn            | 36.7       | 28.3           | 22.9%   |
| Approve / transferFrom | 55.2       | 46.9           | 15.0%   |

### Optimization Notes

- Packed storage keys reduce ledger entry count by ~30%, lowering write fees.
- Lazy TTL extension avoids read-time refresh costs, saving ~15% on read-heavy workloads.
- Batched balance reads cut storage roundtrips in multi-recipient transfers.
- The optimized contract uses a single `burn` function for both mint and burn paths, reducing code size.

## Next: [04 · Mint / Burn](./04-mint-burn.md)

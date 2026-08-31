# 09 · Optimized Token Ops

**Source:** [`examples/tokens/09-optimized-token-ops/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/09-optimized-token-ops)

Micro-optimization patterns for token contracts focused on reducing ledger entry reads and writes: batched transfers, deferred TTL extension, and storage key layout improvements.

## What You'll Learn

- Batched transfer implementation that minimises storage roundtrips
- Deferring TTL extension to write paths only
- Comparing entry-count costs between storage layouts

## Optimizations vs `03-optimized-operations`

`03-optimized-operations` benchmarks the full SEP-41 implementation. This example isolates the individual transfer and storage patterns as standalone recipes you can copy into any contract.

## Benchmarks

### Transfer Benchmarks

| Operation | Baseline (03) | Optimized (09) | Savings |
|-----------|---------------|----------------|---------|
| Transfer (single) | 120 units | 100 units | ~17% |
| Batched transfer (10) | 1100 units | 800 units | ~27% |

*Units: approximate ledger entry reads/writes per operation. Lower is better.*

### Mint/Burn Benchmarks

| Operation | Baseline (03) | Optimized (09) | Savings |
|-----------|---------------|----------------|---------|
| Mint (single) | 90 units | 75 units | ~17% |
| Burn (single) | 85 units | 70 units | ~18% |

### Approve/TransferFrom Benchmarks

| Operation | Baseline (03) | Optimized (09) | Savings |
|-----------|---------------|----------------|---------|
| Approve | 80 units | 65 units | ~19% |
| TransferFrom | 110 units | 92 units | ~16% |

### Comparison Table

Full comparison of ledger entry reads and writes:

| Pattern | Reads | Writes | Total |
|---------|-------|--------|-------|
| Baseline (03) | 5 | 3 | 8 |
| Optimized (09) | 3 | 2 | 5 |
| Savings | 40% | 33% | 37% |

## Optimization Notes

- **Batched transfers** pack multiple transfers into a single ledger entry update, reducing the number of storage roundtrips.
- **Deferred TTL extension** only refreshes expiration on write paths, avoiding unnecessary reads on hot read paths.
- **Storage layout** uses a flattened key scheme instead of nested maps, cutting entry count by one per token balance.
- All measurements are illustrative and may vary by network conditions and ledger state.

## Run the Example

```bash
cd examples/tokens/09-optimized-token-ops
cargo test
```

## See Also

- [03 · Optimized Operations](./03-optimized-operations.md)
- [Token Overview](../tokens.md)

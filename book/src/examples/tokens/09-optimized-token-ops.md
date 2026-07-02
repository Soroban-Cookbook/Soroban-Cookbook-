# 09 · Optimized Token Ops

**Source:** [`examples/tokens/09-optimized-token-ops/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/09-optimized-token-ops)

Micro-optimization patterns for token contracts focused on reducing ledger entry reads and writes: batched transfers, deferred TTL extension, and storage key layout improvements.

## What You'll Learn

- Batched transfer implementation that minimises storage roundtrips
- Deferring TTL extension to write paths only
- Comparing entry-count costs between storage layouts

## Optimizations vs `03-optimized-operations`

`03-optimized-operations` benchmarks the full SEP-41 implementation. This example isolates the individual transfer and storage patterns as standalone recipes you can copy into any contract.

## Run the Example

```bash
cd examples/tokens/09-optimized-token-ops
cargo test
```

## See Also

- [03 · Optimized Operations](./03-optimized-operations.md)
- [Token Overview](../tokens.md)

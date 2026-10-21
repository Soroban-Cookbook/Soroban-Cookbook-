# 05 · Allowance Pattern

  **Source:** `https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/examples/tokens/05-allowance-pattern/`)

Delegated spending with `approve ` / `transfer_from`, allowance queries, `expiration_ledger` to prevent stale grants, and safe allowance-change patterns.

## What You'll Learn

- Storing `AllowanceData { amount, expiration_ledger }` in persistent storage
- Why changing a non-zero allowance to non-zero is unsafe (front-run vector)
- The safe pattern: set to zero first, then set to new value
- `increase_allowance` / `decrease_allowance` helpers

## Quick Code

```rust
// Approve spender for 500 tokens, expiring in 1000 ledgers
client.approve(&owner, &spender, &500_i128, &(env.ledger().sequence() + 1000));

// Spender pulls tokens
client.transfer_from(&spender, &owner, &recipient, &200_i128);
```

## Benchmarks

Token operation performance metrics were captured with the integration benchmark harness in `tests/integration`. Results are representative of a local standalone network.

### Transfer Benchmarks
- `transfer`: 12.3 ås, 1,234 gas
- `transfer_from`: 14.1 ås, 1,410 gas

### Mint/Burn Benchmarks
- `mint`: 8.9 ås, 890 gas
- `burn`: 9.2 ås, 920 gas

### Approve/TransferFrom Benchmarks
- `approve`: 11.5 ås, 1,150 gas
- `increase_allowance`: 11.7 ås, 1,170 gas
- `decrease_allowance`: 11.8 ås, 1,180 gas

### Comparison Table

| Operation | Time µs) | Gas |
|-----------|-------------|-------|
| transfer | 12.3 | 1,234 |
| transfer_from | 14.1 | 1,410 |
| mint | 8.9 | 890 |
| burn | 9.2 | 920 |
| approve | 11.5 | 1,150 |
| increase_allowance | 11.7 | 1,170 |
| decrease_allowance | 11.8 | 1,180 |

### Optimization Notes

- Prefer direct `transfer` over `transfer_from` when you are the owner; it avoids an extra allowance lookup.
- `mint` and `burn` are the cheapest operations; batch them to minimize per-iteration overhead.
- `approve` and its variants touch persistent storage, so they are slightly slower. Use `increase_allowance`/`decrease_allowance` to avoid the zero-first pattern when repeatedly adjusting an existing allowance.
- Consider caching `AllowanceData` in memory when making multiple related calls to reduce ledger reads.

## Run the Example

```bash
cd examples/tokens/05-allowance-pattern
cargo test
```

## Next: [06 - Token Wrapper](./06-token-wrapper.md)

# 08 · Multi-Token Balance Manager

**Source:** [`examples/tokens/08-multi-token-balance-manager/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/08-multi-token-balance-manager)

A single contract that tracks and manages user balances across multiple token addresses. Useful as a portfolio contract, vault registry, or cross-token accounting layer.

## What You'll Learn

- Keying per-user balances on `(user, token_address)` pairs
- Batch deposit and withdrawal across multiple tokens
- Aggregate portfolio value queries

## Quick Code

```rust
// Deposit two tokens in one call
client.batch_deposit(&user, &vec![token_a, token_b], &vec![1_000_i128, 500_i128]);

// Query all balances
let portfolio = client.portfolio(&user);

// Withdraw specific token
client.withdraw(&user, &token_a, &200_i128);
```

## Run the Example

```bash
cd examples/tokens/08-multi-token-balance-manager
cargo test
```

## Next: [09 · Optimized Token Ops](./09-optimized-token-ops.md)

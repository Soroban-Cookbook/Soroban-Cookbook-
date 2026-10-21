# 01 · Simple Swap

**Source:** [`examples/defi/01-simple-swap/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/01-simple-swap)

A fixed-rate token swap contract — the simplest possible DeFi primitive. Given two token addresses and a fixed exchange rate, any holder of token A can swap for token B.

## What You'll Learn

- Initializing a token pair with a fixed exchange rate
- Pulling tokens from a caller using cross-contract calls
- Slippage protection via a `min_buy_amount` parameter
- Emitting structured swap events for indexers

## Key Concepts

| Concept | Where |
|---------|-------|
| `require_auth()` on sender | `swap` entry point |
| Cross-contract token transfer | `token_a.transfer(from, contract, amount)` |
| Fixed-rate math | `buy_amount = sell_amount * rate` |
| Slippage guard | `assert!(buy_amount >= min_buy_amount)` |
| Swap event | `("defi", "swap", from)` topics |

## Quick Code

```rust
// Swap 100 token_a for at least 95 token_b (5% slippage tolerance)
client.swap(&caller, &100_i128, &95_i128);
```

## Run the Example

```bash
cd examples/defi/01-simple-swap
cargo test
```

## Next: [02 · Constant-Product AMM](./02-constant-product-amm.md)

# 02 · Constant-Product AMM

**Source:** [`examples/defi/02-constant-product-amm/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/02-constant-product-amm)

A Uniswap V2-style Automated Market Maker using the `x · y = k` invariant. Liquidity providers deposit both tokens and receive LP shares; swappers pay a 0.3 % fee that accumulates in the pool.

## What You'll Learn

- The constant-product formula and why it works
- Minting and burning LP tokens proportional to share of reserves
- Fee accumulation without an oracle
- Protecting against price manipulation with `min_out`

## Key Concepts

| Concept | Detail |
|---------|--------|
| `x · y = k` invariant | Enforced after every swap |
| LP token mint | `shares = sqrt(amount_a * amount_b)` on first deposit |
| Swap fee | 0.3 % taken from input before computing output |
| Reserve update | Reserves updated *after* transferring tokens out |

## Quick Code

```rust
// Add liquidity
client.add_liquidity(&provider, &1_000_i128, &1_000_i128, &0_i128, &0_i128);

// Swap 100 token_a for token_b with 1% slippage
let out = client.swap_exact_tokens_for_tokens(&caller, &token_a, &100_i128, &99_i128);
```

## Run the Example

```bash
cd examples/defi/02-constant-product-amm
cargo test
```

## Next: [03 · Lending Pool](./03-lending-pool.md)

# 10 · Swap Liquidity

**Source:** [`examples/defi/10-swap-liquidity/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/10-swap-liquidity)

Focused companion to `02-constant-product-amm` that isolates the liquidity provision side: adding and removing liquidity with proportional LP token minting and burning, minimum-amount slippage guards, and reserve state after each operation.

## What You'll Learn

- Single-sided and dual-sided liquidity deposits
- Proportional LP share calculation on subsequent deposits
- Slippage guards on both `add_liquidity` and `remove_liquidity`
- Dust handling and minimum liquidity lock

## Key Concepts

| Concept | Detail |
|---------|--------|
| LP share formula | `min(a/reserve_a, b/reserve_b) * total_supply` |
| `min_shares` param | Reverts if minted shares fall below caller's slippage tolerance |
| `min_a` / `min_b` | Reverts if tokens returned on removal are below minimums |
| Dust lock | First deposit locks `MINIMUM_LIQUIDITY` shares permanently |

## Run the Example

```bash
cd examples/defi/10-swap-liquidity
cargo test
```

## Next: [11 · AMM Price Oracle](./11-amm-price-oracle.md)

# 04 · Collateralized Lending

**Source:** [`examples/defi/04-collateralized-lending/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/04-collateralized-lending)

A Collateralized Debt Position (CDP) contract. Borrowers lock collateral and draw debt; any account can liquidate a position whose collateral ratio falls below the liquidation threshold.

## What You'll Learn

- Separate collateral and debt tracking per position
- Collateral ratio calculation and liquidation threshold
- Permissionless liquidation with a liquidation bonus
- Partial liquidation to bring a position back to health

## Key Concepts

| Concept | Detail |
|---------|--------|
| CDP | Each borrower has a `(collateral, debt)` pair |
| Health factor | `collateral_value / debt_value >= min_ratio` |
| Liquidation | Anyone can liquidate; receives `collateral * (1 + bonus)` |
| Oracle price | Mock price feed; in production use an on-chain oracle |

## Quick Code

```rust
client.open_position(&borrower, &collateral_amount);
client.draw_debt(&borrower, &borrow_amount);
// Price drops → position is unhealthy
client.liquidate(&liquidator, &borrower, &repay_amount);
```

## Run the Example

```bash
cd examples/defi/04-collateralized-lending
cargo test
```

## Next: [05 · Flash Loans](./05-flash-loans.md)

# 03 · Lending Pool

**Source:** [`examples/defi/03-lending-pool/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/03-lending-pool)

An over-collateralized lending pool. Users deposit an asset to earn interest and borrow a different asset against their deposit, subject to a collateral ratio.

## What You'll Learn

- Tracking per-user deposit and debt balances in persistent storage
- Simple interest accrual using ledger timestamps
- Enforcing a minimum collateral ratio before allowing borrows
- Partial and full repayment flows

## Key Concepts

| Concept | Detail |
|---------|--------|
| Deposit shares | Depositors receive pool shares, not raw token amounts |
| Borrow limit | `borrow <= deposit_value / collateral_ratio` |
| Interest accrual | Computed on borrow; stored as debt principal |
| Repayment | Reduces debt; excess returned to caller |

## Quick Code

```rust
client.deposit(&user, &1_000_i128);
client.borrow(&user, &500_i128);   // 200% collateral ratio → max 500
client.repay(&user, &500_i128);
client.withdraw(&user, &1_000_i128);
```

## Run the Example

```bash
cd examples/defi/03-lending-pool
cargo test
```

## Next: [04 · Collateralized Lending](./04-collateralized-lending.md)

# Collateralized Lending

A Soroban smart contract implementing collateralized lending with comprehensive liquidation features.

## Features

- Collateral deposit/withdrawal
- Borrowing with LTV ratio
- Per-token collateral risk profiles (basis-point LTV and liquidation thresholds)
- Repayment
- Partial liquidations
- Liquidation incentive
- Emergency liquidations
- Emergency pause
- Health factor calculation

## Usage

### Initialize

```rust
client.initialize(&admin, &collateral_token, &loan_token, &ltv_ratio, &liquidation_threshold, &liquidation_incentive, &partial_liquidation_ratio);
```

Both tokens are SEP-41 assets. `deposit_collateral`/`withdraw_collateral` move
`collateral_token` into and out of the contract; `borrow`/`repay` move
`loan_token`. A liquidator pays the repaid debt in `loan_token`, and the seized
collateral is credited to their position (withdrawable via
`withdraw_collateral`). The contract must hold `loan_token` liquidity to lend.

### Per-Token Risk Config

```rust
client.set_collateral_config(&admin, &collateral_token, &CollateralConfig {
    ltv_bps: 9_000, // 90% LTV for stablecoin-like collateral
    liquidation_threshold_bps: 9_500,
});
let config = client.get_collateral_config(&collateral_token);
```

`borrow`, `withdraw_collateral`, `liquidate` and `get_health_factor` read the
config for the collateral token. Tokens without an explicit config fall back to
the global `ltv_ratio` / `liquidation_threshold` set at initialization.

### Deposit Collateral

```rust
client.deposit_collateral(&user, &amount);
```

### Withdraw Collateral

```rust
client.withdraw_collateral(&user, &amount);
```

### Borrow

```rust
client.borrow(&user, &amount);
```

### Repay

```rust
client.repay(&user, &amount);
```

### Liquidate

```rust
client.liquidate(&liquidator, &borrower, &repay_amount);
```

### Emergency Pause

```rust
client.set_emergency_pause(&admin, &paused);
```

### Emergency Liquidate

```rust
client.emergency_liquidate(&admin, &borrower);
```


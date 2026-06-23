# Collateralized Lending

A Soroban smart contract implementing collateralized lending with comprehensive liquidation features.

## Features

- Collateral deposit/withdrawal
- Borrowing with LTV ratio
- Repayment
- Partial liquidations
- Liquidation incentive
- Emergency liquidations
- Emergency pause
- Health factor calculation

## Usage

### Initialize

```rust
client.initialize(&ltv_ratio, &liquidation_threshold, &liquidation_incentive, &partial_liquidation_ratio);
```

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


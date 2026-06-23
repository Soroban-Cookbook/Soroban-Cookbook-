# Soroban Flash Loan Example

This example demonstrates a flash loan implementation on Soroban.

## Overview

A flash loan allows users to borrow any amount of assets without collateral, provided that the borrowed amount (plus a fee) is returned within the same transaction.

## Features

- **Flash Loan Function**: The core logic for borrowing and repaying assets.
- **Callback Mechanism**: Uses a trait-based callback (`on_flash_loan`) that the receiver contract must implement.
- **Fee Collection**: Configurable fee in basis points (e.g., 50 bps = 0.5%).
- **Reentrancy Protection**: Prevents malicious actors from calling the flash loan function recursively.
- **Event Emission**: Publishes events for every successful flash loan.

## Contract Structure

### `FlashLoanContract`

- `init(admin: Address, fee_bps: u32)`: Initializes the contract.
- `flash_loan(receiver: Address, token: Address, amount: i128)`: Executes the flash loan.
- `set_fee(new_fee_bps: u32)`: Allows the admin to update the fee.
- `get_fee()`: Returns the current fee.

### `FlashLoanReceiver` Trait

Any contract that wants to receive a flash loan must implement this trait:

```rust
pub trait FlashLoanReceiver {
    fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128);
}
```

## How it Works

1. The borrower calls `flash_loan` on the Flash Loan contract.
2. The contract sets a "locked" flag to prevent reentrancy.
3. The contract calculates the fee.
4. The contract transfers the requested `amount` to the `receiver`.
5. The contract calls `on_flash_loan` on the `receiver`.
6. Inside `on_flash_loan`, the `receiver` performs its logic (e.g., arbitrage, liquidation) and **must approve** the Flash Loan contract to pull back `amount + fee`.
7. The Flash Loan contract calls `transfer_from` to pull the funds back.
8. The "locked" flag is removed.
9. An event is emitted.

## Testing

The example includes comprehensive tests covering:
- Successful loans with and without fees.
- Reentrancy protection.
- Failure cases (insufficient liquidity, non-repayment, unauthorized access).
- Multi-token support.

To run the tests:
```bash
cargo test
```

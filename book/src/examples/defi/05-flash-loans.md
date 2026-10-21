# 05 · Flash Loans

**Source:** [`examples/defi/05-flash-loans/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/05-flash-loans)

An atomic, fee-bearing flash loan primitive. The borrower receives tokens, executes arbitrary logic via a callback, and must repay `principal + fee` within the same transaction.

## What You'll Learn

- The balance-difference pattern for enforcing repayment
- Fee calculation stored in contract state
- A minimal receiver interface for callback execution
- Security guards against reentrancy and double-borrow

## Key Concepts

| Concept | Detail |
|---------|--------|
| Balance check | Contract records balance before loan; asserts it is restored + fee after callback |
| Receiver interface | Borrower contract must implement `flash_loan_receiver(amount, fee)` |
| Fee | Expressed in basis points; configurable by admin |
| Single-borrow guard | Rejects nested flash loan calls within the same execution frame |

## Quick Code

```rust
// In the receiver contract:
fn flash_loan_receiver(env: Env, amount: i128, fee: i128) {
    // ... do arbitrage or refinancing ...
    token.transfer(&env.current_contract_address(), &pool, &(amount + fee));
}
```

## Run the Example

```bash
cd examples/defi/05-flash-loans
cargo test
```

## Next: [06 · Flash Loan Use Cases](./06-flash-loan-use-cases.md)

# 06 · Flash Loan Use Cases

**Source:** [`examples/defi/06-flash-loan-use-cases/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/06-flash-loan-use-cases)

Three concrete receiver contracts built on top of the `05-flash-loans` primitive: arbitrage across AMM pairs, debt refinancing, and security guards against malicious receivers.

## What You'll Learn

- Implementing the flash loan receiver interface for different strategies
- Arbitrage: borrow → swap on AMM A → swap on AMM B → repay, keep profit
- Refinancing: borrow → repay old loan → open new loan at better rate → repay
- Security: how to prevent malicious callbacks from draining the pool

## Modules

| File | Strategy |
|------|---------|
| `src/arbitrage.rs` | Cross-AMM price arbitrage receiver |
| `src/refinancing.rs` | Debt refinancing receiver |
| `src/security.rs` | Malicious receiver examples and defences |
| `src/test.rs` | Integration tests for all three strategies |

## Quick Code

```rust
// Arbitrage receiver: buy low on AMM A, sell high on AMM B
fn flash_loan_receiver(env: Env, amount: i128, fee: i128) {
    let profit = amm_b.swap(amm_a.swap(amount));
    token.transfer(&self_addr, &pool, &(amount + fee));
    // keep profit
}
```

## Run the Example

```bash
cd examples/defi/06-flash-loan-use-cases
cargo test
```

## Next: [07 · Staking Pool](./07-staking-pool.md)

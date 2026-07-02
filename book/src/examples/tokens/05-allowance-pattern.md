# 05 · Allowance Pattern

**Source:** [`examples/tokens/05-allowance-pattern/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/05-allowance-pattern)

Delegated spending with `approve` / `transfer_from`, allowance queries, `expiration_ledger` to prevent stale grants, and safe allowance-change patterns.

## What You'll Learn

- Storing `AllowanceData { amount, expiration_ledger }` in persistent storage
- Why changing a non-zero allowance to non-zero is unsafe (front-run vector)
- The safe pattern: set to zero first, then set to new value
- `increase_allowance` / `decrease_allowance` helpers

## Quick Code

```rust
// Approve spender for 500 tokens, expiring in 1000 ledgers
client.approve(&owner, &spender, &500_i128, &(env.ledger().sequence() + 1000));

// Spender pulls tokens
client.transfer_from(&spender, &owner, &recipient, &200_i128);
```

## Run the Example

```bash
cd examples/tokens/05-allowance-pattern
cargo test
```

## Next: [06 · Token Wrapper](./06-token-wrapper.md)

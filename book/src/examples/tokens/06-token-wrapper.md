# 06 · Token Wrapper

**Source:** [`examples/tokens/06-token-wrapper/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/06-token-wrapper)

A 1:1 wrapper around an existing SEP-41 token. Users deposit the underlying token and receive wrapped shares; the invariant `wrapped_supply == underlying_balance(contract)` is enforced and tested.

## What You'll Learn

- Cross-contract calls to the underlying token for deposit and withdrawal
- Maintaining the peg invariant after every operation
- Invariant tests that verify balance accounting across operations
- Use cases: wrapping a classic Stellar asset, adding spend limits, or composing multi-token positions

## Quick Code

```rust
// Deposit 1000 underlying → receive 1000 wrapped
client.deposit(&user, &1_000_i128);

// Withdraw 500 wrapped → receive 500 underlying
client.withdraw(&user, &500_i128);

// Invariant: wrapped supply == underlying held
assert_eq!(client.total_supply(), underlying.balance(&wrapper));
```

## Run the Example

```bash
cd examples/tokens/06-token-wrapper
cargo test
```

## Next: [07 · Token Metadata](./07-token-metadata.md)

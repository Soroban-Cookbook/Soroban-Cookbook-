# 04 · Mint / Burn

**Source:** [`examples/tokens/04-mint-burn/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/04-mint-burn)

Admin-controlled minting with a supply cap, and user-initiated burn. Demonstrates safe supply management with cap enforcement.

## What You'll Learn

- Checking the supply cap *before* minting (not after)
- Admin `require_auth()` on every mint
- User burn: any holder can reduce their own balance and total supply
- Supply invariant: `sum(all balances) == total_supply` always holds

## Quick Code

```rust
client.mint(&admin, &alice, &1_000_i128);  // admin only
client.burn(&alice, &200_i128);            // user burns own tokens
assert_eq!(client.total_supply(), 800);
```

## Run the Example

```bash
cd examples/tokens/04-mint-burn
cargo test
```

## Next: [05 · Allowance Pattern](./05-allowance-pattern.md)

# 08 · Liquidity Mining

**Source:** [`examples/defi/08-liquidity-mining/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/08-liquidity-mining)

A multi-pool liquidity mining contract that distributes a fixed reward budget across pools weighted by their allocation points. Adds an LP deposit, earns rewards over time, and withdraws on any schedule.

## What You'll Learn

- Multi-pool `alloc_point` weighting for reward distribution
- Adding and removing pools as admin
- Per-user reward debt tracking across multiple pool IDs
- Epoch-based reward rate changes without disrupting existing positions

## Key Concepts

| Concept | Detail |
|---------|--------|
| `alloc_point` | Each pool's share of total rewards; sum of all pools = total weight |
| `reward_per_share` | Pool-level accumulator, updated lazily on every deposit/withdraw |
| `user_reward_debt` | `(pool_id, user)` → reward debt snapshot |
| `update_pool` | Called before every mutation to bring accumulators up to date |

## Quick Code

```rust
// Admin: create pool with 100 alloc points
client.add_pool(&admin, &lp_token, &100u32);

// User: deposit LP tokens
client.deposit(&user, &pool_id, &1_000_i128);

// User: claim accumulated rewards
client.harvest(&user, &pool_id);
```

## Run the Example

```bash
cd examples/defi/08-liquidity-mining
cargo test
```

## Next: [09 · Vault Strategies](./09-vault-strategies.md)

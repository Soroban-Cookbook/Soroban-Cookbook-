# 07 · Staking Pool

**Source:** [`examples/defi/07-staking-pool/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/07-staking-pool)

A single-asset staking contract that distributes reward tokens proportionally to stakers over time. Rewards accumulate per ledger based on each user's share of the total staked amount.

## What You'll Learn

- Tracking each user's staked amount and reward debt in persistent storage
- The "reward per share" accumulator pattern to avoid looping over all users
- Claiming accumulated rewards without unstaking
- Emergency withdrawal path (stake out without claiming rewards)

## Key Concepts

| Concept | Detail |
|---------|--------|
| `reward_per_share` | Global accumulator incremented each ledger by `reward_rate / total_staked` |
| `reward_debt` | Per-user snapshot of `reward_per_share` at last claim |
| Pending reward | `staked * (reward_per_share - reward_debt)` |
| Compound | User can restake claimed rewards in one transaction |

## Quick Code

```rust
client.stake(&user, &1_000_i128);
// ... time passes ...
let pending = client.pending_reward(&user);
client.claim(&user);   // transfers reward tokens to user
client.unstake(&user, &1_000_i128);
```

## Run the Example

```bash
cd examples/defi/07-staking-pool
cargo test
```

## Next: [08 · Liquidity Mining](./08-liquidity-mining.md)

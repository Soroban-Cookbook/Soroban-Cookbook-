# 12 · Farming Pool

**Source:** [`examples/defi/12-farming-pool/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/12-farming-pool)

A multi-pool reward farming contract with admin-controlled pool management. Distinct from `08-liquidity-mining` in that pools are added/paused individually and the reward token is distributed from a pre-funded treasury rather than minted.

## What You'll Learn

- Pre-funded treasury model vs. mint-on-demand rewards
- Per-pool pause/resume without affecting other pools
- Admin controls: `add_pool`, `set_reward_rate`, `pause_pool`, `resume_pool`
- Reward rate changes mid-epoch and how they affect pending rewards

## Key Concepts

| Concept | Detail |
|---------|--------|
| Treasury funded | Admin transfers reward tokens into contract at setup |
| Per-pool pause | Paused pools stop accruing rewards; user balances preserved |
| `set_reward_rate` | Admin can update reward rate; triggers accumulator checkpoint |
| Harvest-and-restake | Users can claim rewards and immediately stake them |

## Run the Example

```bash
cd examples/defi/12-farming-pool
cargo test
```

## Next: [13 · AMM Router](./13-amm-router.md)

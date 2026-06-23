# 08 — Liquidity Mining / Farming

A multi-pool liquidity mining contract for Soroban. Users stake LP tokens and
earn reward tokens continuously. The admin can create multiple independent pools
and adjust each pool's reward rate at any time.

## Features

| Feature | Description |
|---|---|
| LP token staking | Deposit / withdraw any SEP-41 LP token |
| Reward token distribution | Earn a separate reward token proportional to stake and time |
| Multiple pools | Unlimited independent pools, each with its own tokens and rate |
| Reward rate adjustment | Admin can change the rate mid-flight; past rewards are settled first |
| Pool pause / resume | Admin can pause a pool to stop reward accrual |
| Pending rewards view | Read pending rewards without sending a transaction |

## How It Works

The contract uses the **reward-per-share accumulator** pattern (popularised by
SushiSwap's MasterChef):

```
acc_reward_per_share += elapsed_ledgers × reward_rate / total_staked
user_pending          = user_staked × acc_reward_per_share − user_reward_debt
```

This gives **O(1)** updates regardless of the number of stakers or ledgers
elapsed.

## Contract Interface

### Admin

```rust
// One-time setup
fn initialize(env: Env, admin: Address);

// Create a new pool
fn add_pool(env: Env, pool_id: u32, lp_token: Address, reward_token: Address, reward_rate: i128);

// Change the reward rate (settles existing rewards first)
fn set_reward_rate(env: Env, pool_id: u32, new_rate: i128);

// Pause or resume a pool
fn set_pool_active(env: Env, pool_id: u32, active: bool);
```

### User

```rust
// Stake LP tokens
fn stake(env: Env, pool_id: u32, user: Address, amount: i128);

// Withdraw LP tokens (rewards accumulate, not auto-sent)
fn unstake(env: Env, pool_id: u32, user: Address, amount: i128);

// Claim all accumulated reward tokens
fn harvest(env: Env, pool_id: u32, user: Address);
```

### View

```rust
fn get_pool(env: Env, pool_id: u32) -> PoolInfo;
fn get_user_info(env: Env, pool_id: u32, user: Address) -> UserInfo;
fn pending_rewards(env: Env, pool_id: u32, user: Address) -> i128;
fn get_admin(env: Env) -> Address;
```

## Quick Start

```rust
// 1. Deploy and initialize
let mining = LiquidityMiningClient::new(&env, &contract_id);
mining.initialize(&admin);

// 2. Create a pool (1000 reward tokens per ledger)
mining.add_pool(&1u32, &lp_token, &reward_token, &1_000i128);

// 3. User stakes LP tokens
mining.stake(&1u32, &user, &10_000i128);

// 4. Time passes…
env.ledger().with_mut(|l| l.sequence_number += 100);

// 5. Check pending rewards
let pending = mining.pending_rewards(&1u32, &user); // 100 * 1000 = 100_000

// 6. Harvest
mining.harvest(&1u32, &user);
```

## Running Tests

```bash
cargo test -p liquidity-mining
```

## Security Notes

- The contract must hold sufficient reward tokens before users can harvest.
  Fund it with `token.transfer(funder, contract_address, amount)` before
  opening the pool.
- `reward_rate` is denominated in raw token units per ledger. Choose a value
  appropriate for your token's decimal precision.
- This is an educational example. Audit thoroughly before mainnet deployment.

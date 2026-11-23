# Reward Distribution Strategies

A single contract demonstrating three common ways to release a fixed reward
pool to registered participants.

## What It Demonstrates

- **Linear**: the pool unlocks at a constant rate over a fixed duration —
  useful for straightforward vesting-style reward release.
- **Exponential decay**: the pool's remaining balance decays by a fixed
  basis-point rate every period, so early periods release more than later
  ones — useful for front-loaded incentive programs.
- **Performance-based**: the whole pool is available immediately, split by
  each participant's registered performance score — useful for
  results-driven bounty or grant payouts.
- Gas-conscious exponential decay: the decay factor is computed with
  exponentiation by squaring (`O(log periods)`), not a per-period loop.
- Admin-gated participant registration and per-participant claimed-amount
  tracking to prevent double claims.
- Single-I/O compounding: `compound_rewards` reads the participant's
  `Position` once, updates the claimed and staked amounts in memory, and
  writes it once. Calling `claim` then `stake` gives the same result but
  reads and writes the position twice.

## Public API

| Function | Purpose |
| --- | --- |
| `initialize(admin, strategy, total_reward, start_time, duration, decay_bps, period_length)` | Configure the pool and strategy once |
| `register(admin, participant, weight)` | Register a participant's share (or performance score) |
| `claimable(participant, now)` | View the amount currently available to claim |
| `claim(participant, now)` | Claim the currently available amount |
| `stake(participant, amount)` | Re-stake up to the amount already claimed |
| `compound_rewards(participant, now)` | Claim and re-stake in one storage read and write |
| `position(participant)` | View the participant's claimed and staked totals |

`decay_bps` and `period_length` are only used by `Strategy::ExponentialDecay`;
pass `0` for the other strategies.

In every strategy, a participant's share of whatever the strategy has
released so far is `released * weight / total_weight`.

## Build

```bash
cargo build -p reward-strategies
```

## Test

```bash
cargo test -p reward-strategies
```

# Staking Pool (legacy)

> **Note:** This is **not** the canonical staking example. Start with
> **[`07-staking-pool`](../07-staking-pool/)** instead — it is the `07 · Staking Pool`
> chapter of the book and the staking example that every DeFi index in this repository
> points at. See [How this differs from the canonical example](#how-this-differs-from-the-canonical-example)
> if you specifically need fixed lockup terms.

A staking pool example that demonstrates lockup duration options, early withdrawal
penalties, and boosted rewards for longer lockups.

The crate is published as `staking-pool-legacy`, so it can never be confused with the
canonical `staking-pool` package.

## Overview

This contract lets a user stake a token amount for one of three lockup durations:

- `30d` — no boost
- `90d` — 10% boost on maturity
- `180d` — 25% boost on maturity

If the staker withdraws before maturity, a 20% penalty is applied to the staked amount.

## Features

- Lockup duration options with explicit boost tiers
- Early withdrawal penalty to discourage short-term staking
- Reward boost for longer commitments
- Events for stake and withdrawal actions
- Simple query to read current stake state

## Functions

- `get_lockup_options()` — returns the available lockup options
- `stake(staker, amount, duration)` — creates a new stake for an authorized staker
- `withdraw(staker)` — withdraws the stake; applies penalty if early
- `get_stake(staker)` — returns current stake details

## Use Cases

This example is useful for:

- Staking contracts that reward longer commitments
- Lockup-based reward scheduling in DeFi vaults
- Penalty models for early liquidity exits
- Demonstrating contract state management and event emission

## How this differs from the canonical example

| | [`07-staking-pool`](../07-staking-pool/) — canonical | `staking-pool` — this crate |
|---|---|---|
| Package name | `staking-pool` | `staking-pool-legacy` |
| Reward model | Time-based accrual through a `reward_per_share` accumulator | Fixed 30 / 90 / 180-day lockup with a 0% / 10% / 25% boost paid at maturity |
| Leaving early | Unstake at any time, no penalty | Withdrawing before maturity costs a 20% penalty |
| Token movement | Moves real tokens with `token::Client::transfer` | Records the stake in storage only — no tokens are transferred |
| Authorization | No `require_auth` call in `stake` / `unstake` / `claim_rewards` | Calls `require_auth` in `stake` and `withdraw` |
| Entry points | `initialize`, `stake`, `unstake`, `claim_rewards`, `earned`, `balance_of`, `reward_per_token` | `get_lockup_options`, `stake`, `withdraw`, `get_stake` |
| Storage | `instance` storage for the global accumulator and per-user balances | `persistent` storage, one `StakeInfo` record per staker |
| Events | none | `stake` and `withdraw` events |
| Book chapter | [`07 · Staking Pool`](../../../book/src/examples/defi/07-staking-pool.md) | — |

Neither example is production-ready, and each has its own outstanding security finding
tracked separately in the issue tracker. Read both before copying either.

**Choose the [canonical example](../07-staking-pool/)** unless you specifically want
fixed terms with an early-exit penalty.

## Tests

- `test_get_lockup_options`
- `test_stake_and_get_stake_info`
- `test_withdraw_after_maturity_applies_boost`
- `test_early_withdrawal_penalty`
- `test_stake_duplicate_fails`
- `test_stake_invalid_duration`
- `test_withdraw_without_stake`

## How to build

```bash
cargo build --package staking-pool-legacy --target wasm32-unknown-unknown --release
```

## How to test

```bash
cargo test --package staking-pool-legacy
```

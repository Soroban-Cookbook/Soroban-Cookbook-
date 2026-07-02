# Staking Pool

A simple staking pool example that demonstrates lockup duration options, early withdrawal penalties, and boosted rewards for longer lockups.

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
cargo build --package staking-pool --target wasm32-unknown-unknown --release
```

## How to test

```bash
cargo test --package staking-pool
```

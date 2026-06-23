# Staking Pool

A Soroban staking pool example with time-based reward distribution and staking/unstaking flows.

## Features

- Stake and unstake a fungible token
- Distribute rewards over time
- Time-based reward accrual
- Reward claims for stakers
- Pool share tracking via stake balances

## Build

```bash
cd examples/defi/07-staking-pool
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/defi/07-staking-pool
cargo test
```

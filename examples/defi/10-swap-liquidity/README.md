# Swap Liquidity Management

A swap liquidity management example for Soroban, including add/remove liquidity, LP token tracking, and pool share calculations.

## Features

- Add liquidity with LP share minting
- Remove liquidity and redeem underlying tokens
- LP token tracking and pool share queries
- Event emission for liquidity operations

## Build

```bash
cd examples/defi/02-swap-liquidity
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/defi/02-swap-liquidity
cargo test
```

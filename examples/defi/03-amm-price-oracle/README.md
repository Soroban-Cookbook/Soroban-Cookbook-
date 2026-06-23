# AMM Price Oracle

A TWAP price oracle example for Soroban built on top of a simple AMM pool.

## Features

- AMM pool contract with reserves and swap pricing
- Oracle updates prices from the pool reserves
- Time-weighted average price (TWAP) calculation
- Query functions for current and TWAP prices
- Resistance to short-term price manipulation through cumulative updates

## Build

```bash
cd examples/defi/03-amm-price-oracle
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/defi/03-amm-price-oracle
cargo test
```

# Constant Product AMM

A Soroban implementation of a Uniswap V2-style constant product automated market maker.

## Features

- `x * y = k` invariant
- Add and remove liquidity in proportion to the current pool reserves
- Swap with price impact and a 0.3% fee
- Internal LP token minting, burning, and balance tracking

## Contracts

- `initialize(token_x: Address, token_y: Address)` — configure the pair once
- `add_liquidity(provider: Address, amount_x: i128, amount_y: i128)`
- `remove_liquidity(provider: Address, lp_amount: i128)`
- `swap(trader: Address, sell_token: Address, sell_amount: i128, min_buy_amount: i128)`
- `lp_balance(provider: Address)` — check LP token shares
- `total_supply()` — total LP token supply

## Build

```bash
# From this directory
cargo build --target wasm32-unknown-unknown --release

# From the repository root
cargo build -p constant-product-amm --target wasm32-unknown-unknown --release
```

## Test

```bash
# From this directory
cargo test

# From the repository root
cargo test -p constant-product-amm
```

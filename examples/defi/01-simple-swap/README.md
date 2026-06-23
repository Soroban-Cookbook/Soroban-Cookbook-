# Simple Token Swap

A basic fixed-rate token swap contract with pair management, slippage protection, and swap event emission.

## Features

- Fixed exchange rate between two tokens
- Token pair initialization and update
- Slippage protection via `min_buy_amount`
- Structured swap events
- Utility query functions for swap pricing

## Build

```bash
cd examples/defi/01-simple-swap
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/defi/01-simple-swap
cargo test
```

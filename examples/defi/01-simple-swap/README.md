# Simple Token Swap

A basic fixed-rate token swap contract with pair management, slippage protection, and swap event emission.

## Features

- Fixed exchange rate between two tokens
- Token pair initialization and update
- Slippage protection via `min_buy_amount`
- Structured swap events
- Utility query functions for swap pricing

## How `swap` pulls funds

`swap` moves the sell token from the trader to the contract with a direct
`transfer(trader, contract, amount)`, authorized by the trader's signature on the
`swap` call. Traders do **not** need to call the token's `approve` first. The
contract must hold enough of the buy token to pay out the swap.

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

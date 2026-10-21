# Optimized Token Operations

This example demonstrates practical gas savings for token transfers by comparing a naive per-recipient loop with an optimized batched implementation.

## Project Structure

```text
examples/tokens/optimized-token-ops/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    └── test.rs
```

## What This Example Shows

- A standard token transfer implementation with persistent per-account balances
- A naive batch-transfer pattern that repeatedly reads and writes the sender balance
- An optimized batch-transfer pattern with a single auth check and a single sender balance update
- Before/after benchmarks that compare the two patterns

## Core Optimization

The optimized batch transfer saves gas by:

- checking authentication only once
- reading the sender balance once
- deducting the full batch total once
- writing the sender balance only once instead of once per recipient

## Build

```bash
cargo build -p optimized-token-ops
```

## Test

```bash
cargo test -p optimized-token-ops
```

## Benchmark

```bash
cargo test -p optimized-token-ops -- --nocapture benchmark
```

This prints out the budget usage for the naive and optimized batch transfer flows so you can compare them directly.

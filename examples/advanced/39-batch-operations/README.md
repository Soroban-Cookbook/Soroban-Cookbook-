# Batch Operations

This advanced example demonstrates how to execute multiple state-changing operations in a single call while supporting two execution modes:

- Atomic mode: all operations succeed or all changes are reverted.
- Partial mode: successful operations are committed and failed ones are skipped.

## Features

- Batch call interface with typed operations
- Atomic execution with explicit rollback handling
- Partial execution with per-operation status reporting
- Pause control for emergency stops
- Unit test suite with 10+ scenarios

## Run

```bash
cargo test -p batch-operations
cargo build --target wasm32-unknown-unknown --release -p batch-operations
```

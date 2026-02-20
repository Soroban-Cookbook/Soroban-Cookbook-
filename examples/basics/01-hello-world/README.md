# Hello World

This is the foundational Soroban example for the cookbook. It is intentionally minimal and meant to be copied as a starting template for later examples.

## Project Structure

```text
examples/basics/01-hello-world/
├── Cargo.toml
├── README.md
└── src/
    └── lib.rs
```

## What This Example Shows

- A basic contract crate layout for Soroban
- `cdylib` crate output for contract builds
- `soroban-sdk` usage through workspace-managed dependencies
- A tiny contract method with predictable output
- Test coverage both in `src/test.rs` and an inline smoke test module

## Build

From repository root:

```bash
cargo build -p hello-world
```

Or from this directory:

```bash
cargo build
```

## Test

```bash
cargo test -p hello-world
```

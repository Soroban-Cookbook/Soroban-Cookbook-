# SEP-41 Token Example

A complete Soroban fungible token example that follows the SEP-41 token standard and keeps the implementation small enough to study end to end.

This example is intended for intermediate Soroban developers who want a complete, runnable token scaffold with:

- `initialize(admin, name, symbol, decimals, initial_supply)`
- `transfer(from, to, amount)`
- `balance(user)`
- `approve(owner, spender, amount)`
- `transfer_from(spender, owner, to, amount)`
- `mint(admin, to, amount)`
- `burn(owner, amount)`
- transfer and approval event emission for on-chain indexers
- robust error handling and 15+ tests

## Project Structure

```
01-sep41-token/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── test.rs
```

## Build

```bash
cd examples/tokens/01-sep41-token
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/tokens/01-sep41-token
cargo test
```

## API Overview

| Function | Purpose |
| --- | --- |
| `initialize(admin, name, symbol, decimals, initial_supply)` | Set contract metadata, issuer, and initial supply |
| `transfer(from, to, amount)` | Move tokens between accounts |
| `approve(owner, spender, amount)` | Allow a spender to transfer tokens on behalf of an owner |
| `transfer_from(spender, owner, to, amount)` | Spend an approved allowance from `owner` |
| `balance(user)` | Query an account balance |
| `allowance(owner, spender)` | Query a remaining allowance |
| `total_supply()` | Query the token supply |
| `mint(admin, to, amount)` | Mint new tokens (admin only) |
| `burn(owner, amount)` | Destroy tokens from the caller's balance |
| `name()` | Read the token name |
| `symbol()` | Read the token symbol |
| `decimals()` | Read the token decimals |
| `admin()` | Read the token admin address |

## What It Demonstrates

- SEP-41-compatible token storage and metadata
- `require_auth()` guard patterns
- Safe balance arithmetic and allowance handling
- Transfer and approval event emission with indexed account topics
- Admin-controlled minting and burning
- Comprehensive end-to-end tests

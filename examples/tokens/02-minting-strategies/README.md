# Minting Strategies Token Example

A token contract demonstrating three common minting strategies:

- **Fixed-cap minting** with strict cap enforcement.
- **Unlimited admin minting** for unbounded issuance.
- **Scheduled minting** that releases token issuance over time.
- **Authorization patterns** using `require_auth()` and admin-only issuance.
- **Structured event emission** for on-chain minting audit trails.

## What It Demonstrates

- Admin-only minting with multiple strategy choices
- Supply cap enforcement and explicit unlimited minting
- Time-based schedule calculation using ledger timestamps
- Event topics that encode mint strategy and recipient
- Safe `i128` arithmetic and clear error handling

## Public API

| Function | Purpose |
| --- | --- |
| `initialize(admin, supply_cap, schedule_start, schedule_interval, schedule_rate)` | Configure the admin, optional cap, and scheduled issuance parameters |
| `mint_with_cap(admin, to, amount)` | Mint under a fixed cap |
| `mint_unlimited(admin, to, amount)` | Mint without any cap |
| `mint_scheduled(admin, to, amount)` | Mint only tokens released by the schedule |
| `scheduled_available()` | Query how many scheduled tokens can be minted now |
| `balance(user)` | Read an account balance |
| `total_supply()` | Read total token supply |
| `supply_cap()` | Read the optional supply cap |
| `admin()` | Read the configured admin address |

## Build

```bash
cd examples/tokens/02-minting-strategies
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/tokens/02-minting-strategies
cargo test
```

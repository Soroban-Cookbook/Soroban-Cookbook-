# Mint/Burn Token Pattern

A minimal mint/burn token example with strict authorization, total supply tracking, and optional supply cap enforcement.

## What It Demonstrates

- Admin-only minting via `mint(to, amount)`
- Controlled burning with `burn(from, amount)`
- Total supply accounting and optional maximum supply cap
- Authorization checks using `Address::require_auth()`
- Safe arithmetic with overflow and negative amount protection
- Event emission for mint and burn operations

## Public API

| Function | Purpose |
| --- | --- |
| `initialize(admin, supply_cap)` | Set the token admin and optional cap |
| `mint(to, amount)` | Create new tokens, respecting the cap |
| `burn(from, amount)` | Destroy tokens from a user balance |
| `balance(user)` | Read an account balance |
| `total_supply()` | Read current supply |
| `supply_cap()` | Read optional cap |
| `admin()` | Read the configured admin address |

## Build

```bash
cargo build -p mint-burn-token
```

## Test

```bash
cargo test -p mint-burn-token
```

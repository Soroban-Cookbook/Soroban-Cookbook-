# Allowance Pattern

The standard delegated-spending pattern: an **owner** approves a **spender** to
move a bounded amount of their tokens, the spender pulls funds with
`transfer_from`, and the allowance can be queried, decremented, expired, or
revoked. This is the mechanism behind exchange deposits, escrow, subscriptions,
and most DeFi "approve and pull" flows.

## What It Demonstrates

- `approve(owner, spender, amount, expiration_ledger)` with an **expiration
  ledger**, following the Soroban/SEP-41 convention
- `transfer_from(spender, owner, to, amount)` drawing down the allowance
- Allowance **queries** that report expired entries as `0`
- **Revocation** via `revoke()` or `approve(..., 0, ...)`
- Authorization with `Address::require_auth()` (only the spender signs
  `transfer_from`)
- Edge-case handling: negative amounts, already-expired approvals, insufficient
  allowance, insufficient balance, and checked arithmetic
- Event emission for `approve` and `transfer_from`

## Why an Expiration Ledger?

A plain `(owner, spender) -> amount` mapping never forgets. A forgotten but
still-live approval is a classic way tokens get drained long after the user
intended the permission to lapse. Every allowance here carries an
`expiration_ledger`; once the ledger sequence passes it, the allowance is
treated as `0` even though the stale amount may still sit in storage. Querying
`allowance` therefore always reflects what is actually spendable *now*.

## Public API

| Function | Purpose |
| --- | --- |
| `initialize(admin, initial_supply)` | Seed an opening balance for `admin` |
| `approve(owner, spender, amount, expiration_ledger)` | Grant/replace an allowance (`amount = 0` revokes) |
| `transfer_from(spender, owner, to, amount)` | Spend from the owner via the allowance |
| `revoke(owner, spender)` | Explicitly clear an allowance |
| `allowance(owner, spender)` | Read the spendable allowance (expired → `0`) |
| `allowance_details(owner, spender)` | Read the raw `{ amount, expiration_ledger }` entry |
| `balance(user)` | Read an account balance |
| `admin()` | Read the configured admin address |

## Errors

| Error | When |
| --- | --- |
| `AlreadyInitialized` | `initialize` called twice |
| `NotInitialized` | Any call before `initialize` |
| `InvalidAmount` | Negative approve amount, or non-positive `transfer_from` amount |
| `InvalidExpiration` | Non-zero allowance given an already-expired ledger |
| `InsufficientAllowance` | `transfer_from` exceeds the spendable allowance |
| `InsufficientBalance` | Owner lacks the tokens for the transfer |
| `ArithmeticOverflow` | A balance update would overflow `i128` (defensive) |

## Build

```bash
cargo build -p allowance-pattern
```

## Test

```bash
cargo test -p allowance-pattern
```

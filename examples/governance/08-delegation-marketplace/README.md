# 01 — Delegation Marketplace

A marketplace contract where token holders can list their voting power for rent
and other accounts can pay to use it.

## Concepts

- **`#[contracterror]`** — typed error codes returned via `try_*` client methods
- **`require_auth()`** — every state-mutating call is gated on the caller's signature
- **Persistent storage** — offers and delegations survive across ledger boundaries
- **Event topics** — `(namespace, action, primary, secondary)` layout for off-chain indexing
- **Incentive mechanism** — fee transfer from renter to delegator on rental

## Contract Functions

### Offer management
| Function | Description |
|---|---|
| `list_offer(delegator, voting_power, price_per_unit)` | List voting power for rent |
| `cancel_offer(delegator)` | Remove an open offer |
| `get_offer(delegator)` | Read the current offer (returns `Option`) |

### Renting
| Function | Description |
|---|---|
| `rent_voting_power(renter, delegator, units, duration)` | Rent `units` for `duration` seconds, paying the fee |
| `get_delegation(renter, delegator)` | Read an active delegation (returns `Option`) |

### Expiry
| Function | Description |
|---|---|
| `expire_delegation(renter, delegator)` | Clean up an expired delegation and return units to offer |

### Balance helpers
| Function | Description |
|---|---|
| `fund_account(account, amount)` | Credit tokens (test helper / mint substitute) |
| `get_balance(account)` | Read token balance |

## Error Codes

| Code | Name | Meaning |
|---|---|---|
| 1 | `OfferAlreadyExists` | A delegator can only have one open offer at a time |
| 2 | `OfferNotFound` | No offer exists for the given delegator |
| 3 | `InsufficientVotingPower` | Requested units exceed offer supply |
| 4 | `InsufficientBalance` | Renter cannot cover the rental fee |
| 5 | `DelegationAlreadyExists` | This renter already has an active delegation from this delegator |
| 6 | `DelegationNotFound` | No active delegation for the given pair |
| 7 | `InvalidAmount` | `voting_power` or `price_per_unit` must be > 0 |
| 8 | `DelegationNotExpired` | Cannot expire a delegation that is still active |

## Build

```bash
# From the repository root
cargo build -p delegation-marketplace --target wasm32-unknown-unknown --release
```

## Test

```bash
cargo test -p delegation-marketplace
```

## Use Cases

- **Passive holders** list their governance tokens' voting power and earn fees.
- **Active participants** rent voting power to reach quorum or influence proposals.
- **DAOs** can integrate this marketplace to improve participation rates.

## Prerequisites

- [03-authentication](../../basics/03-authentication/) — `require_auth()` patterns
- [04-events](../../basics/04-events/) — structured event emission
- [Governance README](../README.md) — category overview

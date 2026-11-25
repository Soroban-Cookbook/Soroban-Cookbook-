# Rate Limiting

A reusable fixed-window rate limiter that caps how often and how much each
caller may use an operation. Drop `consume` at the top of any function you want
throttled — withdrawals, mints, bridge transfers, faucet drips.

## What You'll Learn

- Enforcing **time-based** limits with ledger timestamps and window rollover
- Enforcing **amount-based** limits on cumulative value per window
- Keeping **per-user** state so one caller cannot spend another's budget
- Letting an admin grant per-user overrides on top of a contract-wide default
- Failing closed with typed errors instead of panicking

## Overview

```
consume(user, amount)
        │
        ├─ window elapsed? ──► start a fresh window (calls = 0, amount = 0)
        │
        ├─ calls  >= max_calls  ──► Err(CallLimitExceeded)
        ├─ amount +  amount > max_amount ──► Err(AmountLimitExceeded)
        │
        └─ record usage, emit event, Ok(())
```

A window is anchored to the caller's first `consume`, not to fixed wall-clock
boundaries. That keeps a caller from collecting two full budgets by straddling a
boundary — the classic weakness of naive fixed-window counters.

## Key Concepts

### The three caps live in one struct

```rust
pub struct Limit {
    pub window: u64,      // time-based: seconds before usage resets
    pub max_calls: u32,   // call cap per window
    pub max_amount: i128, // cumulative amount cap per window
}
```

### Per-user limits override the default

`limit_of(user)` returns the caller's override if the admin set one, otherwise
the contract-wide default. Usage itself is always keyed by address, so limits
and consumption are both isolated per user.

```rust
client.set_user_limit(&vip, &Limit { window: 60, max_calls: 10, max_amount: 5_000 });
client.clear_user_limit(&vip); // back to the default
```

### Rejections record nothing

A call that trips either cap returns an error before writing usage, so a
rejected attempt does not eat the caller's remaining budget.

## API

| Function | Auth | Purpose |
|----------|------|---------|
| `initialize(admin, default_limit)` | — | Set the admin and contract-wide default |
| `consume(user, amount)` | `user` | Guard call: records usage or errors |
| `limit_of(user)` | — | The limit in force for `user` |
| `usage_of(user)` | — | Usage in the current window |
| `remaining_calls(user)` | — | Calls left this window |
| `remaining_amount(user)` | — | Amount left this window |
| `window_reset_at(user)` | — | When the current window closes |
| `set_default_limit(limit)` | admin | Change the default |
| `set_user_limit(user, limit)` | admin | Grant a per-user override |
| `clear_user_limit(user)` | admin | Drop an override |
| `reset(user)` | admin | Clear a caller's usage immediately |

## Use Cases

- **Withdrawal throttling** — cap value out of a vault per address per day, so a
  stolen key drains a bounded amount rather than the whole balance.
- **Faucet / airdrop** — one claim per address per window on a testnet faucet.
- **Bridge transfer caps** — bound per-user volume crossing a bridge while an
  incident is investigated.
- **Mint throttling** — limit how fast a minter role can issue new supply.
- **Anti-spam on writes** — cap how often an address can open proposals or
  submit oracle updates.

## Error Codes

| Code | Name | Meaning |
|------|------|---------|
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `NotInitialized` | No default limit configured yet |
| 3 | `InvalidLimit` | A limit field was zero or negative |
| 4 | `InvalidAmount` | Consumed amount was zero or negative |
| 5 | `CallLimitExceeded` | Caller is out of calls for this window |
| 6 | `AmountLimitExceeded` | Call would exceed the amount cap |

## Security Notes

- `consume` requires the caller's own auth, so nobody can burn another user's budget.
- Only the admin can change limits or reset usage; both paths are `require_auth`ed.
- The limiter bounds *rate*, not *authorization* — pair it with a role check when
  the underlying operation is privileged.
- Limits are enforced per address. An adversary with many funded addresses is
  limited per address, not overall; combine with allowlists when Sybil resistance
  matters.

## Running Tests

```bash
cargo test -p rate-limiting
```

## Related Examples

- [17-bridge-security](../17-bridge-security/) — Applies a *global*, amount-only
  rate limit inside a bridge contract. This example is the standalone,
  **per-user** counterpart that also caps call frequency.
- [02-timelock](../02-timelock/) — Delaying execution with ledger timestamps
- [01-multi-party-auth](../01-multi-party-auth/) — Multi-party authorization
- [03-oracle-pattern](../03-oracle-pattern/) — Freshness checks on timestamped data

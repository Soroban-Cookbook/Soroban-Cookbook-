# Token Vesting Pattern

> **⚠️ UNAUDITED EXAMPLE — NOT FOR PRODUCTION USE**
>
> The contract code and patterns shown on this page have **not been audited**.
> They are provided solely as a learning resource to illustrate Soroban
> development techniques.  **Do not deploy this contract with real funds
> or in a production environment without a professional security audit.**
>
> **Reentrancy:** Soroban's execution model does not support re-entrant
> cross-contract calls within the same transaction — re-entry is a
> protocol-level impossibility on Soroban.  The guard shown in this
> example is therefore illustrative; you do **not** need a mutex for
> reentrancy protection in Soroban contracts.
>
> **Storage TTL / data-expiry:** Soroban persistent storage entries
> expire after a ledger-defined TTL (default ~30 days on Mainnet).  A
> vesting schedule stored in persistent storage **will be silently
> deleted** if no ledger entry is touched and bumped before expiry.
> Production deployments **must** either (a) call
> `env.storage().persistent().extend_ttl(key, threshold, extend_to)` on
> every `claim` call, or (b) keep an off-chain monitor that bumps entry
> TTLs regularly.  Failing to do so will cause beneficiary schedules to
> vanish, making tokens permanently unclaimable.

---

## Overview

Token vesting is a mechanism where tokens are released to a beneficiary
over a period of time according to a predetermined schedule.  This
pattern demonstrates:

- **Cliff period** — a duration during which no tokens can be claimed.
- **Linear vesting** — continuous proportional release of tokens after
  the cliff.
- **Admin-controlled schedule creation** with beneficiary-controlled
  claiming.

**Related example crate:**
[`examples/tokens/05-vesting`](../../examples/tokens/05-vesting/)

---

## Key Concepts

### Vesting Schedule

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub beneficiary: Address,
    pub total_allocation: i128,
    pub start_time: u64,       // ledger timestamp (seconds since Unix epoch)
    pub cliff_duration: u64,   // seconds before any tokens vest
    pub vesting_duration: u64, // total vesting window in seconds
    pub claimed_amount: i128,  // cumulative amount already claimed
}
```

The schedule is stored in **persistent storage** keyed by the
beneficiary address.

> **Storage TTL reminder:** Persistent entries expire.  Always call
> `extend_ttl` when reading or writing a schedule in production.

### Linear Vesting Formula

```
vested = total_allocation × (current_time − start_time) / vesting_duration
```

- If `current_time < start_time + cliff_duration` → `vested = 0`
- If `current_time >= start_time + vesting_duration` → `vested = total_allocation`

### Implementation

```rust
fn calculate_vested_amount(
    schedule: &VestingSchedule,
    current_time: u64,
) -> Result<i128, VestingError> {
    if current_time < schedule.start_time + schedule.cliff_duration {
        return Ok(0);
    }

    if current_time >= schedule.start_time + schedule.vesting_duration {
        return Ok(schedule.total_allocation);
    }

    let elapsed_time = current_time - schedule.start_time;

    // Checked arithmetic prevents silent integer overflow.
    let vested = schedule
        .total_allocation
        .checked_mul(elapsed_time as i128)
        .ok_or(VestingError::ArithmeticOverflow)?
        .checked_div(schedule.vesting_duration as i128)
        .ok_or(VestingError::ArithmeticOverflow)?;

    Ok(vested)
}
```

> ⚠️ **Unaudited example.** The arithmetic above is illustrative.
> Edge cases such as zero vesting_duration, integer truncation rounding
> down the final payout, or schedules that end before they start are
> handled with basic guards but have not been formally verified.

---

## Contract API

| Function | Auth required | Description |
|---|---|---|
| `initialize(admin, token)` | — | Sets admin and the token address once. |
| `create_schedule(admin, beneficiary, total, start, cliff, duration)` | `admin` | Creates a vesting schedule. Admin-only; reverts if schedule already exists. |
| `claim(beneficiary)` | `beneficiary` | Claims all currently vested but unclaimed tokens. Reverts before cliff. |
| `get_schedule(beneficiary)` | — | Read-only view of the schedule. |
| `get_vested_amount(beneficiary)` | — | Computes currently vested tokens. |

---

## Reentrancy

Soroban's VM **does not permit re-entrant calls** within a single
transaction: once a contract is executing, no other invocation of that
same contract can interleave in the same ledger operation.  A traditional
reentrancy guard (mutex) is therefore not required for this pattern.

The token transfer in `claim` calls the underlying SEP-41 token contract
cross-contract, but Soroban's model ensures this cannot call back into
the vesting contract mid-execution.

---

## Storage TTL / Data-Expiry Limits

| Storage tier | Key | Default TTL | Notes |
|---|---|---|---|
| Instance | `Admin`, `Token` | 30 days (bump on every call) | Must be extended by an admin/keeper transaction. |
| Persistent | `Schedule(beneficiary)` | 30 days from last write | **Must be bumped on every `claim` and periodically off-chain.** |

**Required TTL bump pattern for production:**

```rust
// After writing the updated schedule inside `claim`:
env.storage().persistent().extend_ttl(
    &DataKey::Schedule(beneficiary.clone()),
    MIN_TTL_THRESHOLD,  // bump if below this many ledgers
    TARGET_TTL,         // extend to this many ledgers
);
```

Failure to implement TTL management will cause schedules to silently
expire, making tokens permanently locked in the vesting contract with no
mechanism to recover them.

---

## Security Considerations

- **Admin Authorization**: Only the stored admin can create schedules.
- **Beneficiary Authorization**: Only the beneficiary can call `claim`.
- **Cliff Enforcement**: `claim` reverts with `ClaimBeforeCliff` until
  the cliff period has elapsed.
- **One Schedule Per Beneficiary**: A second `create_schedule` for the
  same address reverts with `ScheduleAlreadyExists`.
- **Checked Arithmetic**: All multiplications and divisions use
  `checked_mul` / `checked_div` to surface overflows as errors rather
  than silent wrapping.
- **No Admin Revocation**: The example does not implement a way for the
  admin to cancel or revoke a schedule.  Add this only if the token
  economics of your project require it, and document the trust model
  clearly for beneficiaries.

> ⚠️ **Reminder:** This example is **unaudited**.  These considerations
> are a starting checklist, not a complete security guarantee.

---

## Running the Example

```bash
# Build
cargo build --target wasm32-unknown-unknown --release -p vesting-contract

# Test
cargo test -p vesting-contract
```

---

## Related Pages

- [`examples/tokens/05-vesting`](../../examples/tokens/05-vesting/) — full example source
- [`docs/patterns/token-wrapper.md`](./token-wrapper.md) — wrapping tokens
- [`docs/security-best-practices.md`](../security-best-practices.md) — general security guide

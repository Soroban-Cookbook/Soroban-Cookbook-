# Basic Oracle Pattern

A single-source oracle that demonstrates how external data can be submitted by an authorized updater, stored with a timestamp, and queried with freshness validation.

## What You'll Learn

- Implementing an authorized data-submission path
- Storing timestamped values on-chain
- Querying data with configurable freshness checks
- Providing both strict (fail-on-stale) and raw getters
- Rotating the authorized updater via an admin role

## Overview

```
Updater submits value + timestamp  →  Readers query latest data
         ↓                                      ↓
   Admin can rotate updater           Freshness check (max age)
```

**Configuration:**
- `max_age`: maximum seconds before data is considered stale (set at initialization)

## Key Concepts

### Submit Data

Only the authorized updater can submit new values. Each submission records the current ledger timestamp alongside the value.

```rust
pub fn submit(env: Env, updater: Address, value: i128) -> Result<(), OracleError> {
    updater.require_auth();
    env.storage().instance().set(&DataKey::Value, &value);
    env.storage().instance().set(&DataKey::Timestamp, &env.ledger().timestamp());
    Ok(())
}
```

### Query with Freshness

Readers can fetch the raw value or use a strict getter that rejects stale data:

```rust
pub fn get_value_strict(env: Env) -> Result<i128, OracleError> {
    let age = now - last_updated;
    if age > max_age { return Err(OracleError::StaleData); }
    // return value
}
```

### Freshness Check

```rust
pub fn is_fresh(env: Env) -> Result<bool, OracleError> {
    let age = env.ledger().timestamp() - last_submission_timestamp;
    Ok(age <= max_age)
}
```

## Trust Model

| Aspect | Detail |
|--------|--------|
| Data source | Single authorized updater — the contract trusts this address |
| Freshness | Configurable `max_age`; stale data is flagged but not auto-removed |
| Correctness | Not verified on-chain; freshness != correctness |
| Extensions | Multi-signer validation, multiple sources, signed payload verification |

**This is a baseline educational example, not a production oracle network.** For production use, consider decentralized aggregation, cryptographic proof of data origin, and economic incentives for honest reporting.

## Error Codes

| Code | Name | Meaning |
|------|------|---------|
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `NotInitialized` | Contract not yet initialized |
| 3 | `NotAuthorized` | Caller is not the authorized updater |
| 4 | `NoData` | No value has been submitted yet |
| 5 | `StaleData` | Data exceeds configured max age |

## Security Notes

- Only the authorized updater can submit data
- Only the admin can rotate the updater
- Freshness checks reduce stale-data risk but do not prove correctness
- No value is returned for uninitialized reads

## Running Tests

```bash
cargo test -p oracle-pattern
```

## Related Examples

- [02-timelock](../02-timelock/) — Time-based execution with ledger timestamps
- [01-multi-party-auth](../01-multi-party-auth/) — Multi-party authorization patterns

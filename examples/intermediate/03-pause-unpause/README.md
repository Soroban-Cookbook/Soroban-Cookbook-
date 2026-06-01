# Pause / Unpause Pattern

An emergency shutdown mechanism where an admin can halt sensitive state-changing operations and later resume them.

## What You'll Learn

- Implementing a contract-level pause toggle
- Guarding mutable functions with a pause check
- Keeping read-only functions available during pause
- Emitting events for operational visibility
- Authorization patterns for admin-only actions

## Overview

```
Admin pauses contract  →  Guarded functions reject calls
         ↓                          ↓
Admin unpauses         →  Normal operation resumes
                                    ↓
                        Read-only functions always work
```

## Key Concepts

### Pause Guard

Sensitive functions call `require_not_paused` before performing mutations:

```rust
fn require_not_paused(env: &Env) -> Result<(), PauseError> {
    let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
    if paused { return Err(PauseError::ContractPaused); }
    Ok(())
}
```

### Guarded vs Unguarded Functions

| Function | Guarded | Available while paused |
|----------|---------|----------------------|
| `increment` | Yes | No |
| `reset` | Yes | No |
| `get_counter` | No | Yes |
| `is_paused` | No | Yes |
| `pause` | Admin-only | N/A |
| `unpause` | Admin-only | N/A |

### Pause/Unpause Lifecycle

```rust
client.increment(); // succeeds
client.pause();     // admin only
client.increment(); // fails with ContractPaused
client.get_counter(); // still works (read-only)
client.unpause();   // admin only
client.increment(); // succeeds again
```

## Error Codes

| Code | Name | Meaning |
|------|------|---------|
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `NotInitialized` | Contract not yet initialized |
| 3 | `NotAuthorized` | Caller is not the admin |
| 4 | `ContractPaused` | Guarded operation rejected — contract is paused |
| 5 | `AlreadyInState` | Contract is already paused/unpaused |

## Operational Guidance

- **When to pause:** incident response, vulnerability disclosure, planned upgrades
- **What to guard:** state-changing operations that move funds or modify critical state
- **What to keep available:** read-only queries, status checks
- **Pausing is a safety lever, not a substitute for secure design**

## Security Notes

- Only the admin can pause/unpause
- Read-only functions remain available during pause for transparency
- Events are emitted on every state transition for auditability
- Double-pause/double-unpause is rejected to prevent confusion

## Running Tests

```bash
cargo test -p pause-unpause
```

## Related Examples

- [multi-sig-patterns](../multi-sig-patterns/) — Multi-party authorization patterns
- [02-timelock](../../advanced/02-timelock/) — Time-delayed execution

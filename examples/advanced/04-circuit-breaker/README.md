# Circuit Breaker Pattern

A compact example of a contract-level circuit breaker that can be paused manually by an admin and also auto-triggered after repeated failures. The pattern is useful for emergency shutdowns, exploit containment, and graceful recovery without redeploying the contract.

## What You'll Learn

- Emergency pause and resume flows
- Auto-triggering a halt after repeated failures
- Recovery windows that reopen the circuit automatically
- Best-practice guard rails for contract state transitions

## Overview

```
Caller invokes action → success resets failure count
                 ↓
          failures accumulate
                 ↓
       threshold reached → pause circuit
                 ↓
      recovery window expires → reopen circuit
```

## Key Concepts

### Manual Pause

Only the configured admin can pause or resume the contract.

### Automatic Triggers

Each failure increments a tracked counter. Once the counter meets the configured threshold, the contract enters a paused state.

### Recovery Mechanism

After a configured recovery window elapses, the contract automatically reopens and clears the failure count.

## Best Practices

- Keep the admin role narrowly scoped and require authenticated admin actions.
- Default the circuit to `Active` so the contract remains usable before configuration completes.
- Use separate pause and recovery controls so operators can recover without redeploying.
- Prefer explicit errors over silent fallback behavior for paused or misconfigured contracts.

## Testing

```bash
cargo test -p circuit-breaker
```

## Related Examples

- [02-timelock](../02-timelock/) — Emergency pause patterns and time-based safety controls
- [03-oracle-pattern](../03-oracle-pattern/) — Admin-managed contract configuration and state checks

# Multicall

Batch several cross-contract calls into a single Soroban transaction, with
either all-or-nothing or per-call failure handling.

## What it shows

- **Dynamic cross-contract calls:** `env.invoke_contract` and
  `env.try_invoke_contract` with a runtime `Address`, `Symbol` and `Vec<Val>`.
- **Atomic vs. best-effort batching:** one failure reverts everything, or each
  call succeeds or fails on its own.
- **Authorization propagation:** the caller signs once, and that signature
  covers every nested call that checks `caller.require_auth()`.
- **Batch validation:** empty batches, oversized batches and self-calls are
  rejected.

## API

| Function | Returns | Failure semantics |
| --- | --- | --- |
| `aggregate(caller, calls)` | `Vec<Val>` | Atomic. Any failing call reverts the whole batch. |
| `aggregate_results(caller, calls, require_success)` | `Vec<(bool, Val)>` | Each call reports `(success, data)`. A failed call's state changes are rolled back and the others are kept. With `require_success = true`, the first failure returns `CallFailed` and reverts the batch. |

A `Call` is `{ contract: Address, function: Symbol, args: Vec<Val> }`. On
failure, `data` is `()` (void).

Both functions emit `("multicall", caller) -> call_count` when a batch
completes.

### Errors (`MulticallError`)

| Code | Variant | Cause |
| --- | --- | --- |
| 1 | `EmptyBatch` | `calls` is empty |
| 2 | `TooManyCalls` | more than `MAX_CALLS` (32) calls |
| 3 | `SelfCall` | a call targets the multicall contract itself |
| 4 | `CallFailed` | a call failed while `require_success` was set |

## Security notes

- `caller.require_auth()` runs on every batch. Targets that authenticate
  `caller` rely on this. Without it, anyone could batch calls on someone
  else's behalf.
- The contract holds no funds or roles, so batching through it gives no
  more authority than calling the targets directly.
- Self-calls are rejected so batches cannot nest or re-enter the aggregator.
- `MAX_CALLS` limits the work one transaction can request.

## Running the tests

The unit tests in `src/test.rs` are wired from `lib.rs` via
`#[cfg(test)] mod test;`:

```bash
cargo test -p multicall
```

They cover ordered execution, atomic rollback, partial-failure rollback,
`require_success`, auth recording and rejection without auth, batch limits,
self-call rejection and the batch event.

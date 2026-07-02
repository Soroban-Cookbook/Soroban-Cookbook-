# token-lock (token lock pattern)

This example implements a **time-based token lock ledger**.

It tracks, per user:
- multiple lock entries (`amount`, `unlock_time`)
- the total currently locked amount

## APIs

- `lock(amount, unlock_time)`
  - Locks `amount` for the caller until `unlock_time` (ledger timestamp).
  - Fails if `amount <= 0` or `unlock_time <= now`.

- `unlock()`
  - Unlocks all lock entries whose `unlock_time <= now` for the caller.
  - Returns the total unlocked amount.

- `locked_balance()` / `locked_balance_of(user)`
  - Query total currently locked for the caller (or for an arbitrary user).

- `lock_schedule()` / `lock_schedule_of(user)`
  - Query all lock entries for the caller (or for an arbitrary user).

## Notes / Tradeoffs

- This contract is a **ledger only**: it does not move SEP-41 tokens.
  Use it as a vesting/staking accounting primitive, or extend it to integrate
  with your token via a wrapper pattern.

- Storage design:
  - lock entries and locked totals are stored in `persistent` storage.
  - `locked_balance` is kept as a cached total for cheap reads.


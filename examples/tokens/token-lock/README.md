# token-lock (token lock pattern)

This example implements a **time-based token lock ledger**.

It tracks, per user:
- multiple lock entries (`amount`, `unlock_time`)
- the total currently locked amount

## APIs

| Function | Auth | Purpose |
|----------|------|---------|
| `lock(user, amount, unlock_time)` | `user` | Locks `amount` for `user` until `unlock_time` (ledger timestamp). Fails if `amount <= 0` or `unlock_time <= now`. |
| `unlock(user)` | `user` | Releases every entry whose `unlock_time <= now`; returns the total released. |
| `locked_balance(user)` | — | Total currently locked for `user`. |
| `unlockable_balance(user)` | — | Portion of the locked balance claimable right now. |
| `lock_schedule(user)` | — | All lock entries for `user`, matured or not. |

Both mutating calls require the lock owner's own authorization, so a third party
cannot lock or release someone else's balance.

## Example

```rust
// At ledger time 1_000: lock 100 until 1_500 and 50 until 1_200.
client.lock(&user, &100, &1_500);
client.lock(&user, &50, &1_200);
assert_eq!(client.locked_balance(&user), 150);

// At 1_250 only the second entry has matured.
assert_eq!(client.unlockable_balance(&user), 50);
assert_eq!(client.unlock(&user), 50);
assert_eq!(client.locked_balance(&user), 100);
```

## Notes / Tradeoffs

- This contract is a **ledger only**: it does not move SEP-41 tokens.
  Use it as a vesting/staking accounting primitive, or extend it to integrate
  with your token via a wrapper pattern (see
  [06-token-wrapper](../06-token-wrapper/)).

- Storage design:
  - lock entries and locked totals are stored in `persistent` storage.
  - `locked_balance` is kept as a cached total for cheap reads.

- `unlock` walks the caller's full entry list. Keep schedules small, or shard
  them, if you expect a user to accumulate hundreds of entries.

## Running Tests

```bash
cargo test -p token-lock
```

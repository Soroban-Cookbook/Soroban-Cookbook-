# Automatic Snapshot Triggers — Governance

Demonstrates automatic snapshot trigger patterns for governance contracts: time-based voting-power snapshots, event-based delegation snapshots, and snapshot pruning to manage storage costs.

## Why snapshot gating matters

A naive governance contract writes a snapshot on every action (vote, propose,
delegate). A hot proposal with 500 votes then leaves 500 persistent entries
behind, and every one of them needs periodic TTL extension. This example keeps
snapshot storage bounded no matter how busy governance gets.

## Features

- **Interval gating (default)**: `record_value` always updates the owner's
  current value, but writes a historical snapshot at most once per
  `snapshot_interval_ledgers` (the `frequency` passed to `initialize`). Gating is
  per owner, so one busy account never blocks another.
- **Epoch-based snapshots**: `create_proposal` opens a new epoch with a single
  write. The first change to an owner's value inside an epoch checkpoints the
  pre-change value; further changes in the same epoch write nothing.
- **Snapshot-free voting**: `vote` reads power as of proposal creation and
  only writes the tally and a double-vote guard.
- **Input validation**: an interval of `0` or above `MAX_INTERVAL_LEDGERS` is
  rejected with `InvalidFrequency`.
- **Pruning**: admin-controlled removal of old interval snapshots.

## Snapshot Trigger Patterns

### Interval-gated
```rust
client.initialize(&admin, &17_280);           // ~24h at 5s ledgers
let wrote = client.record_value(&user, &500); // true only if the interval passed
client.get_current(&user);                    // always the latest value
client.next_snapshot_ledger(&user);           // when the next snapshot is allowed
client.auto_snapshot(&user);                  // snapshot current value if due
```

### Epoch-based (proposal creation)
```rust
client.set_mode(&admin, &SnapshotMode::Epoch);
let pid = client.create_proposal(&proposer);  // epoch boundary, one write
client.record_value(&user, &900);             // checkpoints old value once per epoch
client.vote(&user, &pid, &true);              // power as of proposal creation
```

### Pruning
```rust
client.prune(&admin, &user, &20000);
```

## Choosing a mode

| | Interval | Epoch |
|---|---|---|
| Snapshot writes | <= 1 per owner per interval | <= 1 per owner per proposal in which the value changed |
| Voting power accuracy | Latest snapshot before the proposal ledger (interval granularity) | Exact value at proposal creation |
| Best for | Analytics, periodic history | Proposal voting |

## Build

```bash
cd examples/governance/11-automatic-snapshot-triggers
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/governance/11-automatic-snapshot-triggers
cargo test
```

## Gas Considerations

- `record_value` always writes the single `Current(owner)` entry; the snapshot Vec is only rewritten when a snapshot is due.
- In a test with 500 updates over 500 ledgers and an interval of 10, only 50 snapshots are written.
- In epoch mode, 200 votes on one proposal write zero snapshot entries.
- `prune` rewrites the Vec without the pruned entries — periodic pruning is recommended.
- All admin config (`set_frequency`, `set_enabled`, `set_prune_threshold`) writes to instance storage (low cost).

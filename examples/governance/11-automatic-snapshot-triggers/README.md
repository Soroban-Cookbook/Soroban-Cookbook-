# Automatic Snapshot Triggers — Governance

Demonstrates automatic snapshot trigger patterns for governance contracts: time-based voting-power snapshots, event-based delegation snapshots, and snapshot pruning to manage storage costs.

## Features

- **Time-based snapshots**: Auto-record voting power snapshots at configurable ledger intervals.
- **Event-based snapshots**: Explicit snapshot recording triggered by governance events (proposals, votes).
- **Snapshot pruning**: Admin-controlled removal of old snapshots to bound storage growth.
- **Gas-efficient**: Minimal storage writes via interval gating; pruning prevents unbounded Vec growth.

## Snapshot Trigger Patterns

### Time-Based
```rust
// auto_snapshot checks if `frequency` ledgers have elapsed since the last snapshot.
// If the interval has passed, it records a new snapshot automatically.
client.auto_snapshot(&user);
```

### Event-Based
```rust
// record_value always snapshots — triggered by any significant on-chain event.
client.record_value(&user, &voting_power);
```

### Pruning
```rust
// Remove snapshots older than ledger 20000.
client.prune(&admin, &user, &20000);
```

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

- `record_value` performs one persistent read + write per call.
- `auto_snapshot` gates writes behind a `last + frequency` check, saving gas on skipped intervals.
- `prune` rewrites the Vec without the pruned entries — periodic pruning is recommended.
- All admin config (`set_frequency`, `set_enabled`, `set_prune_threshold`) writes to instance storage (low cost).

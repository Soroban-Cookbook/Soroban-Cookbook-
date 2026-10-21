# Automatic Snapshot Triggers — DeFi

Demonstrates automatic snapshot trigger patterns for DeFi contracts: time-based reserve snapshots, event-based swap/liquidity snapshots, and snapshot pruning to manage storage costs.

## Features

- **Time-based snapshots**: Auto-record reserve or price snapshots at configurable ledger intervals.
- **Event-based snapshots**: Explicit snapshot recording triggered by swaps, liquidity changes, or price updates.
- **Snapshot pruning**: Admin-controlled removal of old snapshots to bound storage growth.
- **Gas-efficient**: Minimal storage writes via interval gating; pruning prevents unbounded Vec growth.

## Snapshot Trigger Patterns

### Time-Based
```rust
// auto_snapshot checks if `frequency` ledgers have elapsed since the last snapshot.
client.auto_snapshot(&pool_id);
```

### Event-Based
```rust
// record_value always snapshots — triggered by any significant on-chain event.
client.record_value(&pool_id, &reserve_amount);
```

### Pruning
```rust
// Remove snapshots for pool reserves older than ledger 20000.
client.prune(&admin, &pool_id, &20000);
```

## Build

```bash
cd examples/defi/14-automatic-snapshot-triggers
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/defi/14-automatic-snapshot-triggers
cargo test
```

## Gas Considerations

- `record_value` performs one persistent read + write per call.
- `auto_snapshot` gates writes behind a `last + frequency` check, saving gas on skipped intervals.
- `prune` rewrites the Vec without the pruned entries — periodic pruning is recommended.
- All admin config writes to instance storage (low cost).

# Automatic Snapshot Triggers — NFTs

Demonstrates automatic snapshot trigger patterns for NFT contracts: time-based ownership snapshots, event-based transfer/listing snapshots, and snapshot pruning to manage storage costs.

## Features

- **Time-based snapshots**: Auto-record ownership snapshots at configurable ledger intervals.
- **Event-based snapshots**: Explicit snapshot recording triggered by transfers, listings, or sales.
- **Snapshot pruning**: Admin-controlled removal of old snapshots to bound storage growth.
- **Gas-efficient**: Minimal storage writes via interval gating; pruning prevents unbounded Vec growth.

## Snapshot Trigger Patterns

### Time-Based
```rust
// auto_snapshot checks if `frequency` ledgers have elapsed since the last snapshot.
client.auto_snapshot(&owner);
```

### Event-Based
```rust
// record_value always snapshots — triggered by any significant on-chain event.
client.record_value(&owner, &token_count);
```

### Pruning
```rust
// Remove ownership snapshots older than ledger 20000.
client.prune(&admin, &owner, &20000);
```

## Build

```bash
cd examples/nfts/05-automatic-snapshot-triggers
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/nfts/05-automatic-snapshot-triggers
cargo test
```

## Gas Considerations

- `record_value` performs one persistent read + write per call.
- `auto_snapshot` gates writes behind a `last + frequency` check, saving gas on skipped intervals.
- `prune` rewrites the Vec without the pruned entries — periodic pruning is recommended.
- All admin config writes to instance storage (low cost).

# 14 Bridge Monitor

> Issue #765 — monitoring and alerts for a cross-chain bridge.

A Soroban contract that logs bridged transfers, monitors on-chain balance
drift, and keeps a resolvable alert queue. It holds no funds and moves no
tokens — it is the on-chain audit + alert ledger of a larger off-chain
monitoring stack.

## What it tracks

- **Event tracking** — `record_transaction` appends a canonical record
  (direction, from/to chains, amount, token, status, ledger timestamp) and
  emits a `TX` Soroban event for off-chain indexers.
- **Balance monitoring** — `snapshot_balance` records the observed token
  balance; when it moves by more than the configured threshold vs. the last
  snapshot, a `balance_drift` alert is raised automatically.
- **Anomaly surfacing** — a `failed` transfer raises a high-severity
  `failed_transfer` alert immediately.
- **Alerting** — `list_alerts` / `resolve_alert` give watchers a queue to
  page and acknowledge.

## API

```text
- initialize(admin, threshold)
- set_token(admin, token)
- set_threshold(admin, threshold)
- record_transaction(indexer, tx_id, direction, from_chain, to_chain, amount, token, status) -> u32
- transactions(start, limit) -> Vec<BridgeTransaction>
- transaction_count() -> u32
- snapshot_balance(indexer, observed)
- last_snapshot() -> Option<i128>
- list_alerts() -> Vec<Alert>
- alert_count() -> u32
- resolve_alert(admin, id)
```

## Off-chain component guide

Bridge monitoring pipelines commonly have five components; this contract
provides the storage + alerting that each of them reads:

| Component | Responsibility | Interaction |
|-----------|---------------|-------------|
| Relayer/indexer service | observes bridge events on the source chain | calls `record_transaction` for every mint/burn/lock/unlock |
| Balance poller | reads the vault/token balance on each interval | calls `snapshot_balance` with the observed value |
| Alert watcher | fans alerts out to Slack/PagerDuty/e-mail | polls `list_alerts`, then `resolve_alert` after acknowledgement |
| Dashboard | charts volume, drift, and pendings | reads `transactions(start, limit)`, `transaction_count`, `last_snapshot` |
| Incident responder | triages `failed_transfer`/`balance_drift` alerts | resolves via admin after the runbook step |

Operational notes:

- Grant the **indexer** the admin role only to the monitor, and never share a
  key that can also move bridge funds.
- Set the drift threshold to `2 × max expected sweep size` to avoid false
  positives from routine vault sweeps.
- Page `transactions` with `start`/`limit` instead of loading the full list;
  for very high throughput, prune old records with an archival job.

## Verification

```bash
cargo test -p bridge-monitor
cargo build --target wasm32-unknown-unknown --release -p bridge-monitor
cargo clippy -p bridge-monitor --all-targets -- -D warnings
```

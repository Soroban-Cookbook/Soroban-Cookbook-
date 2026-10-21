# Contract Migrations

Advanced example of migrating live contract storage across schema versions without downtime.

## What It Demonstrates

- **Prepare → batch → finalize** migration lifecycle
- **Gas-bounded batches** so large user sets can migrate over multiple transactions
- **Dual-read** during migration (v2 preferred, v1 fallback)
- **Version gates** so v2-only entry points refuse unmigrated state
- **Admin-gated WASM upgrade** hook (`upgrade`) to pair with schema migration

## Schema Change

| Version | Storage |
|---------|---------|
| v1 | `LegacyAccount { balance }` under `DataKey::Legacy(Address)` |
| v2 | `AccountV2 { balance, last_active, tier }` under `DataKey::Account(Address)` |

Tier is derived at migration time: `0` (&lt; 1_000), `1` (≥ 1_000), `2` (≥ 10_000).

## Lifecycle

```text
initialize (v1)
    │
    ▼
add_user × N          ← seed legacy accounts
    │
    ▼
prepare_migration(2)
    │
    ▼
migrate_batch(k) ──┐  ← repeat until user list exhausted
    │              │
    └──────────────┘
    │
    ▼
Version = 2, MigrationState = None
    │
    ▼
credit / get_account  ← v2-only APIs now allowed
```

## Run Tests

```bash
cargo test -p contract-migrations
```

## Related Examples

- [`07-upgrade-patterns`](../07-upgrade-patterns/) — direct WASM upgrade + single-key versioned migration
- Intermediate [`storage-migration`](../../intermediate/storage-migration/) — simpler batched rewrite
- [`04-upgradeable-proxy`](../04-upgradeable-proxy/) — swap implementation address instead of WASM

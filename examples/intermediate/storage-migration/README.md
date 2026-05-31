# Storage Migration Pattern

A versioned storage migration example that shows how to move data from a legacy layout to a new schema safely.

## What It Demonstrates

- Explicit `version` tracking in contract storage
- `prepare_migration()` with target version validation
- Chunked migration support through `migrate_batch()`
- Rollback-friendly migration state and cancellation
- Legacy-to-new data transformation with preserved invariants
- Testing guidance for safe, incremental upgrade workflows

## Public API

| Function | Purpose |
| --- | --- |
| `initialize(admin)` | Set the admin and begin at version `1` |
| `add_user(user, balance)` | Store legacy per-user balances |
| `prepare_migration(target_version)` | Stage a migration before execution |
| `migrate_batch(batch_size)` | Transform a subset of legacy entries |
| `cancel_migration()` | Abort a staged migration safely |
| `get_version()` | Read the current storage version |
| `migration_state()` | Inspect the pending migration status |
| `legacy_balance(user)` | Read pre-migration balances |
| `profile(user)` | Read the migrated profile data |

## Build

```bash
cargo build -p storage-migration
```

## Test

```bash
cargo test -p storage-migration
```

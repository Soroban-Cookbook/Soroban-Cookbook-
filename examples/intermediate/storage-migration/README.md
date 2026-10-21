# Storage Migration Pattern

A versioned storage migration example that shows how to move data from a legacy layout to a new schema safely.

## What It Demonstrates

- Explicit `version` tracking in contract storage
- `prepare_migration()` with target version validation
- An explicit `migrate_v1_to_v2()` entry point for a concrete legacy-to-new schema transform
- Chunked migration support through `migrate_batch()` for safe, incremental upgrades
- Rollback-friendly migration state and cancellation via `cancel_migration()`
- Legacy-to-new data transformation with preserved invariants and migration state tracking
- Testing guidance for safe, incremental upgrade workflows

## Migration Safety Notes

1. The contract stores a monotonically increasing storage version and refuses to downgrade or re-run an already-completed migration.
2. `prepare_migration()` stages the target version and records the next batch index before any data is moved.
3. `migrate_v1_to_v2()` transforms legacy user balances into profile records while removing the old storage keys as part of the same migration step.
4. `cancel_migration()` provides a rollback-friendly escape hatch when a migration is staged but not yet finalized.
5. Tests should cover both happy-path transformations and edge cases, including invalid versions and partial migrations.

## Testing Guide

- Use `env.mock_all_auths()` in tests so admin-only migration entry points can be exercised without extra authentication setup.
- Verify the storage version changes only after the migration reaches completion.
- Assert that migrated profiles contain the transformed data and that legacy keys are removed.
- Exercise batch execution with partial progress to confirm the migration state resumes at the correct index.

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

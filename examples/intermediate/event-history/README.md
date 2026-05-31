# Event History Pattern

An on-chain audit-trail pattern that stores event history in contract storage for deterministic queries.

## What It Demonstrates

- Append-only history records with a ring-buffer cap
- Queryable pagination via `get_events(start, limit)`
- Time-based filtering with `query_by_time(earliest, latest, limit)`
- Storage limit management by trimming oldest entries
- Authenticated history writes using actor authority
- `HistoryStats` for operational observability

## Public API

| Function | Purpose |
| --- | --- |
| `initialize(admin, max_entries)` | Configure history capacity |
| `append_event(actor, action, details)` | Record an on-chain audit entry |
| `get_events(start, limit)` | Read a paginated result set |
| `query_by_time(earliest, latest, limit)` | Filter entries by timestamp |
| `history_stats()` | Inspect storage window and capacity |

## Build

```bash
cargo build -p event-history
```

## Test

```bash
cargo test -p event-history
```

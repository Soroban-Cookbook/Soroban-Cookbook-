# Lazy Loading and Caching

Demonstrates how to reduce gas costs by loading only the data subsets needed
for a given invocation, and by maintaining a bounded in-contract cache so
that repeated reads within the same transaction hit instance storage (cheap)
rather than persistent storage (expensive).

## Key Concepts

### Lazy Loading
`get_item(id)` reads from persistent storage **only when the item is first
requested** — not eagerly on every call. Contracts with large datasets pay
for only the entries they actually use.

### Bounded Cache
A cache in **instance storage** holds the most recently accessed items up to
`CACHE_CAPACITY` (default: 10). When the cache is full the oldest entry is
evicted (FIFO) to keep the cache size predictable and instance storage TTL
bounded.

### Cache Invalidation
`set_item` **writes through**: it updates persistent storage and immediately
removes the corresponding cache entry. The next read re-fetches the fresh
value, preventing stale reads.

### Performance Measurement
`get_item_with_stats` returns a `LoadResult { item, cache_hit }` so callers
can observe whether the value came from the cache or persistent storage.

## Storage Layout

| Key | Type | Storage | Purpose |
|-----|------|---------|---------|
| `Item(u32)` | `Item` | Persistent | Canonical item data |
| `ItemCount` | `u32` | Instance | Highest id ever stored |
| `Cache(u32)` | `Item` | Instance | Cached copy of item |
| `CacheKeys` | `Vec<u32>` | Instance | Ordered list of cached ids (for eviction) |

## Contract Functions

| Function | Description |
|---|---|
| `set_item(owner, id, value)` | Store or update an item; invalidates cache |
| `get_item(id)` | Lazy-load an item (cache miss → persistent storage) |
| `get_item_with_stats(id)` | Same as `get_item` but returns `cache_hit` flag |
| `cache_size()` | Number of items currently in the cache |
| `invalidate(id)` | Manually evict an item from the cache |
| `item_count()` | Highest item id ever stored |

## Error Codes

| Code | Name | Meaning |
|---|---|---|
| 1 | `ItemNotFound` | No item exists with the given id |
| 2 | `InvalidInput` | `id` must be > 0 |

## Build

```bash
cargo build --target wasm32v1-none --release -p lazy-loading
```

## Test

```bash
cargo test -p lazy-loading
```

## Why This Matters

In Soroban, persistent storage reads are metered — loading data you don't need
wastes gas. By combining lazy loading with a bounded cache you pay only for
what you use, and repeated reads within a workflow hit the cheaper instance
storage tier.

## Prerequisites

- [02-storage-patterns](../../basics/02-storage-patterns/) — storage tiers and TTL
- [03-authentication](../../basics/03-authentication/) — `require_auth()` patterns
- [Intermediate README](../README.md) — category overview

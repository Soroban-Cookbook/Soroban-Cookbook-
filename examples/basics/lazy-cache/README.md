# Lazy Cache Patterns

This example demonstrates how to lazily load and cache items from a large persistent storage set using Soroban's temporary storage.

## Overview

The contract stores item values in **persistent storage** and exposes a `get_item(id)` function that:
- checks temporary storage first,
- loads the item from persistent storage only when needed,
- caches the retrieved item in temporary storage,
- keeps only a small bounded working set,
- invalidates entries manually or when cache capacity is exceeded.

## What This Example Shows

- **Lazy loading**: large data sets are not loaded until requested.
- **Bounded cache**: temporary cache size is limited to a small working set.
- **Cache invalidation**: explicit invalidation and cache clearing.
- **Cache stats**: hit/miss counters show how often the cache avoids persistent reads.
- **Gas optimization**: repeated reads return from temporary storage instead of repeated persistent storage access.

## Contract Functions

- `set_item(id, value)` — Store an item in persistent storage.
- `get_item(id)` — Load the item lazily and cache it in temporary storage.
- `invalidate_cache(id)` — Remove one cached entry.
- `clear_cache()` — Remove all cached temporary entries.
- `cache_stats()` — Return `(cache_size, hits, misses)`.

## Performance Notes

This pattern is useful when a contract needs only a small subset of a large data set in any given transaction or ledger. By reading from temporary storage after the first access, the contract reduces repeated persistent storage costs and keeps the active working set bounded.

## Build

```bash
cargo build -p lazy_cache
```

## Test

```bash
cargo test -p lazy_cache
```

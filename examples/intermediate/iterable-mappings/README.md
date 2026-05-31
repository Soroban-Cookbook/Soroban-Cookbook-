# Iterable Mapping

This example implements a small on-chain map that stays enumerable by keeping:

- a `Map<Symbol, u32>` for direct key lookups, and
- a separate `Vec<Symbol>` key index for page-based iteration.

The public helpers are:

- `set(key, value)` to insert or update entries
- `get(key)` to fetch a single value
- `keys(page, page_size)` to retrieve one page of keys
- `values(page, page_size)` to retrieve one page of values
- `remove(key)` to delete entries without leaving stale index data

## Why this pattern matters

Native Soroban iteration over a `Map` is limited, so this pattern gives you a predictable way to enumerate state without scanning the entire contract storage layout. The tradeoff is extra storage write cost because every new key must be appended to the side list.

## Gas and storage guidance

1. Use this pattern when you need page-based iteration rather than random access.
2. Keep `page_size` small to control response size and gas usage.
3. Expect higher write costs than a plain `Map` because the side index must be updated.

# Storage Pagination

Cursor-based pagination for large on-chain collections. The contract stores each
item under its own persistent key and returns pages via an opaque binary cursor
so callers can resume without scanning the whole collection.

## Why per-item storage?

Storing every entry in a single `Vec` forces each `list` call to deserialize the
entire collection before slicing a page. Per-item keys (`Item(index)`) plus a
`NextIndex` counter let `list` read only the keys in `[start, start + page_size)`.

## What it demonstrates

- Append-only indexed persistent storage
- `list(page_size, cursor) -> Result<Page, PaginationError>`
- Opaque, deterministic cursor encoding (`SPG1` + big-endian `u32` index)
- Page-size validation (`1..=MAX_PAGE_SIZE`)
- End-of-collection behavior (empty page, `next_cursor = None`)
- Honest consistency guarantees for live appends

## API

| Function | Description |
|---|---|
| `add_item(item)` | Append an item; returns its absolute index |
| `list(page_size, cursor)` | Return a page and optional next cursor |
| `count()` | Number of stored items |
| `get_item(index)` | Direct accessor (panics if out of bounds) |
| `cursor_from_index(index)` | Helper to build a cursor for demos / tests |

`Page` contains:

- `items`: symbols in ascending index order
- `next_cursor`: opaque `Bytes` resume token, or `None` at the end

## Cursor format

Treat cursors as **opaque**. The documented layout is for implementers only:

```text
[0..4]  magic = b"SPG1"
[4..8]  index = u32 big-endian absolute storage index
```

Exactly 8 bytes. Wrong length, wrong magic, or truncated payloads return
`PaginationError::InvalidCursor`. Pagination never panics on bad input.

## Page-size limits

`MAX_PAGE_SIZE` is `50`. Requests with `page_size == 0` or `page_size > 50`
return `PaginationError::InvalidPageSize`.

## End-of-collection behavior

| Situation | Result |
|---|---|
| Empty collection | Empty `items`, `next_cursor = None` |
| Cursor exactly at `count` | Empty `items`, `next_cursor = None` |
| Cursor beyond `count` (well-formed) | Empty `items`, `next_cursor = None` |
| Final partial page | Remaining items, `next_cursor = None` |

## Consistency model

This example is append-only and **does not provide snapshot isolation**.

- A cursor encodes an absolute index, so previously returned positions stay
  addressable after new appends.
- If writers append while a client is paging, later pages may include the new
  items.
- Deletes, inserts in the middle, and reordering are out of scope; those would
  need a different indexing / tombstone design.

## Build & test

```bash
cargo test -p storage-pagination
cargo build --target wasm32v1-none --release -p storage-pagination
```

## Use cases

- Paginating token holders, listings, or audit-style append logs
- Frontend “load more” flows without loading the whole ledger map
- Keeping per-call storage reads bounded under instruction limits

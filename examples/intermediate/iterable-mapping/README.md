# Iterable Mapping

A key-value map that supports iteration over keys and values via an indexed key list, including pagination helpers.

## Overview

Soroban contracts cannot natively enumerate map entries. This pattern maintains a side list of keys to preserve deterministic iteration order. The contract exposes:

- `set(key, value)` – insert or update
- `get(key)` – read value
- `contains(key)` – check existence
- `remove(key)` – delete an entry
- `keys(page, page_size)` – paginate over keys
- `values(page, page_size)` – paginate over values
- `entries(page, page_size)` – paginate over key-value pairs
- `len()` – total number of entries

## Storage Tradeoffs

Storing the side list duplicates the key set, increasing storage costs. This pattern is best for maps with a bounded number of entries (e.g. token holders, approved signers). For frequently mutated maps, consider using a linked list or tree-based index.

## Build & Test

```bash
cargo test
cargo build --target wasm32-unknown-unknown --release
```

## Contract Functions

### `set(key: Symbol, value: u32)`
Appends `key` to the iteration order if it is not already present, then stores the value.

### `get(key: Symbol) -> Option<u32>`
Returns the value associated with `key`, or `None`.

### `contains(key: Symbol) -> bool`
Returns `true` if `key` exists.

### `remove(key: Symbol) -> bool`
Removes the entry and its position in the iteration order. Returns `true` if the key existed.

### `keys(page: u32, page_size: u32) -> Vec<Symbol>`
Returns a page of keys in insertion order. Pages are zero-indexed.

### `values(page: u32, page_size: u32) -> Vec<u32>`
Returns a page of values in the same order as `keys`.

### `entries(page: u32, page_size: u32) -> Vec<(Symbol, u32)>`
Returns a page of key-value pairs.

### `len() -> u32`
Returns the number of entries.

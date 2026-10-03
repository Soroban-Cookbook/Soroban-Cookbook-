# Iterable Mapping Utilities

**An advanced companion to the canonical iterable map example** — the same enumerable key-value pattern extended with collection utilities for **filtering**, **mapping**, and **reducing**.

> **Not the learning entry point.** Start with [`iterable-mapping`](../iterable-mapping/), the canonical iterable map example used by the Soroban Cookbook. Come back here once you understand the side-list key index and page-bounded iteration.

This example demonstrates how to maintain an enumerable key-value collection on Soroban and manipulate it safely using collection utilities for **filtering**, **mapping**, and **reducing**.

The contract maintains:
- a `Map<Symbol, u32>` for $O(1)$ key lookups, and
- a separate `Vec<Symbol>` index for ordered, page-bounded key iteration.

## How It Differs from the Canonical Example

| | [`iterable-mapping`](../iterable-mapping/) (canonical) | This example |
|---|---|---|
| Page indexing | Zero-indexed (`page * page_size`) | One-indexed (`(page - 1) * page_size`) |
| Page-size cap | `MAX_PAGE_SIZE = 100` | None |
| Storage layout | One instance-storage entry per key, plus a `Vec<Symbol>` index | A single `Map<Symbol, u32>`, plus a `Vec<Symbol>` index |
| Extra reads | `contains(key)`, `entries(page, page_size)` | `is_empty()` |
| `remove(key)` | Returns `bool` (whether the key existed) | Returns `()` |
| Collection utilities | — | `filter_by_*`, `map_values_*`, `reduce_sum*`, and the generic helpers below |

`set`, `get`, `len`, `keys(page, page_size)`, and `values(page, page_size)` share the same names and semantics in both. Page indices are **not** interchangeable between the two.

## Key Features & Helpers

### Core Storage Operations
- `set(key, value)` — Inserts or updates a key-value entry and maintains the side index.
- `get(key)` — Fetches the value for a single key.
- `remove(key)` — Deletes an entry and removes its key from the side index.
- `keys(page, page_size)` — Retrieves a bounded page of keys.
- `values(page, page_size)` — Retrieves values corresponding to a bounded page of keys.

### 1. Filtering (`filter_by`)
- `filter_by_min_value(min_value)` — Evaluates all entries and returns a new `Map<Symbol, u32>` containing only entries where `value >= min_value`.
- `filter_by_page(min_value, page, page_size)` — Filters entries across a single page of keys, bounding gas consumption to `page_size`.
- `filter_by_predicate(&entries, &keys, predicate)` — Generic functional Rust utility to filter entries using a predicate closure (`Fn(&Symbol, u32) -> bool`).

### 2. Mapping (`map_values`)
- `map_values_scale(factor)` — Transforms each value in the map by multiplying it by `factor` using saturating arithmetic to prevent overflow.
- `map_values_scale_page(factor, page, page_size)` — Scales values across a specific page of keys.
- `transform_values(&entries, &keys, transform)` — Generic functional Rust utility to transform values using a mapping closure (`Fn(&Symbol, u32) -> u32`).

### 3. Reducing (`reduce_sum`)
- `reduce_sum()` — Iterates over all keys and calculates the sum of all values as a `u64` to prevent overflow.
- `reduce_sum_page(page, page_size)` — Calculates the sum of values within a specific page.
- `reduce_values(&entries, &keys, init, f)` — Generic functional Rust utility to accumulate map entries into a deterministic result (`Fn(A, &Symbol, u32) -> A`).

---

## Safe Iteration Patterns in Soroban

1. **Avoid In-Place Mutation During Iteration**: Modifying a collection while iterating over it can lead to state inconsistencies or invalid iterator states. Instead, filtering and mapping create a new `Map` or output structure.
2. **Deterministic Traversal**: Iterating over the explicit key index `Vec<Symbol>` guarantees consistent, deterministic traversal order across execution frames.
3. **Overflow Protection**: Arithmetic reductions (like `reduce_sum`) should use larger numeric types (e.g., accumulating `u32` values into `u64`) and saturating/checked arithmetic to prevent panics or overflow vulnerabilities.

---

## Gas & Performance Guidance

- **Linear Complexity**: Filtering, mapping, and reducing require visiting each element in the collection ($O(N)$ complexity relative to the number of processed keys).
- **Page Bounding**: Always prefer page-bounded operations (`page`, `page_size`) for contracts with non-trivial state sizes. Unbounded iterations risk reaching the transaction CPU instruction limit or memory limits.
- **Storage vs. Read Cost**: Reading keys from storage or generating new in-memory `Map` instances consumes host memory and CPU instructions. Page sizes of 10–50 items are recommended for predictable gas costs.

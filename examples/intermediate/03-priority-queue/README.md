# Priority Queue

This intermediate example shows a heap-backed priority queue implementation in Soroban.

## Features

- `push(item, priority)` — insert an item with a given priority
- `peek_max()` — inspect the highest-priority item without removing it
- `pop_max()` — remove and return the highest-priority item
- `len()` / `is_empty()` — query queue size
- `all()` — return all entries
- `initialize(admin)` — set the admin who controls advanced operations
- `bulk_push(items, priorities)` — atomically insert multiple items, heapifying once for efficiency
- `remove(item)` — remove a specific item from the queue regardless of its position
- `merge(other_queue)` — merge all entries from another priority queue into this one

## Advanced Operations

The `bulk_push`, `remove`, and `merge` operations require admin authorization,
demonstrating access control patterns for managing complex queue workflows.

- `bulk_push` is more efficient than repeated `push` calls because it heapifies
  the entire structure once after all insertions (O(n) vs O(n log n)).
- `remove` locates an item by value and removes it while maintaining heap integrity.
- `merge` reads entries from another queue contract and combines them via heapify.
- `push(item, priority)` insertion
- `peek_max()` to inspect the highest-priority item
- `pop_max()` to remove and return the highest-priority item
- heap integrity via binary heap operations stored on-chain

## API

| Function | Purpose |
|----------|---------|
| `push(item, priority)` | Insert `item` and sift it up to its place |
| `peek_max()` | Highest-priority item, or `None` when empty |
| `pop_max()` | Remove and return the highest-priority item; panics when empty |
| `len()` / `is_empty()` | Queue size |
| `all()` | The raw heap array, useful for inspecting heap order |

## How It Works

The queue is a max-heap stored as a single `Vec<HeapEntry>` in persistent
storage. `push` appends and sifts up; `pop_max` swaps the last element into the
root and sifts down. Both are O(log n) in the number of entries, and the whole
vector is read and written per call — so this suits queues of tens to low
hundreds of entries, not unbounded ones.

The heap invariant is that every parent's priority is greater than or equal to
its children's, which makes index `0` the maximum at all times.

## Use Cases

- Ordered withdrawal or payout queues
- Priority-based task scheduling inside a contract
- Auction or bid ranking where the top entry is read repeatedly

## Running Tests

```bash
cargo test -p priority-queue
```

Tests cover push/peek/pop ordering, length tracking, the empty-queue panic, and
heap integrity across interleaved pushes and pops.

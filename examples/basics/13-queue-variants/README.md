# Queue Variants

This example provides two queue implementations for Soroban smart contracts:

- **Bounded Queue** — enforces a maximum size and supports two drop policies: `DropOldest` (discard the oldest element when full) and `DropNewest` (reject new pushes when full).
- **Circular Buffer** — fixed-size ring buffer that overwrites the oldest element automatically when full.

Both implementations expose simple APIs: `initialize`, `push`, `pop`, `len`, and `capacity`.

## When to use each

- Bounded Queue (DropNewest): use when you must never overwrite existing queued items; new pushes should fail when the queue is full.
- Bounded Queue (DropOldest): use when newer items are more valuable than older ones and you prefer to evict the oldest to make space.
- Circular Buffer: use when a fixed-size sliding window is acceptable and overwriting oldest entries is desirable — it has predictable storage footprint.

## Gas & Storage Tradeoffs

High-level comparison:

- Storage layout:
  - Both implementations store each element as a separate persistent storage entry keyed by an index. They also store a small metadata struct (capacity/head/tail/len).
  - The bounded queue (DropOldest) removes the oldest element's storage entry when evicting; the circular buffer overwrites entries in-place.

- Typical operation costs:
  - `push`:
    - Bounded Queue (DropNewest): checks length and writes one storage entry when successful; when full it panics.
    - Bounded Queue (DropOldest): when full it performs one `remove` (oldest) and one `set` (new tail) — two storage ops.
    - Circular Buffer: when full it performs one `remove` (oldest) and one `set` (new tail) but uses modulo arithmetic to reuse slots.
  - `pop`:
    - Both implementations perform one `get` and one `remove` per popped element.

- Storage growth and predictability:
  - Circular Buffer: storage footprint is bounded to `capacity + meta` entries regardless of churn — predictable and usually preferable if you want tight storage budgets.
  - Bounded Queue (DropNewest): storage grows until capacity and stays there; no extra removes are done on push when full (pushes fail instead), so storage is also bounded but application must handle push failures.
  - Bounded Queue (DropOldest): storage also remains bounded, but eviction causes extra remove operations which may affect gas patterns.

- Gas considerations:
  - Removing a storage key can be cheaper or yield refunds depending on the host economics; however, removes are still computations and consume gas. Overwriting an existing key (circular buffer) avoids repeated key allocation and may be slightly cheaper in practice.
  - If your workload frequently saturates the queue and pushes many new items, the circular buffer avoids the repeated allocation/removal churn and typically uses less gas over time.
  - If you rarely reach capacity, then simpler designs (bounded queue with DropNewest) are cheapest because pushes only do a single `set`.

## Example usage

Look at `src/test.rs` for comprehensive tests demonstrating expected behaviors (push/pop ordering, capacity enforcement, and overwrite/eviction semantics).

## Running tests

Run the example's tests with:

```bash
cargo test -p queue-variants
```

## Recommendation

For predictable on-chain costs and strict storage caps, prefer the circular buffer. For semantics that must fail when full, choose `DropNewest`. Use `DropOldest` when newer data should replace older data but you still want explicit eviction semantics.

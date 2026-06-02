# Compressed Storage

This example shows how to compress byte payloads before storing them in Soroban
persistent storage. It also documents the trade-offs between raw and
compressed storage in terms of saved bytes and effective gas cost.

## What This Example Shows

- How to compress raw bytes using a simple run-length encoding (RLE) helper
- How to store both raw and compressed payloads for direct comparison
- Why compression helps for repeated or highly-structured data
- Why compression may not help for already-random or small payloads

## Storage Pattern

This contract uses **persistent storage** to keep both the raw and compressed
versions of a payload. The compressed payload contains a header with the
original uncompressed length so the contract can decompress it safely.

## Key Functions

- `store_raw(key, data)` – stores raw bytes directly
- `store_compressed(key, data)` – compresses bytes before storing them
- `get_raw(key)` – returns the original stored bytes
- `get_decompressed(key)` – returns the decompressed compressed payload
- `compare_stored_sizes(key)` – returns `(raw_len, compressed_len)`

## Trade-offs and Guidance

### When compression is beneficial

- Data contains many repeated bytes, repeated fields, or predictable patterns.
- The payload is large enough that the compression header and algorithm cost is
  a small fraction of the total.
- Saving storage bytes is more important than the extra compute cost of
  compressing / decompressing on-chain.

### When compression is not beneficial

- Data is short, already random, or has low repetition.
- The compressed payload is the same size or larger than the raw payload.
- You want to avoid additional gas for on-chain compression logic.

### Practical tip

Always compare the stored data length before choosing to compress. In Soroban,
storage rent and gas scale with entry size, so compression only makes sense when
it reduces the on-ledger byte footprint.

## Build

```bash
cd examples/basics/13-compressed-storage
cargo build -p compressed-storage
```

## Test

```bash
cd examples/basics/13-compressed-storage
cargo test -p compressed-storage
```

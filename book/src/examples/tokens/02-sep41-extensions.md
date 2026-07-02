# 02 · SEP-41 Extensions

**Source:** [`examples/tokens/02-sep41-extensions/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/02-sep41-extensions)

Optional extensions on top of the SEP-41 standard: `permit` (EIP-2612 equivalent for gasless approvals), `batch_transfer`, and `batch_approve` for multi-recipient operations in a single transaction.

## Extensions

| Extension | Description |
|-----------|-------------|
| `permit` | Approve by signature — no prior approval transaction needed |
| `batch_transfer` | Transfer to multiple recipients in one call |
| `batch_approve` | Approve multiple spenders in one call |

## Quick Code

```rust
// Batch transfer to 3 recipients
client.batch_transfer(
    &sender,
    &vec![alice, bob, charlie],
    &vec![100_i128, 200_i128, 300_i128],
);
```

## Run the Example

```bash
cd examples/tokens/02-sep41-extensions
cargo test
```

## Next: [03 · Optimized Operations](./03-optimized-operations.md)

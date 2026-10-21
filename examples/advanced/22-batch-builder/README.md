# Batch Builder

Utility contract for composing batch balance operations with a staged builder workflow: accumulate operations, validate them, estimate gas, then execute.

## What It Demonstrates

- **Builder pattern** — `begin_batch`, `add_transfer`, `add_mint`, and `add_burn` stage operations before execution
- **Validation** — rejects empty batches, invalid amounts, insufficient balances, and duplicate operations
- **Gas estimation** — returns a deterministic heuristic cost based on operation count and type
- **Execution guard** — only validated batches can be executed

## Workflow

```rust
let batch_id = client.begin_batch();
client.add_transfer(&batch_id, &alice, &bob, &50);
client.add_mint(&batch_id, &bob, &10);

let gas = client.validate_batch(&batch_id);
let executed = client.execute_batch(&batch_id);
```

## Gas Estimation Model

| Component | Units |
|-----------|------:|
| Base overhead | 12,000 |
| Transfer | 6,000 |
| Mint | 5,000 |
| Burn | 5,000 |

Estimates are deterministic heuristics for planning and client-side UX — not exact host metering.

## Error Codes

| Code | Name | Meaning |
|------|------|---------|
| 1 | `BatchNotFound` | Unknown batch id |
| 2 | `EmptyBatch` | No operations queued |
| 3 | `BatchTooLarge` | Exceeds `MAX_BATCH_SIZE` (32) |
| 4 | `InvalidAmount` | Amount must be positive |
| 5 | `NotValidated` | Execute called before validation |
| 6 | `AlreadyValidated` | Batch is immutable after validation |
| 7 | `InsufficientBalance` | Transfer/burn exceeds balance |
| 8 | `DuplicateOperation` | Identical operation appears twice |

## Running Tests

```bash
cargo test -p batch-builder
cargo clippy --all-targets -p batch-builder -- -D warnings
cargo build --target wasm32-unknown-unknown --release -p batch-builder
```

## Related Examples

- [`08-batch-operations`](../08-batch-operations/) — atomic and partial batch execution
- [`06-gas-optimization`](../06-gas-optimization/) — broader gas optimization patterns

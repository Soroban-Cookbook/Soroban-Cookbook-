# Cross-Contract Optimization

This example demonstrates how to optimize multiple cross-contract updates by batching arguments in a single invocation.

- `TargetContract` stores values keyed by `Symbol` and supports single or batched updates.
- `UnoptimizedCaller` performs repeated cross-contract calls in a loop.
- `OptimizedCaller` packs update arguments into a `Vec<PackedUpdate>` and sends them in one call.

## Run tests

```bash
cd examples/advanced/03-cross-contract-optimization
cargo test
```

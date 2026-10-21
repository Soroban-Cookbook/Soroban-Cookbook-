# Storage Optimization Patterns

Demonstrates storage optimization techniques for Soroban smart contracts.

## Features

- **Packed Storage**: Group related fields into a single struct to reduce storage overhead
- **Lazy Loading**: Load data only when needed rather than at contract start
- **Batch Operations**: Process multiple storage operations in a single call
- **Config Caching**: Store configuration as a single compact instance entry

## Techniques

### Packed Storage
Instead of storing `balance`, `nonce`, `flags`, and `delegate` as separate keys, they are stored as a single `PackedUserData` struct. This reduces the number of storage entries and associated costs.

### Lazy Loading
User data is loaded from storage only when a user interacts with the contract. No upfront loading of all users.

### Batch Operations
`batch_get_balances` and `batch_deposit` process multiple users in a single call, reducing cross-contract overhead.

## Testing

```bash
cargo test -p storage-optimization
```

Build as WASM:
```bash
cargo build --target wasm32-unknown-unknown --release -p storage-optimization
```

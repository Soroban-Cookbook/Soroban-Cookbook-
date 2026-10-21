# Version Registry

A contract version registry with history tracking and rollback support.

## Features

- **Version Registration**: Register new contract versions with metadata
- **History Tracking**: Per-contract version history for auditing
- **Rollback Support**: Roll back to previous versions
- **Admin Controls**: Only authorized admin can register/rollback versions

## Usage

### Initialize
```rust
client.initialize(&admin_address);
```

### Register a Version
```rust
let entry = client.register(&contract_address, &hash, &metadata);
```

### Query Versions
```rust
// Get all versions
let all = client.get_all_versions();

// Get latest version
let latest = client.get_latest_version();

// Get version by number
let v1 = client.get_version_by_number(&1);

// Get contract-specific history
let history = client.get_contract_history(&contract_address);

// Get current version number
let current = client.get_current_version_number();
```

### Rollback
```rust
let removed = client.rollback();
```

## Testing

```bash
cargo test -p version-registry
```

Build as WASM:
```bash
cargo build --target wasm32-unknown-unknown --release -p version-registry
```

# Intermediate Examples

Advanced patterns, real-world use cases, and more complex contracts.

## 🍍 Examples

### Multi-Sig Patterns [./multi-sig-patterns/](../examples/intermediate/multi-sig-patterns/)

Threshold signatures & multi-party auth. N-of-M signers, sequential approvals, and single-transaction multi-auth.

## Key Concepts:
- `#contracterror` for auth failures
- Proposal-based threshold execution
- Atomic multi-signer authorization
- Configurable thresholds

## Quick Code:
```rust
// Collect approvals in a proposal
client.approve(&proposal_id, &signer).unwrap();

// Or require multiple signers in one call
for signer in signers.iter() {
    signer.require_auth();
}
```

## Checklist: [CHECKLIST.md](../examples/intermediate/multi-sig-patterns/CHECKLIST.md)

### Ajo Factory [./ajo-factory/](../examples/intermediate/ajo-factory/)

Contract deployment from within a contract. Spawn isolated instances from Wasm hash.

## Key Concepts:
- `env.deployer()`
- Wasm Hash storage
- Salted address derivation
- Initialization guard

## Quick Code:
```rust
let address = env.deployer()
    .with_current_contract(salt)
    .deploy(wasm_hash);
AjoClient::new(&env, &address).initialize(...);
```

### Pause / Unpause [./03-pause-unpause/](../examples/intermediate/03-pause-unpause/)

Emergency shutdown mechanism. Admin-controlled pause toggle that halts sensitive operations while keeping read-only functions available.

## Key Concepts:
- `#contracterror` for pause-state errors
- Internal `require_not_paused` guard
- Guarded vs unguarded functions
- Event emission on state transitions

## Quick Code:
```rust
// Guard sensitive operations
fn require_not_paused(env: &Env) -> Result<(), PauseError> {
    let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
    if paused { return Err(PauseError::ContractPaused); }
    Ok()
}
```
### Storage Migration [./storage-migration/](../examples/intermediate/storage-migration/)

Versioned storage upgrades. Stage migrations, transform legacy storage, and execute in safe batches with explicit version checks and rollback-friendly state.

## Key Concepts:
- Explicit version tracking with guarded upgrade paths
- Prepared migration state for staged, batched execution
- Concrete legacy-to-new data transforms such as `migrate_v1_to_v2()`
- Safe rollback/cancellation and migration-safety tests

### Event History [./event-history/](../examples/intermediate/event-history/)

On-chain audit history. Record event entries persistently and query them with cursor-based pagination and time filters.

## Key Concepts:
- Append-only audit entries
- Cursor-based pagination with stable next cursors
- Time-based filtering
- Storage cap trimming

### Iterable Mapping [./iterable-mapping/](../examples/intermediate/iterable-mapping/)

Enumerating key-value state. A map that stores keys in a side list to support paginated iteration.

## Key Concepts:
- Side-list key index
- Deterministic insertion order
- Pagination helpers
- Storage cost tradeoffs

## Quick Code:
```rust
// Insert an entry
client.set(&key, &value);
// Paginate over keys
let page = client.keys(&0, &10);
```

## Prerequisites

- [Basics](.e/basics.md)

## ❰ Run
```bash
cd examples/intermediate/multi-sig-patterns
cargo test
```

## Next: [Advanced](.e/advanced.md)

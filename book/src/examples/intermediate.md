# Intermediate Examples

Real-world patterns: tokens, access control, data structures.

## 📋 Examples

### Multi-Sig Patterns [./multi-sig-patterns/](../examples/intermediate/multi-sig-patterns/)
**Threshold signatures & multi-party auth.** N-of-M signers, sequential approvals, and single-transaction multi-auth.

**Key Concepts:**
- `#[contracterror]` for auth failures
- Proposal-based threshold execution
- Atomic multi-signer authorization
- Configurable thresholds

**Quick Code:**
```rust
// Collect approvals in a proposal
client.approve(&proposal_id, &signer).unwrap();

// Or require multiple signers in one call
for signer in signers.iter() {
    signer.require_auth();
}
```

**Checklist:** [CHECKLIST.md](../examples/intermediate/multi-sig-patterns/CHECKLIST.md)

### Ajo Factory [./ajo-factory/](../examples/intermediate/ajo-factory/)
**Contract deployment from within a contract.** Spawn isolated instances from Wasm hash.

**Key Concepts:**
- `env.deployer()`
- Wasm Hash storage
- Salted address derivation
- Initialization guard

**Quick Code:**
```rust
let address = env.deployer()
    .with_current_contract(salt)
    .deploy(wasm_hash);
AjoClient::new(&env, &address).initialize(...);
```

### Pause / Unpause [./03-pause-unpause/](../examples/intermediate/03-pause-unpause/)
**Emergency shutdown mechanism.** Admin-controlled pause toggle that halts sensitive operations while keeping read-only functions available.

**Key Concepts:**
- `#[contracterror]` for pause-state errors
- Internal `require_not_paused` guard
- Guarded vs unguarded functions
- Event emission on state transitions

**Quick Code:**
```rust
// Guard sensitive operations
fn require_not_paused(env: &Env) -> Result<(), PauseError> {
    let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
    if paused { return Err(PauseError::ContractPaused); }
    Ok(())
}
```
### Storage Migration [./storage-migration/](../examples/intermediate/storage-migration/)
**Versioned storage upgrades.** Stage migrations, transform legacy storage, and execute in safe batches.

**Key Concepts:**
- Explicit version tracking
- Prepared migration state
- Chunked batch execution
- Safe rollback/cancellation

### Event History [./event-history/](../examples/intermediate/event-history/)
**On-chain audit history.** Record event entries persistently and query them with cursor-based pagination and time filters.

**Key Concepts:**
- Append-only audit entries
- Cursor-based pagination with stable next cursors
- Time-based filtering
- Storage cap trimming

---

## Prerequisites
- [Basics](../basics.md)

## 🚀 Run
```bash
cd examples/intermediate/multi-sig-patterns
cargo test
```

## Next: [Advanced](../advanced.md)

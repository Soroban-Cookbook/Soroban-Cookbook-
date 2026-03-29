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

---

## Prerequisites
- [Basics](../basics.md)

## 🚀 Run
```bash
cd examples/intermediate/multi-sig-patterns
cargo test
```

## Next: [Advanced](../advanced.md)

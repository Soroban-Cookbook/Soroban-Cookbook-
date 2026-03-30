# Intermediate Examples

Real-world patterns: tokens, access control, data structures.

## 📋 Examples

### Multi-Sig Patterns [./multi-sig-patterns/](../examples/intermediate/multi-sig-patterns/)
**Threshold signatures & multi-party auth.** N-of-N, M-of-N approvals.

**Key Concepts:**
- Signature tracking in storage
- Threshold execution
- Proposal systems

**Quick Code:**
```rust
for signer in signers.iter() {
    signer.require_auth();
}
```

**Checklist:** [CHECKLIST.md](../examples/intermediate/multi-sig-patterns/CHECKLIST.md)

---

**[More coming...]** Token ops, factories, proxies.

## Prerequisites
- [Basics](../basics.md)

## 🚀 Run
```bash
cd examples/intermediate/multi-sig-patterns
cargo test
```

## Next: [Advanced](../advanced.md)

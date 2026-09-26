# Advanced Examples

Complex protocols & optimizations for production systems.

## Upgradeability Sequence

Follow these examples in order, from a single implementation pointer to fleet
management and lower-level WASM and storage upgrade techniques:

1. [Upgradeable Proxy](../examples/advanced/04-upgradeable-proxy/) — one proxy with proxy-owned state; no beacon or upgrade-governance workflow.
2. [Proxy Admin Controls](../examples/advanced/03-proxy-admin/) — timelock, cancellation, and pause controls; no call forwarding or beacon.
3. [Beacon Proxy](../examples/advanced/02-beacon-proxy/) — multiple proxies can share one implementation through a beacon; no fleet factory.
4. [Beacon Proxy Factory](../examples/advanced/03-beacon-proxy-factory/) — deploy and track a fleet sharing one beacon; no independent named-beacon registry.
5. [Beacon Management](../examples/advanced/06-beacon-management/) — version and roll back multiple named beacons; no proxy deployment or call forwarding.
6. [Upgrade Patterns](../examples/advanced/07-upgrade-patterns/) — direct WASM upgrades, schema migration, and initialization guards; not a proxy or beacon system.

## 📋 Examples

### [01-multi-party-auth](../examples/advanced/01-multi-party-auth/)
**Advanced multi-party authorization** beyond simple multisig.

**Key Concepts:**
- Dynamic signer lists
- Weighted voting
- Time-bound approvals

---

### [02-timelock](../examples/advanced/02-timelock/)
**Delayed execution** for governance & security.

---

### [03-state-channel-disputes](../examples/advanced/03-state-channel-disputes/)
**State channel dispute resolution** with challenge submission, response mechanics, timeout handling, and fraud proofs.

**Key Concepts:**
- Challenge window & dispute deadlines
- Sequence-based state updates
- Fraud proof slashing mechanisms

---

### [03-permit-pattern](../examples/advanced/03-permit-pattern/)
**Permit-based approvals** with signature-backed authorization and deadline enforcement.

**Key Concepts:**
- Off-chain authorization envelopes
- Permit-based allowance setup
- Deadline validation and expiry handling

**Key Concepts:**
- Ledger-timestamp gates
- Queue-based execution
- Emergency overrides

**Quick Code:**
```rust
if env.ledger().timestamp() < unlock_time {
    return Err(Error::TimeLocked);
}
```

---

### [03-oracle-pattern](../examples/advanced/03-oracle-pattern/)
**Single-source oracle** with authorized submission and freshness validation.

**Key Concepts:**
- Authorized data updater
- Ledger-timestamp freshness checks
- Strict (fail-on-stale) vs raw getters
- Updater rotation by admin

**Quick Code:**
```rust
// Submit data (authorized updater only)
client.submit(&updater, &42_i128);
// Query with freshness guard
let value = client.get_value_strict(); // errors if stale
```

---

### [05-diamond-security](../examples/advanced/05-diamond-security/)
**Secure Multi-Facet Proxy (Diamond)** with access controls, upgrade safety, and isolated namespaced storage.

**Key Concepts:**
- Access control per facet (restricting direct execution to proxy)
- Upgrade checks & interface supports verification
- Namespaced key isolation to prevent shared storage collisions

### [11-version-registry](../examples/advanced/11-version-registry/)
**Contract version tracking** with history and rollback support.

**Key Concepts:**
- Version registration with metadata
- Per-contract version history
- Admin-controlled rollback

---

### [08-batch-operations](../examples/advanced/08-batch-operations/)
**Batch operations** with atomic and partial execution.

**Key Concepts:**
- Batch call interface
- Atomic execution with rollback
- Partial execution mode

---

### [09-storage-optimization](../examples/advanced/09-storage-optimization/)
**Storage optimization** patterns for efficient contracts.

**Key Concepts:**
- Packed storage (grouping fields)
- Lazy loading patterns
- Batch operations

**[More coming...]** Factories, bonding curves, merkle proofs.

## ⚠️ Warning
Advanced patterns increase complexity - audit thoroughly!

## Prerequisites
- [Basics](../basics.md), [Intermediate](../intermediate.md)

## Next: [DeFi](../defi.md)

# Contract Upgrade Patterns

A collection of idiomatic upgrade patterns for Soroban smart contracts. Each pattern is independent and addresses a specific aspect of safe upgradeability — they can be combined as needed for production contracts.

## Scope In the Upgradeability Sequence

This is step 6 of 6, following [Beacon Management](../06-beacon-management/).

- **In scope:** direct WASM replacement, versioned storage migration, and
    guarded post-upgrade initialization.
- **Out of scope:** proxy routing, shared-beacon deployment, and a complete
    governance system. For the sequence's starting point, see
    [Upgradeable Proxy](../04-upgradeable-proxy/); for timelocked admin controls,
    see [Proxy Admin Controls](../03-proxy-admin/).

## What It Demonstrates

- **Direct WASM upgrade** — minimal admin-gated `update_current_contract_wasm` call
- **Versioned storage + migration** — safe schema evolution across WASM upgrades without data corruption
- **Safe initialization guards** — preventing double-init and managing post-upgrade setup hooks

## Patterns Overview

### Pattern 1 — Direct WASM Upgrade

`src/direct_upgrade.rs`

The simplest possible upgrade: a single admin address can swap the contract's WASM binary at any time. The storage is untouched; all keys, values, and types carry over exactly as-is.

**Use when:**
- Prototyping or iterating quickly
- You have a trusted single admin or simple access-control setup
- The upgrade is non-contentious

**Don't use alone for:**
- Production contracts where stakeholders need a review window (→ add a timelock; see `03-proxy-admin`)
- Contracts where a single compromised key could push a malicious upgrade (→ add multi-sig; see `01-multi-party-auth`)

**Key entry points:**
- `initialize(admin)` — one-time first-deploy setup
- `upgrade(new_wasm_hash)` — replace the WASM binary (admin-only)

**Storage keys:**
- `Admin` — the authorized address

---

### Pattern 2 — Versioned Storage & Migration

`src/versioned_upgrade.rs`

Demonstrates how to safely change the on-chain storage *schema* when upgrading from v1 to v2. The pattern uses a `StorageVersion` sentinel key to track which schema is currently on-chain, and a `migrate()` function to transform old data shapes into new ones.

**Use when:**
- Your v2 code expects different types or new fields compared to v1
- You want to upgrade a live contract without losing existing data
- Multiple versions may be deployed over time (v1 → v2 → v3…)

**Simulated schema change:**
- v1: `Counter { val: i64 }`
- v2: `CounterV2 { val: i64, last_updated: u64 }` — adds a timestamp field

**Key entry points:**
- `initialize(admin)` — sets storage to v1 schema
- `upgrade(new_wasm_hash)` — swaps the WASM
- `migrate()` — reads v1 data, transforms it to v2, bumps `StorageVersion` (idempotent)
- `increment(amount)` — v2-only business logic; panics if storage not migrated

**Storage versioning rules:**
1. Never rename or remove a `DataKey` variant between versions — the encoded bytes are baked into on-chain storage.
2. Never change the type under an existing key without a migration step.
3. Adding a new key is always safe (use `unwrap_or` defensively).
4. Keep a linear migration chain (v1→v2, v2→v3) so missed upgrades catch up in one call.

**Storage keys:**
- `Admin`
- `StorageVersion` — tracks the schema version on-chain
- `Counter` — the data key whose *value type* changes between v1 and v2

---

### Pattern 3 — Safe Initialization Guards

`src/init_guard.rs`

Covers two complementary guards to prevent incorrect re-initialization:

#### Guard A — Double-init prevention

The `initialize` function writes an `Initialized` flag to instance storage on first call and returns `AlreadyInitialized` on subsequent calls. This prevents an attacker (or a confused operator) from overwriting the admin address after deployment.

#### Guard B — Post-upgrade initialization hook

After a WASM upgrade, the contract may need to seed *new* state that the v1 code never created (e.g. a feature flag, a new config key). This is different from a storage *migration* (which transforms existing values).

The `post_upgrade_init(expected_version)` function:
- Checks that the stored `SetupVersion` is exactly `expected_version - 1` (ordered guard)
- Runs the new setup logic
- Bumps `SetupVersion` to `expected_version`
- Is idempotent: calling it a second time returns `AlreadyRan`

**Use when:**
- You have a one-time setup phase (admin, config, seed data)
- You need to seed new state after an upgrade without touching existing keys
- You want explicit version-scoped setup steps that can be retried safely

**Key entry points:**
- `initialize(admin)` — first-deploy setup; errors on repeat
- `upgrade(new_wasm_hash)` — WASM swap
- `post_upgrade_init(expected_version)` — version-scoped post-upgrade setup
- `is_initialized()` — query: was `initialize` called?
- `setup_version()` — query: which post-upgrade init version has run?

**Storage keys:**
- `Initialized` — boolean flag (presence = initialized)
- `Admin`
- `SetupVersion` — tracks which `post_upgrade_init` version was run
- `FeatureFlagV2` — example new-in-v2 state

---

## Best Practices

### Access Control on Upgrades

All three patterns gate `upgrade()` behind admin authorization. For production:
- Use a **multi-sig** admin (see `01-multi-party-auth`) so no single key can unilaterally upgrade.
- Add a **timelock** (see `03-proxy-admin`) so stakeholders can review the new WASM hash before it becomes live.

### Storage Key Stability

When planning for future upgrades:
- Define `DataKey` as an enum (not raw `Symbol` constants) so the compiler tracks all keys.
- Document the type stored under each key in a comment next to the variant.
- Treat the encoded `DataKey` bytes as **immutable ABI** once deployed.

### Testing Upgrades

`update_current_contract_wasm` produces a host error in unit tests because the test environment has no real WASM registry. All guard logic (auth, version checks, init flags) runs *before* that host call and is therefore fully testable. The test pattern:

```rust
let result = client.try_upgrade(&new_hash);
match result {
    Ok(_) => {} // real WASM swap would succeed
    Err(Ok(e)) => {
        // Must not be our guard errors — proves guards passed
        assert_ne!(e, UpgradeError::Unauthorized);
    }
    Err(Err(_)) => {} // host-level deployer stub error — expected in tests
}
```

For full upgrade flows (deploy v1 → call functions → upgrade to v2 → verify v2 behavior), use integration tests against a live testnet or a full Soroban node with a WASM registry.

### Versioning Conventions

- Bump `CURRENT_VERSION` in `versioned_upgrade.rs` and `UPGRADE_INIT_VERSION` in `init_guard.rs` in lockstep if they coexist in the same contract.
- Document each version's changes in a `CHANGELOG.md` or inline comments so future maintainers understand the migration history.

---

## Common Pitfalls

### Forgetting to call `migrate()` after an upgrade

If you deploy v2 WASM but forget to run `migrate()`, v2 entry points that expect the new schema will panic when they try to read old-shaped data. The fix: call `migrate()` once, then retry the v2 function.

### Storage key collisions after adding new keys

If v1 uses `DataKey::Counter` and v2 adds `DataKey::CounterV2`, both are safe. But if v2 *renames* `Counter` to `CounterV2` (changing the enum variant itself), the encoded key bytes differ and v2 code will not find the old data. Solution: keep the variant name unchanged; transform the *value type* via migration.

### Running `post_upgrade_init` out of sequence

If you skip a version (e.g. jump from setup v1 to v3 without running v2's init), the `OutOfSequence` error fires. Solution: run each version's `post_upgrade_init` in order, or consolidate skipped versions into a single migration step.

### Re-deploying the contract without versioning

If you re-deploy a contract from scratch (new address) rather than upgrading in-place, the storage starts empty and `migrate()` is unnecessary. But if you later *do* upgrade that new instance, the version tracking must be consistent. Recommended: always set `StorageVersion` during `initialize` so future upgrades have a known starting point.

---

## Run Tests

```bash
cargo test -p upgrade-patterns
```

Tests cover:
- ✅ Direct upgrade auth rejection by non-admin
- ✅ Double-init prevention
- ✅ Versioned migration transforms v1 → v2 correctly
- ✅ Double-migration is idempotent (returns `AlreadyMigrated`)
- ✅ v2 entry points refuse to run before migration
- ✅ `post_upgrade_init` is idempotent and version-ordered
- ✅ `post_upgrade_init` out-of-sequence rejection

## Build for WASM

```bash
cargo build --target wasm32-unknown-unknown --release -p upgrade-patterns
```

The resulting WASM is under `target/wasm32-unknown-unknown/release/upgrade_patterns.wasm`.

---

## Related Examples

- [03-proxy-admin](../03-proxy-admin/) — Adds a timelock + proposal workflow on top of the direct upgrade call
- [04-upgradeable-proxy](../04-upgradeable-proxy/) — Delegation proxy pattern (swapping implementation *address* rather than WASM binary)
- [01-multi-party-auth](../01-multi-party-auth/) — Multi-sig and threshold patterns for admin access control
- [02-timelock](../02-timelock/) — Core timelock pattern used by `03-proxy-admin`

---

## Security Checklist

- [ ] Admin key is a multi-sig or DAO address in production, not a single EOA
- [ ] Upgrade authority has a timelock (or justify why instant upgrades are acceptable for your use case)
- [ ] New WASM hash is verified off-chain (code review, audit, reproducible build) before calling `upgrade()`
- [ ] `migrate()` is called exactly once per WASM upgrade, immediately after the upgrade
- [ ] All v2 entry points check `StorageVersion` before reading storage (or panic clearly if migration is incomplete)
- [ ] `post_upgrade_init` is called in version order without skipping steps
- [ ] Storage key enum (`DataKey`) has inline comments documenting the type stored under each variant
- [ ] Test coverage includes: auth guards, version checks, migration correctness, idempotency

---

**Implementation Notes:**

- All patterns use `instance` storage for admin / version / init-flag keys so they persist for the lifetime of the contract (TTL management not needed).
- Event emission uses structured tuples `(NS, ACTION, ...)` for off-chain indexing.
- The `#[contracterror]` enum approach (vs. panics) gives callers `Result<T, Error>` for better composability and clearer error surfaces.

For questions or contributions, see [CONTRIBUTING.md](../../../CONTRIBUTING.md).

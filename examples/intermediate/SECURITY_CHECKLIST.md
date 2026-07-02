# Intermediate Patterns — Security Checklist

A pre-release safety gate for contracts in `examples/intermediate/*`.

Use this checklist before submitting a PR that touches intermediate patterns such as:
- authorization/RBAC-style gating
- pause/unpause switches
- storage migration
- iterable mappings / collections
- event emission integrity
- multisig / timelock governance flows

---

## 1) Repo-wide gates (run for the whole workspace)

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo build --workspace`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo audit --deny warnings --deny unsound` (if used in CI)
- [ ] No orphaned test files (every `test.rs` is wired via `mod test;` in `lib.rs` or an inline `#[cfg(test)]`)

---

## 2) Contract-level review items (mark per PR)

### A. Authorization / auth model
- [ ] Every state-mutating entrypoint enforces the correct authorization:
  - [ ] Uses the **stored** privileged address(es) or role membership from storage.
  - [ ] Does **not** call `require_auth()` on caller-supplied addresses unless it also validates identity against stored config.
- [ ] If using RBAC-like patterns, role assignment/updating is protected (only admin can mutate roles).
- [ ] If using pause/unpause, pause does **not** block admin’s ability to unpause.
- [ ] If using multisig/proposal workflows:
  - [ ] replay protection exists (proposal cannot execute twice)
  - [ ] duplicate approvals are rejected or idempotent
  - [ ] execution requires the threshold to be met at execution time

### B. Time / timelock / delayed actions
- [ ] Uses `env.ledger().timestamp()` (or consistent time source) for unlock/execution checks.
- [ ] Validates that “unlock_time must be in the future” (or equivalent precondition) when scheduling.
- [ ] Cancellation/rollback paths exist for queued actions when appropriate.

### C. Storage and migration safety (critical)
- [ ] Versioned storage layout changes are explicitly tracked (e.g., `Version`, `MigrationState`).
- [ ] Migration functions enforce:
  - [ ] admin auth
  - [ ] correct preconditions (only prepared when staging is done)
  - [ ] correct ordering (no skipping invariants)
- [ ] Migration is chunked or bounded to avoid out-of-gas risks.
- [ ] Migration uses deterministic iteration order (or explicitly documents nondeterminism).
- [ ] On completion, migration state is finalized and staged data is removed/archived correctly.
- [ ] `cancel_migration()`/abort paths restore a safe state (no half-migrated invariants).

### D. Storage tiers and TTL assumptions
- [ ] Financial/state-critical data uses `persistent` storage (not instance/temporary) unless the expiry behavior is intentionally acceptable.
- [ ] Cache fields have a clear rebuild/refresh strategy if TTL expires.
- [ ] Any `extend_ttl()` usage is documented and bounded.

### E. Arithmetic and invariants
- [ ] Uses checked arithmetic (`checked_add`, `checked_sub`, etc.) for all external-input-derived values.
- [ ] Enforces non-negative/positive constraints where applicable (balances, amounts, fees).
- [ ] Maintains accounting invariants atomically within a single call when updating multiple related values.

### F. Event integrity & observability
- [ ] Every externally observable state transition emits an event.
- [ ] Event topic schema is consistent (namespace + action + key identifiers).
- [ ] Events include enough fields for indexers to reconstruct history (avoid putting large payloads in topics).
- [ ] Event emission matches the actual state transitions (no “event before write” bugs where the write can fail).

### G. Error handling and safety
- [ ] No `unwrap()`/`expect()`/panic in contract code on attacker-influenced input.
  - [ ] If panics remain, justify why they are unreachable and/or convert to typed errors.
- [ ] Custom error types are used for common failure modes (auth, bounds, invalid state).
- [ ] Inputs are validated (range checks, length checks for vectors, bounds for indexes).

---

## 3) Testing requirements (what to run / what to cover)

- [ ] Unit tests cover:
  - [ ] happy paths for each entrypoint
  - [ ] auth failures for privileged operations
  - [ ] state-machine transitions (init → active → paused → etc.)
  - [ ] invalid arguments and boundary conditions
- [ ] Migration tests cover:
  - [ ] prepared → batch migration → finalize
  - [ ] cancel before finalize leaves contract in safe state
  - [ ] repeated calls cannot corrupt state
- [ ] Event tests (when applicable):
  - [ ] event count and topics match expectations
  - [ ] payload values correspond to state

---

## 4) PR release gate (final)

Before merging:
- [ ] Update the example README API list if entrypoints changed.
- [ ] Confirm tests are wired and run.
- [ ] Confirm storage layout invariants and migration conditions are correct.
- [ ] Confirm event schema changes are documented.

---

## Example scope mapping (quick reference)

- Authorization flows: RBAC/multisig/timelock examples
- Storage migration: `storage-migration/*`
- Iterable collections: `iterable-mappings/*`, `storage-pagination/*` (if applicable)
- Event history: `event-history/*`

---

*This document is intended as an actionable checklist for contributors and reviewers—not as an audit report.*


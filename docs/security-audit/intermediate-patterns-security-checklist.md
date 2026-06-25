# Security Checklist — Intermediate Patterns (Actionable)

This checklist provides a repeatable pre-release safety gate for contracts in
`examples/intermediate/*`.

It focuses on the most failure-prone areas for intermediate patterns:
- authorization correctness (`require_auth` placement and identity checks)
- storage migration safety (versioning, chunking, cancellation)
- event integrity and auditability

---

## A. Authentication & authorization

- [ ] Every state-mutating entrypoint has the correct auth gate.
- [ ] Authorization is checked against **stored** roles/config, not caller-supplied parameters.
- [ ] Admin-only actions verify admin identity by reading from instance storage.
- [ ] Pause/unpause implementations:
  - [ ] pause guards block only non-admin operations
  - [ ] unpause remains callable by admin

---

## B. Storage migration safety

- [ ] Version tracking exists and is enforced (`Version`, `MigrationState`, etc.).
- [ ] Migration functions validate preconditions:
  - [ ] prepared state required before migration batches
  - [ ] invalid transitions rejected (no skipping)
- [ ] Migration is bounded:
  - [ ] batch size validated
  - [ ] iteration is chunked
- [ ] Cancellation/rollback:
  - [ ] leaves contract in a safe state
  - [ ] does not strand invariants across partial migration
- [ ] Data transformation preserves invariants:
  - [ ] each legacy entry is migrated exactly once (or idempotently)
  - [ ] legacy data is removed/archived as designed

---

## C. Event integrity

- [ ] Every meaningful state transition emits an event.
- [ ] Event topics follow a consistent schema:
  - namespace at topic[0]
  - action at topic[1]
  - key identifiers at topic[2+] where helpful
- [ ] Event payloads match state changes.
- [ ] No events are emitted for operations that can fail after emission without reverting.

---

## D. Testing & quality gates

- [ ] Unit tests for auth failures, boundary conditions, and state transitions.
- [ ] Migration tests for prepare → batch → finalize and cancel behavior.
- [ ] Event-related tests validate emitted values.
- [ ] `cargo fmt`, `cargo clippy -D warnings`, and `cargo test` are clean.

---

## Suggested reviewer workflow

1. Confirm auth model and intended threat surface.
2. Confirm migration state machine and invariants.
3. Confirm event schema and observability.
4. Confirm bounded execution and storage tier choices.
5. Confirm tests cover failures and state transitions.


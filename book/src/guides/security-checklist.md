# Security Checklist for Intermediate Patterns

A repeatable pre-release safety gate for intermediate Soroban smart contracts.
Use this checklist during code review, testing, and audit preparation for
contracts built on the patterns covered in the intermediate examples.

## Authorization & Access Control

### Code Review
- [ ] Every state-mutating entry point has an explicit `require_auth()` or
      role-based guard before any storage write.
- [ ] Admin-only functions (pause, unpause, migration staging) verify the
      caller against a stored admin or role, not a hard-coded address.
- [ ] Multi-sig proposals enforce threshold bounds at initialization: `0 <
      threshold ≤ signer_count`.
- [ ] No double-approval path exists — proposals reject repeat signatures from
      the same signer.
- [ ] Once executed or cancelled, a proposal cannot be re-executed or
      re-cancelled.
- [ ] Initialization is one-shot: repeated calls to `initialize` abort cleanly.
- [ ] Admin functions reject double-state transitions (double pause / double
      unpause).

### Testing
- [ ] Auth bypass tests cover unauthorized callers for every privileged action.
- [ ] Multi-sig tests exercise below-threshold, at-threshold, and above-threshold
      approval counts.
- [ ] Role hierarchy tests verify that lower roles cannot perform higher-privilege
      actions.
- [ ] Pause/unpause tests confirm read-only functions remain accessible while
      sensitive operations are blocked.
- [ ] Tests assert that re-initialization is rejected.

## Storage Migration Safety

### Code Review
- [ ] Storage version is tracked explicitly and incremented only after a
      successful migration.
- [ ] `prepare_migration(target_version)` validates that `target_version >
      current_version` before staging.
- [ ] Legacy-to-new data transformation preserves all invariants (balances,
      ownership, references).
- [ ] Chunked batch execution is bounded: batch size and gas limits are
      configurable and documented.
- [ ] A cancellation path exists and is callable by an authorized role before or
      during migration.
- [ ] Migration state is inspectable via a read-only query during staging and
      execution.
- [ ] Pre-migration balances/data are queryable for post-migration reconciliation.

### Testing
- [ ] Tests cover happy-path migration across all legacy versions.
- [ ] Tests verify cancellation before execution and mid-batch abortion.
- [ ] Tests assert that invalid target versions (≤ current) are rejected.
- [ ] Batch overflow tests confirm chunking behaves correctly when data exceeds
      one batch.
- [ ] Post-migration state is compared against pre-migration snapshots.

## Event Integrity & Auditability

### Code Review
- [ ] All state transitions emit events with stable, documented topics.
- [ ] Events include actor identity (`Address`) where relevant, not just
      opaque IDs.
- [ ] Event history is appended only by authorized callers or the contract
      itself (no external write path).
- [ ] Cursor-based pagination encodes absolute storage indices; cursors are
      invalidated predictably when older entries are trimmed.
- [ ] Storage trimming / ring-buffer caps are configured and documented; growth
      is bounded.
- [ ] Event topics are not used to leak sensitive data (no secrets in
      `publish` arguments).

### Testing
- [ ] Tests assert that unauthorized callers cannot append events.
- [ ] Tests verify cursor invalidation after trimming and recovery via
      `history_stats()`.
- [ ] Tests confirm that the maximum number of retained events never exceeds the
      configured cap.
- [ ] Integration tests inspect emitted events to validate topic schemas and
      ordering.

## Arithmetic & Input Safety

### Code Review
- [ ] All external-input arithmetic uses checked operations (`checked_add`,
      `checked_sub`, etc.); no unchecked `+`, `-`, `*`, `/` on user data.
- [ ] Zero and negative inputs are rejected with descriptive error codes before
      storage reads.
- [ ] Subtraction is guarded by explicit balance sufficiency checks.
- [ ] No `unwrap()`, `expect()`, or `panic!` on attacker-influenced input in
      non-test code, or each is documented as unreachable.

### Testing
- [ ] Tests cover overflow, underflow, zero-input, and negative-input paths.
- [ ] Tests verify that insufficient-balance operations return clean errors, not
      panics.

## General Contract Hardening

### Code Review
- [ ] Error types are custom `#[contracterror]` enums with meaningful variants;
      no generic panics for expected failures.
- [ ] Contract storage uses the appropriate tier:
      `instance` for config, `persistent` for canonical state, `temporary` for
      short-lived locks.
- [ ] No `unsafe` blocks exist in contract (non-test) code.
- [ ] Dependencies are pinned; `Cargo.lock` is present and `cargo audit`
      passes in CI.
- [ ] Public API in documentation matches actual contract entry points.

### Testing
- [ ] `cargo build --workspace` is clean (no errors, no warnings).
- [ ] `cargo test --workspace` compiles and passes.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Test coverage for intermediate contracts exceeds 80% and exercises the
      threat-model categories listed in `docs/security-audit/audit-scope.md`.

## Pre-Release Gate

Mark the example **ready for review** only when every item above is checked.

- [ ] Build, test, lint, and clippy gates pass.
- [ ] Every `require_auth` usage is justified; no state-mutating entry point is
      unguarded.
- [ ] Storage migration examples have dry-run documentation and a rollback
      procedure.
- [ ] Event-emitting examples have a runbook for off-chain indexing and
      cursor-recovery.
- [ ] Known issues from `docs/security-audit/known-issues-log.md` are assessed
      and accepted or resolved.
- [ ] README public API table reflects actual entry points.

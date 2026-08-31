# Docs Audit Tests

Regression tests for the security-audit-prep documentation in
[`docs/security-audit/`](../../docs/security-audit/). These are plain-Rust,
filesystem-based checks — no `soroban-sdk` dependency — so they build and run
independently of the contract examples and of `soroban-sdk`'s `testutils`
feature (see KI-1 in
[`known-issues-log.md`](../../docs/security-audit/known-issues-log.md)).

## What is checked

`tests/audit_scope_consistency.rs` compares the actual
`examples/intermediate/` and `examples/tokens/` directory trees against the
audit-prep documents:

- Every example directory (one containing a `Cargo.toml`) under
  `examples/intermediate/` and `examples/tokens/` is referenced in
  [`audit-scope.md`](../../docs/security-audit/audit-scope.md)'s in-scope
  tables.
- Every such example is also referenced in
  [`audit-prep-checklist.md`](../../docs/security-audit/audit-prep-checklist.md)'s
  per-example readiness tables.
- Any in-scope example with no `README.md` is recorded in
  [`known-issues-log.md`](../../docs/security-audit/known-issues-log.md) (as a
  known documentation gap), so a missing README can't silently go untracked.

These tests exist because the scope/checklist previously drifted: several
examples were added to `examples/intermediate/` and the entire
`examples/tokens/` category was added without the audit-prep documents being
updated to match. The tests fail the same way the drift would have been
caught earlier — by comparing docs to the filesystem instead of trusting them.

## Running

```bash
cargo test -p docs-audit-tests
```

## When this should fail

Add a new example under `examples/intermediate/` or `examples/tokens/`
without updating `docs/security-audit/audit-scope.md` and
`audit-prep-checklist.md`, and the corresponding test fails with the list of
undocumented example directories. Remove an example's `README.md` without
adding it to `known-issues-log.md`, and the README test fails the same way.

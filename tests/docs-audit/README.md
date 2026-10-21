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

## Documentation testing (mdBook)

The filesystem checks above cover the audit-prep documents. The mdBook in
[`book/`](../../book/) is covered separately by `mdbook test`, which compiles
and runs every Rust code block in `book/src/` against the real crates. This
catches examples that no longer compile or whose output has drifted from what
the prose claims.

### Running locally

```bash
cd book
mdbook test
```

`mdbook test` builds each Rust snippet as a doctest, so it needs the same
toolchain and dependencies as the examples themselves. To check that the book
still builds (config, links, SUMMARY) without running snippets:

```bash
cd book
mdbook build
```

Both commands must succeed before opening a PR that touches `book/src/`.

### CI integration and failure reporting

CI runs `mdbook test` (and `mdbook build`) for the book as part of the docs
job. A failing snippet fails the job and the log names the file and line of
the offending code block, e.g.:

```
---- book/src/.../example.md - example (line 42) ----
error[E0425]: cannot find value `foo` in this scope
```

Fix the snippet (or the source it mirrors) and re-run `mdbook test` locally
before pushing. Do not silence failures by removing the code block — either
correct it or replace it with a non-Rust fence if it is intentionally
illustrative.

### Maintaining examples

- **Keep snippets in sync with source.** When an example under `examples/`
  changes, update the matching snippet in `book/src/` in the same PR. The
  audit tests and `mdbook test` are complementary: the former checks that
  documents reference examples, the latter checks that the code in them still
  works.
- **Use mdBook annotations deliberately.** `# ` lines are hidden from the
  rendered page but still compiled; use them for `use` statements, `fn main`
  wrappers, and setup so the visible snippet stays focused. `##` renders a
  literal `#`. `ignore` and `no_run` skip execution — reserve them for
  snippets that genuinely cannot run in the test harness, and prefer `no_run`
  over `ignore` so the code is at least compiled.
- **Prefer `edition2018`/`edition2021` fences** when a snippet depends on
  edition-specific behavior, and keep the fence language tag accurate so
  `mdbook test` picks the block up.
- **Add a page for new examples.** New example categories should get a page
  under `book/src/` and an entry in `book/src/SUMMARY.md`; otherwise the
  snippet is never tested.
=======

## When this should fail

Add a new example under `examples/intermediate/` or `examples/tokens/`
without updating `docs/security-audit/audit-scope.md` and
`audit-prep-checklist.md`, and the corresponding test fails with the list of
undocumented example directories. Remove an example's `README.md` without
adding it to `known-issues-log.md`, and the README test fails the same way.

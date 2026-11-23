# Code Review Checklist

Use this checklist when reviewing a pull request that touches smart-contract example code, documentation, or tooling in the Soroban Cookbook.

Work through each section and mark items **Pass**, **Fail**, or **N/A** in your review comment. A PR should have zero **Fail** marks before it merges.

---

## 1. Correctness

- [ ] The change does what the PR description says it does
- [ ] Edge cases are handled (empty collections, zero amounts, maximum values)
- [ ] Arithmetic uses checked or saturating operations where overflow is possible
- [ ] Contract storage keys are unique and do not collide with other keys in the same contract
- [ ] Return values and error variants are semantically correct
- [ ] Any state mutations are rolled back on error (no partial writes)

---

## 2. Security

- [ ] Authorization checks (`env.require_auth()` / `env.require_auth_for_args()`) are present on every privileged entry point
- [ ] Caller-supplied addresses are validated before being stored or used as authorization subjects
- [ ] Re-entrancy is not possible (no cross-contract call before storage writes are finalized, or a guard is in place)
- [ ] Flash-loan patterns use the callback-and-check pattern with a balance invariant assertion
- [ ] Admin / owner roles are transferable or revocable; no address is permanently privileged by a hardcoded constant
- [ ] Integer arithmetic: division by zero is impossible; no silent truncation converts `u128` → `u64` without a checked cast
- [ ] No `unwrap()` or `expect()` in contract code paths (use `Result` / `Option` with explicit error variants)
- [ ] Secrets or private keys are not committed to the repository
- [ ] Dependency additions are from known, audited crates; no suspicious names or unusual version pins

*See also: [DeFi Security Checklist](../book/src/docs/defi-security-checklist.md), [Governance Security Checklist](../book/src/docs/governance-security-checklist.md)*

---

## 3. Performance

- [ ] Hot paths avoid unnecessary storage reads (cache values in local variables within a single invocation)
- [ ] `Map` and `Vec` sizes are bounded; unbounded growth is documented and justified
- [ ] Cross-contract calls are minimized; no call is made inside a loop unless the loop is bounded by a small constant
- [ ] TTL extension (`bump_instance`, `bump_persistent`) is called only where needed and with documented parameters
- [ ] Gas benchmarks exist for computationally intensive operations; new code does not regress existing benchmarks by more than 10%

*See also: [Performance Benchmarks](../book/src/docs/benchmarks.md)*

---

## 4. Testing

- [ ] New example or changed logic has unit tests in `src/test.rs`
- [ ] Tests cover the happy path, at least one error path, and relevant boundary conditions
- [ ] Cross-contract interactions have a test in `tests/integration/`
- [ ] Authorization-sensitive code has a test in `tests/security/` that verifies unauthorized callers are rejected
- [ ] `cargo test --workspace` passes locally (or CI is green)
- [ ] Test coverage for the changed crate is ≥ 90% (run `cargo tarpaulin --package <crate>`)

---

## 5. Code Style and Quality

- [ ] `cargo fmt --all -- --check` passes with no diffs
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes with no warnings
- [ ] All contracts use `#![no_std]`
- [ ] Custom error types use `#[contracterror]`
- [ ] Public functions have doc comments (`///`) explaining parameters, return values, and panics
- [ ] Variable and function names follow Rust naming conventions and the project style guide
- [ ] No dead code (`#[allow(dead_code)]`) is introduced without a comment explaining why

---

## 6. Documentation

- [ ] New pages are added to `book/src/SUMMARY.md`
- [ ] New examples have a `README.md` that explains the pattern, when to use it, and how to run it
- [ ] Code comments explain *why*, not just *what*
- [ ] External links are valid (no 404s)
- [ ] If an ADR-worthy decision was made, an ADR is opened or referenced
- [ ] `cd book && mdbook build` succeeds without config errors

---

## 7. Breaking Changes

- [ ] Breaking changes are documented in the PR description under a **Breaking Changes** heading
- [ ] Any public API change in a shared crate (`shared/`) is flagged; consumers across the workspace are updated
- [ ] CHANGELOG or release notes are updated if the project maintains one

---

## Reviewer Sign-off

After completing the checklist, leave a summary comment on the PR:

```
**Review checklist:** all items reviewed.

Approval condition: [list any remaining items needed before merge, or write "ready to merge"]
```

---

*Template version: 1.0 (2026-10-19). For updates, open a PR modifying this file and `.github/pull_request_template.md`.*

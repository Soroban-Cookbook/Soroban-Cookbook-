# Known Issues Log

A baseline of issues known *before* the external audit, so the engagement does
not spend effort re-reporting them. Each entry is reproducible at preparation
time. Entries are **build / tooling / process** facts unless explicitly labeled
otherwise; this log intentionally does **not** assert security
vulnerabilities — that is the audit's job.

Status values: `Open` · `Mitigated` · `Accepted` · `Closed`.

---

## KI-1 — Test & lint builds fail: `soroban-sdk` testutils cannot link `serde_json`/`rand`

- **Type:** Build blocker (repo-wide)
- **Status:** Open
- **Affects:** Entire workspace — any crate built with the `testutils` feature,
  i.e. all example test suites and `cargo clippy --all-targets`.

**Description.** Building any package's tests pulls `soroban-sdk` with the
`testutils` feature, whose `testutils.rs` references `serde_json` and `rand`.
Under the current resolution these crates are not linked, so the SDK itself
fails to compile with `E0433: failed to resolve: use of unresolved module or
unlinked crate` for both `serde_json` and `rand` (see the error excerpt below).

**Evidence.**

```text
$ cargo test -p mint-burn-token        # an already-merged example
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `serde_json`
   --> soroban-sdk-26.0.x/src/testutils.rs
error: could not compile `soroban-sdk` (lib) due to 176 previous errors
```

The same failure occurs for every example, confirming it is pre-existing and
not specific to any single contract.

**Impact.** `cargo test --workspace` and `cargo clippy --all-targets` cannot
complete, so test results cannot be reproduced for the audit. Plain library
builds (`cargo build -p <example>`, no testutils) are unaffected.

**Suggested remediation (for maintainers).** Pin a `soroban-sdk` patch whose
`testutils` feature links its dependencies correctly, and/or add the missing
dev-dependencies, then commit a `Cargo.lock` (see KI-4). Left for the
maintainers per the contribution policy.

---

## KI-2 — Orphaned test files not wired with `mod test;`

- **Type:** Test-coverage gap
- **Status:** Open
- **Affects:** `examples/tokens/01-sep41-token`,
  `examples/intermediate/02-role-based-access-control`,
  `examples/intermediate/03-priority-queue`

**Description.** Each listed example contains a `src/test.rs`, but its `lib.rs`
declares neither `mod test;` nor an inline `#[cfg(test)]` module. The test file
is therefore never compiled or executed, so the example ships with effectively
zero running tests despite appearing to have a suite.

**Evidence.** `grep -rn "mod test\|cfg(test)" src/` returns nothing for these
three examples, while their sibling examples (e.g. `mint-burn`,
`storage-migration`) do declare `mod test;`.

**Impact.** Untested behavior in audit-relevant examples; CI's per-example
`cargo test` passes vacuously for them.

**Suggested remediation.** Add `mod test;` to each `lib.rs` (and fix any
compilation drift the now-compiled tests reveal). Left for the maintainers.

---

## KI-3 — Non-root `[profile]` table is ignored (cargo warning)

- **Type:** Tooling warning
- **Status:** Open
- **Affects:** `examples/advanced/04-cross-contract-integration-testing/Cargo.toml`

**Description.** A `[profile.*]` table is defined in a non-root package, which
Cargo ignores in a workspace, emitting:
`profiles for the non root package will be ignored, specify profiles at the
workspace root`.

**Impact.** Cosmetic, but it adds noise to every build and the intended profile
settings are silently not applied.

**Suggested remediation.** Move the profile settings to the workspace root
`Cargo.toml` or remove them. Left for the maintainers.

---

## KI-4 — No committed `Cargo.lock`; dependency version drift

- **Type:** Reproducibility risk
- **Status:** Open
- **Affects:** Workspace

**Description.** `Cargo.lock` is git-ignored (`.gitignore` line 5). The
workspace declares `soroban-sdk = "26.0.0-rc.1"`, which currently resolves to
`26.0.1`. Without a committed lockfile, two builds can resolve different
dependency versions.

**Impact.** An auditor and the maintainers may not build identical bytes,
undermining reproducibility of any finding tied to a specific dependency
version. Also interacts with KI-1 (which version of `testutils` is compiled).

**Suggested remediation.** Decide on a lockfile policy for the audit: either
commit a `Cargo.lock` for the audited commit or document the exact resolved
versions in the reproducible-build notes. Left for the maintainers.

---

## Areas flagged for review (not findings)

The following are *not* known issues; they are pointers for the auditor derived
from the prep scan, to be confirmed or dismissed during the review:

- `03-priority-queue`, `iterable-mappings`, `ajo-factory`, `event-history`,
  `storage-migration` contain `unwrap()`/`expect()`/`panic!` usages — confirm
  none are reachable from attacker-influenced input in contract (non-test) code.
- `02-role-based-access-control` and `multi-sig-patterns` concentrate the
  authorization logic; verify role/threshold checks cannot be bypassed.
- `storage-migration` should be reviewed for migration ordering, idempotency,
  and authorization.

---

## Change log

| Date | Entry | Change |
| --- | --- | --- |
| 2026-06-02 | KI-1…KI-4 | Initial audit-prep baseline recorded. |

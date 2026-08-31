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
  `examples/tokens/09-optimized-token-ops`,
  `examples/intermediate/02-role-based-access-control`,
  `examples/intermediate/03-priority-queue`

**Description.** Each listed example contains a `src/test.rs`, but its `lib.rs`
declares neither `mod test;` nor an inline `#[cfg(test)]` module. The test file
is therefore never compiled or executed, so the example ships with effectively
zero running tests despite appearing to have a suite.

**Evidence.** `grep -rn "mod test\|cfg(test)" src/` returns nothing for these
four examples, while their sibling examples (e.g. `mint-burn`,
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

## KI-5 — Missing example README

- **Type:** Documentation gap
- **Status:** Open
- **Affects:** `examples/tokens/03-pausable-token`,
  `examples/tokens/06-reward-token`, `examples/intermediate/ajo`

**Description.** These three in-scope examples have no `README.md` at all
(`ls <example>/` shows only `Cargo.toml` and `src/`). Every other example in
both audit-scope tables has one. Without a README the auditor has no
maintainer-authored description of intended behavior to check the code
against, which is one of the two documentation-completeness checks this audit
prep exists to satisfy (§2 "does the documented behavior match the code?").

**Impact.** For these three examples the "README claims match the code" review
step in [`audit-scope.md`](./audit-scope.md) §2 cannot be performed; the
auditor must infer intended behavior from the source and tests alone.

**Suggested remediation.** Add a `README.md` to each, following the structure
already used by sibling examples (purpose, public API, usage example, security
considerations, testing instructions). Left for the maintainers.

---

## KI-6 — `examples/tokens/README.md` is stale relative to the current directory tree

- **Type:** Documentation gap
- **Status:** Open
- **Affects:** `examples/tokens/README.md`

**Description.** The category README's "What's Inside?" and "Examples"
sections predate several renames/additions and no longer match
`examples/tokens/`:

- Broken relative links — the target directory does not exist under the name
  linked: `./allowance-pattern/` (actual: `05-allowance-pattern`),
  `./token-wrapper/` (actual: `06-token-wrapper`), `./optimized-token-ops/`
  (actual: `09-optimized-token-ops`).
- Listed examples that do not exist anywhere in the tree: `02-vesting-contract`
  (the closest match, `05-vesting`, is a different, already-listed contract),
  `04-airdrop-contract`, `05-wrapped-asset`.
- In-scope examples not mentioned at all: `02-sep41-extensions`,
  `03-optimized-operations`, `03-pausable-token`, `05-allowance-pattern`,
  `05-vesting`, `10-custom-token`, `token-lock`.

**Impact.** A reader (including an auditor orienting themselves) following the
category README lands on broken links or looks for contracts that were never
merged, and misses seven of the eighteen in-scope contracts entirely.

**Suggested remediation.** Regenerate the "Examples" list from the actual
`examples/tokens/*` directories, fixing each link target. Left for the
maintainers so the correction can be reviewed alongside whatever prompted the
original drift.

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
- `ajo` (`src/lib.rs:53,60`, `.expect("Not initialized")`) and `lazy-loading`
  (`src/lib.rs:272`, `.unwrap()` on the oldest cache key) and
  `storage-pagination` (`src/lib.rs:150`, `.unwrap_or_else(|| panic!(...))` on
  an out-of-range page index) contain panics reachable from ordinary call
  paths — confirm each is unreachable for a well-formed caller or convert to a
  recoverable `Result`/error code.
- `token-lock` (`src/lib.rs:58,62,82,114,126,167`) uses `panic!` and
  `unwrap_or_else(|| panic!(...))` for both input validation (non-positive
  amount, past unlock time) and arithmetic overflow/underflow guards — confirm
  input-validation panics are intentional (vs. returning a contract error) and
  that the overflow/underflow paths cannot be triggered by a caller.
- `04-snapshot-token` (`src/lib.rs:423,447`) calls `.unwrap()` on
  `history.get(history.len() - 1)` — confirm the history vector cannot be
  empty at either call site.

---

## Change log

| Date | Entry | Change |
| --- | --- | --- |
| 2026-06-02 | KI-1…KI-4 | Initial audit-prep baseline recorded. |
| 2026-08-31 | KI-2, KI-5, KI-6 | Extended scope to `examples/tokens/`; added `09-optimized-token-ops` to KI-2; recorded missing READMEs (KI-5) and a stale category README (KI-6); added panic-path pointers for the newly in-scope examples. |

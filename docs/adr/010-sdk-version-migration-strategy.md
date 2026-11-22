# 10. Soroban SDK Version Migration Strategy

**Date:** 2026-11-20

## Status

**Status:** Accepted

## Context

All examples depend on `soroban-sdk`, which ships breaking changes across major versions in step with Stellar protocol upgrades. The cookbook moved from SDK 26 to the 28 release candidate (`28.0.0-rc.1`) and then to 29.0.0. During these moves some examples stopped compiling, and the templates and fuzz harness drifted from the workspace version.

Alternatives considered:

- **Per-example upgrades over time** — each example bumps its own SDK; the repo ends up with mixed versions and inconsistent teaching material
- **Big-bang upgrade with no fallback** — CI stays blocked until every example is fixed

## Decision

Upgrade the SDK at the workspace level, in one place, with a temporary exclusion list as a safety valve:

1. Dependabot (or a maintainer) bumps `soroban-sdk` in `[workspace.dependencies]`; every example inherits it via `soroban-sdk = { workspace = true }`
2. Examples that no longer compile are fixed in the same PR where practical; otherwise they are added to `scripts/ci-exclude-packages.txt` (see ADR-009) and fixed in follow-up PRs
3. Standalone templates and the fuzz harness are bumped to the same version so they stay in sync with the workspace
4. Release candidates may be adopted to get ahead of protocol changes, then replaced by the stable release
5. Stale version references in comments and docs are updated once the migration is complete

## Consequences

### Positive

- All examples teach a single SDK version
- CI immediately shows which examples a new SDK breaks
- Migrations can land incrementally without blocking unrelated work

### Negative

- A major bump touches many crates at once and needs coordinated review
- Excluded crates are untested until they are fixed

### Neutral

- The current workspace version and its history are tracked in [Book ADR-003: Soroban SDK Version](../../book/src/adr/003-soroban-sdk-version.md)

## References

- Root `Cargo.toml` (`[workspace.dependencies]`)
- `scripts/ci-exclude-packages.txt`
- [Soroban SDK releases](https://github.com/stellar/rs-soroban-sdk/releases)

## Notes

A migration is complete when `scripts/ci-exclude-packages.txt` has no package entries left.

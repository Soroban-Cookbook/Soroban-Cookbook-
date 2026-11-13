# 3. Test Contracts at Multiple Levels

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

Contract examples need fast feedback on contract behavior as well as checks for interactions and security properties that unit tests alone cannot cover. The growing Cargo workspace also needs repeatable formatting, lint, and coverage checks.

## Decision

Use Rust unit tests alongside contract implementations for focused behavior checks. Keep cross-contract integration tests in `tests/integration` and security-focused tests in `tests/security`. Run formatting, Clippy, builds, and the test suites through the GitHub Actions workflow; use `scripts/ci-workspace.sh` for workspace-aware CI execution. Generate workspace coverage with cargo-tarpaulin and publish the report artifact, with Codecov upload configured not to fail CI when the upload itself fails.

See [ADR-002](./002-workspace-structure.md) for the package layout and [ADR-004](./004-ci-cd.md) for required CI jobs.

## Consequences

### Positive

- Unit, integration, and security checks cover different failure modes.
- Shared CI commands make checks repeatable across the workspace.
- Coverage reports provide a trend artifact without making an external upload a build requirement.

### Negative

- Workspace and security checks increase CI runtime.
- Coverage tools and contract toolchains add maintenance to the CI configuration.

### Neutral

- A passing test suite does not replace review of contract-specific threat models.
- The exact test scope can be narrowed for changed examples while shared checks remain in place.

## References

- [Testing guide](../guides/testing.md)
- [Test workflow](../../../.github/workflows/test.yml)
- [CI workspace script](../../../scripts/ci-workspace.sh)
- [ADR-002: Workspace Structure](./002-workspace-structure.md)
- [ADR-004: CI/CD](./004-ci-cd.md)
# 11. Code Coverage Toolchain (tarpaulin + Codecov)

**Date:** 2026-11-20

## Status

**Status:** Accepted

## Context

The cookbook is teaching material, so example tests should exercise the code they demonstrate. The project needs coverage that runs in CI, works for a Cargo workspace, gives reviewers feedback on PRs, and is easy to reproduce locally.

Alternatives considered:

- **cargo-llvm-cov** — accurate source-based coverage, but needs the `llvm-tools` component and extra setup
- **grcov** — flexible, but requires instrumented builds and more configuration
- **Coveralls** — hosted service similar to Codecov, with weaker patch-coverage reporting

## Decision

Use **cargo-tarpaulin** to measure coverage and **Codecov** to report it.

- `tarpaulin.toml` configures workspace-wide coverage, writes Cobertura XML, HTML and LCOV reports to `coverage/`, and sets `fail-under = 90`
- The `coverage` job in `.github/workflows/test.yml` installs (and caches) tarpaulin, applies the CI exclude list, and uploads the XML report with `codecov/codecov-action`
- `codecov.yml` keeps project coverage informational and requires 90% patch coverage on changed lines
- Contributors run `cargo tarpaulin` locally with the same configuration

## Consequences

### Positive

- Single command for local and CI coverage
- PR comments and patch targets make untested new code visible
- Historical trends and badges via Codecov

### Negative

- tarpaulin only runs on Linux x86_64, so macOS/Windows contributors need CI or a container
- Line coverage is less precise than LLVM source-based coverage
- Coverage builds are slower than normal test runs

### Neutral

- Test modules, build artifacts and docs are excluded from coverage metrics

## References

- `tarpaulin.toml`
- `codecov.yml`
- `.github/workflows/test.yml` (coverage job)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)

## Notes

Switching to cargo-llvm-cov would only change the CI job and local command; the Codecov setup could stay the same.

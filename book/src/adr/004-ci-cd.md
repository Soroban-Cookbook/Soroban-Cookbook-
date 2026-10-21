# 4. Validate Changes with GitHub Actions

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

The repository combines Rust contracts with a documentation site. Pull requests need automated checks for source quality, tests, contract builds, and security, while documentation publication should happen only from the main branch.

## Decision

Use GitHub Actions as the CI/CD system. The test workflow validates Rust formatting and linting, runs security and integration suites, builds contracts, and generates coverage on pushes and pull requests to the configured branches. Keep documentation deployment separate: `deploy-docs.yml` builds the mdBook in `book/` and publishes it to `gh-pages` only after a push to `main`.

See [ADR-003](./003-testing-approach.md) for the testing layers and [ADR-005](./005-documentation-publishing.md) for the documentation site boundary.

## Consequences

### Positive

- Pull requests receive repeatable automated validation before merge.
- Documentation changes can be published independently of contract deployment.
- CI configuration is versioned alongside the source it validates.

### Negative

- Build and security jobs depend on external GitHub-hosted runners and package registries.
- Workflow changes can affect contributor feedback and require careful review.

### Neutral

- Deployment of the documentation site is tied to changes merged to `main`.
- Secrets are supplied through GitHub Actions configuration rather than committed to the repository.

## References

- [Test and lint workflow](../../../.github/workflows/test.yml)
- [Documentation deployment workflow](../../../.github/workflows/deploy-docs.yml)
- [ADR-003: Testing Approach](./003-testing-approach.md)
- [ADR-005: Documentation Publishing](./005-documentation-publishing.md)
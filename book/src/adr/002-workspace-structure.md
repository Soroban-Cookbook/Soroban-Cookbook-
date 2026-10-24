# 2. Organize Contracts in a Cargo Workspace

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

The cookbook contains reusable Rust code, many independent contract examples, and integration and security tests. Contributors need a consistent way to discover examples and share workspace-level dependency and build configuration without turning every example into one deployable contract.

## Decision

Keep the Rust projects in the root Cargo workspace. Group examples by learning level or use case under `examples/`, keep reusable code in `shared/`, and keep cross-example tests under `tests/`. Use workspace dependency declarations and package metadata where they are shared. Keep the mdBook source in `book/`, outside the Cargo workspace, because it has its own build and publishing lifecycle.

The current workspace membership and release profiles are defined in the root `Cargo.toml`. See [ADR-003](./003-testing-approach.md) for how this layout supports testing and [ADR-004](./004-ci-cd.md) for workspace CI.

## Consequences

### Positive

- Contributors can run Cargo commands across packages and share dependency versions.
- Examples remain independently understandable and deployable.
- Integration and security tests have clear homes outside individual examples.

### Negative

- Workspace-wide commands can take longer as the number of examples grows.
- Glob-based membership requires new examples to follow the established directory layout.

### Neutral

- Each example may still define package-specific dependencies and configuration.
- Documentation is versioned in the same repository but is built separately from Rust packages.

## References

- [Root Cargo workspace](../../../Cargo.toml)
- [ADR-003: Testing Approach](./003-testing-approach.md)
- [ADR-004: CI/CD](./004-ci-cd.md)
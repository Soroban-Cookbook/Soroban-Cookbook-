# 5. Publish Documentation with mdBook

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

The cookbook's examples and guides need a navigable, searchable website while remaining easy to edit as Markdown in the same repository. Documentation publication should not require a separate source repository or a release of the Rust packages.

## Decision

Maintain the published documentation as an mdBook rooted at `book/`, with Markdown pages under `book/src/` and navigation in `book/src/SUMMARY.md`. Configure the book in `book/book.toml`; build and publish it using the dedicated GitHub Actions workflow when changes reach `main`. Keep complementary project references in `docs/` and link between the two where useful.

See [ADR-004](./004-ci-cd.md) for the deployment trigger and [ADR-002](./002-workspace-structure.md) for the repository layout.

## Consequences

### Positive

- Documentation stays versioned with the examples it describes.
- Markdown source is suitable for review and contribution through pull requests.
- A dedicated workflow keeps docs publication separate from contract builds.

### Negative

- Navigation must be updated in `SUMMARY.md` when new pages are added.
- Content split between `book/` and `docs/` requires cross-links to avoid duplication.

### Neutral

- mdBook output is generated during deployment and is not the source of truth.
- Local preview and deployment use the same mdBook configuration.

## References

- [Book configuration](../../book.toml)
- [Book navigation](../SUMMARY.md)
- [Documentation deployment workflow](../../../.github/workflows/deploy-docs.yml)
- [ADR-002: Workspace Structure](./002-workspace-structure.md)
- [ADR-004: CI/CD](./004-ci-cd.md)
# 8. Use mdBook for Documentation

**Date:** 2026-11-20

## Status

**Status:** Accepted

## Context

The cookbook needs a documentation site that renders the markdown written next to the examples, deploys to GitHub Pages from CI, and can be previewed locally with one command. Most contributors are Rust developers, so a JavaScript toolchain would raise the barrier to editing docs.

Alternatives considered (see the [Documentation Platform Evaluation](../docs/documentation-platform-evaluation.md)):

- **Docusaurus** — React-based and highly customizable, but requires Node.js and React knowledge to maintain
- **MkDocs** — Python-based; adds a second language toolchain with no clear benefit
- **GitBook** — hosted service; vendor lock-in and cost at scale

## Decision

Use **mdBook** as the documentation platform. The book lives in `book/`, its sidebar is `book/src/SUMMARY.md`, and it is deployed by `.github/workflows/deploy-docs.yml`.

- Build: `cd book && mdbook build`
- Preview: `cd book && mdbook serve`

mdBook was chosen over Docusaurus because it is a single Rust binary that fits the project's existing toolchain, uses plain markdown with no MDX or React, and is the same tool used for the Rust Book, which most learners already know.

## Consequences

### Positive

- One toolchain (Rust/Cargo) for code and docs
- Contributors edit plain markdown; no front-end build knowledge needed
- Fast local builds and simple CI deployment

### Negative

- Less UI customization than Docusaurus (no React components, limited theming)
- Features such as versioned docs or rich search need preprocessors or custom JS

### Neutral

- Every new page must be added to `SUMMARY.md` to appear in the sidebar

## References

- [Book ADR-005: Documentation Platform](./005-documentation-platform.md)
- [mdBook documentation](https://rust-lang.github.io/mdBook/)

## Notes

Revisit this decision if the project needs versioned documentation or interactive components that mdBook cannot support.

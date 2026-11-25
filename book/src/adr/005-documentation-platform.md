# 5. Documentation Platform

**Date:** 2026-10-19

## Status

**Status:** Accepted

## Context

The Soroban Cookbook needs a documentation platform that:

- Builds from the markdown files already authored alongside the code
- Deploys automatically to GitHub Pages via CI (`.github/workflows/deploy-docs.yml`)
- Allows contributors to preview changes locally with a single command
- Supports a structured sidebar (`SUMMARY.md`) matching the repository layout
- Keeps the authoring barrier low for Rust-focused contributors who are not JavaScript developers

Two platforms were evaluated in detail (see [Documentation Platform Evaluation](../docs/documentation-platform-evaluation.md)):

**mdBook** — a static site generator written in Rust, designed for book-style documentation

**Docusaurus** — a React-based static site generator with richer UI customization

Additional platforms considered but quickly eliminated:

- **MkDocs** — Python-based; adds a second language toolchain with no benefit over mdBook for this use case
- **GitBook** — cloud-hosted; raises vendor lock-in concerns and costs money at scale
- **Astro** — requires significant front-end work; overkill for reference-heavy content

## Decision

Use **mdBook** as the primary documentation platform, deployed from the `book/` directory.

```
book/
  book.toml         # mdBook configuration
  src/
    SUMMARY.md      # sidebar / table of contents
    *.md            # documentation pages
    adr/            # Architecture Decision Records
    docs/           # reference material
    guides/         # step-by-step guides
    examples/       # example write-ups
```

Build command:

```bash
cd book && mdbook build
```

Local preview:

```bash
cd book && mdbook serve
```

**Why mdBook over Docusaurus:**

| Factor | mdBook | Docusaurus |
|--------|--------|-----------|
| Build tool language | Rust (fits the project) | JavaScript (extra toolchain) |
| Contributor overhead | Minimal | Moderate–high |
| Local build speed | Excellent | Moderate |
| Existing repo alignment | ✅ Already adopted | ❌ Would require migration |
| Migration cost | None | High |
| Interactive UI / search tuning | Limited | Good |
| Marketing / landing pages | Weak | Strong |

The project does not currently need marketing pages, versioned docs, or custom search facets — the three main Docusaurus advantages. Those needs are served by the `webapp/` surface.

**Scope split by platform:**

| Surface | Platform | Purpose |
|---------|----------|---------|
| `book/` | mdBook | Technical docs, guides, examples, ADRs |
| `docs/` | Markdown (no build) | Supporting reference notes, operational docs |
| `webapp/` | Custom React | Interactive playground, community showcase |

**Migration trigger criteria** (if two or more become true, revisit this ADR):

1. Versioned documentation becomes a hard requirement
2. The docs homepage must serve as the primary public marketing site
3. Custom search facets are required inside the docs site
4. Contributor demand shifts strongly toward UI-rich discovery

## Consequences

### Positive

- Zero new toolchain dependencies: mdBook is already installed in CI
- `SUMMARY.md` provides an explicit, reviewable table of contents that new contributors can read
- Fast local previews (`mdbook serve`) encourage documentation contributions
- Pages are pure markdown, making diffs readable and reviews straightforward
- The existing `deploy-docs.yml` workflow continues without changes

### Negative

- Limited built-in search (mdBook's default search is basic)
- UI customization requires Handlebars templates and custom CSS rather than React components
- Community-facing discovery features (filtering, cards) must live in `webapp/` rather than the docs site

### Neutral

- Docusaurus remains a future option; the structured `SUMMARY.md` and markdown-first approach make a future migration straightforward if the trigger criteria are met
- The `docs/` directory continues to hold supporting reference material without a build step; only `book/` is published

## References

- [mdBook documentation](https://rust-lang.github.io/mdBook/)
- [mdBook GitHub](https://github.com/rust-lang/mdBook)
- [Documentation Platform Evaluation](../docs/documentation-platform-evaluation.md)
- Current build workflow: [`.github/workflows/deploy-docs.yml`](../../../.github/workflows/deploy-docs.yml)
- Current configuration: [`book/book.toml`](../../../book/book.toml)

## Notes

To add a new page to the documentation:

1. Create a markdown file in the appropriate `book/src/` subdirectory
2. Add an entry to `book/src/SUMMARY.md`
3. Run `cd book && mdbook build` to verify no broken links
4. Open a PR — the deploy workflow publishes changes automatically on merge to `main`

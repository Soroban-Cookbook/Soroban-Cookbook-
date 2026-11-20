# 12. Example Naming Convention (Numeric Prefixes)

**Date:** 2026-11-20

## Status

**Status:** Accepted

## Context

Examples are grouped by category (`basics`, `intermediate`, `advanced`, `defi`, `tokens`, `nfts`, `governance`, `storage`). Learners need a clear order to follow within each category, and maintainers need directory names that sort predictably in file browsers, the book sidebar and the workspace `members` list.

Alternatives considered:

- **Plain descriptive names** (`hello-world`, `custom-errors`) — no implied learning order; listings sort alphabetically
- **Separate ordering file** — order is defined outside the directory itself and easily drifts

## Decision

Name example directories `examples/<category>/NN-short-name`, where `NN` is a two-digit number giving the suggested reading order within the category, and `short-name` is lowercase kebab-case (for example `examples/basics/01-hello-world`, `examples/tokens/01-sep41-token`).

- Numbers are scoped to a category, not global
- New examples take the next free number in their category
- Closely related variants may share a number (for example `13-collection-types` and `13-queue-variants`)
- Book pages for examples mirror the directory name (for example `book/src/examples/tokens/01-sep41-token.md`)

## Consequences

### Positive

- Directory listings sort in learning order
- The number gives learners an obvious next step
- Book pages and example directories map one-to-one

### Negative

- Inserting an example in the middle of a sequence means reusing a number or renumbering
- A few older directories without a prefix (for example `examples/hello-world`, `examples/basics/lazy-cache`) do not follow the convention

### Neutral

- Crate names in `Cargo.toml` are independent of the numeric prefix

## References

- `examples/README.md`
- `book/src/SUMMARY.md`

## Notes

Renaming existing directories to fill gaps is not required; prefer stable paths over a perfectly contiguous sequence.

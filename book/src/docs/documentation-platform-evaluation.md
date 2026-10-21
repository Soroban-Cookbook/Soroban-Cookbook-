# Documentation Platform Evaluation

This note compares the repository's current `mdBook` setup with a possible move to Docusaurus.

## Scope

The evaluation is based on the current Soroban Cookbook repository shape:

- Rust-heavy examples with many local markdown files
- a docs pipeline that already builds from `book/`
- a need for low-friction contributor edits
- reference-heavy content that benefits from fast local preview and stable linking

## Summary Decision

Keep `mdBook` as the primary documentation platform for the current repository phase.

Docusaurus remains a reasonable future option if the project later needs a richer marketing layer, versioned product docs, or plugin-driven content experiences that exceed the current cookbook scope.

## Evaluation Matrix

| Capability | mdBook | Docusaurus | Repo Fit |
| --- | --- | --- | --- |
| Rust documentation workflow | Excellent | Good | `mdBook` wins |
| Contributor simplicity | Excellent | Moderate | `mdBook` wins |
| Local build speed | Excellent | Moderate | `mdBook` wins |
| Sidebar authoring | Good | Excellent | Docusaurus wins |
| Search customization | Moderate | Good | Docusaurus wins |
| Interactive marketing pages | Weak | Excellent | Docusaurus wins |
| Existing repo alignment | Excellent | Weak | `mdBook` wins |
| Migration cost today | None | High | `mdBook` wins |

## What Was Compared

### mdBook strengths

- Already adopted in this repository.
- Minimal configuration and low contributor overhead.
- Predictable markdown-first authoring for guides, examples, and reference material.
- Good fit for a cookbook where page structure mirrors repository structure.
- Easy to review in pull requests because content changes stay close to source.

### mdBook limitations

- Limited out-of-the-box search tuning and UI customization.
- Weaker support for landing-page style community storytelling.
- Fewer plugin and ecosystem options than modern JS doc stacks.

### Docusaurus strengths

- Better front-end customization, theming, and navigation polish.
- Stronger support for search providers, docs versioning, and hybrid docs plus marketing pages.
- Easier to build community-facing sections with filters, cards, and richer navigation.

### Docusaurus limitations

- Introduces a larger JavaScript toolchain for a Rust-first repository.
- Raises the maintenance bar for contributors making small documentation fixes.
- Requires migration of current mdBook navigation and link structure.
- Creates overlap with the existing `webapp/` surface for UI-heavy content.

## Architecture Plan

### Near term

Keep the documentation stack split by responsibility:

- `book/`: authoritative technical docs, guides, and example write-ups
- `docs/`: supporting reference material and operational notes
- `webapp/`: interactive or community-facing pages that benefit from filtering and richer UI

This preserves the current authoring model while letting UI-rich content evolve independently.

### Medium term

If discoverability becomes the main bottleneck, add incremental improvements before migrating platforms:

1. tighten `SUMMARY.md` coverage
2. standardize page metadata and naming
3. move community showcase and discovery features into the webapp
4. keep mdBook focused on technical documentation

## Migration Trigger Criteria

Revisit Docusaurus only if at least two of these become true:

- versioned docs are required
- the docs homepage must double as the primary public marketing site
- custom search facets become a hard requirement inside the docs site
- contributor demand shifts toward UI-rich discovery over code-adjacent reference pages

## Migration Plan If Triggered Later

1. Freeze the current `mdBook` information architecture and export a page inventory.
2. Move only community and landing content first, not the full reference set.
3. Keep example source and technical reference pages in markdown with stable slugs.
4. Add redirects for existing mdBook paths before switching the primary docs URL.
5. Migrate the remaining technical sections only after parity checks pass.

## Recommendation

Use `mdBook` for technical docs now, and use the `webapp/` for richer discovery experiences such as project showcase pages. That split matches the repository's current structure with the least operational cost.
# Soroban Cookbook Documentation Setup

## Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install mdbook soroban-cli
```

## Local Development

### Preview Docs Site

```bash
cd book
mdbook serve  # http://localhost:2345
```

### Build Docs

```bash
./scripts/build.sh  # Builds contracts + mdbook
mdbook build       # Docs only
```

## Structure

```
book/
├── book.toml          # mdBook config
├── src/
│   ├── SUMMARY.md     # Navigation
│   ├── guides/        # Tutorials
│   ├── examples/      # Example overviews
│   └── docs/          # Reference
└── book-output/       # Built HTML (gitignored)
```

## Deployment

- GitHub Actions: `.github/workflows/deploy-docs.yml`
- Push to `main` → automatic build/deploy to GitHub Pages
- Live site: https://soroban-cookbook.github.io/Soroban-Cookbook/

## Contributing Docs

1. Add content to `book/src/`
2. Update `SUMMARY.md` navigation
3. Preview: `mdbook serve`
4. Commit/push → auto-deploy

## mdBook Tips

- Frontmatter: `---\nkey: value\n---`
- Shortcodes: `{{#include}}`, `{{#rust}}`
- Themes: Customize in `book.toml`

See [mdBook Book](https://rust-lang.github.io/mdBook/) for advanced usage.

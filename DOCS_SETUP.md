# Documentation Site Setup

This document explains the documentation site setup for Soroban Cookbook.

## Tool Selection: mdBook

**Selected:** mdBook  
**Rationale:**
- Native to Rust ecosystem
- Lightweight and fast
- Markdown-based (existing docs compatible)
- Built-in search functionality
- Easy GitHub Pages deployment
- Active maintenance and community

## Structure

```
Soroban-Cookbook/
├── book.toml              # mdBook configuration
├── book/                  # Documentation source
│   └── src/
│       ├── SUMMARY.md     # Table of contents
│       ├── README.md      # Introduction
│       ├── guides/        # Tutorial guides
│       ├── examples/      # Example overviews
│       └── docs/          # Reference docs
└── book-output/           # Generated site (gitignored)
```

## Local Development

### Prerequisites

Install mdBook:
```bash
cargo install mdbook
```

### Build and Serve

```bash
# Build the book
mdbook build

# Serve locally with live reload
mdbook serve
# Opens at http://localhost:3000
```

### Watch for changes
```bash
mdbook watch
```

## Deployment

### GitHub Pages (Automated)

The site automatically deploys to GitHub Pages on push to `main` via `.github/workflows/deploy-docs.yml`.

**Setup Steps:**
1. Go to repository Settings → Pages
2. Set Source to "GitHub Actions"
3. Push to main branch
4. Site will be available at: `https://soroban-cookbook.github.io/Soroban-Cookbook/`

### Manual Deployment

```bash
# Build the book
mdbook build

# Deploy the book-output/ directory to your hosting service
```

## Adding Content

### Add a New Page

1. Create the markdown file in `book/src/`
2. Add entry to `book/src/SUMMARY.md`
3. Build and test locally

Example:
```markdown
# In SUMMARY.md
- [New Guide](./guides/new-guide.md)
```

### Link to Repository Files

Use relative links to reference actual code:
```markdown
[View example code](https://github.com/Soroban-Cookbook/Soroban-Cookbook/tree/main/examples/basics/01-hello-world)
```

## Maintenance

### Update Existing Docs

Docs in `book/src/guides/` and `book/src/docs/` are symlinked to the original files. Edit the originals in `guides/` and `docs/` directories.

### Add New Examples

When adding new examples to `examples/`, update the corresponding overview page in `book/src/examples/`.

## Configuration

Edit `book.toml` to customize:
- Title and metadata
- Theme settings
- Search configuration
- Output directory

## Testing

```bash
# Check for broken links
mdbook test

# Build to verify no errors
mdbook build
```

## Troubleshooting

**Issue:** Links not working  
**Solution:** Ensure paths in SUMMARY.md match actual file locations

**Issue:** Changes not showing  
**Solution:** Clear `book-output/` and rebuild: `rm -rf book-output && mdbook build`

**Issue:** GitHub Pages not updating  
**Solution:** Check Actions tab for workflow errors

## Future Enhancements

- Add mdbook-mermaid for diagrams
- Add mdbook-linkcheck for link validation
- Custom theme/styling
- Multi-language support
- API documentation integration

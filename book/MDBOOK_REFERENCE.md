# mdBook Quick Reference

## Installation
```bash
cargo install mdbook
```

## Common Commands

### Development
```bash
# Serve with live reload (http://localhost:3000)
mdbook serve

# Serve on custom port
mdbook serve --port 8080

# Watch for changes (build only)
mdbook watch
```

### Building
```bash
# Build the book
mdbook build

# Build and open in browser
mdbook build --open

# Clean build
rm -rf book-output && mdbook build
```

### Testing
```bash
# Test code samples in documentation
mdbook test

# Check configuration
mdbook build --verbose
```

## File Structure
```
book.toml           # Configuration
book/src/
  ├── SUMMARY.md    # Table of contents (required)
  ├── README.md     # Introduction page
  └── ...           # Other markdown files
```

## SUMMARY.md Syntax
```markdown
# Summary

[Introduction](./README.md)

# Section Name
- [Chapter 1](./chapter1.md)
  - [Subsection](./chapter1/subsection.md)
- [Chapter 2](./chapter2.md)

---

[Appendix](./appendix.md)
```

## Configuration (book.toml)
```toml
[book]
title = "My Book"
authors = ["Author Name"]
language = "en"

[output.html]
default-theme = "rust"
git-repository-url = "https://github.com/user/repo"

[output.html.search]
enable = true
```

## Useful Links
- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [mdBook Guide](https://rust-lang.github.io/mdBook/guide/creating.html)

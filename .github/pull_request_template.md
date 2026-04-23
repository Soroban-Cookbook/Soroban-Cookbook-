## Description

<!-- Provide a clear and concise description of your changes -->

<!-- Please read our Code of Conduct: https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/CODE_OF_CONDUCT.md -->

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] New example or improvement to existing example
- [ ] Infrastructure/tooling change
- [ ] Code cleanup or refactoring

## Related Issues

<!-- Link to related issues using keywords like "Closes", "Fixes", or "Resolves" -->

Closes #

## Changes Made

<!-- List the specific changes you made -->

-
-
-

## Testing

<!-- Describe how you tested your changes -->

### Test Steps

1.
2.
3.

### Test Results

<!-- Paste relevant test output or screenshots -->

```bash
# Example: cargo test output
```

## Checklist

### Code Quality
- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] All contracts use `#![no_std]`
- [ ] Custom errors use `#[contracterror]` where applicable

### Testing
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Integration tests are included for multi-contract interactions (if applicable)

### Documentation
- [ ] I have made corresponding changes to the documentation
- [ ] I have updated the main README.md if needed
- [ ] I have added/updated the example's README.md (if this is a new/modified example)
- [ ] I have updated SUMMARY.md if adding new content to the book

### Pre-submission Validation
- [ ] Code formatting passes: `cargo fmt --all --check`
- [ ] Linting passes with no warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] All tests pass: `cargo test --workspace --all-features`
- [ ] WASM build succeeds: `cargo build --workspace --target wasm32-unknown-unknown --release`

### Other
- [ ] Any dependent changes have been merged and published
- [ ] I have read the [CONTRIBUTING.md](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/CONTRIBUTING.md) guidelines

## Additional Notes

<!-- Any additional information, context, or screenshots -->

## Screenshots (if applicable)

<!-- Add screenshots to help explain your changes -->

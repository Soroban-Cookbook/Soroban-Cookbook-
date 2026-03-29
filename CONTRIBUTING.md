# Contributing to Soroban Cookbook

Thank you for your interest in contributing to the Soroban Cookbook! This guide will help you get started.

Please also read our [Code of Conduct](./CODE_OF_CONDUCT.md) before participating in our community.

## 🎯 Ways to Contribute

### 1. Add New Examples

Create clear, well-documented smart contract examples that demonstrate specific patterns or use cases.

### 2. Improve Documentation

Enhance existing guides, fix typos, add clarifications, or translate content.

### 3. Review Pull Requests

Help maintain quality by reviewing PRs and providing constructive feedback.

### 4. Report Issues

Found a bug or have a suggestion? Open an issue with a clear description.

## 🛠️ Development Environment Setup

### Prerequisites

- Basic knowledge of Rust programming
- Command line comfort
- Git installed

### Quick Setup

1. **Install Rust**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add WASM target**

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Install Soroban CLI**

   ```bash
   cargo install --locked soroban-cli --features opt
   ```

4. **Clone and verify**

   ```bash
   git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git
   cd Soroban-Cookbook-
   cargo test
   ```

For detailed network configuration and deployment, see the [Getting Started Guide](./guides/getting-started.md).

## 📝 Example Guidelines

When adding a new example, ensure it includes:

### Required Elements

- ✅ **Clear purpose statement** - What does this contract do?
- ✅ **Well-commented code** - Explain key concepts inline
- ✅ **Comprehensive tests** - Both unit and integration tests
- ✅ **README.md** - Usage instructions and deployment steps
- ✅ **Cargo.toml** - Proper dependencies and metadata

### Code Quality Standards

- ✅ Compiles with latest stable Soroban SDK
- ✅ Follows Rust naming conventions and idioms
- ✅ Includes error handling
- ✅ Passes `cargo clippy` with no warnings
- ✅ Formatted with `cargo fmt`
- ✅ All tests pass with `cargo test`

### Documentation Standards

- Clear and concise explanations
- Code comments for complex logic
- Usage examples in README
- Link to relevant official documentation

## 🏗️ Project Structure

```
examples/
├── basics/              # Beginner-friendly examples
│   └── 01-hello-world/
│       ├── src/
│       │   ├── lib.rs
│       │   └── test.rs
│       ├── Cargo.toml
│       └── README.md
├── intermediate/        # Common patterns
├── advanced/           # Complex systems
├── defi/               # DeFi protocols
├── nfts/               # NFT implementations
├── governance/         # DAOs and voting
└── tokens/             # Token standards
```

## 🔄 Pull Request Process

1. **Fork the repository** and create a new branch

   ```bash
   git checkout -b feature/your-example-name
   ```

2. **Add your example** following the structure above

3. **Test thoroughly**

   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

4. **Update documentation**
   - Add entry to main README.md
   - Create detailed README.md for your example
   - Update relevant guides if needed

5. **Submit PR** with:
   - Clear title describing the change
   - Description of what the example demonstrates
   - Any additional context or considerations

## ✅ Validation Steps

Before submitting, run the following checks:

### Targeted Checks

Run tests and checks for the specific files you changed:

```bash
# Test a specific example
cargo test -p <package-name>

# Lint a specific package
cargo clippy -p <package-name> -- -D warnings

# Format a specific package
cargo fmt -- --check
```

### Workspace Checks

If your changes affect shared tooling, docs, or examples:

```bash
# Run full workspace tests
cargo test --workspace

# Run full workspace linting
cargo clippy --workspace -- -D warnings

# Check formatting across workspace
cargo fmt -- --check
```

### Documentation Verification

- Ensure all links in documentation are valid
- Verify README.md files are accurate after any changes
- Check that guides remain consistent with code examples

## 📋 Pull Request Template

When submitting a pull request, use this template:

```markdown
## Description

<!-- Describe your changes clearly and concisely -->

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

## Testing

<!-- Describe how you tested your changes -->

### Test Steps

1.
2.
3.

### Test Results

<!-- Paste relevant test output or screenshots -->

## Checklist

- [ ] My code follows the style guidelines of this project (`cargo fmt`)
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings (`cargo clippy`)
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes (`cargo test`)
- [ ] Any dependent changes have been merged and published
- [ ] I have updated the README.md if needed
- [ ] I have added an entry to the example's README if this is a new example

## Additional Notes

<!-- Any additional information, context, or screenshots -->
```

## ✅ Checklist for New Examples

Before submitting, verify:

- [ ] Code compiles without errors
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code is formatted with `cargo fmt`
- [ ] README.md included with usage instructions
- [ ] Inline comments explain key concepts
- [ ] Contract includes proper error handling
- [ ] Example is placed in the correct category
- [ ] Main README.md is updated

## 🎨 Code Style

Follow standard Rust conventions:

```rust
// Good: Clear naming, proper documentation
/// Transfers tokens from one account to another.
///
/// # Arguments
/// * `from` - Source account address
/// * `to` - Destination account address
/// * `amount` - Number of tokens to transfer
///
/// # Panics
/// Panics if the sender has insufficient balance.
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();
    // Implementation...
}
```

## 🧪 Testing Guidelines

### Unit Tests

Test individual functions in isolation:

```rust
#[test]
fn test_transfer() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    // Test logic...
}
```

### Integration Tests

Test complete workflows:

```rust
#[test]
fn test_complete_workflow() {
    // Set up multiple contracts
    // Execute a full user journey
    // Verify end state
}
```

## 🤝 Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help others learn and grow

## ❓ Questions?

- Open a [discussion](https://github.com/Soroban-Cookbook/Soroban-Cookbook/discussions)
- Join [Stellar Discord](https://discord.gg/stellardev)
- Check [official Soroban docs](https://developers.stellar.org/docs/smart-contracts)

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for helping make Soroban more accessible to developers worldwide! 🚀

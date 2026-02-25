# Contributing to Soroban Cookbook

Thank you for your interest in contributing to the Soroban Cookbook! This guide will help you get started.

## 🎯 Ways to Contribute

### 1. Add New Examples

Create clear, well-documented smart contract examples that demonstrate specific patterns or use cases.

### 2. Improve Documentation

Enhance existing guides, fix typos, add clarifications, or translate content.

### 3. Review Pull Requests

Help maintain quality by reviewing PRs and providing constructive feedback.

### 4. Report Issues

Found a bug or have a suggestion? Open an issue with a clear description.

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

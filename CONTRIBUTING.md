# Contributing to Soroban Cookbook

Thank you for your interest in contributing to the Soroban Cookbook! This project aims to be a comprehensive resource for Soroban developers, and your contributions are what make it great.

Please read our [Code of Conduct](./CODE_OF_CONDUCT.md) before participating.

## 🎯 Ways to Contribute

1.  **Add New Examples**: Create well-documented smart contract examples demonstrating specific patterns.
2.  **Improve Documentation**: Fix typos, clarify guides, or add new documentation.
3.  **Bug Reports & Feature Requests**: Use [GitHub Issues](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues) to report bugs or suggest new features.
4.  **Code Review**: Review open pull requests and provide constructive feedback.

---

## 🛠️ Development Environment Setup

### 1. Prerequisites

- **Rust**: Latest stable version.
- **WASM Target**: Required for compiling Soroban contracts.
- **Stellar CLI**: Used for building, testing, and deploying (formerly `soroban-cli`).

### 2. Installation Steps

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Add WASM target
rustup target add wasm32-unknown-unknown

# 3. Install Stellar CLI (version 22.1.0+ recommended)
cargo install --locked stellar-cli --version 22.1.0

# 4. Clone the repository
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git
cd Soroban-Cookbook-

# 5. Verify installation
cargo test --workspace
```

For more detailed setup, see the [Getting Started Guide](./guides/getting-started.md).

---

## 📝 Code Style Guidelines

To maintain a consistent and high-quality codebase, please follow these guidelines:

- **Naming**: Follow standard [Rust naming conventions](https://rust-lang.github.io/api-guidelines/naming.html).
- **Formatting**: Always run `cargo fmt` before committing.
- **Linting**: Ensure `cargo clippy` passes with no warnings (`-D warnings`).
- **Error Handling**: Use custom errors with `#[contracterror]` for meaningful error messages.
- **Comments**: Use `///` for documentation comments and `//` for internal logic explanation.
- **No-std**: All contract code must be `#![no_std]`.

---

## 🏗️ Project Structure

- `examples/`: Categorized smart contract examples.
- `docs/`: General documentation and ADRs.
- `guides/`: Step-by-step tutorials and guides.
- `book/`: Source for the mdBook documentation.
- `tests/`: Integration tests for the workspace.

---

## 🔄 Pull Request Process

1.  **Branching**: Create a feature branch from `main`.
    ```bash
    git checkout -b feature/your-feature-name
    ```
2.  **Development**: Implement your changes following the style guidelines.
3.  **Local Testing**: Run the validation suite (see below).
4.  **Commit**: Use descriptive commit messages.
5.  **Documentation**: If adding an example, ensure it has a `README.md` and is added to the main `README.md` and `SUMMARY.md` if applicable.
6.  **Submit PR**: Fill out the [Pull Request Template](.github/pull_request_template.md).

---

## 🧪 Testing Requirements

All contributions must include tests:

- **Unit Tests**: In `src/test.rs` for individual function logic.
- **Integration Tests**: In `tests/` for multi-contract or complex interactions.
- **Mocking**: Use `env.mock_all_auths()` for testing authorization flows.
- **Coverage**: Aim for high test coverage. You can check coverage locally using `cargo tarpaulin`.

---

## 📋 Example Contribution Template

When adding a new example in `examples/category/name/`:

```text
name/
├── src/
│   ├── lib.rs       # Contract implementation
│   └── test.rs      # Unit tests
├── Cargo.toml       # Metadata and dependencies
└── README.md        # Description, how to run, and explanation
```

The `README.md` for the example should include:
1.  **What it does**: Clear purpose statement.
2.  **Key Concepts**: Explanation of Soroban features used.
3.  **How to Run**: Commands for testing and building.

---

## ✅ Validation Steps

Before submitting your PR, ensure all these checks pass:

```bash
# 1. Format check
cargo fmt --all --check

# 2. Lint check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Run all tests
cargo test --workspace

# 4. Build Wasm (for contracts)
cargo build --workspace --target wasm32-unknown-unknown --release
```

---

## 🚀 Definition of Done

- [ ] Acceptance criteria of the issue are met.
- [ ] Code follows style guidelines and passes all checks.
- [ ] Tests are included and passing.
- [ ] Documentation (README, guides, SUMMARY.md) is updated.
- [ ] PR is linked to relevant issues.

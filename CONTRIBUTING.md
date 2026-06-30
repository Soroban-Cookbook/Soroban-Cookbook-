Markdown
# Contributing to Soroban Cookbook 🍳

Thank you for your interest in contributing to the Soroban Cookbook! This project aims to be a comprehensive resource for Soroban developers, and your contributions are what make it great.

Please read our [Code of Conduct](./CODE_OF_CONDUCT.md) before participating.

---

## 📍 Table of Contents
- [Ways to Contribute](#-ways-to-contribute)
- [Development Environment Setup](#️-development-environment-setup)
- [Code Style Guidelines](#-code-style-guidelines)
- [Project Structure](#️-project-structure)
- [Pull Request Process](#-pull-request-process)
- [Testing Requirements](#-testing-requirements)
- [Example Contribution Template](#-example-contribution-template)
- [Validation Steps](#-validation-steps)
- [Bug Bounty Program](#-bug-bounty-program)
- [Recognition System](#-recognition-system)

---

## 🎯 Ways to Contribute

1.  **Add New Examples**: Create well-documented smart contract examples demonstrating specific patterns.
2.  **Improve Documentation**: Fix typos, clarify guides, or add new documentation.
3.  **Bug Reports & Feature Requests**: Use [GitHub Issues] to report bugs or suggest new features.
4.  **Code Review**: Review open pull requests and provide constructive feedback.

---

## 🛠️ Development Environment Setup

### 1. Prerequisites

- **Rust**: Latest stable version.
- **WASM Target**: Required for compiling Soroban contracts.
- **Stellar CLI**: Used for building, testing, and deploying (Note: `stellar-cli` has replaced `soroban-cli`).

### 2. Installation Steps

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh

# 2. Add WASM target
rustup target add wasm32-unknown-unknown

# 3. Install Stellar CLI (version 22.1.0+ recommended)
cargo install --locked stellar-cli --version 22.1.0

# 4. Clone the repository
git clone [https://github.com/username/Soroban-Cookbook-.git]
cd Soroban-Cookbook-

# 5. Verify installation
cargo test --workspace
For more detailed setup, see the Getting Started Guide.

To maintain a consistent and high-quality codebase, please follow our [Style Guide](./docs/style-guide.md).

Key highlights:
- **Naming**: Follow standard [Rust naming conventions](https://rust-lang.github.io/api-guidelines/naming.html) and our specific contract patterns.
- **Formatting**: Always run `cargo fmt` before committing.
- **Linting**: Ensure `cargo clippy` passes with no warnings (`-D warnings`).
- **Documentation**: Use `///` for public interface docs and `//!` for module-level explanations.
- **Testing**: Every example must include comprehensive unit tests.
- **No-std**: All contract code must be `#![no_std]`.

🏗️ Project Structure
examples/: Categorized smart contract examples.

docs/: General documentation and ADRs.

guides/: Step-by-step tutorials and guides.

book/: Source for the mdBook documentation.

tests/: Integration tests for the workspace.

🔄 Pull Request Process
Branching: Create a feature branch from main.

Bash
git checkout -b feature/your-feature-name
Development: Implement your changes following the style guidelines.

Local Testing: Run the validation suite (see below).

Commit: Use descriptive commit messages.

Documentation: If adding an example, ensure it has a README.md and is added to the main README.md and SUMMARY.md if applicable.

Submit PR: Fill out the Pull Request Template.

🧪 Testing Requirements
All contributions must include tests:

- **Unit Tests**: In `src/test.rs` for individual function logic.
- **Integration Tests**: In `tests/` for multi-contract or complex interactions.
- **Mocking**: Use `env.mock_all_auths()` for testing authorization flows.
- **Coverage**: Aim for high test coverage. Run coverage locally with:
  ```bash
  cargo tarpaulin
  # Reports are written to coverage/ (XML, HTML, LCOV)
  # Open coverage/tarpaulin-report.html in a browser for a line-by-line view
  ```

📋 Example Contribution Template
When adding a new example in examples/category/name/:

Plaintext
name/
├── src/
│   ├── lib.rs       # Contract implementation
│   └── test.rs      # Unit tests
├── Cargo.toml       # Metadata and dependencies
└── README.md        # Description, how to run, and explanation
The README.md for the example should include:

What it does: Clear purpose statement.

Key Concepts: Explanation of Soroban features used.

How to Run: Commands for testing and building.

✅ Validation Steps
Before submitting your PR, ensure all these checks pass:

Bash
# 1. Format check
cargo fmt --all --check

# 2. Lint check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Run all tests
cargo test --workspace

# 4. Build Wasm (for contracts)
cargo build --workspace --target wasm32-unknown-unknown --release
🚀 Definition of Done
[ ] Acceptance criteria of the issue are met.

[ ] Code follows style guidelines and passes all checks.

[ ] Tests are included and passing.

[ ] Documentation (README, guides, SUMMARY.md) is updated.

[ ] PR is linked to relevant issues.
### CI Testing Strategy

- We run targeted tests for changed paths on pull requests to enable fast feedback.
- For merges to main, the CI fallback runs the entire workspace check to ensure full compatibility.

---

## 🛡️ Bug Bounty Program

The Soroban Cookbook Bug Bounty Program rewards community members who responsibly disclose security vulnerabilities in the contract examples and shared library code.

### Scope

**In scope:**
- Smart contract examples in `examples/` that contain logic vulnerabilities, incorrect auth patterns, or integer overflow/underflow bugs
- The `shared/` library (validation helpers, test utilities)
- Integration tests in `tests/` that mask rather than catch security issues
- Documentation that provides insecure guidance or incorrect security advice

**Out of scope:**
- Third-party dependencies (report upstream)
- Issues in the `webapp/` Next.js frontend
- Theoretical vulnerabilities without a working proof-of-concept
- Style or formatting issues

### Reward Tiers

| Severity | Description | Reward |
|----------|-------------|--------|
| **Critical** | Auth bypass, fund theft, or arbitrary state manipulation in any example contract | $500–$1,000 USD equivalent |
| **High** | Logic flaw enabling incorrect behavior affecting core contract functions | $200–$500 USD equivalent |
| **Medium** | Incorrect documentation that teaches insecure patterns; test gaps hiding real bugs | $50–$200 USD equivalent |
| **Low** | Minor issues, missing input validation, documentation inaccuracies | $10–$50 USD equivalent or public acknowledgment |

Rewards are paid in XLM at the time of payout. The triage team makes final severity determinations.

### Rules and Guidelines

1. **Responsible Disclosure**: Report vulnerabilities privately before public disclosure. Do not open a public GitHub issue for security bugs.
2. **Report Channel**: Email `security@soroban-cookbook.dev` or open a [GitHub Security Advisory](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/security/advisories/new) with full details.
3. **Proof of Concept**: Include a minimal failing test or reproduction steps. Reports without a PoC may not qualify for rewards.
4. **No Destructive Testing**: Do not test on mainnet. All testing must be done locally or on testnet using the provided tooling.
5. **One Report Per Issue**: Duplicate reports are not rewarded. The first valid submission receives the reward.
6. **Good Faith**: Researchers must not access, modify, or delete data beyond what is needed to demonstrate the vulnerability.
7. **No Social Engineering**: Do not attempt to compromise maintainer accounts or CI infrastructure.
8. **Response SLA**: The team will acknowledge reports within 5 business days and provide a resolution timeline within 14 business days.

### Disclosure Timeline

- **Day 0**: Vulnerability reported privately
- **Day 1–5**: Team acknowledges receipt
- **Day 1–14**: Severity triage and fix timeline communicated
- **Day 14–60**: Fix developed, reviewed, and merged
- **Day 60+**: Coordinated public disclosure with researcher credit (if desired)

### Budget

The program is funded from the project's community treasury. Total annual budget: **$5,000 USD**. Awards are distributed on a first-come, first-served basis until the annual budget is exhausted.

For full details, see [`docs/security-audit/bug-bounty.md`](./docs/security-audit/bug-bounty.md).

## 🏆 Recognition System

All contributors are recognized for their work. We have a tiered recognition system with rewards ranging from public acknowledgment to Stellar swag and community spotlights.

See [docs/recognition-system.md](./docs/recognition-system.md) for the full criteria, tiers, rewards, and automation plan.

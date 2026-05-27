# GitHub Workflows

Automated CI/CD pipelines for the Soroban Cookbook project.

## 📋 Available Workflows

### [CI (Continuous Integration)](./ci.yml)
Runs on every pull request and push to the main branch.
- **Tasks:** Lints the codebase, checks formatting, runs workspace tests, validates each basic example crate, builds Wasm targets, and publishes a Wasm size report (before/after `wasm-opt`).

### [Test Suite](./test.yml)
Runs all unit and integration tests across the examples and shared tooling.
- **Tasks:** Executes `cargo test` for all packages in the workspace.

### [Deploy Docs](./deploy-docs.yml)
Automatically builds and deploys the mdBook documentation to GitHub Pages.
- **Tasks:** Builds the book from `book/src/` and pushes the output to the `gh-pages` branch.

### [Security Audit](./security-audit.yml)
Audits all workspace dependencies for known vulnerabilities using `cargo audit`.
- **Triggers:** Push/PR to `main` when `Cargo.toml` or `Cargo.lock` changes, weekly on Mondays at 08:00 UTC, and manual dispatch.
- **Tasks:** Runs `cargo audit --deny warnings --deny unsound` against the RustSec Advisory Database. Fails the job on any HIGH or CRITICAL severity advisory. Uploads a full JSON audit report as a workflow artifact (retained for 30 days).

### [Dependabot](./dependabot-auto-merge.yml)
Automates dependency updates and auto-merges safe PRs.
- **Tasks:** Keeps Rust and Node.js dependencies up-to-date.

## 🛠️ Local Development

You can run the CI checks locally before pushing your changes:

```bash
# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets -- -D warnings

# Run all tests
cargo test --all
```

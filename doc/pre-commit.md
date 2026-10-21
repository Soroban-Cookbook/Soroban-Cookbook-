# Pre-Commit Hooks

The repository configures Rust formatting and typo checks before commits, and
Clippy plus integration tests before pushes. Install
[`pre-commit`](https://pre-commit.com) and run these commands from the repository
root:

```bash
pre-commit install
pre-commit install --hook-type pre-push
```

The checks can also be run manually:

```bash
cargo fmt --all -- --check
cargo clippy --tests --lib -- -D warnings
cargo test -p integration-tests
pre-commit run --all-files
```

The `typos` hook uses a prebuilt executable from GitHub releases and reports
findings without modifying files. For details on contribution checks and the
broader test suites, see [CONTRIBUTING.md](../CONTRIBUTING.md).

# IDE and Tooling Integration

This guide sets up your editor and command-line tools so they flag the same problems CI flags, before you push. It covers VS Code (configured in-repo), IntelliJ IDEA / RustRover (optional), linting tools, and testing tools.

## Contents

1. [VS Code](#1-vs-code)
2. [IntelliJ IDEA / RustRover (optional)](#2-intellij-idea--rustrover-optional)
3. [Linting Tools](#3-linting-tools)
4. [Testing Tools](#4-testing-tools)
5. [Troubleshooting](#5-troubleshooting)
6. [External Resources](#6-external-resources)

---

## 1. VS Code

The repo ships a shared VS Code configuration in [`.vscode/`](../.vscode/). Open the repository root folder (not a single example) and VS Code will offer to install the recommended extensions.

### What is in `.vscode/`

| File | Purpose |
|---|---|
| [`extensions.json`](../.vscode/extensions.json) | Recommended extensions (see below) |
| [`settings.json`](../.vscode/settings.json) | rust-analyzer runs clippy with CI flags, format on save, typos config, coverage gutters, and `target/` exclusions |
| [`tasks.json`](../.vscode/tasks.json) | One-click tasks for tests, smoke tests, coverage, lint, typos, WASM build, and `mdbook serve` |
| [`launch.json`](../.vscode/launch.json) | CodeLLDB debug configs for smoke tests and for any integration test by name |
| [`soroban.code-snippets`](../.vscode/soroban.code-snippets) | Soroban snippets: contract skeleton, error enum, `DataKey`, admin guard, TTL, events, and tests |

Everything else under `.vscode/` is gitignored, so personal files such as launch configs for your own branches stay local.

### Recommended extensions

| Extension | Why |
|---|---|
| [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) | Completion, go-to-definition, inline errors, and test code lenses. Expands `#[contract]` / `#[contractimpl]` macros. |
| [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) | `Cargo.toml` validation and formatting |
| [Dependi](https://marketplace.visualstudio.com/items?itemName=fill-labs.dependi) | Shows outdated crate versions inline, which helps keep `soroban-sdk` in sync |
| [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) | Debugging tests with breakpoints |
| [Typos](https://marketplace.visualstudio.com/items?itemName=tekumara.typos-vscode) | The same spell checker CI runs, using [`_typos.toml`](../_typos.toml) |
| [Code Spell Checker](https://marketplace.visualstudio.com/items?itemName=streetsidesoftware.code-spell-checker) | Prose spelling in Markdown, using [`cspell.json`](../cspell.json) |
| [markdownlint](https://marketplace.visualstudio.com/items?itemName=davidanson.vscode-markdownlint) | Consistent Markdown in the book and docs |
| [Coverage Gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters) | Shows `cargo tarpaulin` line coverage in the editor gutter |

### Snippets

Type a prefix in any `.rs` file and press <kbd>Tab</kbd>:

| Prefix | Inserts |
|---|---|
| `soroban-contract` | `#![no_std]` contract with `#[contract]`, `#[contractimpl]`, and a test module |
| `soroban-error` | `#[contracterror]` enum with common variants |
| `soroban-datakey` | `#[contracttype]` storage key enum |
| `soroban-admin` | Loads the admin from instance storage and calls `require_auth()` |
| `soroban-ttl` | Instance `extend_ttl` call |
| `soroban-event` | `env.events().publish(...)` |
| `soroban-test` | Test with `mock_all_auths`, contract registration, and a client |
| `soroban-try` | `assert_eq!(client.try_x(..), Err(Ok(Error::Variant)))` |

### Tasks

Open them with <kbd>Ctrl/Cmd</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd> → **Tasks: Run Task**:

- `cookbook: test current example`: runs `cargo test` in the example that contains the open file. It assumes the file is in that example's `src/`. This is the default test task.
- `cookbook: smoke tests`: runs the fast critical-path suite in under 2 minutes. See [`tests/integration/SMOKE_TESTS.md`](../tests/integration/SMOKE_TESTS.md).
- `cookbook: integration tests` / `cookbook: workspace tests`
- `cookbook: coverage (tarpaulin)`: writes `coverage/lcov.info`. Run **Coverage Gutters: Display Coverage** to see it in the editor.
- `cookbook: lint (fmt + clippy)` / `cookbook: spelling (typos)`
- `cookbook: build contract (wasm)`: runs `stellar contract build` in the current example. This is the default build task.
- `cookbook: serve book`: runs `mdbook serve --open` for live documentation preview.

### Debugging

Contracts registered with `env.register_contract(None, Contract)` (or `env.register(Contract, ())`) run as native Rust in tests, so CodeLLDB breakpoints inside contract functions are hit. Use **Run and Debug** → *Debug smoke tests* or *Debug integration test by name*, or click the **Debug** code lens above any `#[test]`.

---

## 2. IntelliJ IDEA / RustRover (optional)

JetBrains users can use [RustRover](https://www.jetbrains.com/rust/), or IntelliJ IDEA Ultimate with the [Rust plugin](https://plugins.jetbrains.com/plugin/22407-rust). The repo does not commit `.idea/`, so configure the IDE once:

1. **Open the repository root** as a Cargo project. The IDE detects the workspace `Cargo.toml` and `rust-toolchain.toml`.
2. **Enable clippy on save:** *Settings → Rust → External Linters*. Set the tool to **Clippy**, tick *Run external linter on the fly*, and set additional arguments to `-- -D warnings -A deprecated`.
3. **Enable rustfmt:** *Settings → Rust → Rustfmt*. Tick *Use rustfmt instead of built-in formatter* and *Run rustfmt on Save*.
4. **Expand procedural macros:** *Settings → Languages & Frameworks → Rust*. Make sure *Expand declarative and procedural macros* is on, so the generated `ContractClient` types resolve.
5. **Exclude build output:** right-click `target/` and `book/book-output/` and choose *Mark Directory as → Excluded*.
6. **Run configurations:** click the gutter icon next to any `#[test]`, or add a *Cargo* configuration with the command `test -p integration-tests --test smoke_tests`.
7. **Spelling:** install the [Typos plugin](https://plugins.jetbrains.com/search?search=typos), or run `typos` from the terminal.
8. **Coverage:** *Run → Run with Coverage* works for native tests. CI uses tarpaulin, so treat the IDE's numbers as approximate.

To add the Soroban snippets, create *Live Templates* (*Settings → Editor → Live Templates → Rust*) using the bodies from [`.vscode/soroban.code-snippets`](../.vscode/soroban.code-snippets). Replace `${1:Name}` placeholders with `$NAME$`.

---

## 3. Linting Tools

These are the tools CI runs (see [`.github/workflows/test.yml`](../.github/workflows/test.yml)). Run them locally before you push.

| Tool | Command | Config | Install |
|---|---|---|---|
| **rustfmt** | `cargo fmt --all -- --check` | default | `rustup component add rustfmt` |
| **clippy** | `cargo clippy --tests --lib -- -D warnings -A deprecated` | [`.clippy.toml`](../.clippy.toml) | `rustup component add clippy` |
| **typos** | `typos` | [`_typos.toml`](../_typos.toml) | `cargo install typos-cli` |
| **cspell** (webapp/docs prose) | `cd webapp && bun run spellcheck` | [`cspell.json`](../cspell.json) | `bun install` in `webapp/` |
| **cargo-audit** | `cargo audit` | — | `cargo install cargo-audit` |
| **Combined script** | `./scripts/lint.sh [example-path]` | — | — |

Tips:

- `./scripts/lint.sh -c examples/basics/01-hello-world` runs clippy on a single example, which is much faster than the whole workspace.
- `cargo clippy --fix --allow-dirty` applies the fixes clippy can make automatically. Review the diff before you commit.
- `.clippy.toml` sets complexity, argument-count, and function-length thresholds. If clippy flags one, split the function rather than adding `#[allow]`.

---

## 4. Testing Tools

| Tool | Command | Use |
|---|---|---|
| **cargo test** | `cargo test` in an example, or `cargo test --workspace` | Unit and integration tests |
| **Smoke tests** | `cargo test -p integration-tests --test smoke_tests` or `./scripts/smoke-test.sh` | Fast critical-path check before merging |
| **Integration suite** | `cargo test -p integration-tests` | Cross-contract scenarios ([`tests/integration/`](../tests/integration/)) |
| **Security suite** | `cargo test --package security-tests` | Auth bypass, reentrancy, and other regressions |
| **cargo-nextest** (optional) | `cargo nextest run -p integration-tests` | Faster parallel runner with clearer failure output. Install with `cargo install cargo-nextest --locked`. |
| **proptest** | runs as part of `cargo test` | Property-based tests in `tests/integration/tests/*fuzz*` |
| **cargo-fuzz** | see [`docs/fuzz-testing.md`](./fuzz-testing.md) | Coverage-guided fuzzing (`tests/fuzz/`) |
| **cargo-tarpaulin** | `cargo tarpaulin` | Coverage. Reports go to `coverage/` ([`tarpaulin.toml`](../tarpaulin.toml)). |
| **Test snapshots** | written automatically to `test_snapshots/` (gitignored) | Soroban records the ledger state and events for each test. Diff them locally before and after a change to spot unexpected state changes. |
| **Stellar CLI** | `stellar contract build`, `stellar contract invoke` | Build WASM and run against local, testnet, or mainnet. See [Deployment Best Practices](./deployment-best-practices.md). |

For patterns and fixtures, see the [Testing Guide](./testing-guide.md) and [Testing Best Practices](./testing-best-practices.md).

---

## 5. Troubleshooting

| Symptom | Fix |
|---|---|
| rust-analyzer shows `unresolved import` for `*Client` types | Make sure procedural macros are enabled. Then run *rust-analyzer: Restart server*. |
| `cargo build` waits on "Blocking waiting for file lock" | `rust-analyzer.cargo.targetDir` is already `true` in the shared settings. If you overrode it, set it back. |
| `Address::generate` not found in tests | Import `soroban_sdk::testutils::Address as _`, and check that the example's `[dev-dependencies]` enables `soroban-sdk` with `features = ["testutils"]`. |
| clippy passes locally but fails in CI | Run it with `-D warnings -A deprecated`. CI also sets `RUSTFLAGS` with these flags. |
| Typos flags a valid word | Add it to `[default.extend-words]` in `_typos.toml`. |
| The editor is slow on the whole workspace | Open a single example folder for focused work. `cargo test` still works there through the workspace. |

---

## 6. External Resources

- [rust-analyzer manual](https://rust-analyzer.github.io/manual.html)
- [RustRover documentation](https://www.jetbrains.com/help/rust/)
- [Stellar CLI reference](https://developers.stellar.org/docs/tools/developer-tools/cli)
- [Soroban SDK docs](https://docs.rs/soroban-sdk)
- [Clippy lint list](https://rust-lang.github.io/rust-clippy/master/)
- [typos](https://github.com/crate-ci/typos) · [cargo-nextest](https://nexte.st) · [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)

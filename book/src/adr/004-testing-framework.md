# 4. Testing Framework

**Date:** 2026-10-19

## Status

**Status:** Accepted

## Context

The Soroban Cookbook contains Rust smart-contract examples that run on the Soroban VM. Testing smart contracts presents unique requirements compared with ordinary Rust programs:

- Contracts execute inside a sandboxed host environment; unit tests must simulate that environment
- Cross-contract calls require orchestrating multiple contract instances within a single test
- Gas and resource accounting must be verifiable
- Security-focused tests (reentrancy, authorization bypass) need a dedicated home
- Fuzz testing is useful for serialization and arithmetic boundaries

The project evaluated three testing layers:

1. **Native Rust unit tests** — fast, in-process, no harness required
2. **Soroban test utilities** (`soroban_sdk::testutils`) — in-process Soroban environment simulation
3. **Standalone integration harness** — separate workspace member with shared fixtures

We also evaluated external tools:

- `cargo-nextest` — parallel test runner; drop-in replacement for `cargo test`
- `cargo-tarpaulin` — coverage measurement for CI
- `cargo-fuzz` — libFuzzer-based fuzz testing

## Decision

Use a **four-layer testing strategy** implemented entirely with standard Cargo tooling:

### Layer 1 — Example unit tests (`src/test.rs`)

Every example crate includes in-module tests using the `soroban_sdk::testutils` environment:

```rust
#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;
    // ...
}
```

These tests cover the happy path and basic error cases for a single contract.

### Layer 2 — Integration tests (`tests/integration/`)

A dedicated workspace member (`tests/integration`) holds cross-contract and multi-step workflow tests. It has path dependencies on the example crates it orchestrates, ensuring the tests always build against the current source.

### Layer 3 — Security tests (`tests/security/`)

A separate workspace member isolates authorization-bypass, reentrancy, and access-control regressions. Keeping them separate prevents them from inflating example-level test counts and makes threat-model coverage easy to audit.

### Layer 4 — Fuzz targets (`tests/fuzz/`)

`cargo-fuzz` targets cover serialization and arithmetic boundaries. These run in CI on a schedule (not on every PR) due to their long execution time.

### Tooling decisions

| Tool | Decision | Reason |
|------|----------|--------|
| `cargo test` | Default test runner | No extra installation, works everywhere |
| `soroban_sdk::testutils` | In-process host simulation | Official SDK-provided harness, no external process needed |
| `cargo-tarpaulin` | Coverage measurement | Works with `wasm32` exclusions, uploads to Codecov |
| `cargo-fuzz` | Fuzz testing | libFuzzer-backed; standard Rust fuzz tool |

`cargo-nextest` is **not** adopted as a required tool because it adds a non-trivial installation step. Contributors may use it locally, but CI relies on `cargo test`.

### Coverage target

Maintain **≥ 90%** line coverage across all example crates, measured by `cargo-tarpaulin` in CI.

## Consequences

### Positive

- Zero additional build-tool installation required beyond a standard Rust toolchain
- Tests run identically on Windows, macOS, and Linux (cross-platform CI)
- Separation of concerns: unit, integration, security, and fuzz tests do not interfere
- Coverage gate in CI prevents regressions from reaching `main`
- `soroban_sdk::testutils` provides a deterministic, reproducible host environment

### Negative

- Tests do not run against a live Soroban network; on-chain edge cases require manual testing
- The four-crate layout adds navigation overhead for new contributors (mitigated by README files in each directory)
- `cargo-tarpaulin` does not support WASM targets; WASM tests are excluded from coverage counts

### Neutral

- `cargo-nextest` can be used locally to speed up test runs without affecting CI parity
- Fuzz targets accumulate coverage corpora over time; those corpora are `.gitignore`-d to keep repo size bounded

## References

- [soroban-sdk testutils documentation](https://docs.rs/soroban-sdk/latest/soroban_sdk/testutils/index.html)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)
- Integration test README: [`tests/integration/README.md`](../../../tests/integration/README.md)
- Security test README: [`tests/security/README.md`](../../../tests/security/README.md)

## Notes

When adding a new example:

1. Add unit tests in `src/test.rs` for single-contract behavior
2. If the example interacts with other contracts, add a test in `tests/integration/`
3. If the example handles authorization or untrusted input, add a case in `tests/security/`
4. Run `cargo tarpaulin --package <crate-name>` locally to verify coverage before opening a PR

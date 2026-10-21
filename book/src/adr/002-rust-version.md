# 2. Rust Version

**Date:** 2026-10-19

## Status

**Status:** Accepted

## Context

The Soroban Cookbook workspace compiles a large set of smart-contract examples and tests against the Soroban SDK. Rust versions affect:

- WASM output compatibility with the Soroban host
- Available language features in example code
- Compiler diagnostics and Clippy lint availability
- Reproducibility of CI builds across contributor machines

We need a single, pinned Rust version so that:

- Every contributor produces the same build artefacts
- CI is deterministic and not affected by nightly regressions
- The WASM target (`wasm32-unknown-unknown`) is always present

Several constraints apply:

- The Soroban SDK specifies a minimum supported Rust version (MSRV) of **1.74** as of SDK 29.x
- Some example patterns require `const` generics and `let`-chains that need ≥ 1.74
- The project wants a recent stable compiler but not `nightly` (which is unstable)
- All contributors must be able to reproduce builds by running `rustup show`

## Decision

Pin Rust to **1.96.0 (stable)** using a `rust-toolchain.toml` at the workspace root:

```toml
[toolchain]
channel = "1.96.0"
targets = ["wasm32-unknown-unknown"]
components = ["rust-src"]
```

**Why 1.96.0:**

- Well above the SDK's MSRV of 1.74
- Latest stable release tested against Soroban SDK 29.x at the time of this ADR
- Ships with improved const-evaluation and Clippy lints used in the codebase
- Supported by all major CI runners (ubuntu-latest, windows-latest, macos-latest)

**What is pinned:**

- Compiler version: `1.96.0`
- Target: `wasm32-unknown-unknown` (required for Soroban contract compilation)
- Component: `rust-src` (required for `-Z build-std` in WASM profiles)

**Update policy:**

- Pin is updated when the Soroban SDK releases a new MSRV requirement or when a new stable fixes a reproducibility or security issue
- Updates are tracked as new ADRs that supersede this one
- Dependabot monitors `rust-toolchain.toml` via the `rust` ecosystem entry in `.github/dependabot.yml`

## Consequences

### Positive

- Fully reproducible builds across all contributors and CI environments
- `rustup` automatically installs the correct toolchain when contributors clone the repo
- No "it works on my machine" issues caused by compiler version drift
- Clippy and `rustfmt` versions are also pinned, keeping style checks consistent

### Negative

- Contributors must update their toolchain when the pin is bumped
- We occasionally miss language features available in newer stable releases until the pin is updated
- A pinned old toolchain may mask future deprecation warnings earlier than necessary

### Neutral

- The `rust-toolchain.toml` override is respected by Cargo, rust-analyzer, and most editors without any extra configuration
- Nightly features are out of scope; if a future example requires them, a separate ADR must justify the change

## References

- [rust-toolchain.toml specification](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file)
- [Soroban SDK 29.x changelog](https://github.com/stellar/rs-soroban-sdk/releases)
- Current pin: [`rust-toolchain.toml`](../../rust-toolchain.toml)

## Notes

Verify compatibility after each SDK major version upgrade by running:

```bash
rustup update 1.96.0
cargo test --workspace
cargo build --workspace --target wasm32-unknown-unknown --release
```

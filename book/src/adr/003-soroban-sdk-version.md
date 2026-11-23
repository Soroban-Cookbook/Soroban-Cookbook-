# 3. Soroban SDK Version

**Date:** 2026-10-19

## Status

**Status:** Accepted

## Context

Every smart-contract example in the Soroban Cookbook depends on the `soroban-sdk` crate. The SDK:

- Defines the core types (`Env`, `Address`, `Map`, `Vec`, …) used in every contract
- Determines which host functions are available and their signatures
- Has a release cadence tied to the Stellar Protocol upgrade schedule
- Can introduce breaking API changes between major versions

We need a workspace-level SDK version policy so that:

- All examples compile against the same SDK
- Contributors are not surprised by version mismatches
- Security patches and protocol changes are adopted quickly

Two broad approaches were considered:

1. **Per-crate pinning** — each example `Cargo.toml` specifies its own version range
2. **Workspace-level pinning** — a single `[workspace.dependencies]` entry is the source of truth

## Decision

Pin `soroban-sdk` at **29.0.0** as a workspace dependency:

```toml
# Cargo.toml (workspace root)
[workspace.dependencies]
soroban-sdk = "29.0.0"
```

Each example crate inherits the version with:

```toml
# examples/<category>/<name>/Cargo.toml
[dependencies]
soroban-sdk = { workspace = true }
```

**Why 29.0.0:**

- Production-stable release aligned with Stellar Protocol 21
- Includes the `soroban_sdk::token` interface required by the DeFi and token examples
- SEP-41 token interface is stable in this version
- Compatible with Rust 1.96.0 (see ADR-002)

**Update policy:**

- The workspace pin is bumped in a dedicated PR that also updates all example code for breaking API changes
- RC versions (`-rc.N`) may be used in a `next` branch to test upcoming SDK changes before they land in `main`
- Major version bumps require a new ADR superseding this one
- Minor/patch bumps update this ADR's version table in-place (no new ADR required)

**Version history:**

| SDK Version | Date Adopted | Notes |
|-------------|--------------|-------|
| 29.0.0      | 2026-10-19   | Initial ADR; production stable, Protocol 21 |

## Consequences

### Positive

- One place to update when the SDK changes; all examples stay in sync automatically
- CI catches any example that fails to compile after an SDK bump
- Contributors never need to hunt for the "correct" SDK version for a new example

### Negative

- All examples must be updated simultaneously on a major version bump — a larger but bounded one-time cost
- RC releases in the workspace affect every example, so they are kept to the `next` branch
- Examples temporarily broken by an SDK breaking change block merges until fixed

### Neutral

- The `soroban-sdk` crate bundles `soroban-env-host` as an internal dependency; host version mismatches are therefore invisible to example authors
- Workspace inheritance does not prevent individual crates from overriding the version if truly needed (they should document the reason)

## References

- [soroban-sdk crates.io page](https://crates.io/crates/soroban-sdk)
- [soroban-sdk GitHub releases](https://github.com/stellar/rs-soroban-sdk/releases)
- Current workspace pin: [`Cargo.toml`](../../Cargo.toml)

## Notes

When bumping the SDK version:

1. Update `[workspace.dependencies]` in the root `Cargo.toml`
2. Run `cargo test --workspace` and fix any compilation errors
3. Run `cargo build --workspace --target wasm32-unknown-unknown --release`
4. Update the version history table in this ADR
5. Open a PR titled `chore: bump soroban-sdk to X.Y.Z`

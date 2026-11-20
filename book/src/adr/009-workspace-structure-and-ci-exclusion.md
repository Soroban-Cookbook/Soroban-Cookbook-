# 9. Workspace Structure and CI Exclusion Mechanism

**Date:** 2026-11-20

## Status

**Status:** Accepted

## Context

The cookbook holds many independent example contracts. They need to build and test together so that SDK upgrades and shared code changes are verified everywhere, but a few crates cannot always build with the rest:

- Learner templates (`templates/dao-governance-dapp`, `templates/token-dapp`) are standalone Cargo workspaces meant to be copied out, and a package cannot belong to two workspaces
- During an SDK migration, some examples may temporarily fail to compile
- Host-only crates (integration tests, fuzz harness) cannot be built for `wasm32v1-none`

Alternatives considered:

- **Independent crates, no workspace** — every example pins its own dependencies; SDK versions drift and CI must loop over crates
- **One workspace with every crate always required** — a single broken example blocks all CI

## Decision

Use a single root Cargo workspace (`Cargo.toml`) that lists each example under `members` and pins shared dependencies such as `soroban-sdk` in `[workspace.dependencies]`.

- Standalone template workspaces are listed under `exclude` in the root `Cargo.toml`, with a comment explaining why
- CI runs cargo through `scripts/ci-workspace.sh`, which reads `scripts/ci-exclude-packages.txt` and passes `--exclude <pkg>` for each entry; crates that are temporarily broken go in that file and are removed once fixed
- The same exclude list is applied to the tarpaulin coverage job
- `ci-workspace.sh build` also excludes host-only crates (`integration-tests`, `security-tests`, `soroban-cookbook-fuzz`) for the wasm build

## Consequences

### Positive

- One `cargo test --workspace` verifies every example against the same SDK
- A broken crate can be parked without blocking unrelated PRs
- Exclusions are explicit, version-controlled and reviewable

### Negative

- New examples must be added to `members` or they are silently not built
- Excluded crates can rot if entries are not removed after they are fixed

### Neutral

- Templates are built by their own CI workflow (`templates.yml`), outside the root workspace

## References

- Root `Cargo.toml` (`members` / `exclude`)
- `scripts/ci-workspace.sh` and `scripts/ci-exclude-packages.txt`
- `.github/workflows/test.yml`

## Notes

Keep `scripts/ci-exclude-packages.txt` free of package entries except while a migration or fix is in progress.

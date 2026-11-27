# Soroban Cookbook — Fuzz Targets

This crate contains [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) targets
for the Soroban Cookbook examples. It is excluded from the root workspace so that
`cargo build` and `cargo test --workspace` never try to compile it for a non-fuzz
host (the `libfuzzer-sys` crate is only available on certain platforms).

## Available fuzz targets

| Target | Contract |
|--------|----------|
| `example_fuzz` | `examples/basics/01-hello-world` |
| `advanced_claimable_balance` | `examples/advanced/09-fuzz-testing` |
| `advanced_timelock` | `examples/advanced/02-timelock` |
| `advanced_multi_party_auth` | `examples/advanced/01-multi-party-auth` |

## Running a target

```bash
# Install cargo-fuzz once
cargo install cargo-fuzz

# Run a target from this directory (not the workspace root)
cd tests/fuzz
cargo fuzz run example_fuzz
cargo fuzz run advanced_timelock -- -max_total_time=60
```

## SDK version synchronisation (#1485)

`tests/fuzz/Cargo.toml` declares:

```toml
soroban-sdk = { workspace = true, features = ["testutils"] }
```

Because `workspace = true` delegates version resolution to the root
`Cargo.toml`, the fuzz targets always use the **same `soroban-sdk` version**
as every other crate in the cookbook. There is no separate pinned version.

### How to keep it in sync

No manual steps are needed as long as `soroban-sdk` remains a workspace
dependency. If you ever need to override the version for local testing:

1. Add a `[patch.crates-io]` block to the **root** `Cargo.toml` — do **not**
   add it only to `tests/fuzz/Cargo.toml`.
2. After updating `workspace.dependencies.soroban-sdk` in the root, run
   `cargo update -p soroban-sdk` from the workspace root to refresh the lock
   file. The fuzz crate picks up the change automatically.

### CI version-divergence check

The CI workflow (`scripts/check-fuzz-sdk-version.sh`) fails the build if
`tests/fuzz/Cargo.toml` ever diverges from the workspace root by pinning its
own `soroban-sdk` version. The check is intentionally simple:

```bash
# If the fuzz Cargo.toml declares a soroban-sdk version literal (not
# "workspace = true"), the check fails and prints the offending line.
grep -n 'soroban-sdk' tests/fuzz/Cargo.toml | grep -v 'workspace = true' && exit 1 || exit 0
```

This ensures fuzz-discovered bugs are always reproduced against the same SDK
version that the rest of the cookbook ships with.

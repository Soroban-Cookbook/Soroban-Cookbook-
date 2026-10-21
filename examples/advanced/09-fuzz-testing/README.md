# Fuzz Testing for Advanced Examples

Shows how to fuzz-test Soroban contracts using `cargo-fuzz` and reusable `proptest` property tests. The contract under test is a claimable-balance / timelock pattern — the same shape as `02-timelock`, structured so every deposit/claim path is reachable with arbitrary inputs.

## What It Demonstrates

- A deposit/claim contract with time bounds and multi-claimant auth
- Invariant checks that must hold for *any* input (state machine consistency, token conservation)
- Property tests that run in CI without nightly
- A dedicated fuzz target under `tests/fuzz/` for continuous exploration

## Run Unit + Property Tests

```bash
cargo test -p fuzz-testing
```

## Run Continuous Fuzzing

Requires nightly Rust and [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz):

```bash
cargo install cargo-fuzz
cargo +nightly fuzz run --fuzz-dir tests/fuzz advanced_claimable_balance
```

Also available for other advanced crates:

```bash
cargo +nightly fuzz run --fuzz-dir tests/fuzz advanced_timelock
cargo +nightly fuzz run --fuzz-dir tests/fuzz advanced_multi_party_auth
```

See [`guides/fuzz-testing.md`](../../../guides/fuzz-testing.md) for setup details.

## Invariants Checked

After every deposit or claim attempt:

1. Init flag and balance entry are a valid pair (never balance-without-init)
2. Contract token balance is non-negative
3. If a claimable balance exists, its amount equals tokens held by the contract
4. Remaining claimable amount never exceeds the original deposit

## Related Examples

- [`02-timelock`](../02-timelock/) — production-style delayed execution
- [`01-multi-party-auth`](../01-multi-party-auth/) — multi-signer authorization vectors

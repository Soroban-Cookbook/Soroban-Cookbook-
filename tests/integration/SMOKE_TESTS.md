# Smoke Tests

Smoke tests are quick checks on the cookbook's critical paths. They run before the full test suite so a broken core example fails CI in minutes, before the slower integration, fuzz, and coverage jobs start.

- **Location:** [`tests/smoke_tests.rs`](./tests/smoke_tests.rs)
- **Run time:** a few seconds once compiled. The budget is **2 minutes**.
- **CI job:** `Smoke Tests` in [`.github/workflows/test.yml`](../../.github/workflows/test.yml). The `Test Suite` job has `needs: smoke`, so it only starts once the smoke tests pass.

## Running

```bash
# Run only the smoke tests
cargo test -p integration-tests --test smoke_tests

# Same run, but fail if execution goes over the 2-minute budget
./scripts/smoke-test.sh

# Use a different budget, in seconds
./scripts/smoke-test.sh 60
```

Run these before you push. They catch most breakages caused by SDK bumps, shared-crate changes, and example API changes.

## What Is Covered

| Test | Example | Happy path | Failure path checked |
|---|---|---|---|
| `smoke_hello_world_responds` | `basics/01-hello-world` | `hello` returns the greeting | — |
| `smoke_storage_round_trip` | `basics/02-storage-patterns` | persistent set → has → get | missing key reports absent |
| `smoke_auth_transfer_and_custom_errors` | `basics/03-authentication`, `basics/03-custom-errors` | transfer moves balances | zero-amount transfer and invalid input are rejected |
| `smoke_sep41_token_core_flow` | `tokens/01-sep41-token` | mint, transfer, approve, transfer_from | spending past the allowance fails |
| `smoke_mint_burn_respects_cap` | `tokens/04-mint-burn` | mint up to the cap | minting past the cap fails |
| `smoke_pause_guard` | `intermediate/03-pause-unpause` | increment, pause, unpause | writes fail while paused; reads still work |
| `smoke_simple_voting_lifecycle` | `governance/01-simple-voting` | create, vote, tally, execute | — |
| `smoke_amm_liquidity_and_swap` | `defi/02-constant-product-amm` | add liquidity, swap, reserves update | slippage guard rejects the swap |

These cover the main categories: basics, storage, auth and errors, tokens, access control, governance, and DeFi.

## Pre-merge Validation

1. Run `./scripts/smoke-test.sh` locally.
2. Open the PR. The `Smoke Tests` job runs first.
3. When it passes, `Test Suite` runs the full integration, security, and gas-reporter suites.

## Adding a Smoke Test

Add a smoke test when a new example becomes part of a critical path, such as a new core token standard or a contract that other examples depend on. Everything else belongs in the regular integration suite.

Rules:

- **Keep it fast.** No proptest, no benchmarks, no loops over large inputs, no `Instant` timing.
- **Use one contract per test**, or one tight cross-contract pair (for example, AMM + Stellar asset tokens).
- **Check the happy path plus the single most important guard rail.**
- **Register contracts natively** with `env.register_contract(None, Contract)`. Never build WASM.
- **Add the example as a path dependency** in [`Cargo.toml`](./Cargo.toml) if it is not there already.
- **Name the test `smoke_<area>_<behavior>`** and add a row to the table above.

If the suite starts getting close to the 2-minute budget, move the slowest test to the regular integration suite. Do not raise the budget.

# Fuzz Testing Report

Status of fuzz and property-based testing in the Soroban Cookbook: what is covered, what the runs have found, and how it is wired into CI.

For *how to write and run* fuzz tests, see the [Fuzz Testing Guide](../guides/fuzz-testing.md). This document is the report; that one is the manual.

---

## 1. What Is Fuzzed

Three layers, in increasing order of cost:

| Layer | Where | Harness | Runs on |
|-------|-------|---------|---------|
| Boundary / adversarial tests | `tests/integration/tests/` | plain `#[test]`, hand-picked edge values | every PR, stable |
| Property tests | `tests/integration/tests/defi_fuzz_tests.rs`, `examples/advanced/09-fuzz-testing/src/proptest.rs` | `proptest`, randomized inputs per run | every PR, stable |
| Coverage-guided fuzzing | `tests/fuzz/fuzz_targets/` | `cargo-fuzz` / libFuzzer, nightly | every PR, 30 s per target |

The split matters: property tests are the ones that gate merges, because they need no nightly toolchain and finish in under a minute. The libFuzzer targets explore the same surfaces far more aggressively, but only for as long as CI is willing to wait.

### Test inventory

| Suite | Tests | Kind | Contracts exercised |
|-------|------:|------|---------------------|
| `tests/integration/tests/fuzz_tests.rs` | 14 | boundary values | `storage-patterns`, `authentication`, `soroban-error-handling-example`, `custom-errors`, `events` |
| `tests/integration/tests/access_control_fuzz.rs` | 24 | adversarial | `authentication`, `role-based-access-control`, `rbac-modifiers`, `pause-unpause`, `registry-access-controls`, `proxy-admin`, `multi-sig-patterns`, `timelock`, `multi-party-auth` |
| `tests/integration/tests/defi_fuzz_tests.rs` | 32 (29 properties + 3 regression) | property + boundary | `constant-product-amm`, `lending-pool`, `collateralized-lending` |
| `examples/advanced/09-fuzz-testing` | 5 (4 unit + 1 property) | property | `fuzz-testing` (claimable balance) |
| `tests/fuzz/fuzz_targets/` | 4 targets | coverage-guided | `hello-world`, `fuzz-testing`, `timelock`, `multi-party-auth` |

**Generated cases per CI run.** `proptest` defaults to 256 cases per property, and the claimable-balance property pins 64:

- `defi_fuzz_tests.rs` — 29 properties × 256 = **7,424 generated cases**
- `09-fuzz-testing` — 1 property × 64 = **64 generated cases**
- 4 libFuzzer targets × 30 s = **~2 minutes of coverage-guided exploration**

Deterministic tests (14 + 24 + 3 + 4 = **45**) run the same inputs every time and are the regression floor.

---

## 2. Coverage Metrics

### Volume

| Metric | Value |
|--------|------:|
| Deterministic tests (boundary + adversarial + regression) | 45 |
| `proptest` properties | 30 |
| Generated cases per CI run | 7,488 |
| libFuzzer targets | 4 |
| Coverage-guided exploration per CI run | ~2 min |
| Contract crates reached by the fuzz layer | 18 |

`proptest` defaults to 256 cases per property; the claimable-balance property pins 64. So 29 × 256 + 64 = 7,488 generated cases every run, on top of the 45 fixed inputs that form the regression floor.

### Line coverage — not currently measurable for this layer

Line coverage for the fuzz layer alone could not be produced. Two `cargo-tarpaulin` runs, both green, reported only 30 instrumented lines:

```
|| shared/src/lib.rs: 2/25
|| tests/integration/tests/helpers/mod.rs: 0/5
||
6.67% coverage, 2/30 lines covered
```

The 32 `defi_fuzz_tests` and 24 `access_control_fuzz` tests ran and passed inside that measurement, so the tests execute plenty of contract code — tarpaulin simply did not attribute any of it. The example contracts enter `integration-tests` as path dependencies, and under this invocation their sources were not instrumented. Naming all 16 contract packages explicitly with `-p` changed nothing:

```bash
cargo tarpaulin --ignore-config \
  -p integration-tests -p constant-product-amm -p lending-pool -p collateralized-lending ... \
  --test fuzz_tests --test access_control_fuzz --test defi_fuzz_tests \
  --out Html --output-dir coverage
```

Both runs passed `--skip-clean`, so this is an observation about that invocation, not a proven limitation of the tool — a clean instrumented rebuild has not yet been completed against this workspace. Either way, **the number the repository would publish today for fuzz-layer line coverage is not meaningful**, which is why none is quoted here. See F-4.

Whole-workspace line coverage is a separate, working pipeline: the `coverage` job in [`test.yml`](../.github/workflows/test.yml) runs tarpaulin over the workspace per [`tarpaulin.toml`](../tarpaulin.toml) and uploads to Codecov ([`codecov.yml`](../codecov.yml)). That number covers all tests together and cannot be read as a fuzzing metric.

### Entry-point coverage

Because line coverage was unavailable, reach is reported as entry points: which contract functions the fuzz layer actually calls. The four libFuzzer targets are enumerated exactly below — their surfaces are small enough to read off the source.

An equivalent per-contract table for the 16 crates driven by the integration suites was attempted and is **not** included: deriving it needs call-site type resolution (clients arrive through tuple-returning helpers, struct fixtures, and `env.invoke_contract` name dispatch), and three successive heuristics produced three different answers — 58%, 31%, and 39% overall. None was trustworthy enough to publish. Doing this properly means either fixing the tarpaulin invocation or generating the table from `cargo`'s own analysis rather than from regexes.

| Target | Contract | Entry points driven by fuzzed input | Invariants asserted |
|--------|----------|-------------------------------------|---------------------|
| `example_fuzz` | `hello-world` | `hello` | Output shape for arbitrary UTF-8 symbols |
| `advanced_claimable_balance` | `fuzz-testing` | `deposit`, `claim` | Init flag and balance entry are a valid pair; contract token balance non-negative; claimable amount equals tokens held; remaining ≤ original deposit |
| `advanced_timelock` | `timelock` | `queue`, `cancel`, `execute` | Reachable states only; a queued operation implies the delay was within bounds |
| `advanced_multi_party_auth` | `multi-party-auth` | `encode_auth_vec`, `decode_auth_vec`, `validate_auth_vec`, `auth_vec_len`, `auth_vec_contains` | Encode/decode round-trips; length agrees with the decoded vector; membership holds |

### Known gaps

- **No fuzzing of the token, NFT, or governance categories.** Property tests stop at DeFi; access-control coverage is adversarial unit tests, not generated input.
- **No stateful sequence fuzzing.** Every target drives one or two calls. Multi-step sequences — deposit, partial claim, re-deposit, claim — are only covered by the deterministic tests.
- **30 seconds per target is a smoke test, not a campaign.** It is enough to catch a regression that fails immediately; it is not enough to find a deep bug. A longer scheduled run is the obvious next step.
- **No trustworthy line-coverage number for this layer.** See F-4; reach is reported as entry points instead.
- **Cross-contract fuzzing is shallow.** `defi_fuzz_tests.rs` composes AMM and lending contracts with a test token, but the oracle, bridge, and proxy examples are not driven by generated input.

---

## 3. Issues Found and Fixed

### F-1 — Dust swaps failed the AMM k-invariant properties

**Found by:** `fuzz_amm_swap_increases_k` and `fuzz_amm_swap_output_less_than_reserve`, shrunk to `sell_amount = 1`.
Recorded in [`defi_fuzz_tests.proptest-regressions`](../tests/integration/tests/defi_fuzz_tests.proptest-regressions).

**Cause:** the properties generated swap sizes from `1` and asserted every swap would succeed and return a positive output. At 50,000/50,000 reserves, `constant-product-amm` refuses dust:

- `sell_amount = 1` → `apply_fee` computes `1 × 997 / 1000 = 0` → `AmmError::InvalidAmount`
- `sell_amount = 2` → fee-adjusted input is `1`, and `1 × 50_000 / 50_001 = 0` → `AmmError::InsufficientOutputAmount`

Both rejections are correct contract behaviour. The defect was in the property's assumption, not in the AMM.

**Fixed:** commit `a02654e` raised the generators' lower bound to `100`.

**Follow-up (this report):** narrowing the range removed the boundary from coverage entirely — the contract's dust rejection was no longer asserted anywhere. Three deterministic regression tests now pin it down:

- `amm_swap_below_the_fee_floor_is_rejected`
- `amm_swap_yielding_zero_output_is_rejected`
- `amm_dust_rejection_leaves_reserves_untouched`

### F-2 — Liquidation properties failed on thin collateral

**Found by:** the liquidation properties, shrunk to `collateral = 1000, repay = 50` and `collateral = 500, repay = 50`.
Recorded in the same regressions file.

**Cause:** positions built from small collateral could not be made reliably underwater by the shared `make_underwater_borrower` helper, so `liquidate` rejected a position the property assumed was liquidatable.

**Fixed:** the committed generators start at `collateral ≥ 2_000` and `repay ≥ 100`, which keeps every generated position inside the regime the property describes.

**Residual risk:** the same class of gap as F-1 — the thin-collateral regime is now outside the generators. Unlike the AMM dust boundary, the correct behaviour there is not yet pinned by a deterministic test. Tracked under "Known gaps" above.

### F-3 — Fuzz targets were not reachable from CI

**Found by:** auditing this report against the workflow.

**Cause:** `fuzz.yml` ran only `example_fuzz`, a hello-world smoke target. The three targets that exercise real contract logic — claimable balance, timelock, multi-party auth — existed in `tests/fuzz/` and were documented in the guide, but no CI job ever ran them.

The property suites were in a weaker position than they looked. `test.yml` runs `cargo test -p integration-tests` on every PR, so the three integration fuzz suites did execute — but inside a job named "Test Suite", with nothing identifying them as the fuzzing layer or reporting them separately. The `fuzz-testing` example's property test was weaker still: it ran only on pushes to `main`, through the workspace fallback step, or on a PR that happened to touch that example's directory.

**Fixed:** `fuzz.yml` now runs all four targets as a matrix, and a stable-toolchain `property-tests` job runs the property and boundary suites by name on every PR — including `cargo test -p fuzz-testing`, which no PR-triggered job reliably covered before. The overlap with "Test Suite" is deliberate: it makes the fuzzing layer a named, independently readable signal. See §4.

### F-4 — Fuzz-layer line coverage is not measurable with the current tooling

**Found by:** attempting to produce §2 of this report.

**Cause:** `cargo-tarpaulin` scoped to the three fuzz suites attributes coverage only to `shared/src/lib.rs` and a test helper — 2 of 30 lines — while the suites themselves run 70 tests to completion against 16 contract crates. The contract crates arrive as path dependencies of `integration-tests`, and their sources were not instrumented under the invocations tried. Listing every contract package explicitly with `-p` did not change the result.

**Not fixed.** Both attempts used `--skip-clean`; a clean instrumented rebuild is the obvious next thing to try, and `--engine llvm` after that. Until one of those works, the repository has no trustworthy line-coverage number for the fuzz layer, and this report quotes none rather than quoting the misleading 6.67%.

**Why it matters beyond this document:** the same shape of invocation underlies the workspace `coverage` job. If path-dependency sources are being dropped there too, the Codecov number is measuring less than it appears to. Worth confirming separately.

### Non-findings worth recording

No fuzz run to date has found a defect in contract code. Every counterexample so far has been a wrong assumption in a test. That is a normal early result — it says the properties are still being calibrated — but it also means the fuzzing layer has not yet earned the claim that it protects the contracts. Longer campaigns and stateful sequences are what would change that.

---

## 4. CI Integration

[`.github/workflows/fuzz.yml`](../.github/workflows/fuzz.yml) has two jobs, both on push to `main` and on every PR:

**`property-tests`** — stable toolchain, no extra install:

```bash
cargo test -p fuzz-testing
cargo test -p integration-tests \
  --test fuzz_tests \
  --test access_control_fuzz \
  --test defi_fuzz_tests
```

**`fuzz`** — nightly toolchain, one matrix leg per target, `fail-fast: false` so one failing target does not hide the others:

```bash
cargo +nightly fuzz run --fuzz-dir tests/fuzz <target> -- -max_total_time=30
```

A failing leg uploads `tests/fuzz/artifacts/` as `fuzz-artifacts-<target>`, so the crashing input can be downloaded and replayed locally:

```bash
cargo +nightly fuzz run --fuzz-dir tests/fuzz <target> <artifact-file>
```

Counterexamples found by `proptest` are written to `*.proptest-regressions` next to the test file and **are committed**. They re-run before any new cases are generated, so a fixed bug stays fixed.

Related jobs:

- [`test.yml`](../.github/workflows/test.yml) → `coverage` — whole-workspace `cargo-tarpaulin`, uploaded to Codecov ([`tarpaulin.toml`](../tarpaulin.toml), [`codecov.yml`](../codecov.yml))
- [`security-audit.yml`](../.github/workflows/security-audit.yml) — `cargo audit` for dependency advisories

---

## 5. Test Scenarios

### Boundary values — `fuzz_tests.rs`

| Area | Scenarios |
|------|-----------|
| Storage | `u64::MAX` and `0` in persistent and temporary storage; overwrite three times; missing key returns `None` |
| Authorization | Transfer of an entire balance; many small sequential transfers |
| Error handling | Minimum deposit; exact withdraw boundary; consecutive deposits accumulate; overdraft rejected |
| Custom errors | Zero input rejected; one accepted |
| Events | High increment count |

### Adversarial — `access_control_fuzz.rs`

| Area | Scenarios |
|------|-----------|
| Authorization bypass | Unauthorized admin action, role action, pause, registry owner action, proxy-admin propose, proxy-admin pause |
| Role management | Grant requires admin auth; revoke prevents escalation; hierarchy enforced; symbol role guard; revoked role loses access |
| Multi-sig | Partial approval cannot execute; duplicate approval rejected; unauthorized signer blocked; cancel after execute blocked; execute of a nonexistent proposal fails |
| Timelock | Early execute blocked; replay after execution blocked; pause blocks queue |
| Misc | Auth vector round-trip; excess allowance spend rejected; delay bounds enforced; non-initializer cannot reinitialize; registration without whitelist fails when whitelist-only |

### Properties — `defi_fuzz_tests.rs`

| Group | Properties |
|-------|-----------:|
| AMM — `k` never decreases, output bounded by reserves, slippage rejection, sqrt LP mint, ratio mismatch, proportional removal, LP supply matches balances, sequential swaps, fee reduces output, reserves match token balances, bidirectional swaps | 11 |
| Lending pool — deposit increases total, withdraw reduces deposit, borrow within 80% limit, repay reduces debt, utilization bounded 0–100, borrow rate rises with utilization, positions isolated per user, deposit/withdraw round-trip, full repay zeros debt | 9 |
| Liquidation — reduces debt, transfers collateral, rejects healthy positions, health factor falls with borrow, partial liquidation capped, emergency liquidation clears position, repay improves health, incentive seizure amount, collateral deposit increases position | 9 |
| AMM dust boundary (deterministic, added by this report) | 3 |

### Invariants — `09-fuzz-testing` and `tests/fuzz/`

After every deposit or claim, for arbitrary amounts:

1. Init flag and balance entry are a valid pair — never a balance without initialization
2. Contract token balance is non-negative
3. If a claimable balance exists, its amount equals the tokens the contract holds
4. Remaining claimable amount never exceeds the original deposit

---

## 6. Running Locally

```bash
# Property and boundary suites (stable, ~1 min)
cargo test -p integration-tests --test fuzz_tests --test access_control_fuzz --test defi_fuzz_tests
cargo test -p fuzz-testing

# Coverage-guided fuzzing (nightly)
cargo install cargo-fuzz
cargo +nightly fuzz run --fuzz-dir tests/fuzz advanced_claimable_balance -- -max_total_time=300

# Coverage for the fuzz suites — see F-4: this currently reports only
# shared/src/lib.rs, not the contract crates under test
cargo tarpaulin -p integration-tests \
  --test fuzz_tests --test access_control_fuzz --test defi_fuzz_tests \
  --out Html --output-dir coverage
```

Raise `PROPTEST_CASES` to widen a property run without editing the source:

```bash
PROPTEST_CASES=2048 cargo test -p integration-tests --test defi_fuzz_tests
```

---

## 7. Next Steps

1. Get a real line-coverage number for the fuzz layer (F-4) — retry tarpaulin without `--skip-clean`, then with `--engine llvm`, and confirm the workspace `coverage` job is not dropping path-dependency sources the same way.
2. Pin the thin-collateral liquidation boundary with deterministic tests, closing the residual risk under F-2.
3. Add stateful sequence fuzzing — drive a list of operations from one fuzz input rather than a fixed one or two calls.
4. Extend property coverage to the token and governance categories.
5. Run a longer scheduled campaign (hours, not seconds) on a cron workflow, separate from the PR gate.

---

## See Also

- [Fuzz Testing Guide](../guides/fuzz-testing.md) — setup, writing targets, interpreting failures
- [`examples/advanced/09-fuzz-testing`](../examples/advanced/09-fuzz-testing/) — the fuzzable contract and its property tests
- [Testing Guide](./testing-guide.md) and [Testing Best Practices](./testing-best-practices.md)
- [Security Best Practices](./security-best-practices.md)

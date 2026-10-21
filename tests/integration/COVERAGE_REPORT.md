# Coverage Report — Advanced and Use-Case Examples

This report documents the coverage added for the two coverage issues that the
integration suite tracks:

| Issue | Scope | Test file |
|-------|-------|-----------|
| [#904](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/904) | 90%+ coverage for the **advanced** examples | `tests/advanced_coverage_tests.rs` |
| [#901](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/901) | 90%+ coverage for the **use-case** examples | `tests/use_case_coverage_tests.rs` |

Both suites register the example contracts natively
(`env.register_contract(None, Example)`), so they run with a plain Rust
toolchain and do not need a Wasm build.

---

## 1. How to run

```bash
# The two new suites only
cargo test -p integration-tests --test advanced_coverage_tests
cargo test -p integration-tests --test use_case_coverage_tests

# The whole integration suite (this is what CI runs)
cargo test -p integration-tests

# Line coverage of the whole workspace
cargo tarpaulin --out Html      # report in ./coverage/index.html
cargo tarpaulin --out Xml       # Cobertura XML for Codecov
```

`tarpaulin.toml` enforces `fail-under = 90`, and the CI workflow
(`.github/workflows/test.yml`) runs the coverage job on every push and pull
request.

---

## 2. Advanced examples (issue #904)

| Example | Path | Covered behaviour |
|---------|------|-------------------|
| Circuit breaker | `examples/advanced/04-circuit-breaker` | Trip after the failure threshold, reject calls while paused, automatic recovery after the recovery window, admin rotation, invalid configuration |
| Rate limiting | `examples/advanced/05-rate-limiting` | Call and amount limits, per-user overrides, window reset, explicit reset, invalid amounts |
| Reentrancy guard | `examples/advanced/15-reentrancy-guard` | Deposit, withdraw through a cross-contract call, insufficient balance |
| Permit pattern | `examples/advanced/30-permit-pattern` | Permit grant, `transfer_from` spending, allowance accounting, expired permits, invalid amounts |
| Merkle proofs | `examples/advanced/21-merkle-proofs` | Root publication, proof verification, claim-once semantics, out-of-bounds indices, unauthorized root update |
| Merkle airdrop | `examples/advanced/28-merkle-airdrop` | Claim against a Merkle proof, single-claim enforcement, wrong-amount proof rejection, SAC payout |
| Merkle whitelist | `examples/advanced/29-merkle-whitelist` | Role assignment, governed root update with quorum and timelock, whitelist registration, duplicate registration |
| Batch transfer | `examples/advanced/20-batch-transfer` | Multi-recipient payout, empty batch, invalid amount, insufficient balance, all-or-nothing behaviour |
| Batch builder | `examples/advanced/22-batch-builder` | Batch drafting, gas estimation, validation, execution, duplicate operation, executing twice |
| Multicall | `examples/advanced/42-multicall` | Aggregating several calls, empty batch, self-call rejection, per-result failure reporting, strict failure mode |
| Hierarchical access control | `examples/advanced/16-hierarchical-access-control` | Role grant/revoke/renounce, inherited permissions, role members |
| Data aggregation oracle | `examples/advanced/26-data-aggregation-oracle` | Trusted sources, median/mean aggregation, pause and resume |
| Upgrade patterns | `examples/advanced/07-upgrade-patterns` | Versioned storage migration, migration guards, post-upgrade initializer ordering |
| Beacon management | `examples/advanced/37-beacon-management` | Beacon registration, upgrade, rollback, listing |

### Edge cases covered

- Threshold and window boundaries (exactly at the threshold, one ledger
  before and after the recovery window).
- Zero and negative amounts everywhere an amount is accepted.
- Expired approvals and permits (allowance becomes worthless).
- Double claims (Merkle airdrop, Merkle proofs, rewards).
- Batches that are empty, too large, duplicated, or executed twice.
- Unauthorized administration (root update, role grant, proposal execution).
- Proposals that do not reach quorum.

---

## 3. Use-case examples (issue #901)

| Category | Example | Path | Covered behaviour |
|----------|---------|------|-------------------|
| DeFi | Constant product AMM | `examples/defi/02-constant-product-amm` | Adding liquidity, LP accounting, fee-adjusted swaps, slippage guard, wrong token, ratio mismatch |
| DeFi | Snapshot triggers | `examples/defi/14-automatic-snapshot-triggers` | Recording values, frequency-gated snapshots, pruning, disabling |
| DeFi | Reward strategies | `examples/defi/15-reward-strategies` | Linear vesting of rewards, claim-once, unauthorized registration, invalid weight |
| NFT | NFT metadata | `examples/nfts/02-nft-metadata` | Off-chain and on-chain metadata, double mint, invalid token id, metadata updates |
| NFT | NFT metadata standards | `examples/nfts/03-nft-metadata-standards` | Metadata validation (image URI, empty fields), approvals, transfers, balances |
| NFT | Snapshot triggers | `examples/nfts/05-automatic-snapshot-triggers` | History tracking, index bounds, configuration |
| Governance | Vote delegation | `examples/governance/06-vote-delegation` | Voting power movement, un-delegation, self-delegation rejection |
| Governance | Timelock governance | `examples/governance/09-timelock-governance` | Queue, delay enforcement, execution, cancellation, invalid delay |
| Governance | DAO treasury | `examples/governance/10-dao-treasury` | Deposits, proposal lifecycle, quorum, execution, double execution |
| Governance | Snapshot triggers | `examples/governance/11-automatic-snapshot-triggers` | Snapshot history and configuration |
| Token | Vesting management | `examples/tokens/01-vesting-management` | Cliff and linear vesting, claims, invalid schedules, unauthorized access |
| Token | Minting strategies | `examples/tokens/02-minting-strategies` | Supply cap, unlimited minting, scheduled minting, unauthorized minting |
| Token | Allowance pattern | `examples/tokens/05-allowance-pattern` | Approve, `transfer_from`, revocation, expired allowances |
| Token | Reward token | `examples/tokens/06-reward-token` | Pools, deposits, reward claims, nothing to claim, unknown pool |
| Token | Token metadata | `examples/tokens/07-token-metadata` | Mint, transfer, burn, metadata reads and updates, empty-name rejection |
| Token | Multi-token balance manager | `examples/tokens/08-multi-token-balance-manager` | Token registration, batch balances, batch transfers, empty batch, unregistered token |
| Token | Custom token | `examples/tokens/10-custom-token` | Transfers, pause, multi-sig transfers, signer rotation |
| Token | Token lock | `examples/tokens/token-lock` | Locking, unlock schedule, matured releases |

---

## 4. Coverage of the example tree

The suites above add **31 examples** to the integration matrix. Combined with
the pre-existing suites (`integration_tests.rs`, `advanced_integration_tests.rs`,
`token_integration_tests.rs`, `governance_tests.rs`, `defi_scenarios.rs`,
`regression_tests.rs`, `compatibility_tests.rs`, `security_tests.rs`) the
workspace-wide line coverage is measured by `cargo tarpaulin` and gated at
90% by `tarpaulin.toml` and the CI coverage job.

Examples that are intentionally not linked from the integration suite:

| Example | Reason |
|---------|--------|
| `examples/governance/02-token-voting` | The crate's `src/lib.rs` is a broken merge of two different contracts and does not compile; it is tracked separately. |
| `examples/governance/08-delegation-marketplace` | The crate is `cdylib`-only, so it cannot be linked as a path dependency from a test crate. |
| `examples/advanced/27-gasless-relayer`, `examples/advanced/43-payment-channels` | They require `ed25519-dalek` signature fixtures; the signature scheme is covered by the contracts' own unit tests. |

---

## 5. Adding more coverage

1. Add the example as a path dependency in `tests/integration/Cargo.toml`.
2. Add a test file (or extend an existing one) under `tests/integration/tests/`.
3. Use `env.mock_all_auths()` for the happy path and `try_*` for the error
   paths.
4. Run `cargo test -p integration-tests` and `cargo clippy -p integration-tests --tests`.
5. Update the tables in this report.

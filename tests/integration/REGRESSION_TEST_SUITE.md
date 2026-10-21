# Regression Test Suite Architecture & Execution Report

This document specifies the regression testing architecture, invariant validation matrix, and automated execution guidelines for the Soroban Cookbook smart contract suite (Issue #946, Phase 6 Issue #305).

---

## 1. Overview & Objectives

As the Soroban Cookbook expands with new examples, patterns, and SDK version updates, a regression test suite ensures:
- **Zero Breaking Changes**: Contract interfaces, error codes, and storage patterns remain backward-compatible.
- **Invariant Preservation**: Core economic, mathematical, and cryptographic invariants remain strictly true.
- **Cross-Contract Composability**: Multi-contract workflows maintain deterministic behavior across example boundaries.
- **Automated CI Execution**: Fast, reliable execution in GitHub Actions on every pull request.

---

## 2. Invariant Validation Matrix

The suite is implemented in [`tests/integration/tests/regression_tests.rs`](./tests/regression_tests.rs) and covers six critical domains:

| Domain | Contracts Covered | Invariants Validated | Failure Behavior |
|--------|-------------------|----------------------|------------------|
| **Basics** | `authentication`, `storage_patterns`, `custom_errors` | Balance non-negativity, storage key persistence, strict error variant mapping (`ZeroAmount`, `NegativeAmount`) | Reverts on invalid input / mutation |
| **Tokens** | `sep41_token`, `mint_burn_token` | Conservation of total supply ($\sum \text{balances} = \text{total\_supply}$), allowance decrement on `transfer_from`, supply cap enforcement | Reverts on allowance or cap breach |
| **Pausable** | `pausable_token` | State machine isolation: operations fail when paused and resume upon unpause without state loss | Reverts while paused |
| **Governance** | `proposal_lifecycle` | Unique voter enforcement (no double voting), deterministic voting window bounds, accurate ballot tallying | Reverts on duplicate vote |
| **NFTs** | `basic_nft` | Unique token ownership, balance transfer preservation, revocation of non-owner transfer privileges | Reverts on non-owner transfer |
| **Intermediate** | `contract_registry`, `ajo_factory` | Dynamic contract discovery by symbol name, version tracking, non-existent entries return `None` | Deterministic `None` / Lookup |
| **DeFi** | `simple_swap` | Valid liquidity checks, zero-amount swap prevention, fee deduction integrity | Reverts on zero swap |

---

## 3. Test Cases Summary

### `test_regression_basics_auth_storage_and_custom_errors`
- Validates that unauthorized transfers and negative amounts are blocked.
- Validates that persistent storage retains written values across invocations.
- Validates that custom error discriminants are faithfully preserved.

### `test_regression_token_supply_and_allowance_invariants`
- Validates that minting tokens increases both recipient balance and total supply in exact parity.
- Validates that allowances are cleanly tracked and decremented upon `transfer_from`.
- Verifies that capped tokens reject mint transactions that would exceed the maximum supply.

### `test_regression_pausable_token_invariants`
- Verifies that pausing immediately halts token transfers and minting.
- Verifies that unpausing restores full operation without corrupting existing balances.

### `test_regression_governance_proposal_and_voting_invariants`
- Tests the complete proposal lifecycle: creation, active status, voting, and tallying.
- Proves that double voting by the same account is rejected.

### `test_regression_nft_ownership_and_transfer_invariants`
- Verifies that NFT minting assigns unique ownership.
- Proves that transfers update owner registries and balances atomically.
- Confirms that former owners cannot initiate transfers.

### `test_regression_registry_and_factory_invariants`
- Validates contract registration and discovery in the registry contract.
- Confirms safe handling of queries for unregistered names.

### `test_regression_defi_swap_invariants`
- Tests automated market maker swap initiation and zero-amount safeguards.

---

## 4. Execution & CI Integration

### Local Execution

Run the regression test suite with:

```bash
cargo test -p integration-tests --test regression_tests
```

Run with verbose test output:

```bash
cargo test -p integration-tests --test regression_tests -- --nocapture
```

### CI Pipeline

The regression test suite is automatically executed as part of the `Test Suite` job in `.github/workflows/test.yml`:

```yaml
- name: Run integration tests
  run: cargo test -p integration-tests --target x86_64-unknown-linux-gnu
```

All integration test files under `tests/integration/tests/` are compiled and executed in parallel during CI.

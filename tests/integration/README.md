# Soroban Integration Tests - Shared Test Utilities

This directory contains comprehensive integration tests for Soroban smart contracts, along with a reusable library of test utilities and helpers.

## Table of Contents

- [Overview](#overview)
- [Test Helpers](#test-helpers)
- [Mock Contracts](#mock-contracts)
- [Test Data Generators](#test-data-generators)
- [Custom Assertions](#custom-assertions)
- [Running Tests](#running-tests)
- [Best Practices](#best-practices)
- [Migration Guide](#migration-guide)

> **Smoke tests:** for a quick check of the critical paths before running everything, run `cargo test -p integration-tests --test smoke_tests`. See [SMOKE_TESTS.md](./SMOKE_TESTS.md).

## Overview

The test utilities library (`src/lib.rs`) provides reusable components to simplify Soroban integration testing:

- **Common Helpers**: Contract deployment, invocation, environment setup
- **Mock Contracts**: Reusable mock token and auth contracts
- **Test Data Generators**: Factories and fixtures for common test data
- **Custom Assertions**: Macros and functions for contract-specific assertions

## Test Helpers

### Basic Setup

Initialize a test environment with default Soroban configuration:

```rust
use integration_test_utils::helpers::*;

#[test]
fn example_test() {
    let env = init_test_env();
    env.mock_all_auths();
    
    // Your test code here
}
```

7. **Validation + Custom Errors Integration** (`test_validation_and_errors_integration`)
   - Combines `validation-patterns` and `custom-errors`
   - Demonstrates how to handle different error types in a single workflow

8. **Ajo Factory + Authentication Lifecycle** (`test_ajo_factory_lifecycle_integration`)
   - Combines `ajo_factory` and `authentication`
   - Tests the complete lifecycle of a factory-deployed contract with initialization

9. **Multi-Sig Governance + Events Tracking** (`test_multi_sig_governance_integration`)
   - Combines `multi_sig_patterns` and `events-counter`
   - Demonstrates proposal-based governance with audit logs in a separate contract

10. **Token Wrapper End-to-End Flow** (`test_token_wrapper_multi_user_flow`)
   - Combines the token wrapper with a Stellar asset token
   - Demonstrates multi-user wrapping, transfer, unwrapping, and backing checks

## Token Integration Tests (`token_integration_tests.rs`)

Cross-crate flows spanning multiple token examples (Issue #119):

1. **SEP-41 + Mint/Burn + Pausable** (`test_sep41_mint_burn_pausable_multi_user`)
   - Multi-user mint, transfer, allowance, supply-cap, and pause edge cases
2. **Wrapper + Vesting** (`test_wrapper_and_vesting_cross_contract_flow`)
   - Shared underlying asset, wrap/unwrap alongside timed vesting claims
3. **Edge Cases** (`test_token_edge_cases_across_contracts`)
   - Zero amounts, overdraft, and pause-already-in-state across contracts

## Token Security Tests (`token_security_tests.rs`)

Security-focused tests for token examples (Issue #795) — reentrancy,
arithmetic issues, and authorization bypass attempts. See
[`SECURITY_REVIEW_TOKEN_EXAMPLES.md`](./SECURITY_REVIEW_TOKEN_EXAMPLES.md)
for the findings (including a real reentrancy fix in `06-token-wrapper`)
these tests guard against regressing.

1. **Authorization bypass** (`sep41_token`) — `transfer`/`approve`/
   `transfer_from`/`mint`/`burn` each rejected without their required
   signer's real (non-mocked) authorization, plus a real-but-non-admin
   signer correctly rejected by `mint`'s own admin check.
2. **Arithmetic issues** (`sep41_token`) — `mint` at `i128::MAX` overflow,
   full-balance transfer/burn boundaries, and `total_supply` consistency
   across a mint/transfer/burn cycle.
3. **Reentrancy** (`token_wrapper`) — a `MaliciousUnderlyingToken` whose
   `transfer` reenters `wrap`; asserts the `DataKey::Entered` guard blocks
   the double-mint attempt.

## Governance Integration Tests (`governance_tests.rs`)

30 tests across 8 categories covering the complete governance stack. All tests use
`env.register_contract` (no WASM binary required).

## RBAC, Multisig, and Timelock Workflows (`rbac_multisig_timelock_tests.rs`)

6 tests simulating a multi-step governance action integrating Role-Based Access Control, Multisig, and Timelocks.

| # | Flow | Result |
|---|----------|-------|
| 1 | Full Pipeline (Happy Path) | Success |
| 2 | Multisig threshold not met | Revert / Failure |
| 3 | Timelock executed too early | Revert / Failure |
| 4 | Unauthorized timelock queueing | Revert / Failure |
| 5 | Cancellation workflow | Success (Timelock execution fails after cancellation) |
| 6 | RBAC Validation failure | Revert / Failure |


| # | Category | Tests | File |
|---|----------|-------|------|
| 1 | Proposal lifecycle | 5 | `governance_tests.rs` |
| 2 | Voting | 4 | `governance_tests.rs` |
| 3 | DAO treasury | 3 | `governance_tests.rs` |
| 4 | Authorization | 2 | `governance_tests.rs` |
| 5 | Multiple / concurrent proposals | 4 | `governance_tests.rs` |
| 6 | Delegation (inline mock) | 4 | `governance_tests.rs` |
| 7 | Voting-time-constraints | 4 | `governance_tests.rs` |
| 8 | End-to-end | 4 | `governance_tests.rs` |

### Category Details

**Category 1 – Proposal Lifecycle**
- `test_gov_proposal_creation` — initialize and create a Draft proposal
- `test_gov_create_proposal_not_initialized_error` — NotInitialized error path
- `test_gov_proposal_submit` — Draft → Active transition
- `test_gov_proposal_execute` — cross-contract execution of a Passed proposal
- `test_gov_proposal_reject_and_cancel` — Failed (quorum miss) and Cancelled states

**Category 2 – Voting**
- `test_gov_vote_successful` — yes vote recorded with correct weight
- `test_gov_vote_duplicate_prevention` — AlreadyVoted error on second vote
- `test_gov_vote_deadline_enforcement` — VotingEnded error after deadline
- `test_gov_vote_quorum_not_met` — proposal resolves to Failed below quorum

**Category 3 – DAO Treasury**
- `test_dao_treasury_deposit` — repeated deposits accumulate correctly
- `test_dao_treasury_withdrawal_via_governance` — Transfer proposal reduces balance
- `test_dao_treasury_over_withdrawal_guard` — InsufficientTreasuryBalance error

**Category 4 – Authorization**
- `test_gov_auth_invalid_proposer_cancel` — Unauthorized error for third-party cancel
- `test_gov_auth_invalid_executor_while_active` — VotingNotEnded error on early execute

**Category 5 – Multiple / Concurrent Proposals**
- `test_gov_multiple_proposals_independent_ids` — sequential IDs and independent state
- `test_gov_concurrent_proposals_different_outcomes` — Passed vs Failed simultaneously
- `test_gov_concurrent_dao_proposals_independent_votes` — votes isolated per proposal
- `test_gov_vote_isolation_between_proposals` — voting one proposal leaves others clean

**Category 6 – Delegation**
- `test_gov_delegation_create` — delegate weight tracked on delegatee
- `test_gov_delegation_remove` — undelegate zeroes weight
- `test_gov_delegated_weight_voting` — delegatee uses accumulated weight in governance
- `test_gov_delegation_zero_weight_edge_case` — zero delegation → proposal fails quorum

**Category 7 – Voting-time-constraints**
- `test_vtc_proposal_creation` — initialize and create Active proposal
- `test_vtc_post_deadline_vote_rejected` — VotingClosed after deadline
- `test_vtc_vote_within_window_accepted` — for/against counts within window
- `test_vtc_full_lifecycle` — Active → GracePeriod → Executable → Executed

**Category 8 – End-to-end**
- `test_e2e_cross_contract_governance_action` — governance mutates a target contract
- `test_e2e_full_community_vote_workflow` — 5-voter weighted community vote
- `test_e2e_dao_treasury_full_flow` — deposit → propose → vote → execute → verify balance
- `test_e2e_simple_voting_full_workflow` — admin creates, users vote, result tallied

### Run governance tests only

```bash
cargo test -p integration-tests governance
```

### Contracts used

| Dependency | Package | Path |
|-----------|---------|------|
| `proposal_lifecycle` | `proposal-lifecycle` | `examples/governance/04-proposal-lifecycle` |
| `simple_voting` | `simple-voting` | `examples/governance/01-simple-voting` |
| `voting_time_constraints` | `voting-time-constraints` | `examples/governance/01-voting-time-constraints` |
| `dao_treasury` | `dao-treasury` | `examples/governance/10-dao-treasury` |

## Regression Test Suite (`regression_tests.rs`)

Comprehensive cross-category regression suite (Issue #946 / Phase 6 #305). Validates critical contract invariants, state transitions, allowance boundaries, and backwards compatibility across Basics, Tokens, Governance, NFTs, Intermediate, and DeFi examples.

Full specification and failure behavior documentation: [`REGRESSION_TEST_SUITE.md`](./REGRESSION_TEST_SUITE.md).

| # | Test | Scope | Invariant Validated |
|---|------|-------|---------------------|
| 1 | `test_regression_basics_auth_storage_and_custom_errors` | Basics | Balance non-negativity, persistent storage integrity, error variants |
| 2 | `test_regression_token_supply_and_allowance_invariants` | Tokens | Supply conservation ($\sum B = \text{Supply}$), allowance decrement, supply caps |
| 3 | `test_regression_pausable_token_invariants` | Tokens | Mutation blocked while paused, clean resumption after unpause |
| 4 | `test_regression_governance_proposal_and_voting_invariants` | Governance | Unique voter enforcement, lifecycle transitions, deterministic tallies |
| 5 | `test_regression_nft_ownership_and_transfer_invariants` | NFTs | Atomic ownership handoff, non-owner transfer rejection |
| 6 | `test_regression_registry_and_factory_invariants` | Intermediate | Dynamic contract discovery by symbol, version tracking |
| 7 | `test_regression_defi_swap_invariants` | DeFi | AMM reserves integrity, zero-amount swap rejection |

Run regression tests:
```bash
cargo test -p integration-tests --test regression_tests
```

## Compatibility Test Suite (`compatibility_tests.rs`)

Cross-version and environment compatibility suite (Issue #950 / Phase 6 #306). Tests SDK type serialization, network protocol simulation (Protocols 20, 21, and 22), upgrade state preservation, storage schema evolution, and error code numeric stability.

Full report and version matrix: [`COMPATIBILITY_REPORT.md`](./COMPATIBILITY_REPORT.md).

| # | Test | Scope | Verification |
|---|------|-------|--------------|
| 1 | `test_sdk_version_type_compatibility` | SDK Types | `Address`, `Val`, `Symbol`, `String`, `Vec`, `Map`, `Bytes`, `BytesN` representation |
| 2 | `test_network_compatibility_ledger_protocol_simulation` | Network Protocol | Ledger simulation across Protocol 20, 21, and 22 parameters |
| 3 | `test_upgrade_compatibility_state_preservation` | Upgrades | Proxy state preservation across implementation contract upgrades |
| 4 | `test_storage_layout_backward_compatibility` | Storage Layout | Schema evolution from legacy structs to modern structs with new fields |
| 5 | `test_custom_error_code_stability` | Error Codes | Stable enum integer discriminators preventing RPC deserialization failures |

Run compatibility tests:
```bash
cargo test -p integration-tests --test compatibility_tests
```

## Advanced Coverage Tests (`advanced_coverage_tests.rs`)

Cross-example integration tests for the **advanced** examples (issue #904).
Covers the circuit breaker, rate limiter, reentrancy guard, permit pattern,
Merkle proofs / airdrop / whitelist, batch transfer, batch builder, multicall,
hierarchical access control, data aggregation oracle, upgrade patterns and
beacon management — including the edge cases listed in
[`COVERAGE_REPORT.md`](./COVERAGE_REPORT.md).

```bash
cargo test -p integration-tests --test advanced_coverage_tests
```

## Use-Case Coverage Tests (`use_case_coverage_tests.rs`)

Cross-example integration tests for the **use-case** examples (issue #901):
DeFi (AMM, snapshot triggers, reward strategies), NFTs (metadata, metadata
standards, snapshot triggers), governance (vote delegation, timelock, DAO
treasury, snapshot triggers) and tokens (vesting, minting strategies,
allowance, rewards, metadata, multi-token manager, custom token, lock).

```bash
cargo test -p integration-tests --test use_case_coverage_tests
```

## Coverage Report

[`COVERAGE_REPORT.md`](./COVERAGE_REPORT.md) maps every example to the suite
that covers it and documents how the 90% coverage gate is measured.

## Running the Tests

Deploy a contract to the test environment:

```rust
let hello_id = deploy_contract(&env, hello_world::HelloContract);
```

### Contract Invocation

Invoke contract functions with simplified syntax:

```rust
use soroban_sdk::{symbol_short, IntoVal, Vec};

let greeting: Vec<Symbol> = invoke_contract(
    &env,
    &hello_id,
    &symbol_short!("hello"),
    &Vec::from_array(&env, [symbol_short!("Alice").into_val(&env)])
);
```

### Ledger Management

Jump forward in ledger sequence for time-dependent testing:

```rust
// Advance by 100 ledgers
jump_ledger_sequence(&env, 100);
```

## Mock Contracts

### Mock Token Contract

Create a configurable mock token for testing token interactions:

```rust
use integration_test_utils::mocks::MockTokenBuilder;

let admin = Address::generate(&env);
let token_config = MockTokenBuilder::new(admin)
    .with_supply(1_000_000)
    .with_decimals(7)
    .build();
```

### Mock Auth Contract

Create a mock authorization contract:

```rust
use integration_test_utils::mocks::MockAuthBuilder;

let admin = Address::generate(&env);
let auth_config = MockAuthBuilder::new(admin)
    .with_authorized(user1.clone())
    .with_authorized(user2.clone())
    .build();
```

## Test Data Generators

### Factory Functions

Generate common test data types:

```rust
use integration_test_utils::generators::factories::*;

let addr = address(&env, 42);           // Deterministic address from seed
let sym = symbol("count");              // Symbol from string
let val = int128(-100);                 // Signed integer
let uval = uint128(1_000_000);          // Unsigned integer
let time = timepoint(1609459200);       // Unix timestamp
```

### Token Fixture

Create pre-configured token fixtures:

```rust
use integration_test_utils::generators::fixtures::*;

let admin = Address::generate(&env);
let token = TokenFixtureBuilder::new(admin)
    .with_supply(5_000_000)
    .with_decimals(6)
    .build();
```

### Auth Fixture

Create pre-configured auth fixtures:

```rust
use integration_test_utils::generators::fixtures::*;

let admin = Address::generate(&env);
let auth = AuthFixtureBuilder::new(admin)
    .with_authorized(user1)
    .with_authorized(user2)
    .build();
```

## Custom Assertions

### Contract Success/Error Assertions

```rust
use integration_test_utils::assert_contract_success;
use integration_test_utils::assert_contract_error;

let result: Vec<Symbol> = env.invoke_contract(...);
assert_contract_success!(&result);

let error_result: Result<(), Error> = env.invoke_contract(...);
assert_contract_error!(&error_result);
```

### State Value Assertions

```rust
use integration_test_utils::helpers::assertions::assert_state_value;

assert_state_value(actual_balance, expected_balance, "balance");
```

### Collection Length Assertions

```rust
use integration_test_utils::helpers::assertions::assert_collection_length;

assert_collection_length(&events, 2, "event log");
```

### Conditional Assertions

```rust
use integration_test_utils::assert_condition;

assert_condition!(value > 0, "value should be positive, got: {}", value);
```

## Running Tests

Run all integration tests:

```bash
cargo test -p integration-tests
```

Run a specific test:

```bash
cargo test -p integration-tests test_greeting_system_workflow
```

Run tests with output:

```bash
cargo test -p integration-tests -- --nocapture
```

## Best Practices

### 1. Use `init_test_env()` for Setup

Always initialize test environment using the provided helper:

```rust
let env = init_test_env();
```

This ensures consistent environment configuration across all tests.

### 2. Use Fixtures for Complex Scenarios

For tests involving multiple contracts or data structures, use fixtures:

```rust
let admin = Address::generate(&env);
let token = TokenFixtureBuilder::new(admin)
    .with_supply(1_000_000)
    .build();
```

### 3. Use Custom Assertions for Clarity

Instead of bare `assert_eq!`, use domain-specific assertions:

```rust
// Better
assert_state_value(actual, expected, "balance");

// vs.
assert_eq!(actual, expected);
```

### 4. Group Related Tests

Use module organization to group related integration tests:

```rust
mod token_tests {
    #[test]
    fn test_transfer() { }
    
    #[test]
    fn test_balance() { }
}
```

### 5. Keep Test Data Consistent

Use factories for consistent test data generation:

```rust
let addr1 = address(&env, 1);
let addr2 = address(&env, 2);
```

## Migration Guide

### Before: Manual Contract Setup

```rust
#[test]
fn test_old_style() {
    let env = Env::default();
    env.mock_all_auths();
    
    let auth_id = env.register_contract(None, authentication::AuthContract);
    let storage_id = env.register_contract(None, storage_patterns::StorageContract);
    
    let admin = Address::generate(&env);
    
    env.invoke_contract::<()>(
        &auth_id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );
}
```

### After: Using Helpers

```rust
use integration_test_utils::helpers::*;

#[test]
fn test_new_style() {
    let env = init_test_env();
    
    let auth_id = deploy_contract(&env, authentication::AuthContract);
    let storage_id = deploy_contract(&env, storage_patterns::StorageContract);
    
    let admin = Address::generate(&env);
    
    invoke_contract(
        &env,
        &auth_id,
        &Symbol::new(&env, "initialize"),
        &Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );
}
```

### Key Improvements

- Reduced boilerplate with `init_test_env()`
- Simplified contract deployment with `deploy_contract()`
- Cleaner invocation syntax with `invoke_contract()`

## API Reference

### Helpers Module

| Function | Purpose |
|----------|---------|
| `init_test_env()` | Initialize test environment |
| `deploy_contract()` | Deploy a contract to the environment |
| `invoke_contract()` | Invoke a contract function |
| `jump_ledger_sequence()` | Advance ledger by N ledgers |
| `setup_ledger_with_data()` | Initialize ledger state |

### Generators Module

| Function | Purpose |
|----------|---------|
| `address(seed)` | Generate deterministic address |
| `symbol(s)` | Create symbol from string |
| `int128(val)` | Create signed integer |
| `uint128(val)` | Create unsigned integer |
| `u64_val(val)` | Create u64 value |
| `u32_val(val)` | Create u32 value |

### Assertion Module

| Macro/Function | Purpose |
|---|---|
| `assert_contract_success!()` | Assert successful contract execution |
| `assert_contract_error!()` | Assert contract error |
| `assert_state_value()` | Assert state field value |
| `assert_collection_length()` | Assert collection size |
| `assert_condition!()` | Assert boolean condition |

## Contributing

When adding new tests:

1. Use the provided helpers and fixtures
2. Include documentation with examples
3. Ensure all tests pass with `cargo test -p integration-tests`
4. Run clippy: `cargo clippy -p integration-tests`
5. Update this README if adding new utilities

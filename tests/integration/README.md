# Integration Tests

This directory contains integration tests for the Soroban Cookbook basic examples. These tests demonstrate cross-contract interactions and end-to-end scenarios.

## Overview

The integration tests combine multiple basic examples to showcase real-world usage patterns:

1. **Multi-Contract Workflow** (`test_greeting_system_workflow`)
   - Combines Hello World, Storage, and Events contracts
   - Demonstrates a user greeting system with persistent storage and event emission

2. **Authentication + Storage Integration** (`test_authenticated_storage_workflow`)
   - Tests authenticated users storing and retrieving their own data
   - Shows proper data isolation between users

3. **Cross-Contract Event Tracking** (`test_cross_contract_event_tracking`)
   - Tracks operations across multiple contracts with events
   - Demonstrates admin initialization, configuration storage, and event emission

4. **Storage Type Comparison** (`test_storage_types_comparison`)
   - End-to-end demonstration of persistent, temporary, and instance storage
   - Shows independence of different storage types

5. **Complex Multi-Party Workflow** (`test_multi_party_workflow`)
   - Multiple users interacting with authentication, storage, and events
   - Simulates a complete application flow with greetings, transfers, and balance updates

6. **State Management Across Contracts** (`test_coordinated_state_management`)
   - Coordinates state changes across multiple contracts
   - Demonstrates configuration updates with event tracking and audit trails

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
| `dao_treasury` | `dao-treasury` | `examples/governance/03-dao-treasury` |

## Running the Tests

### Prerequisites

1. Build the WASM files for all required contracts:
```bash
cd /home/luckify/wave/Soroban-Cookbook-
cargo build --release --target wasm32-unknown-unknown
```

Or build individual contracts:
```bash
cd examples/basics/01-hello-world && cargo build --release --target wasm32-unknown-unknown
cd examples/basics/02-storage-patterns && cargo build --release --target wasm32-unknown-unknown
cd examples/basics/03-authentication && cargo build --release --target wasm32-unknown-unknown
cd examples/basics/04-events && cargo build --release --target wasm32-unknown-unknown
```

### Run Tests

```bash
cd tests/integration
cargo test
```

Run a specific test:
```bash
cargo test test_greeting_system_workflow
```

Run with output:
```bash
cargo test -- --nocapture
```

## Test Architecture

The integration tests use WASM binaries directly via `env.register_contract_wasm()` and invoke contract functions using `env.invoke_contract()`. This approach:

- Tests contracts as they would be deployed on-chain
- Validates cross-contract interactions
- Ensures WASM compilation works correctly
- Provides realistic end-to-end scenarios

## Key Patterns Demonstrated

### Cross-Contract Communication
Tests show how contracts can work together to build complex applications.

### Storage Patterns
- Persistent storage for long-term data
- Temporary storage for transaction-scoped data
- Instance storage for contract configuration

### Authentication Flows
- User authentication before operations
- Admin-only functions
- Multi-user scenarios

### Event Emission
- Tracking operations across contracts
- Audit trails
- Configuration changes

## Adding New Integration Tests

1. Ensure the required contracts are built as WASM
2. Register contracts using `env.register_contract_wasm()`
3. Use `Symbol::new(&env, "function_name")` for function names (not `symbol_short!`)
4. Invoke contracts with `env.invoke_contract()`
5. Add assertions to verify expected behavior

Example:
```rust
#[test]
fn test_my_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let wasm = Bytes::from_slice(&env, include_bytes!("../../../target/wasm32-unknown-unknown/release/my_contract.wasm"));
    let contract_id = env.register_contract_wasm(None, wasm);

    let result: u64 = env.invoke_contract(
        &contract_id,
        &Symbol::new(&env, "my_function"),
        Vec::from_array(&env, [42u64.into_val(&env)]),
    );

    assert_eq!(result, 42);
}
```

## Troubleshooting

### "MissingValue" Error
- Ensure WASM files are built and up-to-date
- Check function names match the contract exports exactly
- Use `Symbol::new(&env, "full_function_name")` not `symbol_short!`

### Contract Not Found
- Build the WASM files first
- Check the path in `include_bytes!` is correct

### Type Mismatch
- Ensure return types match the contract function signatures
- Use `env.invoke_contract::<()>` for functions that return void

## CI/CD

These tests are automatically run in the CI pipeline to ensure all basic examples work together correctly.

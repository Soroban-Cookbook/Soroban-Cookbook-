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

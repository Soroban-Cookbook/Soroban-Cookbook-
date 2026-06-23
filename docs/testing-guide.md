# Soroban Testing Guide

A comprehensive guide to testing Soroban smart contracts effectively, covering unit tests, integration tests, test utilities, and best practices.

## 📖 Overview

Testing is critical for smart contract development. This guide covers:

- Unit testing individual contract functions
- Integration testing multi-contract interactions
- Test organization and best practices
- Advanced testing techniques and utilities
- Snapshot testing and coverage tools
- Common testing patterns and anti-patterns

## 🧪 Test Types

### Unit Tests

Unit tests verify individual contract functions in isolation. They're fast, focused, and ideal for testing business logic.

**When to use**:

- Testing single function behavior
- Verifying state changes
- Testing error conditions
- Validating input validation

#### Example from Hello World Contract

```rust
use super::*;
use soroban_sdk::{symbol_short, vec, Env, Symbol};

#[test]
fn test_hello_returns_greeting_vec() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let result = client.hello(&symbol_short!("World"));

    assert_eq!(
        result,
        vec![&env, symbol_short!("Hello"), symbol_short!("World")]
    );
}

#[test]
fn test_hello_with_different_names() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    for name in [
        symbol_short!("Alice"),
        symbol_short!("Bob"),
        symbol_short!("Dev"),
    ] {
        let result = client.hello(&name);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap(), symbol_short!("Hello"));
        assert_eq!(result.get(1).unwrap(), name);
    }
}
```

### Integration Tests

Integration tests verify interactions between multiple contracts or complex workflows involving multiple function calls.

**When to use**:

- Testing multi-contract interactions
- Verifying complex workflows
- Testing cross-contract calls
- Validating state consistency across contracts

**Example**:

```rust
#[test]
fn test_multi_contract_interaction() {
    let env = Env::default();

    // Deploy multiple contracts
    let token_id = env.register_contract(None, TokenContract);
    let vault_id = env.register_contract(None, VaultContract);

    let token = TokenContractClient::new(&env, &token_id);
    let vault = VaultContractClient::new(&env, &vault_id);

    // Test interaction
    token.mint(&user, &1000);
    vault.deposit(&user, &token_id, &500);

    assert_eq!(vault.balance(&user), 500);
}
```

## 🏗️ Test Organization

### Recommended Structure

```
src/
├── lib.rs           # Contract implementation
└── test.rs          # Unit tests

tests/
├── integration.rs   # Integration tests
└── common/
    └── mod.rs       # Shared test utilities
```

### Test Module Pattern

**In `src/lib.rs`**:

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}

#[cfg(test)]
mod test;
```

**In `src/test.rs`**:

```rust
#![cfg(test)]

use super::*;
use soroban_sdk::Env;

#[test]
fn test_add() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    assert_eq!(client.add(&2, &3), 5);
}
```

## 🛠️ Testing Utilities

### Environment Setup

```rust
use soroban_sdk::{Env, Address, testutils::Address as _};

#[test]
fn setup_test() {
    let env = Env::default();

    // Mock the ledger to enable authorization
    env.mock_all_auths();

    // Create test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // ... test logic
}
```

### Time Manipulation

```rust
#[test]
fn test_with_time() {
    let env = Env::default();

    // Set specific ledger timestamp
    env.ledger().with_mut(|li| {
        li.timestamp = 1640000000;
    });

    // Advance time by 100 seconds
    env.ledger().with_mut(|li| {
        li.timestamp += 100;
    });
}
```

### Authorization Mocking

```rust
use soroban_sdk::testutils::MockAuth;

#[test]
fn test_auth() {
    let env = Env::default();
    env.mock_all_auths(); // Mock all authorization checks

    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // This will succeed even without real auth
    client.transfer(&user, &recipient, &100);

    // Verify auth was called
    assert_eq!(
        env.auths(),
        std::vec![(
            user.clone(),
            AuthorizedInvocation { ... }
        )]
    );
}
```

## ✅ Best Practices

### 1. Use Descriptive Test Names

Test names should clearly describe what is being tested and the expected outcome.

✅ **DO**:

```rust
#[test]
fn test_transfer_succeeds_with_sufficient_balance() { }

#[test]
fn test_transfer_fails_with_insufficient_balance() { }

#[test]
fn test_transfer_fails_when_sender_not_authorized() { }
```

❌ **DON'T**:

```rust
#[test]
fn test_transfer() { }

#[test]
fn test_1() { }

#[test]
fn test_error() { }
```

### 2. Test Both Happy Path and Error Cases

Every function should have tests for success and failure scenarios.

✅ **DO**:

```rust
#[test]
fn test_withdraw_succeeds_with_sufficient_balance() {
    // ... test successful withdrawal
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn test_withdraw_fails_with_insufficient_balance() {
    // ... test withdrawal failure
}
```

### 3. Use Assertions with Descriptive Messages

Include context in assertion messages to aid debugging.

✅ **DO**:

```rust
assert_eq!(
    balance,
    expected_balance,
    "Balance should be {} after transfer, got {}",
    expected_balance,
    balance
);
```

### 4. Keep Tests Focused and Independent

Each test should verify one behavior. Tests should not depend on other tests.

✅ **DO**:

```rust
#[test]
fn test_persistent_storage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("balance");
    let value = 1000u64;

    // Initially, key should not exist
    assert!(!client.has_persistent(&key));

    // Set value
    client.set_persistent(&key, &value);

    // Key should now exist
    assert!(client.has_persistent(&key));

    // Retrieved value should match
    assert_eq!(client.get_persistent(&key), Some(value));
}
```

### 5. Mock Authorization Appropriately

Use `env.mock_all_auths()` for unit tests, but test authorization logic explicitly when needed.

✅ **DO**:

```rust
#[test]
fn test_transfer_logic_with_mocked_auth() {
    let env = Env::default();
    env.mock_all_auths();  // Focus on transfer logic

    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);

    // Test transfer logic
}

#[test]
fn test_transfer_requires_sender_authorization() {
    let env = Env::default();
    // Don't mock auth - test authorization explicitly

    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);

    // Test that unauthorized transfers fail
}
```

### 6. Test Edge Cases

Include tests for boundary conditions and edge cases.

✅ **DO**:

```rust
#[test]
fn test_hello_with_single_character_name() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let name = symbol_short!("A");
    let result = client.hello(&name);

    assert_eq!(result, vec![&env, symbol_short!("Hello"), name]);
}

#[test]
fn test_zero_and_boundary_values() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("boundary");

    // Test zero value
    client.set_persistent(&key, &0);
    assert_eq!(client.get_persistent(&key), Some(0));

    // Test max u64 value
    client.set_persistent(&key, &u64::MAX);
    assert_eq!(client.get_persistent(&key), Some(u64::MAX));
}
```

### 7. Use Fixtures for Common Setup

Create helper functions to reduce test boilerplate.

✅ **DO**:

```rust
fn setup_test_env() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    (env, user1, user2)
}

#[test]
fn test_transfer() {
    let (env, user1, user2) = setup_test_env();
    // ... test logic
}
```

### 8. Test Storage Behavior

Example from Storage Patterns Contract:

```rust
#[test]
fn test_persistent_storage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("balance");
    let value = 1000u64;

    // Initially, key should not exist
    assert!(!client.has_persistent(&key));

    // Set value
    client.set_persistent(&key, &value);

    // Verify set event
    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();
    assert_eq!(topics.len(), 2);
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("persist"));
    assert_eq!(t1, symbol_short!("set"));
    let (d_key, d_value): (Symbol, u64) = <(Symbol, u64)>::try_from_val(&env, &data).unwrap();
    assert_eq!(d_key, key);
    assert_eq!(d_value, value);

    // Key should now exist
    assert!(client.has_persistent(&key));

    // Retrieved value should match
    assert_eq!(client.get_persistent(&key), Some(value));

    // Remove value
    client.remove_persistent(&key);

    // Verify remove event
    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();
    assert_eq!(topics.len(), 2);
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("persist"));
    assert_eq!(t1, symbol_short!("set")); // or "remove" depending on event
    // ... assertions
}
```

## 🧩 Shared Test Utilities

For multi-file test suites, keep reusable setup code in `tests/common/mod.rs`.

```rust
// tests/common/mod.rs
use soroban_sdk::{Address, Env, testutils::Address as _};

pub fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

pub fn test_user(env: &Env) -> Address {
    Address::generate(env)
}
```

## 📸 Snapshot Testing

Snapshot testing helps lock down serialized outputs, event topics/data, and human-readable responses.

```rust
#[test]
fn emits_expected_event_shape() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    client.create_order(&42);

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    // Keep snapshots stable by comparing deterministic debug output.
    let rendered = format!("{events:?}");
    insta::assert_snapshot!(rendered);
}
```

**When to use**:

- Testing complex data structures
- Verifying serialization/deserialization
- Regression testing for output changes
- Testing event emissions

## 📊 Coverage Tools

The repository CI already uses `cargo-tarpaulin` and uploads Cobertura XML to Codecov.

### Cargo Tarpaulin

**Installation**:

```bash
cargo install cargo-tarpaulin --locked
```

**Usage**:

```bash
cargo tarpaulin --out Xml --out Html
```

Reports are written to `coverage/`. Open `coverage/tarpaulin-report.html` in a browser for a line-by-line view of what your tests are (and aren't) hitting.

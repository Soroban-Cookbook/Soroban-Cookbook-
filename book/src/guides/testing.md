# Testing Guide

Comprehensive guide to testing Soroban smart contracts effectively.

## 📖 Overview

Testing is crucial for smart contract development. This guide covers:

- Unit testing individual functions
- Integration testing multi-contract interactions
- Test organization and best practices
- Advanced testing techniques
- Snapshot testing and coverage tools
- Common testing patterns and anti-patterns

## 🧪 Test Types

### Unit Tests

Test individual contract functions in isolation.

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

Test interactions between multiple contracts.

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

## 🏗️ Test Structure

### Recommended Organization

```
src/
├── lib.rs           # Contract code
└── test.rs          # Unit tests

tests/
├── integration.rs   # Integration tests
└── common/
    └── mod.rs       # Shared test utilities
```

### Test Module Pattern

```rust
// In src/lib.rs
#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}

// Keep tests in separate file
#[cfg(test)]
mod test;
```

```rust
// In src/test.rs
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
    assert_eq!(t1, symbol_short!("remove"));
    let d_key: Symbol = Symbol::try_from_val(&env, &data).unwrap();
    assert_eq!(d_key, key);

    // Key should no longer exist
    assert!(!client.has_persistent(&key));
}
```

### 9. Test Error Conditions

```rust
#[test]
#[should_panic(expected = "insufficient balance")]
fn test_insufficient_balance() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TokenContract);
    let client = TokenContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Should panic
    client.transfer(&user, &recipient, &1000);
}
```

### 10. Test Events

```rust
#[test]
fn test_events() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    client.transfer(&from, &to, &100);

    // Get emitted events
    let events = env.events().all();

    // Verify event was emitted
    assert_eq!(events.len(), 1);
    // ... verify event data
}
```

## ⚠️ Common Testing Pitfalls

### 1. Forgetting to Mock Authorization

❌ **DON'T**:

```rust
#[test]
fn test_transfer() {
    let env = Env::default();
    // Forgot env.mock_all_auths()!

    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);

    // This will fail due to missing authorization
    client.transfer(&from, &to, &100);
}
```

✅ **DO**:

```rust
#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();  // Always mock auth for unit tests

    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);

    client.transfer(&from, &to, &100);
}
```

### 2. Not Extending TTL for Persistent Storage

❌ **DON'T**:

```rust
#[test]
fn test_persistent_storage() {
    let env = Env::default();

    let key = symbol_short!("balance");
    env.storage().persistent().set(&key, &1000);
    // Forgot to extend TTL!

    let value: u64 = env.storage().persistent().get(&key).unwrap();
    assert_eq!(value, 1000);
}
```

✅ **DO**:

```rust
#[test]
fn test_persistent_storage() {
    let env = Env::default();

    let key = symbol_short!("balance");
    env.storage().persistent().set(&key, &1000);
    env.storage().persistent().extend_ttl(&key, 100, 100);  // Extend TTL

    let value: u64 = env.storage().persistent().get(&key).unwrap();
    assert_eq!(value, 1000);
}
```

### 3. Testing Multiple Behaviors in One Test

❌ **DON'T**:

```rust
#[test]
fn test_everything() {
    // Tests initialization, transfer, and withdrawal in one test
    // Hard to debug when it fails
}
```

✅ **DO**:

```rust
#[test]
fn test_initialization() {}

#[test]
fn test_transfer() {}

#[test]
fn test_withdrawal() {}
```

### 4. Ignoring Error Cases

❌ **DON'T**:

```rust
#[test]
fn test_transfer() {
    // Only test the happy path
    client.transfer(&from, &to, &100);
    assert_eq!(client.balance(&to), 100);
}
```

✅ **DO**:

```rust
#[test]
fn test_transfer_succeeds() {
    client.transfer(&from, &to, &100);
    assert_eq!(client.balance(&to), 100);
}

#[test]
#[should_panic]
fn test_transfer_fails_with_insufficient_balance() {
    client.transfer(&from, &to, &1000000);
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

```rust
// tests/integration.rs
mod common;

#[test]
fn deposit_updates_balance() {
    let env = common::setup_env();
    let user = common::test_user(&env);
    // ... contract setup + assertions
}
```

This pattern keeps contract behavior assertions focused while avoiding duplicated setup logic.

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

**Tips**:

- Prefer deterministic inputs (fixed timestamps/amounts) before snapshot assertions.
- Snapshot only stable values (avoid non-deterministic IDs unless normalized first).
- Review snapshot diffs in PRs the same way you review code diffs.

Add to dev-dependencies when using `insta`:

```toml
[dev-dependencies]
insta = "1"
```

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

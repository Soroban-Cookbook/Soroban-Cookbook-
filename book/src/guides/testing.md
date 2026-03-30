# Testing Guide

Comprehensive guide to testing Soroban smart contracts effectively.

## 📖 Overview

Testing is crucial for smart contract development. This guide covers:

- Unit testing individual functions
- Integration testing multi-contract interactions
- Test organization and best practices
- Advanced testing techniques

## 🧪 Test Types

### Unit Tests

Test individual contract functions in isolation.

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_single_function() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MyContract);
        let client = MyContractClient::new(&env, &contract_id);

        let result = client.my_function(&42);
        assert_eq!(result, 42);
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

### 1. Test Edge Cases

```rust
#[test]
fn test_edge_cases() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    // Test zero
    assert_eq!(client.divide(&10, &0), Err(...));

    // Test maximum values
    assert_eq!(client.add(&i128::MAX, &1), Err(...));

    // Test negative values
    assert_eq!(client.absolute(&-42), 42);
}
```

### 2. Test Error Conditions

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

### 3. Use Descriptive Test Names

```rust
// Good ✅
#[test]
fn transfer_succeeds_with_sufficient_balance() { }

#[test]
fn transfer_fails_when_balance_insufficient() { }

#[test]
fn transfer_emits_event_on_success() { }

// Bad ❌
#[test]
fn test1() { }

#[test]
fn transfer() { }
```

### 4. Test Storage Behavior

```rust
#[test]
fn test_storage_persistence() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    // Set value
    client.set_value(&42);

    // Verify persistence
    assert_eq!(client.get_value(), 42);

    // Update value
    client.set_value(&100);
    assert_eq!(client.get_value(), 100);
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

Tips:

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

Run coverage locally:

```bash
cargo install cargo-tarpaulin --locked
cargo tarpaulin --workspace --all-features --out xml --output-dir ./coverage --timeout 300
```

Alternative (LLVM-based):

```bash
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Use coverage reports to identify untested error paths, auth branches, and storage edge cases.

### 5. Test Events

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

## 🚀 Running Tests

### Basic Test Run

```bash
cargo test
```

### Run Specific Test

```bash
cargo test test_transfer
```

### Run with Output

```bash
cargo test -- --nocapture
```

### Run with Multiple Threads

```bash
cargo test -- --test-threads=4
```

### Run Integration Tests Only

```bash
cargo test --test integration
```

## 📊 Test Coverage

### Install Tarpaulin (Linux only)

```bash
cargo install cargo-tarpaulin
```

### Generate Coverage Report

```bash
cargo tarpaulin --out Html
```

### View Coverage

```bash
open tarpaulin-report.html
```

## 🐛 Debugging Tests

### Print Debugging

```rust
#[test]
fn debug_test() {
    let env = Env::default();

    // Use env.logs() for debugging
    env.logs().enable();

    // Your test code
    let result = some_function();

    // Check logs
    println!("{:?}", env.logs().all());
}
```

### Test Isolation

Each test runs in isolation - tests don't share state:

```rust
#[test]
fn test_a() {
    // Has its own Env
}

#[test]
fn test_b() {
    // Completely separate Env
}
```

## 📚 Examples

Check out our test examples:

- [Basic Tests](../examples/basics/01-hello-world/src/test.rs)
- [Storage Tests](../examples/basics/02-storage-patterns/src/test.rs)
- [Auth Tests](../examples/basics/03-authentication/src/test.rs)

## 🔗 Resources

- [Soroban Testing Documentation](https://developers.stellar.org/docs/smart-contracts/testing)
- [Rust Testing Best Practices](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [SDK Test Utilities](https://docs.rs/soroban-sdk/latest/soroban_sdk/testutils/)

---

**Next:** Learn about [Deployment](./deployment.md)

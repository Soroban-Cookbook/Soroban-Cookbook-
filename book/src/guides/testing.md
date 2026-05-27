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

### 1. Use Descriptive Test Names

Test names should clearly describe what is being tested and the expected outcome.

✅ **DO:**

```rust
#[test]
fn test_transfer_succeeds_with_sufficient_balance() { }

#[test]
fn test_transfer_fails_with_insufficient_balance() { }

#[test]
fn test_transfer_fails_when_sender_not_authorized() { }
```

❌ **DON'T:**

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

✅ **DO:**

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

✅ **DO:**

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

✅ **DO:**

```rust
#[test]
fn test_increment_increases_counter_by_one() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Counter);
    let client = CounterClient::new(&env, &contract_id);

    assert_eq!(client.value(), 0);
    client.increment();
    assert_eq!(client.value(), 1);
}
```

### 5. Mock Authorization Appropriately

Use `env.mock_all_auths()` for unit tests, but test authorization logic explicitly when needed.

✅ **DO:**

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

✅ **DO:**

```rust
#[test]
fn test_transfer_zero_amount() {
    // Test edge case: zero transfer
}

#[test]
fn test_transfer_max_u64_amount() {
    // Test edge case: maximum value
}

#[test]
fn test_transfer_to_self() {
    // Test edge case: self-transfer
}
```

### 7. Use Fixtures for Common Setup

Create helper functions to reduce test boilerplate.

✅ **DO:**

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

### 8. Test Edge Cases (Existing)

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

### 10. Use Descriptive Test Names (Existing)

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

### 11. Test Storage Behavior

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

### 12. Test Events

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

❌ **DON'T:**

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

✅ **DO:**

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

❌ **DON'T:**

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

✅ **DO:**

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

❌ **DON'T:**

```rust
#[test]
fn test_everything() {
    // Tests initialization, transfer, and withdrawal in one test
    // Hard to debug when it fails
}
```

✅ **DO:**

```rust
#[test]
fn test_initialization() { }

#[test]
fn test_transfer() { }

#[test]
fn test_withdrawal() { }
```

### 4. Ignoring Error Cases

❌ **DON'T:**

```rust
#[test]
fn test_transfer() {
    // Only test the happy path
    client.transfer(&from, &to, &100);
    assert_eq!(client.balance(&to), 100);
}
```

✅ **DO:**

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

**When to use:**

- Testing complex data structures
- Verifying serialization/deserialization
- Regression testing for output changes
- Testing event emissions

**Tips:**

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

**Installation:**

```bash
cargo install cargo-tarpaulin --locked
```

**Usage:**

```bash
# Generate HTML coverage report
cargo tarpaulin --workspace --all-features --out Html --output-dir ./coverage --timeout 300

# Generate Cobertura XML for CI/CD
cargo tarpaulin --workspace --all-features --out Xml --output-dir ./coverage --timeout 300

# Exclude specific files
cargo tarpaulin --exclude-files tests/* --workspace
```

### Alternative: LLVM-based Coverage

```bash
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

### Coverage Goals

- **Minimum:** >80% line coverage
- **Target:** >90% line coverage
- **Ideal:** >95% line coverage

Focus on covering critical paths and error conditions, not just achieving high percentages.

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

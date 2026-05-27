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

---

## 🧪 Test Types

### Unit Tests

Unit tests verify individual contract functions in isolation. They're fast, focused, and ideal for testing business logic.

**When to use:**

- Testing single function behavior
- Verifying state changes
- Testing error conditions
- Validating input validation

**Example:**

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_increment_increases_counter() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, Counter);
        let client = CounterClient::new(&env, &contract_id);

        // Initial state
        assert_eq!(client.value(), 0);

        // Increment
        client.increment();
        assert_eq!(client.value(), 1);
    }
}
```

### Integration Tests

Integration tests verify interactions between multiple contracts or complex workflows involving multiple function calls.

**When to use:**

- Testing multi-contract interactions
- Verifying complex workflows
- Testing cross-contract calls
- Validating state consistency across contracts

**Example:**

```rust
#[test]
fn test_token_transfer_to_vault() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy contracts
    let token_id = env.register_contract(None, Token);
    let vault_id = env.register_contract(None, Vault);

    let token = TokenClient::new(&env, &token_id);
    let vault = VaultClient::new(&env, &vault_id);

    let user = Address::generate(&env);

    // Mint tokens
    token.mint(&user, &1000);
    assert_eq!(token.balance(&user), 1000);

    // Deposit to vault
    vault.deposit(&user, &token_id, &500);
    assert_eq!(vault.balance(&user), 500);
    assert_eq!(token.balance(&user), 500);
}
```

### Snapshot Testing

Snapshot tests capture the output of a function and compare it against a stored snapshot. Useful for testing complex data structures or serialization.

**When to use:**

- Testing complex data structures
- Verifying serialization/deserialization
- Regression testing for output changes
- Testing event emissions

**Example:**

```rust
#[test]
fn test_contract_state_snapshot() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    // Perform operations
    client.initialize(&Address::generate(&env), &1000);

    // Verify state matches expected snapshot
    let state = client.get_state();
    assert_eq!(state.balance, 1000);
    assert_eq!(state.initialized, true);
}
```

---

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

**In `src/lib.rs`:**

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

mod test;
```

**In `src/test.rs`:**

```rust
#![cfg(test)]

use super::*;
use soroban_sdk::Env;

#[test]
fn test_add_positive_numbers() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    assert_eq!(client.add(&5, &3), 8);
}
```

---

## 🛠️ Test Utilities

### Environment Setup

Every test needs an `Env` instance. Use `Env::default()` for most cases.

```rust
let env = Env::default();
```

### Authorization Mocking

Use `env.mock_all_auths()` to bypass authorization checks during testing. This keeps tests focused on business logic.

```rust
#[test]
fn test_transfer_with_auth() {
    let env = Env::default();
    env.mock_all_auths();  // Mock all auth checks

    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.transfer(&from, &to, &100);
    assert_eq!(client.balance(&to), 100);
}
```

### Time Manipulation

Advance ledger time for testing time-dependent logic.

```rust
#[test]
fn test_timelock_expires() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, Timelock);
    let client = TimelockClient::new(&env, &contract_id);

    // Lock funds for 100 ledgers
    client.lock(&100);

    // Advance time
    env.ledger().with_mut(|ledger| {
        ledger.sequence_number = 150;
    });

    // Verify lock has expired
    assert!(client.can_unlock());
}
```

### Storage Testing

Test storage operations directly.

```rust
#[test]
fn test_persistent_storage() {
    let env = Env::default();

    let key = symbol_short!("balance");
    let value: u64 = 1000;

    // Write to persistent storage
    env.storage().persistent().set(&key, &value);
    env.storage().persistent().extend_ttl(&key, 100, 100);

    // Read from persistent storage
    let retrieved: u64 = env.storage().persistent().get(&key).unwrap();
    assert_eq!(retrieved, value);
}
```

### Event Testing

Verify that events are emitted correctly.

```rust
#[test]
fn test_transfer_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.transfer(&from, &to, &100);

    // Verify event was emitted
    let events = env.events().all();
    assert!(!events.is_empty());
}
```

---

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

❌ **DON'T:**

```rust
#[test]
fn test_withdraw() {
    // Only test the happy path
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

❌ **DON'T:**

```rust
assert_eq!(balance, expected_balance);
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

❌ **DON'T:**

```rust
#[test]
fn test_counter_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Counter);
    let client = CounterClient::new(&env, &contract_id);

    // Multiple behaviors in one test
    client.increment();
    client.increment();
    client.decrement();
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

---

## 🧩 Test Utilities Library

### Creating Reusable Test Helpers

For projects with multiple contracts or complex test setups, create a shared test utilities module.

**File: `tests/common/mod.rs`**

```rust
use soroban_sdk::{Address, Env, testutils::Address as _};

/// Standard test environment setup with auth mocking
pub fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

/// Generate a test address
pub fn test_user(env: &Env) -> Address {
    Address::generate(env)
}

/// Setup multiple test users
pub fn test_users(env: &Env, count: usize) -> Vec<Address> {
    (0..count).map(|_| Address::generate(env)).collect()
}

/// Advance ledger time by N seconds
pub fn advance_time(env: &Env, seconds: u64) {
    env.ledger().with_mut(|ledger| {
        ledger.timestamp += seconds;
    });
}

/// Set ledger to specific timestamp
pub fn set_time(env: &Env, timestamp: u64) {
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = timestamp;
    });
}
```

**Usage in tests:**

```rust
mod common;

#[test]
fn test_with_utilities() {
    let env = common::setup_env();
    let users = common::test_users(&env, 3);

    common::advance_time(&env, 100);

    // Test logic here
}
```

### Common Test Fixtures

Create fixtures for frequently-used contract setups:

```rust
pub struct TestFixture {
    pub env: Env,
    pub admin: Address,
    pub user: Address,
    pub contract_id: Address,
}

impl TestFixture {
    pub fn new<T: soroban_sdk::contract::ContractType>() -> Self {
        let env = setup_env();
        let admin = test_user(&env);
        let user = test_user(&env);

        let contract_id = env.register_contract(None, T);

        TestFixture {
            env,
            admin,
            user,
            contract_id,
        }
    }
}

#[test]
fn test_with_fixture() {
    let fixture = TestFixture::new::<MyContract>();

    // Use fixture.env, fixture.admin, fixture.user, fixture.contract_id
}
```

---

## 📸 Snapshot Testing

Snapshot testing helps lock down serialized outputs, event topics/data, and human-readable responses.

**Installation:**

Add to `Cargo.toml`:

```toml
[dev-dependencies]
insta = "1"
```

**Example:**

```rust
#[test]
fn emits_expected_event_shape() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    client.create_order(&42);

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    // Keep snapshots stable by comparing deterministic debug output
    let rendered = format!("{events:?}");
    insta::assert_snapshot!(rendered);
}
```

**When to use:**

- Testing complex data structures
- Verifying serialization/deserialization
- Regression testing for output changes
- Testing event emissions with complex payloads

**Tips:**

- Prefer deterministic inputs (fixed timestamps/amounts) before snapshot assertions
- Snapshot only stable values (avoid non-deterministic IDs unless normalized first)
- Review snapshot diffs in PRs the same way you review code diffs
- Use `insta review` to approve snapshot changes

---

## 🧪 Integration Testing Patterns

### Multi-Contract Testing

```rust
#[test]
fn test_token_vault_integration() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy both contracts
    let token_id = env.register_contract(None, TokenContract);
    let vault_id = env.register_contract(None, VaultContract);

    let token = TokenContractClient::new(&env, &token_id);
    let vault = VaultContractClient::new(&env, &vault_id);

    let user = Address::generate(&env);

    // Test interaction flow
    token.mint(&user, &1000);
    assert_eq!(token.balance(&user), 1000);

    vault.deposit(&user, &token_id, &500);
    assert_eq!(vault.balance(&user), 500);
    assert_eq!(token.balance(&user), 500);
}
```

### Cross-Contract Calls

```rust
#[test]
fn test_cross_contract_authorization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_a_id = env.register_contract(None, ContractA);
    let contract_b_id = env.register_contract(None, ContractB);

    let contract_a = ContractAClient::new(&env, &contract_a_id);
    let contract_b = ContractBClient::new(&env, &contract_b_id);

    let user = Address::generate(&env);

    // Contract A calls Contract B
    contract_a.invoke_contract_b(&contract_b_id, &user, &100);

    // Verify state in both contracts
    assert_eq!(contract_a.get_state(&user), 100);
    assert_eq!(contract_b.get_state(&user), 100);
}
```

---

## 🔍 Advanced Testing Techniques

### Property-Based Testing

For complex logic, consider property-based testing with `proptest`:

```toml
[dev-dependencies]
proptest = "1"
```

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_transfer_preserves_total_supply(
        initial_balance in 0u64..1_000_000,
        transfer_amount in 0u64..1_000_000,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, Token);
        let client = TokenClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.mint(&user1, &initial_balance);
        let total_before = client.total_supply();

        if transfer_amount <= initial_balance {
            client.transfer(&user1, &user2, &transfer_amount);
            let total_after = client.total_supply();

            prop_assert_eq!(total_before, total_after, "Total supply should be preserved");
        }
    }
}
```

### Fuzzing

For security-critical contracts, use fuzzing to find edge cases:

```bash
cargo install cargo-fuzz
cargo fuzz init
cargo fuzz add fuzz_target_1
```

### Benchmarking

Measure contract performance:

```rust
#![feature(test)]
extern crate test;

use test::Bencher;

#[bench]
fn bench_transfer(b: &mut Bencher) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.mint(&from, &1_000_000);

    b.iter(|| {
        client.transfer(&from, &to, &100);
    });
}
```

---

## 📊 Coverage Tools

### Cargo Tarpaulin

Measure code coverage for your contracts.

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

### LLVM-based Coverage

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

---

## 🧩 Test Utilities

### Testing Authorization

```rust
#[test]
fn test_only_admin_can_initialize() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);

    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    // Admin can initialize
    env.mock_auths(&[MockAuth {
        address: admin.clone(),
        nonce: 0,
        sign_args: vec![&env],
    }]);
    client.initialize(&admin);

    // Non-admin cannot initialize
    env.mock_auths(&[MockAuth {
        address: non_admin.clone(),
        nonce: 0,
        sign_args: vec![&env],
    }]);

    // This should panic or return error
    // client.initialize(&non_admin);
}
```

### Testing State Transitions

```rust
#[test]
fn test_contract_state_transitions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StateMachine);
    let client = StateMachineClient::new(&env, &contract_id);

    // Initial state
    assert_eq!(client.state(), State::Idle);

    // Transition to Active
    client.activate();
    assert_eq!(client.state(), State::Active);

    // Transition to Completed
    client.complete();
    assert_eq!(client.state(), State::Completed);
}
```

### Testing Error Conditions

```rust
#[test]
#[should_panic(expected = "insufficient balance")]
fn test_withdraw_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, Vault);
    let client = VaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit 100
    client.deposit(&user, &100);

    // Try to withdraw 200 (should panic)
    client.withdraw(&user, &200);
}
```

### Testing Storage Persistence

```rust
#[test]
fn test_data_persists_across_calls() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, Storage);
    let client = StorageClient::new(&env, &contract_id);

    // Set value
    client.set_value(&42);

    // Retrieve value in same test
    assert_eq!(client.get_value(), 42);

    // In real scenarios, persistence is verified across transactions
}
```

---

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

---

## 🚀 Running Tests

### Run All Tests

```bash
# From repository root
cargo test --workspace

# From specific example
cargo test -p hello-world
```

### Run Specific Test

```bash
# Run single test
cargo test test_transfer

# Run tests matching pattern
cargo test test_transfer_
```

### Run with Output

```bash
# Show println! output
cargo test -- --nocapture

# Show test names as they run
cargo test -- --nocapture --test-threads=1
```

### Generate Coverage Report

```bash
# HTML report
cargo tarpaulin --workspace --out Html --output-dir coverage

# Open in browser
open coverage/index.html
```

---

## ✅ Testing Validation Checklist

Before submitting a PR with new tests, verify:

- [ ] All tests have descriptive names following `test_<function>_<scenario>` pattern
- [ ] Both happy path and error cases are tested
- [ ] Edge cases are covered (zero, max values, self-operations)
- [ ] Authorization is tested appropriately (mocked for unit tests, explicit for auth tests)
- [ ] Storage operations include TTL extensions
- [ ] Events are verified when emitted
- [ ] Test coverage is >90% for new code
- [ ] All tests pass locally: `cargo test --workspace`
- [ ] No Clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all`
- [ ] Tests are independent and don't rely on execution order
- [ ] Assertions include descriptive messages
- [ ] No hardcoded values that should be constants
- [ ] Test utilities are reused where applicable

---

## 📚 Additional Resources

- [Soroban SDK Documentation](https://docs.rs/soroban-sdk/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Tarpaulin](https://github.com/xd009642/tarpaulin)
- [Best Practices](./best-practices.md)
- [Style Guide](./style-guide.md)
- [Soroban Testing Docs](https://developers.stellar.org/docs/smart-contracts/testing)

---

**Last Updated:** April 2025

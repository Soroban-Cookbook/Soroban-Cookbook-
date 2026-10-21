# Testing Pitfalls

Common testing mistakes in Soroban smart contract development and how to avoid them.

---

## Table of Contents

1. [Insufficient Test Coverage](#1-insufficient-test-coverage)
2. [Missing Edge Cases](#2-missing-edge-cases)
3. [Flaky Tests](#3-flaky-tests)
4. [Poor Test Organization](#4-poor-test-organization)
5. [Missing Integration Tests](#5-missing-integration-tests)
6. [Best Practices](#6-best-practices)

---

## 1. Insufficient Test Coverage

### Problem
Tests only cover happy paths, leaving critical error conditions and edge cases untested.

### Examples
- Testing only successful transfers, not insufficient balance cases
- Testing only valid inputs, not boundary values or invalid types
- Testing only authorized calls, not unauthorized access attempts

### Solution
```rust
// BAD: Only testing happy path
#[test]
fn test_transfer() {
    let env = create_test_env();
    env.mock_all_auths();
    
    let (token, alice, bob) = setup_token(&env);
    token.transfer(&alice, &bob, &100);
    
    assert_eq!(token.balance(&alice), 900);
    assert_eq!(token.balance(&bob), 100);
}

// GOOD: Testing multiple scenarios
#[test]
fn test_transfer_happy_path() { /* ... */ }

#[test]
#[should_panic(expected = "Error(Contract, #3)")] // InsufficientBalance error
fn test_transfer_insufficient_balance() {
    let env = create_test_env();
    env.mock_all_auths();
    
    let (token, alice, bob) = setup_token(&env);
    // Try to transfer more than balance
    token.transfer(&alice, &bob, &1000);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")] // Unauthorized error  
fn test_transfer_unauthorized() {
    let env = create_test_env();
    // DON'T mock auths - test that unauthorized calls fail
    let (token, alice, bob) = setup_token(&env);
    token.transfer(&alice, &bob, &100);
}
```

### Checklist
- [ ] Test every error variant in your `#[contracterror]` enum
- [ ] Test boundary values (0, 1, max values)
- [ ] Test invalid input types and values
- [ ] Test both authorized and unauthorized access
- [ ] Use `cargo tarpaulin` to measure coverage (aim for >90%)

---

## 2. Missing Edge Cases

### Problem
Tests don't cover edge cases that can lead to security vulnerabilities or incorrect behavior.

### Common Missing Edge Cases
1. **Integer overflow/underflow**: Operations near `i128::MAX` or `i128::MIN`
2. **Zero values**: Transfers of 0, division by 0
3. **Duplicate operations**: Same operation called twice
4. **Concurrent access**: Multiple users interacting simultaneously
5. **State exhaustion**: Storage limits, TTL expiration

### Solution
```rust
// Test integer overflow
#[test]
#[should_panic(expected = "Error(Contract, #5)")] // ArithmeticOverflow
fn test_transfer_overflow() {
    let env = create_test_env();
    env.mock_all_auths();
    
    let (token, alice, bob) = setup_token(&env);
    // Transfer amount that would cause overflow when added to bob's balance
    token.mint(&bob, &i128::MAX);
    token.transfer(&alice, &bob, &1);
}

// Test zero amount
#[test]
fn test_transfer_zero_amount() {
    let env = create_test_env();
    env.mock_all_auths();
    
    let (token, alice, bob) = setup_token(&env);
    // Should either succeed or fail with specific error, not panic unexpectedly
    token.transfer(&alice, &bob, &0);
}

// Test duplicate operations
#[test]
fn test_double_spend_prevention() {
    let env = create_test_env();
    env.mock_all_auths();
    
    let (token, alice, bob) = setup_token(&env);
    
    // First transfer should succeed
    token.transfer(&alice, &bob, &100);
    
    // Second identical transfer should fail if balance is now insufficient
    // or succeed with reduced amount depending on logic
    let alice_balance = token.balance(&alice);
    if alice_balance >= 100 {
        token.transfer(&alice, &bob, &100);
    } else {
        // Should fail with insufficient balance
    }
}
```

### Checklist
- [ ] Test maximum and minimum integer values
- [ ] Test zero values and empty collections
- [ ] Test duplicate/sequential operations
- [ ] Test TTL expiration and storage limits
- [ ] Test concurrent interactions (simulated with multiple addresses)

---

## 3. Flaky Tests

### Problem
Tests that pass sometimes and fail other times, usually due to non-deterministic behavior.

### Common Causes
1. **Timestamp dependence**: Tests that rely on `env.ledger().timestamp()`
2. **Random values**: Tests using random data without fixed seeds
3. **State leakage**: Tests that don't clean up between runs
4. **Order dependence**: Tests that depend on execution order

### Solution
```rust
// BAD: Flaky due to timestamp
#[test]
fn test_time_sensitive_operation() {
    let env = Env::default();
    env.set_default_info();
    // Ledger timestamp is 0 by default, but could change
    let current_time = env.ledger().timestamp();
    // ... test logic that depends on timestamp
}

// GOOD: Fixed timestamp
#[test]
fn test_time_sensitive_operation() {
    let env = create_test_env();
    env.ledger().with_mut(|li| {
        li.timestamp = 1234567890; // Fixed timestamp
        li.sequence_number = 100; // Fixed sequence
        li.protocol_version = 23; // Fixed protocol version
    });
    
    // Now test is deterministic
    let contract = deploy_contract(&env);
    contract.time_sensitive_function();
}

// BAD: State leakage between tests
#[test]
fn test_user_balance() {
    let env = Env::default(); // Fresh env each test
    // But contract registration persists in some cases
}

// GOOD: Fresh environment per test
fn create_test_env() -> Env {
    let env = Env::default();
    env.set_default_info();
    env.ledger().with_mut(|li| {
        li.protocol_version = 23;
        li.timestamp = 1234567890;
        li.sequence_number = 100;
    });
    env
}
```

### Checklist
- [ ] Use fixed ledger timestamps and sequence numbers
- [ ] Use fixed random seeds if randomness is needed
- [ ] Create fresh `Env` for each test
- [ ] Tests are independent and order-independent
- [ ] No reliance on external state or network

---

## 4. Poor Test Organization

### Problem
Tests are disorganized, making them hard to maintain and understand.

### Symptoms
- Giant test functions testing multiple scenarios
- No clear separation of setup, execution, verification
- Duplicated setup code
- Missing descriptive test names

### Solution
```rust
// BAD: Monolithic test
#[test]
fn test_token() {
    // Setup
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    
    // Deploy token
    let token_id = env.register_stellar_asset_contract(admin.clone());
    let token = token::Client::new(&env, &token_id);
    
    // Test mint
    token.mint(&alice, &1000);
    assert_eq!(token.balance(&alice), 1000);
    
    // Test transfer
    token.transfer(&alice, &bob, &100);
    assert_eq!(token.balance(&alice), 900);
    assert_eq!(token.balance(&bob), 100);
    
    // Test burn
    token.burn(&alice, &50);
    assert_eq!(token.balance(&alice), 850);
    
    // Too many assertions in one test!
}

// GOOD: Organized tests with helper functions
mod token_tests {
    use super::*;
    
    fn setup_token(env: &Env) -> (Address, token::Client, Address, Address) {
        let admin = Address::generate(env);
        let alice = Address::generate(env);
        let bob = Address::generate(env);
        
        let token_id = env.register_stellar_asset_contract(admin.clone());
        let token = token::Client::new(env, &token_id);
        
        (token_id, token, alice, bob)
    }
    
    #[test]
    fn test_mint_tokens() {
        let env = create_test_env();
        env.mock_all_auths();
        
        let (_, token, alice, _) = setup_token(&env);
        token.mint(&alice, &1000);
        
        assert_eq!(token.balance(&alice), 1000);
    }
    
    #[test]
    fn test_transfer_tokens() {
        let env = create_test_env();
        env.mock_all_auths();
        
        let (_, token, alice, bob) = setup_token(&env);
        token.mint(&alice, &1000);
        token.transfer(&alice, &bob, &100);
        
        assert_eq!(token.balance(&alice), 900);
        assert_eq!(token.balance(&bob), 100);
    }
    
    #[test]
    fn test_burn_tokens() {
        let env = create_test_env();
        env.mock_all_auths();
        
        let (_, token, alice, _) = setup_token(&env);
        token.mint(&alice, &1000);
        token.burn(&alice, &50);
        
        assert_eq!(token.balance(&alice), 950);
    }
}
```

### Checklist
- [ ] One test per scenario (not per function)
- [ ] Descriptive test names that indicate what's being tested
- [ ] Helper functions for common setup
- [ ] Clear Arrange-Act-Assert structure
- [ ] Tests are in logical modules/groups

---

## 5. Missing Integration Tests

### Problem
Only unit tests exist, missing tests for cross-contract interactions and end-to-end workflows.

### Impact
- Undetected integration issues
- Missing authorization chain testing
- Unverified event emission across contracts
- Untested upgrade paths

### Solution
```rust
// GOOD: Integration test example
#[test]
fn test_defi_lending_workflow() {
    let env = create_test_env();
    env.mock_all_auths();
    
    // 1. Setup tokens
    let usdc = setup_token(&env, "USDC");
    let eth = setup_token(&env, "ETH");
    
    // 2. Deploy lending pool
    let lending_pool = deploy_lending_pool(&env, &usdc, &eth);
    
    // 3. User deposits collateral
    eth.mint(&alice, &10_000); // 10 ETH
    eth.approve(&alice, &lending_pool.address(), &10_000, &1000);
    lending_pool.deposit_collateral(&alice, &10_000);
    
    // 4. User borrows against collateral
    lending_pool.borrow(&alice, &5_000); // 5000 USDC
    
    // 5. Verify state
    assert_eq!(lending_pool.collateral_balance(&alice), 10_000);
    assert_eq!(lending_pool.debt_balance(&alice), 5_000);
    assert_eq!(usdc.balance(&alice), 5_000);
    
    // 6. Test liquidation scenario
    // Simulate price drop
    oracle.update_price(&eth, &900); // ETH price drops
    lending_pool.liquidate(&liquidator, &alice, &5_000);
    
    // 7. Verify liquidation outcomes
    assert_eq!(lending_pool.collateral_balance(&alice), 5_000); // Half collateral liquidated
    assert_eq!(lending_pool.debt_balance(&alice), 0); // Debt cleared
}
```

### Checklist
- [ ] Cross-contract interaction tests
- [ ] End-to-end workflow tests
- [ ] Authorization chain tests (multi-hop auth)
- [ ] Event emission verification across contracts
- [ ] Upgrade and migration tests
- [ ] Integration tests in `tests/integration/` directory

---

## 6. Best Practices

### Test Structure
1. **Unit tests**: In `#[cfg(test)]` modules within each contract
2. **Integration tests**: In `tests/integration/` directory
3. **Property tests**: Using `proptest` for fuzzing
4. **Security tests**: Specific tests for security vulnerabilities

### Testing Tools
- **cargo test**: Run all tests
- **cargo tarpaulin**: Measure code coverage
- **proptest**: Property-based testing
- **cargo fuzz**: Fuzz testing
- **UPDATE_EXPECT=true**: Update snapshot tests

### Common Test Helpers
```rust
// Standard test environment setup
fn create_test_env() -> Env {
    let env = Env::default();
    env.set_default_info();
    env.ledger().with_mut(|li| {
        li.protocol_version = 23; // Match SDK version
        li.timestamp = 1_000_000;
        li.sequence_number = 100;
    });
    env
}

// Mock auth helper
fn with_mocked_auths<F>(test_fn: F) 
where
    F: FnOnce(&Env),
{
    let env = create_test_env();
    env.mock_all_auths();
    test_fn(&env);
}

// Snapshot testing
#[test]
fn test_contract_snapshot() {
    let env = create_test_env();
    // ... contract operations
    
    // Compare against saved snapshot
    assert_eq!(env.dump_state(), include_str!("../test_snapshots/test_name.json"));
}
```

### Continuous Integration
- Run tests on every PR and push
- Enforce minimum coverage thresholds
- Run security audits as part of CI
- Use snapshot testing to detect regressions

---

## Related Resources

- [Testing Best Practices](./testing-best-practices.md)
- [Security Best Practices](./security-best-practices.md)
- [Common Pitfalls](./common-pitfalls.md)
- [DeFi Security Checklist](./defi-security-checklist.md)

## Examples

See the test suites in the examples directory for practical implementations:
- `examples/basics/`: Unit test patterns
- `tests/integration/`: Integration test patterns  
- `tests/fuzz/`: Fuzz testing examples
- `tests/security/`: Security test patterns


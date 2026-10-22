# Testing Best Practices

Testing is a critical part of developing smart contracts on Soroban. A comprehensive testing strategy ensures that your contracts behave as expected and protects against regressions and security vulnerabilities.

## Unit Test Patterns
- **Test each function in isolation:** Ensure you test the happy path and edge cases.
- **Mock dependencies:** Use `Env::mock_all_auths()` to simplify authorization testing when not explicitly testing `require_auth()`.
- **Test error handling:** Use `#[should_panic(expected = "...")]` to verify that your contracts fail correctly.

## Integration Testing
- **Cross-contract calls:** Test workflows spanning multiple contracts using integration tests.
- **State transitions:** Ensure that sequences of actions transition the contract state correctly.
- **Integration Test Suite:** Keep integration tests in the `tests/integration/` directory, using `env.register_contract()` to load and execute contracts.

## Fuzz Testing
- **Property-based testing:** Fuzz inputs using tools like `proptest` to automatically verify contract invariants across thousands of random inputs.
- **Boundary conditions:** Fuzzers often find edge cases near the limits of integer sizes (`u32::MAX`, `i128::MAX`).

## Coverage Goals
- **Target >90% coverage:** Utilize `cargo tarpaulin` to measure test coverage.
- **Cover all branches:** Ensure that conditional branches are evaluated for both truthy and falsy states.

---

## Worked Examples

Every snippet below is copied from a test that runs in this repository.

### 1. Set up the environment and the contract

```rust
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn deposit_increases_balance() {
    let env = Env::default();
    // Accept every require_auth() in the test.
    env.mock_all_auths();

    // Register the contract natively - no Wasm build required.
    let id = env.register_contract(None, TokenContract);
    let client = TokenContractClient::new(&env, &id);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
}
```

### 2. Assert on typed errors, not panics

```rust
// A contract error is matched exactly.
assert_eq!(
    client.try_transfer(&alice, &bob, &10_000),
    Err(Ok(TokenError::InsufficientBalance))
);

// A missing authorization surfaces as a host error.
assert!(client.try_transfer(&alice, &bob, &10).is_err());
```

### 3. Travel through time

```rust
// Timelocks, vesting and rate limits are all ledger-time driven.
env.ledger().with_mut(|l| l.timestamp += 70);
env.ledger().with_mut(|l| l.sequence_number = 200);

// Or set an absolute value.
env.ledger().set_timestamp(1_000);
```

### 4. Assert on events

```rust
let events = env.events().all();
let (_, topics, data) = events.last().unwrap();

// Topics are indexed; the payload carries the details.
assert_eq!(topics.get(0).unwrap(), symbol_short!("token").into_val(&env));
```

### 5. Test the cross-contract workflow

```rust
// Register both contracts and let them talk to each other.
let amm_id = env.register_contract(None, ConstantProductAmm);
let token_id = env.register_stellar_asset_contract_v2(issuer);
let amm = ConstantProductAmmClient::new(&env, &amm_id);

amm.initialize(&token_id.address(), &token_y.address());
let lp = amm.add_liquidity(&alice, &1_000, &1_000);
assert_eq!(lp, 1_000);
```

### 6. Measure coverage

```bash
# Whole-workspace coverage report (HTML + XML + LCOV).
cargo tarpaulin --out Html

# Only the integration suite.
cargo tarpaulin -p integration-tests --out Html
```

The CI workflow `.github/workflows/test.yml` runs the coverage job and fails
the build when line coverage drops below 90%.

### 7. Fuzz an invariant

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn transfer_never_creates_money(
        balance in 0i128..1_000_000,
        amount in 0i128..1_000_000,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, TokenContract);
        let client = TokenContractClient::new(&env, &id);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        client.initialize(&alice, &balance);
        let _ = client.try_transfer(&alice, &bob, &amount);

        prop_assert!(client.total_supply() == balance);
    }
}
```

---

## Related Pages

- [Testing Guide →](../guides/testing.md)
- [Code Examples →](./code-examples.md)
- [Testing Pitfalls →](./testing-pitfalls.md)
- [Interactive Playground — Intermediate Examples →](../examples/playground-intermediate.md)

# Integration Test Framework

Comprehensive test framework for the Soroban Cookbook.  Provides reusable
helpers, mock contracts, and structured patterns for writing integration tests.

## Architecture

```
tests/integration/
├── Cargo.toml                  # Dependencies on all example contracts
├── build.rs                    # (no-op; native registration, no WASM patching)
├── FRAMEWORK.md                # ← you are here
├── README.md                   # Existing test overview
└── tests/
    ├── integration_tests.rs    # Existing cross-contract tests (12 tests)
    ├── framework_tests.rs      # Framework demo tests (helpers, mocks, governance)
    ├── helpers/
    │   └── mod.rs              # Reusable test utilities and fixtures
    └── mocks/
        └── mod.rs              # Lightweight mock contracts
```

## Modules

### `helpers/` — Test Utilities

| Helper | Purpose |
|--------|---------|
| `setup_env()` | Create fresh `Env` with `mock_all_auths()` |
| `setup_env_with_timestamp(ts)` | Create env at a specific ledger time |
| `generate_addresses(env, n)` | Generate N unique test addresses |
| `invoke_no_args(env, id, fn)` | Invoke a zero-arg contract function |
| `invoke_one_arg(env, id, fn, a)` | Invoke a single-arg contract function |
| `assert_balance(env, id, user, bal)` | Assert auth-contract balance |
| `assert_event_count(env, id, n)` | Assert events-counter value |
| `AuthFixture::setup(env, n, bal)` | Auth contract + N funded users |
| `EventsFixture::setup(env)` | Events counter contract |
| `StorageFixture::setup(env)` | Storage contract with typed helpers |

### `mocks/` — Mock Contracts

| Mock | Purpose |
|------|---------|
| `MockToken` | Minimal mint/transfer/balance token |
| `MockOracle` | Admin-set price feed |
| `MockTimelock` | Time-based lock/unlock |

Mock contracts are lightweight implementations that isolate specific behaviors
without depending on full contract logic.  Use them when testing contracts that
depend on external interfaces (price feeds, tokens, timelocks).

## Setup/Teardown Patterns

Each test function gets its own `Env` — Soroban's test runtime guarantees full
isolation.  For explicit teardown (e.g. verifying cleanup logic), use storage
`remove_*` functions.

```rust
// Setup
let env = helpers::setup_env();
let storage = helpers::StorageFixture::setup(&env);
storage.set_persistent(&env, symbol_short!("key"), 42);

// ... test logic ...

// Teardown
env.invoke_contract::<()>(
    &storage.contract_id,
    &Symbol::new(&env, "remove_persistent"),
    Vec::from_array(&env, [symbol_short!("key").into_val(&env)]),
);
```

## Adding New Tests

1. **Simple unit-style integration:** Add to `integration_tests.rs`.
2. **Framework patterns / governance:** Add to `framework_tests.rs`.
3. **New helper:** Add to `helpers/mod.rs`.
4. **New mock:** Add to `mocks/mod.rs`.

### Template

```rust
#[test]
fn test_my_new_scenario() {
    let env = helpers::setup_env();
    let auth = helpers::AuthFixture::setup(&env, 2, 1000);
    let events = helpers::EventsFixture::setup(&env);

    // ... invoke contracts ...
    events.increment(&env);

    helpers::assert_balance(&env, &auth.contract_id, &auth.users[0], 1000);
    assert_eq!(events.get_count(&env), 1);
}
```

## Running

```bash
# All integration tests (both files)
cargo test -p integration-tests

# Only framework tests
cargo test -p integration-tests --test framework_tests

# Specific test
cargo test -p integration-tests test_governance_voting_lifecycle
```

## CI

The `test.yml` workflow runs `cargo test --workspace` which includes this crate.
All tests use native contract registration (no WASM binaries needed at test time).

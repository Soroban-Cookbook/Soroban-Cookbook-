# Error Handling

Demonstrates proper error handling in Soroban smart contracts: when to use `Result<T, Error>` and when `panic!` is the right choice.

## Overview

This contract compares two approaches to failure handling side-by-side:

- **`Result<T, Error>`** — the preferred pattern for expected, recoverable failures such as invalid user input or insufficient funds. Cheaper in gas (no stack unwinding) and composable with Rust's `?` operator.
- **`panic!`** — appropriate only for invariant violations and unreachable code paths where continuing execution would leave the contract in a corrupt state.

Understanding the difference is foundational to writing production-quality Soroban contracts.

## Project Structure

```text
05-error-handling/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs      # contract and error definitions
    └── test.rs     # unit tests across five test categories
```

## Key Concepts

### `#[contracterror]`

The `#[contracterror]` attribute transforms a plain Rust enum into a Soroban-compatible error type. Each variant maps to a stable `u32` code surfaced to callers across the host–guest boundary.

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    InvalidAmount       = 1,
    InsufficientBalance = 2,
    Unauthorized        = 3,
}
```

Rules when defining contract errors:

- Always use `#[repr(u32)]`. The host encodes errors as `u32` values.
- Assign explicit discriminants starting at `1`. Zero is reserved by the host.
- Never change or reuse a discriminant after deployment. Callers and tooling depend on stable codes across upgrades.
- Derive `Copy + Clone + Eq + PartialEq` so errors can be compared in tests and match arms.

### `Result<T, Error>`

Any contract function that can fail for an expected reason should return `Result<T, Error>`. The host propagates the error code to the caller; the caller can branch on it or convert it to a host trap with `.unwrap()`.

## Code Walkthrough

### `transfer` — Result for validation failures

```rust
pub fn transfer(amount: u64, balance: u64) -> Result<u64, Error> {
    if amount == 0 {
        return Err(Error::InvalidAmount);
    }
    if amount > balance {
        return Err(Error::InsufficientBalance);
    }
    Ok(balance - amount)
}
```

Both failure conditions are expected user errors. Returning `Err` lets the caller decide how to respond without wasting gas on an unwinding panic.

### `transfer_panic` — panic as an anti-pattern

```rust
pub fn transfer_panic(amount: u64, balance: u64) -> u64 {
    if amount == 0 {
        panic!("invalid amount");
    }
    if amount > balance {
        panic!("insufficient balance");
    }
    balance - amount
}
```

Included to illustrate what **not** to do. Panicking on user input aborts the transaction and wastes all gas consumed up to that point. The caller has no path to handle the failure.

### `get_verified_state` — panic for invariant violations

```rust
pub fn get_verified_state(env: Env, key: u32) -> u64 {
    let value: u64 = env.storage().instance().get(&key).unwrap_or(0);
    // Invariant: value must be <= 1000 (enforced by all setters)
    if value > 1000 {
        panic!("invariant violated: state corrupted");
    }
    value
}
```

If storage holds a value above `1000`, every setter that ran previously violated the contract invariant — the contract is in an unrecoverable state. Panicking here is correct: it halts execution and signals that intervention is required.

### `divide` — Result for expected arithmetic errors

```rust
pub fn divide(a: i128, b: i128) -> Result<i128, Error> {
    if b == 0 {
        return Err(Error::InvalidAmount);
    }
    Ok(a / b)
}
```

Division by zero is a foreseeable user error, not a bug. Returning `Err` keeps the transaction alive and gives callers a recoverable path.

## Best Practices

| Scenario | Pattern | Reason |
| -------- | ------- | ------ |
| User supplies `amount = 0` | `Result` | Expected validation failure; caller should handle it |
| User supplies `amount > balance` | `Result` | Business logic error; recoverable |
| Internal storage value violates an invariant | `panic!` | Unrecoverable state; must abort |
| Unreachable branch hit at runtime | `unreachable!()` | Should never happen; signals a bug |
| Division by zero from user input | `Result` | Expected; caller can retry |

**Use `Result` when:**

- The failure is caused by user input or external state.
- The caller can meaningfully recover or retry.
- You want to preserve remaining gas for the rest of the transaction.

**Use `panic!` when:**

- An internal invariant has been violated.
- Continuing execution would produce incorrect results or corrupt state.
- The branch is logically unreachable.

## Custom Error Guide

### Defining error variants

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    InvalidAmount       = 1,  // zero or negative input
    InsufficientBalance = 2,  // not enough funds
    Unauthorized        = 3,  // caller lacks permission
}
```

### Returning errors from functions

```rust
pub fn withdraw(amount: u64, balance: u64, caller: Address, admin: Address)
    -> Result<u64, Error>
{
    if caller != admin {
        return Err(Error::Unauthorized);
    }
    if amount == 0 {
        return Err(Error::InvalidAmount);
    }
    if amount > balance {
        return Err(Error::InsufficientBalance);
    }
    Ok(balance - amount)
}
```

### Propagating errors with `?`

In functions that return `Result`, use `?` to forward an inner error to the caller without an explicit `match`:

```rust
pub fn transfer_and_divide(amount: u64, balance: u64, divisor: i128)
    -> Result<i128, Error>
{
    let remaining = transfer(amount, balance)?;
    let result    = divide(remaining as i128, divisor)?;
    Ok(result)
}
```

### Keeping error codes stable across upgrades

Once a variant and its discriminant are deployed, they are part of your public API. Indexers, client SDKs, and on-chain callers reference codes by number. **Never** renumber, remove, or reuse a discriminant. Add new variants at the end:

```rust
// v1: codes 1 and 2 are fixed for all future versions
pub enum Error {
    InvalidAmount       = 1,
    InsufficientBalance = 2,
    Unauthorized        = 3,  // added in v2 — safe, new code
}
```

## Testing Errors Guide

`src/test.rs` is organised into five sections that together cover all error handling scenarios.

### 1. Happy path — verify success cases

```rust
#[test]
fn test_transfer_success() {
    assert_eq!(ErrorHandlingContract::transfer(50, 100), Ok(50));
}

#[test]
fn test_divide_success() {
    assert_eq!(ErrorHandlingContract::divide(10, 2), Ok(5));
}
```

### 2. Error cases — assert the correct `Err` variant

```rust
#[test]
fn test_transfer_invalid_amount_zero() {
    assert_eq!(
        ErrorHandlingContract::transfer(0, 100),
        Err(Error::InvalidAmount)
    );
}

#[test]
fn test_divide_by_zero() {
    assert_eq!(
        ErrorHandlingContract::divide(10, 0),
        Err(Error::InvalidAmount)
    );
}
```

### 3. Error type verification — confirm variant identity and stable `u32` code

```rust
#[test]
fn test_error_type_invalid_amount() {
    let result = ErrorHandlingContract::transfer(0, 100);
    match result {
        Err(Error::InvalidAmount) => assert_eq!(Error::InvalidAmount as u32, 1),
        _ => panic!("Expected InvalidAmount error"),
    }
}
```

### 4. Recovery patterns — handle errors without aborting

```rust
// match — explicit branch per variant
let handled = match ErrorHandlingContract::transfer(0, 100) {
    Ok(new_balance)                 => new_balance,
    Err(Error::InvalidAmount)       => 100,  // keep original balance
    Err(Error::InsufficientBalance) => 0,
    Err(_)                          => 50,   // fallback
};

// unwrap_or — supply a default on any error
let balance = ErrorHandlingContract::transfer(150, 100).unwrap_or(0);

// cascading — chain two fallible operations
let result = match ErrorHandlingContract::transfer(50, 100) {
    Ok(balance) => ErrorHandlingContract::divide(balance as i128, 2),
    Err(_)      => Ok(25),
};
```

### 5. Panic tests — assert expected panics with `#[should_panic]`

```rust
#[test]
#[should_panic(expected = "invalid amount")]
fn test_transfer_panic_invalid() {
    ErrorHandlingContract::transfer_panic(0, 100);
}

#[test]
#[should_panic(expected = "invariant violated")]
fn test_get_verified_state_corrupted() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ErrorHandlingContract);
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&1u32, &2000u64);
    });
    let client = ErrorHandlingContractClient::new(&env, &contract_id);
    client.get_verified_state(&1);
}
```

## Real-World Patterns

### Validation wrapper

Validate inputs before invoking the contract to avoid paying gas on a predictably failed transaction:

```rust
fn safe_transfer(amount: u64, balance: u64) -> Result<u64, Error> {
    if amount == 0 {
        return Err(Error::InvalidAmount);
    }
    if amount > balance {
        return Err(Error::InsufficientBalance);
    }
    ErrorHandlingContract::transfer(amount, balance)
}
```

### Cascading operations with `?`

Chain multiple fallible steps; the first `Err` short-circuits the rest:

```rust
fn withdraw_and_split(amount: u64, balance: u64) -> Result<(i128, i128), Error> {
    let remaining = ErrorHandlingContract::transfer(amount, balance)?;
    let half      = ErrorHandlingContract::divide(remaining as i128, 2)?;
    Ok((half, remaining as i128 - half))
}
```

### `unwrap_or` for safe defaults

When a missing storage value is acceptable, use `unwrap_or` instead of panicking:

```rust
let value: u64 = env.storage().instance().get(&key).unwrap_or(0);
```

## Build

```bash
# From this directory
cargo build --target wasm32-unknown-unknown --release

# Or from the repository root
cargo build -p soroban-error-handling-example --target wasm32-unknown-unknown --release
```

## Test

```bash
# From this directory
cargo test

# Or from the repository root
cargo test -p soroban-error-handling-example
```

Tests in `src/test.rs`:

| Section | Tests |
| ------- | ----- |
| Happy path | `test_transfer_success`, `test_transfer_full_amount`, `test_divide_success`, `test_get_verified_state_valid` |
| Error cases | `test_transfer_invalid_amount_zero`, `test_transfer_insufficient_balance`, `test_divide_by_zero` |
| Error type verification | `test_error_type_invalid_amount`, `test_error_type_insufficient_balance`, `test_error_equality` |
| Recovery patterns | `test_error_handling_with_match`, `test_error_handling_with_unwrap_or`, `test_cascading_error_handling` |
| Panic tests | `test_transfer_panic_invalid`, `test_transfer_panic_insufficient`, `test_get_verified_state_corrupted` |

## Next Steps

- [03-custom-errors](../03-custom-errors/) — rate-limiting and more complex error hierarchies
- [06-validation-patterns](../06-validation-patterns/) — precondition checks and overflow-safe arithmetic
- [docs/best-practices.md](../../../../docs/best-practices.md) — full error-handling best-practices reference

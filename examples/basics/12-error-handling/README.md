# Error Handling Example

This example demonstrates foundational error handling patterns in Soroban, focusing on the `Result<T, E>` pattern for recoverable errors and `panic!` for irrecoverable invariants.

## Project Structure

```text
examples/basics/12-error-handling/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    └── test.rs
```

## What This Example Shows

- Defining custom error enums with `#[contracterror]`
- Using `Result<T, Error>` return types for fallible operations
- Error code enumeration with explicit `u32` representations
- Testing both success and error cases
- Client-side error handling with `try_*` methods

## Key Concepts

### Custom Error Types

The contract defines a custom error enum using the `#[contracterror]` attribute:

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ZeroInput = 1,
    Overflow = 2,
    Unauthorized = 3,
}
```

### Result-Based Functions

Contract functions return `Result<T, Error>` to handle failures gracefully:

```rust
pub fn divide(a: i128, b: i128) -> Result<i128, Error> {
    if b == 0 {
        return Err(Error::ZeroInput);
    }
    Ok(a / b)
}
```

### Testing Errors

The test suite demonstrates both success and error scenarios:

- `test_divide_success()` - validates successful execution
- `test_client_try_divide_error()` - validates error handling using `try_*` client methods

## Build

From repository root:

```bash
cargo build -p error-handling
```

Or from this directory:

```bash
cargo build
```

## Test

```bash
cargo test -p error-handling
```

## Learn More

- [Soroban Error Handling Documentation](https://soroban.stellar.org/docs/learn/errors)
- [Custom Error Types](https://soroban.stellar.org/docs/learn/errors#custom-errors)

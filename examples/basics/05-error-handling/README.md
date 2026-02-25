# Error Handling

This example demonstrates proper error handling patterns in Soroban smart contracts using custom error types and the Result pattern.

## Project Structure

```text
examples/basics/05-error-handling/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    └── test.rs
```

## What This Example Shows

- Defining custom error types with `#[contracterror]`
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
    LimitExceeded = 1,
}
```

### Result-Based Functions

Contract functions return `Result<T, Error>` to handle failures gracefully:

```rust
pub fn hello(env: Env, count: u32) -> Result<Symbol, Error> {
    if count > 10 {
        return Err(Error::LimitExceeded);
    }
    Ok(symbol_short!("Hello"))
}
```

### Testing Errors

The test suite demonstrates both success and error scenarios:

- `test_hello()` - validates successful execution
- `test_hello_error()` - validates error handling using `try_*` client methods

## Build

From repository root:

```bash
cargo build -p soroban-error-handling-example
```

Or from this directory:

```bash
cargo build
```

## Test

```bash
cargo test -p soroban-error-handling-example
```

## Learn More

- [Soroban Error Handling Documentation](https://soroban.stellar.org/docs/learn/errors)
- [Custom Error Types](https://soroban.stellar.org/docs/learn/errors#custom-errors)

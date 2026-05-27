# Soroban Cookbook Style Guide

This guide outlines the coding standards and best practices for the Soroban Cookbook. Adhering to these guidelines ensures consistency, readability, and high quality across all examples.

## 1. Naming Conventions

Follow standard [Rust naming conventions](https://rust-lang.github.io/api-guidelines/naming.html) with Soroban-specific refinements:

### Contracts and Types
- **Contract Structs**: `PascalCase` (e.g., `HelloContract`, `LiquidityPool`).
- **Custom Types/Enums**: `PascalCase` (e.g., `DataKey`, `ContractError`).
- **Traits**: `PascalCase` (e.g., `VestingInterface`).

### Functions and Variables
- **Contract Functions**: `snake_case` (e.g., `get_balance`, `initialize`).
- **Internal Variables**: `snake_case` (e.g., `user_address`, `token_amount`).
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_SUPPLY`).

### Events and Symbols
- **Event Tags**: `snake_case` (e.g., `transfer`, `mint`). Usually defined using `symbol_short!`.
- **Storage Keys**: Descriptive names using `Symbol` or `DataKey` enums.

---

## 2. Documentation Standards

Documentation is critical for a cookbook. Every example must be self-explanatory.

### Module-Level Documentation (`//!`)
Every `lib.rs` must start with module-level docs:
- **Title**: Clear name of the example.
- **Description**: What the contract does and why it matters.
- **Key Concepts**: List of Soroban features demonstrated (e.g., "Instance Storage", "Custom Errors").
- **Design Decisions**: Explanation of *why* specific patterns were used (e.g., "Using `Persistent` storage for user balances to prevent expiration").

### Function-Level Documentation (`///`)
Public contract functions must include:
- **Summary**: A single line describing the action.
- **Arguments**: Detailed list of parameters.
- **Returns**: Explanation of the return value.
- **Errors**: List of possible `contracterror` variants the function might return.
- **Example**: A short snippet or description of how to call the function.

### Example `README.md` Structure
Every example folder must contain a `README.md` with:
1. **Overview**: Brief description.
2. **Key Soroban Patterns**: Technical highlights.
3. **Usage**: Step-by-step commands to build and test.
4. **Code Deep Dive**: Explanation of the most important logic blocks.

---

## 3. Testing Standards

High-quality examples require robust testing.

### Structure
- **Unit Tests**: Place in a dedicated `src/test.rs` file.
- **Integration Tests**: Place in the workspace `tests/` directory if they involve multiple contracts.
- **Separation**: Use `#[cfg(test)] mod test;` in `lib.rs` to keep the testing logic separate from implementation.

### Best Practices
- **Naming**: Use descriptive test names: `test_transfer_insufficient_funds_fails`.
- **Coverage**: Aim for 100% path coverage for business logic.
- **Mocking Auth**: Always use `env.mock_all_auths()` for testing authorization-protected functions.
- **Assertions**: Use `assert_eq!`, `assert!`, and `expect_err` to verify state and failures.

---

## 4. Clippy and Lints

Clean code is enforced through Clippy and standard Rust lints.

### Mandatory Directives
```rust
#![no_std] // All contracts must be no_std
```

### Recommended Lints
In CI, we run:
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Developer Flow
1. Run `cargo fmt` before every commit.
2. Ensure `cargo clippy` has zero warnings.
3. Use `cargo fix` for automated improvements where possible.

---

## 5. Soroban Specifics

### Error Handling
- Use `#[contracterror]` for all user-facing errors.
- Prefer `Result<T, ContractError>` return types over `panic!`.
- Assign explicit error codes (e.g., `InvalidAmount = 1`).

### Storage Optimization
- **Instance**: Use for configuration and contract metadata.
- **Persistent**: Use for user data and balances.
- **Temporary**: Use for short-lived state like nonces or temporary locks.
- **TTL**: Always consider the Time-To-Live for storage and include bump logic where necessary.

### Best Practices
- **Type Safety**: Use custom enums for storage keys (`DataKey`) rather than raw symbols.
- **Events**: Emit events for all significant state changes.
- **Validation**: Validate all inputs at the entry point of the function.
### Best Practices Reference
For security-specific patterns and performance optimization, refer to the [Best Practices Guide](./best-practices.md). The Style Guide and Best Practices Guide together form the full standard for Soroban Cookbook contributions.

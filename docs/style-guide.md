Markdown
# Soroban Cookbook Style Guide

This document outlines the coding standards for smart contract examples in the Soroban Cookbook. Adhering to these styles ensures that our "recipes" remain readable, consistent, and educational for the entire community.

---

## 1. Naming Conventions 🏷️
We follow standard [Rust naming conventions](https://rust-lang.github.io/api-guidelines/naming.html) to ensure code remains idiomatic:

| Category | Convention | Example |
| :--- | :--- | :--- |
| **Contracts / Structs** | PascalCase | `pub struct LiquidityPool;` |
| **Functions / Methods** | snake_case | `pub fn swap_tokens(...)` |
| **Variables / Fields** | snake_case | `let user_balance = ...` |
| **Constants** | SCREAMING_SNAKE_CASE | `const MAX_SUPPLY: u64 = 100;` |
| **Enums / Errors** | PascalCase | `enum ContractError { ... }` |

## 2. Documentation Standards 📚
Every example is a teaching tool. Documentation is mandatory for clarity.

* **Top-level Docs**: Every contract file must start with `//!` or `///` explaining the recipe's purpose.
* **Function Comments**: Use `///` to document public functions, specifically noting inputs and return values.
* **Complex Logic**: Use `//` within functions to explain "the why" behind specific Soroban SDK calls.
* **No-Std**: All contract entries must begin with `#![no_std]` to remain compatible with the WASM environment.

## 3. Testing Standards 🧪
A recipe isn't finished until it's tested and verified.

* **Module Placement**: Use a `mod test { ... }` block at the bottom of your `lib.rs` or a dedicated `test.rs` file.
* **Auth Testing**: Use `env.mock_all_auths()` for standard authorization tests to keep the code concise and focused on the recipe logic.
* **Assertions**: Provide descriptive messages in assertions: `assert_eq!(val, 10, "Value should be 10 after increment")`.

## 4. Clippy & Quality Rules 🛠️
We enforce zero-warning code to maintain production standards. All examples must pass:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
Error Handling: Prefer panic! with descriptive messages or ContractError over unwrap().

Storage Optimization: Minimize storage writes. Cache data in local variables if accessed multiple times in one transaction.
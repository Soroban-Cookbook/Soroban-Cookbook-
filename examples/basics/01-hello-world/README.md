# Hello World

The simplest possible Soroban smart contract. Use this as a starting template or to understand the fundamental building blocks of every Soroban contract.

## Overview

This contract exposes a single `hello(to: Symbol) -> Vec<Symbol>` function that returns `["Hello", <to>]`. Its purpose is to demonstrate contract structure, not business logic.

## Key Concepts

- **`#[contract]`** — marks a plain unit struct as a Soroban contract; the host uses this to route invocations.
- **`#[contractimpl]`** — exposes public methods as callable contract functions.
- **`Env`** — the execution environment injected by the host; required as the first parameter of every contract function.
- **`Symbol`** — the most gas-efficient type for short identifiers (≤ 32 alphanumeric characters). Prefer it over `String` for names and labels.
- **`Vec<Symbol>`** — idiomatic return type for multiple values; allocated in host memory via the `vec![&env, …]` macro.
- **`#![no_std]`** — Soroban contracts run in a `no_std` Wasm sandbox; the standard library is not available.

## Code Walkthrough

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};

#[contract]
pub struct HelloContract;          // (1) plain unit struct — no fields needed

#[contractimpl]
impl HelloContract {
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {   // (2) Env is always first
        vec![&env, symbol_short!("Hello"), to]            // (3) host-allocated Vec
    }
}
```

1. `#[contract]` registers `HelloContract` with the Soroban host.
2. Every contract function receives `Env` as its first argument — it provides access to storage, events, auth, and more.
3. `vec![&env, …]` is the Soroban equivalent of `std::vec![]`. The `&env` borrow is required because the vector lives in host memory.
4. `symbol_short!("Hello")` compiles the string literal into a compact `Symbol` at compile time (no runtime allocation).

## Build

```bash
# From this directory
cargo build --target wasm32-unknown-unknown --release

# Or from the repository root
cargo build -p hello-world --target wasm32-unknown-unknown --release
```

The compiled `.wasm` file is written to `target/wasm32-unknown-unknown/release/hello_world.wasm`.

## Test

```bash
# From this directory
cargo test

# Or from the repository root
cargo test -p hello-world
```

Tests live in `src/test.rs` and cover:

| Test | What it checks |
| ---- | -------------- |
| `test_hello_returns_greeting_vec` | Full output equals `["Hello", "World"]` |
| `test_hello_first_element_is_hello` | First element is always the literal `"Hello"` |
| `test_hello_second_element_is_name` | Second element echoes the supplied name |
| `test_hello_with_different_names` | Works correctly for multiple different inputs |

## Project Structure

```
01-hello-world/
├── Cargo.toml       # crate manifest (cdylib + soroban-sdk dependency)
├── README.md
└── src/
    ├── lib.rs       # contract definition
    └── test.rs      # unit tests
```

## Common Pitfalls

### Missing `&env` in `vec![]`

```rust
// Wrong — does not compile
vec![symbol_short!("Hello"), to]

// Correct — &env is always the first argument
vec![&env, symbol_short!("Hello"), to]
```

The `vec!` macro allocates in host memory and requires `&env` as its first argument. Forgetting it produces a compile-time error.

### Using `std` types or macros

```rust
// Wrong — std is not available in no_std Wasm
let s = format!("Hello, {}!", to);
let v = std::vec!["Hello"];

// Correct — use Soroban SDK equivalents
let v = vec![&env, symbol_short!("Hello"), to];
```

`#![no_std]` is mandatory for all Soroban contracts. The Rust standard library is not available in the Wasm sandbox — use Soroban SDK types (`Symbol`, `Vec`, `String`, `Bytes`) instead.

### Adding state fields to the contract struct

```rust
// Wrong — struct fields are not contract storage
#[contract]
pub struct HelloContract {
    greeting: Symbol,  // this field is never persisted on-chain
}

// Correct — use env.storage() for any state that must survive calls
#[contract]
pub struct HelloContract;
```

The contract struct is a zero-sized marker type. On-chain state must be written and read through `env.storage()`, not as struct fields.

## Next Steps

- [02-storage-patterns](../02-storage-patterns/) — persist data between invocations
- [03-authentication](../03-authentication/) — restrict who can call your functions
- [04-events](../04-events/) — emit events for off-chain consumers

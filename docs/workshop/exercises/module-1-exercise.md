# Module 1 Exercise: Environment Setup & Hello World

**Objective:** Set up your Soroban development environment and run your first contract.

**Duration:** 30 minutes

---

## Prerequisites

- A Unix-like terminal (macOS, Linux, or WSL2)
- Internet access for package downloads

---

## Part 1: Install the Toolchain (15 min)

### 1.1 Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version    # should print rustc 1.74+
cargo --version
```

### 1.2 Add WebAssembly Target

```bash
rustup target add wasm32-unknown-unknown
rustup target list --installed | grep wasm32
```

### 1.3 Install Stellar CLI

```bash
cargo install --locked stellar-cli --features opt
stellar --version
```

> **Note:** If you have the older `soroban-cli` installed, uninstall it first with `cargo uninstall soroban-cli`.

---

## Part 2: Run the Hello World Contract (10 min)

### 2.1 Clone and Navigate

```bash
git clone https://github.com/Stellar-Cookbook/Soroban-Cookbook.git
cd Soroban-Cookbook
```

### 2.2 Run the Hello World Tests

```bash
cd examples/basics/01-hello-world
cargo test
```

You should see output like:

```
running 7 tests
test test::test_hello_returns_greeting_vec ... ok
test test::test_hello_first_element_is_hello ... ok
test test::test_hello_second_element_is_name ... ok
...
test result: ok. 7 passed
```

### 2.3 Build the WASM

```bash
cargo build --target wasm32-unknown-unknown --release
```

Verify the WASM file was created:

```bash
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

---

## Part 3: Modify the Contract (5 min)

### 3.1 Open the Contract

Open `examples/basics/01-hello-world/src/lib.rs` in your editor.

### 3.2 Add a Greeting with String

Add a new function that returns a `String` instead of `Vec<Symbol>`:

```rust
use soroban_sdk::{String};

// Add this to the impl block:
pub fn greet(env: Env, name: String) -> String {
    String::from_str(&env, "Hello, ");
    // Hint: you'll need to concatenate strings
    // Look at examples/basics/08-soroban-types for string manipulation
}
```

> **Challenge:** Can you make it return "Hello, {name}!"? Check `examples/basics/08-soroban-types/src/lib.rs` for string concatenation patterns.

### 3.3 Write a Test

Add a test for your new function in `src/test.rs`:

```rust
#[test]
fn test_greet() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let name = String::from_str(&env, "Alice");
    let result = client.greet(&name);

    // Adjust this assertion based on your implementation
    assert!(result.to_string().contains("Alice"));
}
```

### 3.4 Run Your Test

```bash
cargo test test_greet
```

---

## Verification Checklist

- [ ] `rustc --version` shows 1.74+
- [ ] `rustup target list --installed` includes wasm32-unknown-unknown
- [ ] `stellar --version` prints a version number
- [ ] `cargo test` passes in the hello-world example
- [ ] `cargo build --target wasm32-unknown-unknown --release` produces a .wasm file
- [ ] You added a new function and test

---

## Solution

The Hello World contract as-is is the reference solution for parts 1 and 2. For Part 3, here's a working implementation:

```rust
pub fn greet(env: Env, name: String) -> String {
    let prefix = String::from_str(&env, "Hello, ");
    let suffix = String::from_str(&env, "!");
    prefix.append(&name).append(&suffix)
}
```

Note: `String::append()` returns a new `String`. Check the `soroban_sdk::String` docs for available methods.

---

## Next Steps

Continue to [Module 2: Storage & Data Types](../slides/module-2-storage.md) to learn about Soroban's storage system.

# Troubleshooting Guide

Common issues encountered when building, testing, and deploying Soroban smart contracts, with solutions and workarounds.

---

## Build Errors

### `error[E0463]: can't find crate for 'std'`

**Cause:** The contract crate is missing `#![no_std]` or the WASM target is not installed.

**Solution:**

1. Add `#![no_std]` at the top of `src/lib.rs`.
2. Install the WASM target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. Build with the correct target:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

---

### `error: the 'wasm32-unknown-unknown' target may not be installed`

**Cause:** The WASM target is missing from the active Rust toolchain.

**Solution:**
```bash
rustup target add wasm32-unknown-unknown
rustup target list --installed   # verify
```

---

### `error[E0277]: the trait bound ... is not satisfied` on a contract type

**Cause:** A type used in a contract function signature or storage does not implement the required Soroban traits (`IntoVal`, `TryFromVal`, etc.).

**Solution:** Annotate the type with `#[contracttype]`:

```rust
use soroban_sdk::contracttype;

#[contracttype]
pub struct MyData {
    pub value: i128,
    pub owner: soroban_sdk::Address,
}
```

Custom enums used as storage keys or return values also need `#[contracttype]`.

---

### `error: proc-macro derive panicked` / `#[contract]` or `#[contractimpl]` errors

**Cause:** Mismatched `soroban-sdk` versions between `[dependencies]` and `[dev-dependencies]`, or a missing `crate-type`.

**Solution:**

1. Ensure `Cargo.toml` has:
   ```toml
   [lib]
   crate-type = ["cdylib", "rlib"]
   ```
2. Pin both dependency entries to the same version:
   ```toml
   [dependencies]
   soroban-sdk = "21.7.0"

   [dev-dependencies]
   soroban-sdk = { version = "21.7.0", features = ["testutils"] }
   ```

---

### `error: linking with 'cc' failed` or linker errors on WASM build

**Cause:** The system linker is being invoked instead of the WASM linker, or `lld` is missing.

**Solution:** Add a `.cargo/config.toml` at the workspace root:

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=+multivalue"]
```

Or install `lld` via your system package manager and ensure it is on `PATH`.

---

### `overflow evaluating the requirement` / stack overflow during compilation

**Cause:** Deeply nested `#[contracttype]` structs or recursive types.

**Solution:** Break the type into smaller, non-recursive pieces. Soroban does not support recursive contract types.

---

## Test Failures

### `called 'Option::unwrap()' on a 'None' value` in a test

**Cause:** A storage key was never written before being read, or the wrong key was used.

**Solution:** Ensure the contract is initialized before calling read methods:

```rust
client.initialize(&admin);   // write state first
let val = client.get_value(); // then read
```

Use `has()` to guard optional reads:

```rust
if env.storage().instance().has(&key) {
    let val: MyType = env.storage().instance().get(&key).unwrap();
}
```

---

### `HostError: Error(Auth, InvalidAction)` in tests

**Cause:** An `Address::require_auth()` call was not satisfied. The test did not mock authorization.

**Solution:** Add `env.mock_all_auths()` before invoking the client:

```rust
let env = Env::default();
env.mock_all_auths();
```

To assert that specific auth was required:

```rust
use soroban_sdk::testutils::AuthorizedFunction;

let auths = env.auths();
assert!(!auths.is_empty());
```

---

### `HostError: Error(WasmVm, ...)` or `unreachable` panic in tests

**Cause:** A `panic!` inside the contract was triggered (e.g., overflow, out-of-bounds, explicit guard).

**Solution:** Use `#[should_panic]` for expected panics, or return `Result<_, ContractError>` instead of panicking:

```rust
#[test]
#[should_panic(expected = "overflow")]
fn test_overflow() {
    // ...
}
```

For production contracts, prefer returning errors over panicking:

```rust
pub fn safe_transfer(env: Env, amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    Ok(())
}
```

---

### Tests pass locally but fail in CI

**Cause:** Toolchain version mismatch, missing `rust-toolchain.toml`, or non-deterministic test ordering.

**Solution:**

1. Pin the toolchain in `rust-toolchain.toml`:
   ```toml
   [toolchain]
   channel = "stable"
   targets = ["wasm32-unknown-unknown"]
   ```
2. Run the same checks locally that CI runs:
   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   ```
3. Check the CI workflow file (`.github/workflows/`) for any environment variables or setup steps that differ from your local environment.

---

### `error[E0599]: no method named 'mock_all_auths'`

**Cause:** The `testutils` feature is not enabled for `soroban-sdk` in `[dev-dependencies]`.

**Solution:**
```toml
[dev-dependencies]
soroban-sdk = { version = "21.7.0", features = ["testutils"] }
```

---

## Deployment Issues

### `error: No such file or directory: target/wasm32-unknown-unknown/release/*.wasm`

**Cause:** The contract was not built for the WASM target before deploying.

**Solution:**
```bash
cargo build --target wasm32-unknown-unknown --release
# or use the Stellar CLI shorthand:
stellar contract build
```

---

### `stellar: command not found`

**Cause:** The Stellar CLI is not installed or not on `PATH`.

**Solution:**
```bash
cargo install --locked stellar-cli --version 22.1.0
# Verify:
stellar --version
```

---

### `error: account not found` when deploying to testnet

**Cause:** The source account does not exist on testnet or has no XLM balance.

**Solution:** Fund the account using Friendbot:
```bash
stellar keys generate alice --network testnet
stellar keys fund alice --network testnet
# Or via curl:
curl "https://friendbot.stellar.org?addr=$(stellar keys address alice)"
```

---

### `error: transaction failed: op_underfunded`

**Cause:** The deploying account does not have enough XLM to cover the transaction fee and minimum balance.

**Solution:** Fund the account (see above). Each deployed contract increases the minimum balance requirement by 0.5 XLM base reserve.

---

### Contract deployed but `invoke` returns `HostError: Error(Storage, MissingValue)`

**Cause:** The contract was deployed but not initialized. A read was attempted before the corresponding write.

**Solution:** Call the contract's `initialize` function immediately after deployment:

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --admin $(stellar keys address alice)
```

---

### `error: contract wasm too large`

**Cause:** The compiled WASM binary exceeds the protocol limit (~128 KB for most networks).

**Solution:**

1. Build in release mode (not debug):
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```
2. Enable size optimizations in `Cargo.toml`:
   ```toml
   [profile.release]
   opt-level = "z"
   overflow-checks = true
   debug = false
   strip = "symbols"
   lto = true
   codegen-units = 1
   ```
3. Split large contracts into multiple smaller contracts that call each other.

---

### `error: network passphrase mismatch`

**Cause:** The CLI is configured for one network (e.g., mainnet) but the contract ID or account belongs to another (e.g., testnet).

**Solution:** Always pass `--network` explicitly:
```bash
stellar contract invoke --network testnet ...
stellar contract invoke --network mainnet ...
```

Or configure a default network:
```bash
stellar network use testnet
```

---

## Solutions and Workarounds

### Workaround: No `println!` in contracts

Soroban contracts run in a `no_std` environment. Use `env.logs()` for debug output during testing:

```rust
env.logs().print();   // in tests, prints all log entries
```

In the contract:
```rust
// Not available in production; only visible in test environments
```

For production debugging, emit events instead:
```rust
env.events().publish((symbol_short!("debug"),), value);
```

---

### Workaround: No `HashMap` or `BTreeMap` from `std`

Use Soroban's `Map<K, V>` instead:

```rust
use soroban_sdk::Map;

let mut map: Map<soroban_sdk::Symbol, i128> = Map::new(&env);
map.set(soroban_sdk::symbol_short!("key"), 42);
```

---

### Workaround: No `Vec` from `std`

Use Soroban's `Vec<T>`:

```rust
use soroban_sdk::{vec, Vec};

let v: Vec<i128> = vec![&env, 1, 2, 3];
```

---

### Workaround: Simulating time in tests

Soroban's test environment does not advance time automatically. Set the ledger timestamp manually:

```rust
env.ledger().with_mut(|li| {
    li.timestamp = 1_700_000_000;
    li.sequence  = 1000;
});
```

---

### Workaround: Testing TTL / storage expiry

Advance the ledger sequence to simulate TTL expiry:

```rust
env.ledger().with_mut(|li| {
    li.sequence += 10_000; // advance past the TTL
});
```

---

## Getting More Help

- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [soroban-sdk API Reference](https://docs.rs/soroban-sdk/)
- [Stellar Developer Discord](https://discord.gg/stellardev) — `#soroban` channel
- [GitHub Issues](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues) — report bugs or ask questions
- [Quick Reference](./quick-reference.md) — common patterns cheat sheet
- [Best Practices](./best-practices.md) — security and code quality guidelines

# Module 2 Exercise: Storage & Data Types

**Objective:** Build a storage-backed counter contract with custom types, events, and error handling.

**Duration:** 45 minutes

---

## Prerequisites

- Complete Module 1 (environment set up, hello-world builds and tests)
- Familiarity with basic Rust syntax

---

## Reference Examples

Study these before starting:

- [`examples/basics/02-storage-patterns/`](../../../examples/basics/02-storage-patterns/) — All three storage types
- [`examples/basics/03-custom-errors/`](../../../examples/basics/03-custom-errors/) — Error handling
- [`examples/basics/04-events/`](../../../examples/basics/04-events/) — Events
- [`examples/basics/10-custom-structs/`](../../../examples/basics/10-custom-structs/) — Custom types

---

## Part 1: Create the Contract (20 min)

Create a new file `src/lib.rs` with the following contract. Your task is to fill in the TODO sections.

### Contract Requirements

1. Initialize with an admin address and starting count
2. `increment()` — increases the counter by 1
3. `decrement()` — decreases the counter by 1
4. `reset()` — sets counter to 0 (admin only)
5. `get_count()` — returns the current count
6. `set_metadata(key, value)` — stores arbitrary metadata (admin only)
7. `get_metadata(key)` — retrieves metadata
8. Emit events for increment, decrement, and reset

### Starter Code

```rust
#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Env, Symbol, Address};

// TODO: Define a DataKey enum with:
//   - Count
//   - Admin
//   - Metadata(Symbol)

// TODO: Define a ContractError enum with:
//   - NotInitialized = 1
//   - Unauthorized = 2
//   - ArithmeticOverflow = 3

// TODO: Define event data structs for Increment, Decrement, Reset

#[contract]
pub struct CounterContract;

#[contractimpl]
impl CounterContract {
    // TODO: initialize(env, admin: Address, initial_count: i128) -> Result<(), ContractError>
    //   - Guard against re-initialization
    //   - Store admin in instance storage
    //   - Store count in persistent storage

    // TODO: increment(env) -> Result<i128, ContractError>
    //   - Load current count, add 1, store back
    //   - Use checked_add for overflow protection
    //   - Emit Increment event

    // TODO: decrement(env) -> Result<i128, ContractError>
    //   - Load current count, subtract 1, store back
    //   - Use checked_sub for overflow protection
    //   - Emit Decrement event

    // TODO: reset(env, caller: Address) -> Result<(), ContractError>
    //   - Require auth, check admin
    //   - Set count to 0
    //   - Emit Reset event

    // TODO: get_count(env) -> i128
    //   - Return current count (0 if not initialized)

    // TODO: set_metadata(env, caller: Address, key: Symbol, value: i128) -> Result<(), ContractError>
    //   - Require auth, check admin
    //   - Store in persistent storage

    // TODO: get_metadata(env, key: Symbol) -> Option<i128>
    //   - Return from persistent storage
}
```

---

## Part 2: Write Tests (15 min)

Create `src/test.rs` with tests covering:

### Test Cases

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, Address, symbol_short, Symbol};

    #[test]
    fn test_initialize() {
        // Initialize with admin and count=10
        // Assert get_count() returns 10
    }

    #[test]
    fn test_increment() {
        // Initialize, increment, assert count is 11
    }

    #[test]
    fn test_decrement() {
        // Initialize with 10, decrement, assert count is 9
    }

    #[test]
    fn test_reset_admin_only() {
        // Initialize with admin
        // Reset with admin → should succeed
        // Reset with non-admin → should fail with Unauthorized
    }

    #[test]
    fn test_double_initialize_fails() {
        // Initialize twice → second should fail with AlreadyInitialized
    }

    #[test]
    fn test_metadata_set_get() {
        // Initialize, set metadata, get metadata, assert equal
    }

    #[test]
    fn test_increment_event() {
        // Initialize, increment, check event was emitted
        // Hint: use env.events().all() or check topics
    }
}
```

---

## Part 3: Build and Test (10 min)

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Run all tests
cargo test

# Run a specific test
cargo test test_increment
```

---

## Verification Checklist

- [ ] Contract compiles without warnings
- [ ] `initialize()` works and prevents re-initialization
- [ ] `increment()` and `decrement()` modify the count correctly
- [ ] `reset()` requires admin auth
- [ ] Events are emitted for increment, decrement, and reset
- [ ] Metadata can be set and retrieved
- [ ] All tests pass
- [ ] WASM builds successfully

---

## Solution

<details>
<summary>Click to reveal the solution</summary>

```rust
#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Count,
    Admin,
    Metadata(Symbol),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ArithmeticOverflow = 4,
}

#[contracttype]
#[derive(Clone)]
pub struct IncrementEvent {
    pub new_value: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct DecrementEvent {
    pub new_value: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct ResetEvent {
    pub reset_by: Address,
}

#[contract]
pub struct CounterContract;

#[contractimpl]
impl CounterContract {
    pub fn initialize(env: Env, admin: Address, initial_count: i128) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Count, &initial_count);
        Ok(())
    }

    pub fn increment(env: Env) -> Result<i128, ContractError> {
        let current = read_count(&env);
        let new_val = current.checked_add(1).ok_or(ContractError::ArithmeticOverflow)?;
        env.storage().persistent().set(&DataKey::Count, &new_val);
        env.events()
            .publish((symbol_short!("counter"), symbol_short!("incr")), IncrementEvent { new_value: new_val });
        Ok(new_val)
    }

    pub fn decrement(env: Env) -> Result<i128, ContractError> {
        let current = read_count(&env);
        let new_val = current.checked_sub(1).ok_or(ContractError::ArithmeticOverflow)?;
        env.storage().persistent().set(&DataKey::Count, &new_val);
        env.events()
            .publish((symbol_short!("counter"), symbol_short!("decr")), DecrementEvent { new_value: new_val });
        Ok(new_val)
    }

    pub fn reset(env: Env, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();
        let admin = read_admin(&env)?;
        if caller != admin {
            return Err(ContractError::Unauthorized);
        }
        env.storage().persistent().set(&DataKey::Count, &0i128);
        env.events()
            .publish((symbol_short!("counter"), symbol_short!("reset")), ResetEvent { reset_by: caller });
        Ok(())
    }

    pub fn get_count(env: Env) -> i128 {
        read_count(&env)
    }

    pub fn set_metadata(env: Env, caller: Address, key: Symbol, value: i128) -> Result<(), ContractError> {
        caller.require_auth();
        let admin = read_admin(&env)?;
        if caller != admin {
            return Err(ContractError::Unauthorized);
        }
        env.storage().persistent().set(&DataKey::Metadata(key), &value);
        Ok(())
    }

    pub fn get_metadata(env: Env, key: Symbol) -> Option<i128> {
        env.storage().persistent().get(&DataKey::Metadata(key))
    }
}

fn read_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ContractError::NotInitialized)
}

fn read_count(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Count)
        .unwrap_or(0)
}
```

</details>

---

## Next Steps

Continue to [Module 3: Authentication & Access Control](../slides/module-3-auth.md) to learn about authorization patterns.

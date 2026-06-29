# Interactive Playground — Basic Examples

Run, edit, and explore all 14 basic Soroban examples locally. Each example is
a self-contained Rust crate you can clone, modify, and test in minutes.

---

## How to Use This Playground

### Prerequisites

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Add the Wasm target
rustup target add wasm32-unknown-unknown

# 3. Install Soroban CLI
cargo install --locked soroban-cli

# 4. Clone the cookbook
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git
cd Soroban-Cookbook-
```

### Run Any Example

```bash
# Pattern: cd examples/basics/<example-name>
cd examples/basics/01-hello-world

# Run tests
cargo test

# Build the Wasm artifact
cargo build --target wasm32-unknown-unknown --release
```

### Edit and Experiment

Every example is isolated. Open `src/lib.rs` in your editor, make a change,
then run `cargo test` again — the test suite will catch regressions immediately.

---

## Example 01 — Hello World

> **Concepts:** `#[contract]`, `#[contractimpl]`, `Env`, `Symbol`, `Vec`, `symbol_short!`

The simplest possible Soroban contract. It receives a name and returns a
greeting vector. A great starting point for understanding how contracts are
structured.

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    /// Returns ["Hello", <to>] as a Vec<Symbol>.
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol_short!("Hello"), to]
    }
}
```

**Try it:**

```bash
cd examples/basics/01-hello-world
cargo test
```

**What to edit:** Change `symbol_short!("Hello")` to `symbol_short!("Hola")` and
re-run the tests — watch one assertion fail and learn how to fix it.

**Key concepts:**
- `#[contract]` marks the struct as a deployable contract unit.
- `#[contractimpl]` exposes public functions as callable entry points.
- `Env` is always the first parameter; it provides access to storage, events, and ledger.
- `symbol_short!` creates a `Symbol` from a string literal of up to 9 characters.

---

## Example 02 — Storage Patterns

> **Concepts:** Persistent, Instance, and Temporary storage; TTL management; `#[contracttype]`

Soroban has three storage tiers with different lifetimes and costs. This
example covers all three in one contract.

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Symbol};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Persistent(Symbol),
    Temporary(Symbol),
    Instance(Symbol),
}

#[contract]
pub struct StorageContract;

#[contractimpl]
impl StorageContract {
    // Persistent: survives upgrades; requires per-key TTL management.
    pub fn set_persistent(env: Env, key: Symbol, value: u64) {
        let storage_key = DataKey::Persistent(key.clone());
        env.storage().persistent().set(&storage_key, &value);
        env.storage().persistent().extend_ttl(&storage_key, 1000, 10000);
        env.events().publish(
            (symbol_short!("persist"), symbol_short!("set")),
            (key, value),
        );
    }

    pub fn get_persistent(env: Env, key: Symbol) -> Option<u64> {
        env.storage().persistent().get(&DataKey::Persistent(key))
    }

    // Instance: tied to the contract instance lifetime; shared TTL.
    pub fn set_instance(env: Env, key: Symbol, value: u64) {
        env.storage().instance().set(&DataKey::Instance(key.clone()), &value);
        env.storage().instance().extend_ttl(1000, 10000);
    }

    pub fn get_instance(env: Env, key: Symbol) -> Option<u64> {
        env.storage().instance().get(&DataKey::Instance(key))
    }

    // Temporary: single-ledger lifetime; lowest cost; no TTL needed.
    pub fn set_temporary(env: Env, key: Symbol, value: u64) {
        env.storage().temporary().set(&DataKey::Temporary(key.clone()), &value);
    }

    pub fn get_temporary(env: Env, key: Symbol) -> Option<u64> {
        env.storage().temporary().get(&DataKey::Temporary(key))
    }
}
```

**Try it:**

```bash
cd examples/basics/02-storage-patterns
cargo test
```

**What to edit:** Remove the `extend_ttl` call from `set_persistent` and observe
that the TTL-related tests fail — this demonstrates why TTL management matters.

**Storage cheat-sheet:**

| Type | Lifetime | Cost | Best for |
|------|----------|------|----------|
| Persistent | Until TTL expires | High | Balances, ownership |
| Instance | Contract instance | Medium | Admin config, fees |
| Temporary | Current ledger | Lowest | Reentrancy guards, caches |

---

## Example 03 — Custom Errors

> **Concepts:** `#[contracterror]`, `Result<T, E>`, error codes, structured error handling

Soroban contracts return typed errors via Rust's `Result`. Using
`#[contracterror]` assigns each variant a stable integer code that clients
can match on.

```rust
#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, Address, Env, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    InvalidInput        = 1,
    Unauthorized        = 2,
    NotFound            = 3,
    InsufficientBalance = 4,
    OperationNotAllowed = 5,
    RateLimitExceeded   = 6,
    ContractPaused      = 7,
    AlreadyExists       = 8,
}

#[contract]
pub struct CustomErrorsContract;

#[contractimpl]
impl CustomErrorsContract {
    /// Returns InvalidInput if value <= 0.
    pub fn validate_input(_env: Env, value: i64) -> Result<(), ContractError> {
        if value <= 0 {
            Err(ContractError::InvalidInput)
        } else {
            Ok(())
        }
    }

    /// Returns Unauthorized if caller != admin.
    pub fn check_authorization(
        _env: Env,
        caller: Address,
        admin: Address,
    ) -> Result<(), ContractError> {
        if caller != admin {
            Err(ContractError::Unauthorized)
        } else {
            Ok(())
        }
    }

    /// Returns InsufficientBalance or InvalidInput on bad transfers.
    pub fn transfer_tokens(
        _env: Env,
        from_balance: u64,
        amount: u64,
    ) -> Result<(), ContractError> {
        if amount == 0 {
            Err(ContractError::InvalidInput)
        } else if from_balance < amount {
            Err(ContractError::InsufficientBalance)
        } else {
            Ok(())
        }
    }
}
```

**Try it:**

```bash
cd examples/basics/03-custom-errors
cargo test
```

**What to edit:** Add a new variant `Overflow = 9` to `ContractError`, then write
a function that triggers it when two numbers would exceed `u64::MAX`.

---

## Example 04 — Authentication

> **Concepts:** `require_auth()`, admin patterns, RBAC, timelocks, allowances

`require_auth()` is Soroban's core security primitive. Every state-mutating
function that touches user funds should call it on the relevant address.

```rust
#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role { Admin = 0, Moderator = 1, User = 2 }

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuthError {
    Unauthorized       = 1,
    NotAdmin           = 2,
    AlreadyInitialized = 3,
    InsufficientBalance = 4,
    TimeLocked         = 5,
    CooldownActive     = 6,
    InvalidState       = 7,
    InsufficientRole   = 8,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey { Admin, Balance(Address) }

#[contract]
pub struct AuthContract;

#[contractimpl]
impl AuthContract {
    /// One-time initialization — stores the admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), AuthError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AuthError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Transfer tokens — `from` must authorize the call.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
        from.require_auth();    // <-- this is the core pattern

        let from_bal: i128 = env.storage().persistent()
            .get(&DataKey::Balance(from.clone())).unwrap_or(0);

        if from_bal < amount {
            return Err(AuthError::InsufficientBalance);
        }

        let to_bal: i128 = env.storage().persistent()
            .get(&DataKey::Balance(to.clone())).unwrap_or(0);

        env.storage().persistent().set(&DataKey::Balance(from), &(from_bal - amount));
        env.storage().persistent().set(&DataKey::Balance(to),   &(to_bal   + amount));
        Ok(())
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage().persistent()
            .get(&DataKey::Balance(user)).unwrap_or(0)
    }
}
```

**Try it:**

```bash
cd examples/basics/03-authentication
cargo test
```

**What to edit:** Remove `from.require_auth()` from `transfer` and run the tests.
The contract will no longer enforce ownership — a critical security hole.

**Auth patterns in this example:**
- Basic `require_auth()` on the caller
- Admin-only gates using a stored admin address
- Role-based access control (Admin / Moderator / User)
- Time-lock restrictions
- Per-address cooldown periods
- N-of-N multi-sig using `require_auth()` on a list

---

## Example 05 — Events

> **Concepts:** `env.events().publish()`, topic layout, indexed fields, structured payloads

Events are how Soroban contracts communicate state changes to off-chain
consumers. Topics are indexed; the data payload is decoded after filtering.

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
pub struct TransferEventData {
    pub amount: i128,
    pub memo:   u64,
}

#[contract]
pub struct EventsContract;

#[contractimpl]
impl EventsContract {
    /// Emits a 4-topic transfer event.
    ///
    /// Topic layout:
    ///   [0] "events"    — namespace (filter all contract events)
    ///   [1] "transfer"  — action    (filter all transfers)
    ///   [2] sender      — indexed   (filter by sender)
    ///   [3] recipient   — indexed   (filter by recipient)
    ///   data: TransferEventData { amount, memo }
    pub fn transfer(env: Env, sender: Address, recipient: Address, amount: i128, memo: u64) {
        env.events().publish(
            (symbol_short!("events"), symbol_short!("transfer"), sender, recipient),
            TransferEventData { amount, memo },
        );
    }

    /// Emits a simple counter-increment event.
    pub fn increment(env: Env) {
        let mut num: u32 = env.storage().instance()
            .get(&symbol_short!("num")).unwrap_or(0);
        num += 1;
        env.storage().instance().set(&symbol_short!("num"), &num);
        env.events().publish(
            (symbol_short!("number"), symbol_short!("inc")),
            num,
        );
    }

    pub fn get_number(env: Env) -> u32 {
        env.storage().instance().get(&symbol_short!("num")).unwrap_or(0)
    }
}
```

**Try it:**

```bash
cd examples/basics/04-events
cargo test
```

**What to edit:** Add a second event inside `increment` that emits the *previous*
value alongside the new value so off-chain consumers can detect the delta.

**Topic design rules:**
- Put the most-filtered field in the earliest topic slot.
- Max 4 topic slots per `publish()` call.
- Prefer `Symbol`/`Address` in topics — they serialize cheaply.
- Amounts and timestamps belong in `data`, not topics.

---

## Example 06 — Error Handling (Result Propagation)

> **Concepts:** `Result<T, E>`, `?` operator, error chaining, `#[contracterror]`

```bash
cd examples/basics/05-error-handling
cargo test
```

Key pattern — use `?` to propagate errors up the call stack without boilerplate:

```rust
pub fn complex_operation(env: Env, amount: u64) -> Result<u64, ContractError> {
    validate_amount(amount)?;          // returns Err early if invalid
    let balance = load_balance(&env)?; // returns Err if not found
    Ok(balance - amount)
}
```

---

## Example 07 — Auth Context

> **Concepts:** `require_auth_for_args()`, sub-invocation authorization, call chains

```bash
cd examples/basics/05-auth-context
cargo test
```

Auth context lets a contract verify authorization for specific arguments,
not just the bare caller identity:

```rust
// Only authorizes this call if the user signed off on EXACTLY these args.
user.require_auth_for_args((&user, &amount).into_val(&env));
```

---

## Example 08 — Validation Patterns

> **Concepts:** Input validation, checked arithmetic, `panic!` vs `Result`

```bash
cd examples/basics/06-validation-patterns
cargo test
```

Core validation helpers:

```rust
// Checked arithmetic prevents silent overflow
let result = a.checked_add(b).ok_or(ContractError::Overflow)?;

// Range check
if amount == 0 || amount > MAX_AMOUNT {
    return Err(ContractError::InvalidInput);
}
```

---

## Example 09 — Type Conversions

> **Concepts:** `TryFromVal`, `IntoVal`, `Val`, safe cross-boundary type handling

```bash
cd examples/basics/07-type-conversions
cargo test
```

Converting between Rust types and Soroban `Val`:

```rust
// Convert a Rust type into a Val for storage or events
let val: Val = my_struct.into_val(&env);

// Convert back — returns Result, never panics
let my_struct = MyStruct::try_from_val(&env, &val)?;
```

---

## Example 10 — Soroban Types

> **Concepts:** `Address`, `Symbol`, `Map`, `Vec`, `Bytes`, `BytesN`

```bash
cd examples/basics/08-soroban-types
cargo test
```

Quick reference:

```rust
use soroban_sdk::{Address, Bytes, BytesN, Map, Symbol, Vec};

// Symbol: short identifier ≤ 9 chars (compile-time)
let s = symbol_short!("hello");

// Map: key→value store living in host memory
let mut map: Map<Symbol, u64> = Map::new(&env);
map.set(symbol_short!("key"), 42u64);

// Vec: ordered list in host memory
let v: Vec<u64> = vec![&env, 1, 2, 3];

// BytesN: fixed-length byte array (e.g. 32-byte hash)
let hash: BytesN<32> = env.crypto().sha256(&data);
```

---

## Example 11 — Enum Types

> **Concepts:** `#[contracttype]` enums, state machines, dispatch tables

```bash
cd examples/basics/09-enum-types
cargo test
```

Use `#[contracttype]` enums as storage keys or contract state:

```rust
#[contracttype]
#[derive(Clone, Copy, PartialEq)]
pub enum Status { Active, Paused, Closed }

// Store state
env.storage().instance().set(&DataKey::Status, &Status::Active);

// Load and match
let status: Status = env.storage().instance().get(&DataKey::Status)
    .unwrap_or(Status::Active);
match status {
    Status::Active => { /* normal operation */ }
    Status::Paused => return Err(ContractError::Paused),
    Status::Closed => return Err(ContractError::Closed),
}
```

---

## Example 12 — Custom Structs

> **Concepts:** `#[contracttype]` structs, composite storage keys, nested types

```bash
cd examples/basics/10-custom-structs
cargo test
```

Structs annotated with `#[contracttype]` can be stored directly and used as
complex storage keys:

```rust
#[contracttype]
#[derive(Clone)]
pub struct UserProfile {
    pub name:    Symbol,
    pub balance: i128,
    pub tier:    u32,
}

// Use as a value
env.storage().persistent().set(&DataKey::Profile(user.clone()), &profile);

// Use a tuple as a composite key
env.storage().persistent().set(&(owner.clone(), token_id), &metadata);
```

---

## Example 13 — Primitive Types

> **Concepts:** Integer overflow safety, `u32`/`i128`/`u64`, arithmetic patterns

```bash
cd examples/basics/11-primitive-types
cargo test
```

Soroban contracts compile with `overflow-checks = true` in release builds,
so overflows panic rather than wrap. Use checked arithmetic for explicit handling:

```rust
// Panics on overflow in debug; use checked_ for explicit Result
let safe = value.checked_mul(factor).ok_or(ContractError::Overflow)?;

// i128 is the canonical token amount type (128-bit signed integer)
pub fn transfer(_env: Env, amount: i128) { /* ... */ }
```

---

## Example 14 — Data Types

> **Concepts:** Full type system reference — `Bytes`, `String`, `Map`, `Vec`, tuples

```bash
cd examples/basics/12-data-types
cargo test
```

Soroban `String` is a host object — no `format!` or concatenation available:

```rust
use soroban_sdk::String;

// Create from a string literal
let s = String::from_str(&env, "hello world");

// Prefer Symbol for short identifiers (≤ 9 chars)
let sym = symbol_short!("hello");  // more gas-efficient than String
```

---

## Example 15 — Collection Types

> **Concepts:** `Vec`, `Map`, iteration, map merging, sorted access

```bash
cd examples/basics/13-collection-types
cargo test
```

```rust
// Vec — ordered, integer-indexed
let mut v: Vec<u64> = Vec::new(&env);
v.push_back(10);
v.push_back(20);
for item in v.iter() { /* ... */ }

// Map — unordered key→value
let mut m: Map<Address, u64> = Map::new(&env);
m.set(user.clone(), 100);
let val = m.get(user).unwrap_or(0);
```

---

## Example 16 — Event Filtering

> **Concepts:** Off-chain query patterns, topic ordering, indexer-friendly design

```bash
cd examples/basics/14-event-filtering
cargo test
```

Designing events that off-chain indexers can query efficiently:

```rust
// Slot ordering matters for filter efficiency
env.events().publish(
    (
        symbol_short!("token"),   // [0] namespace  → "show me all token events"
        symbol_short!("transfer"),// [1] action     → "show me all transfers"
        from,                     // [2] primary    → "show me transfers FROM Alice"
        to,                       // [3] secondary  → "show me transfers TO Bob"
    ),
    amount,
);
```

Off-chain query progression (most → least broad):
1. `topic[0] == "token"` — all contract events
2. `+ topic[1] == "transfer"` — all transfers
3. `+ topic[2] == Alice` — Alice's outbound transfers
4. `+ topic[3] == Bob` — Alice-to-Bob transfers only

---

## Running the Full Basic Suite

Run every basic example's tests in one command from the workspace root:

```bash
cargo test -p soroban-hello-world-example \
           -p soroban-storage-patterns-example \
           -p soroban-custom-errors-example \
           -p soroban-auth-example \
           -p soroban-events-example
```

Or test all workspace members at once:

```bash
cargo test --workspace
```

## Building All Wasm Artifacts

```bash
# Build all basics
for dir in examples/basics/*/; do
    echo "Building $dir..."
    cargo build --manifest-path "$dir/Cargo.toml" \
                --target wasm32-unknown-unknown \
                --release
done
```

On Windows (PowerShell):

```powershell
Get-ChildItem examples\basics -Directory | ForEach-Object {
    Write-Host "Building $($_.Name)..."
    cargo build --manifest-path "$($_.FullName)\Cargo.toml" `
                --target wasm32-unknown-unknown `
                --release
}
```

---

## Deploying to Testnet

```bash
# Configure the testnet network
soroban network add --rpc-url https://soroban-testnet.stellar.org \
                    --network-passphrase "Test SDF Network ; September 2015" \
                    testnet

# Generate a funded test identity
soroban keys generate --network testnet --fund my-account

# Deploy the hello-world contract
soroban contract deploy \
    --network testnet \
    --source my-account \
    --wasm examples/basics/01-hello-world/target/wasm32-unknown-unknown/release/*.wasm

# Invoke the deployed contract (replace CONTRACT_ID with the output above)
soroban contract invoke \
    --network testnet \
    --source my-account \
    --id CONTRACT_ID \
    -- hello --to World
```

---

## Troubleshooting

**`error[E0463]: can't find crate for std`**
→ Add the Wasm target: `rustup target add wasm32-unknown-unknown`

**`error: package not found in workspace`**
→ Run the command from the workspace root, not from inside the example directory.

**`HostError: General` during tests**
→ You likely forgot `env.mock_all_auths()` before calling an auth-protected function.

**`extend_ttl` panics in tests**
→ This is expected when ledger sequence is 0. Wrap ledger setup with:
```rust
env.ledger().with_mut(|li| { li.sequence_number = 100; li.timestamp = 1_000_000; });
```

**Wasm binary is too large**
→ Check `Cargo.toml` for `opt-level = "z"`, `lto = true`, and `strip = "symbols"` in `[profile.release]`.

---

## Next Steps

- [Intermediate Examples →](./intermediate.md) — tokens, multi-sig, pause/unpause
- [Advanced Examples →](./advanced.md) — cross-contract calls, oracle patterns, upgradeable proxies
- [Testing Guide →](../guides/testing.md) — mocking auth, time manipulation, event assertions
- [Best Practices →](../docs/best-practices.md) — production-ready patterns
- [Common Pitfalls →](../docs/common-pitfalls.md) — mistakes to avoid

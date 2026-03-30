# Common Patterns in Soroban Smart Contracts

Patterns extracted from the basic examples. Each section names the pattern,
shows the minimal code, and explains when to reach for it.

---

## 1. Contract Initialization Guard

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs)

Prevent `initialize` from being called more than once. Without this guard,
anyone can overwrite the admin address after deployment.

```rust
pub fn initialize(env: Env, admin: Address) -> Result<(), AuthError> {
    if env.storage().instance().has(&DataKey::Admin) {
        return Err(AuthError::AlreadyInitialized);
    }
    admin.require_auth();
    env.storage().instance().set(&DataKey::Admin, &admin);
    Ok(())
}
```

**When to use:** Any contract that has a privileged admin or one-time setup
step. Always pair with an idempotency check before writing.

---

## 2. Authenticate → Validate → Execute

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs)

The canonical ordering for every state-mutating function. Reject
unauthenticated callers before any computation or storage access.

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128)
    -> Result<(), AuthError>
{
    from.require_auth();                    // 1. authenticate

    if amount <= 0 {                        // 2. validate
        return Err(AuthError::InsufficientBalance);
    }

    let bal: i128 = env.storage().persistent()
        .get(&DataKey::Balance(from.clone())).unwrap_or(0);
    if bal < amount {
        return Err(AuthError::InsufficientBalance);
    }

    env.storage().persistent()             // 3. execute
        .set(&DataKey::Balance(from), &(bal - amount));
    Ok(())
}
```

**When to use:** Every function that writes state on behalf of a caller.
Deviating from this order — especially calling `require_auth()` after a
storage write — leaves a window for unauthenticated state mutation.

---

## 3. Stored-Admin Check

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs)

Load the admin from storage and compare; never trust the argument alone.
Extract into a private helper so the check is not duplicated.

```rust
fn require_admin(env: &Env, caller: &Address) -> Result<(), AuthError> {
    let admin: Address = env.storage().instance()
        .get(&DataKey::Admin)
        .ok_or(AuthError::NotAdmin)?;
    if caller != &admin {
        return Err(AuthError::NotAdmin);
    }
    Ok(())
}

// Usage in any admin-only function:
pub fn set_config(env: Env, admin: Address, value: u64) -> Result<(), AuthError> {
    admin.require_auth();
    require_admin(&env, &admin)?;
    env.storage().instance().set(&DataKey::Config, &value);
    Ok(())
}
```

**When to use:** Every privileged function. An attacker can pass any address
they control as the `admin` argument; only the value in storage is
authoritative.

---

## 4. Role-Based Access Control (RBAC)

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs)

Assign roles to addresses in persistent storage. A helper checks whether the
caller holds one of the allowed roles before proceeding.

```rust
#[contracttype]
#[repr(u32)]
pub enum Role { Admin = 0, Moderator = 1, User = 2 }

fn require_role(env: &Env, caller: &Address, allowed: &[Role])
    -> Result<(), AuthError>
{
    let role: Role = env.storage().persistent()
        .get(&DataKey::UserRole(caller.clone()))
        .unwrap_or(Role::User);
    for r in allowed {
        if role as u32 <= *r as u32 { return Ok(()); }
    }
    Err(AuthError::InsufficientRole)
}

// Admin-only gate
pub fn admin_action(env: Env, caller: Address, value: u64)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    require_role(&env, &caller, &[Role::Admin])?;
    Ok(value * 2)
}

// Admin or Moderator gate
pub fn moderator_action(env: Env, caller: Address, value: u64)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    require_role(&env, &caller, &[Role::Admin, Role::Moderator])?;
    Ok(value + 10)
}
```

**When to use:** Contracts with multiple privilege tiers — governance,
content moderation, treasury management. Store roles in **persistent**
storage so assignments survive contract upgrades.

---

## 5. Allowance (Delegated Spend)

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs)

Let an owner pre-approve a spender to move tokens on their behalf, without
sharing their private key.

```rust
pub fn approve(env: Env, from: Address, spender: Address, amount: i128)
    -> Result<(), AuthError>
{
    from.require_auth();
    env.storage().persistent()
        .set(&DataKey::Allowance(from, spender), &amount);
    Ok(())
}

pub fn transfer_from(
    env: Env, spender: Address, from: Address, to: Address, amount: i128,
) -> Result<(), AuthError> {
    spender.require_auth();

    let allowance: i128 = env.storage().persistent()
        .get(&DataKey::Allowance(from.clone(), spender.clone()))
        .unwrap_or(0);
    if allowance < amount { return Err(AuthError::Unauthorized); }

    // … update balances, reduce allowance …
    Ok(())
}
```

**When to use:** Token contracts, DeFi protocols where a router or vault
needs to pull funds from a user's account. Always check the allowance
**before** modifying balances.

---

## 6. N-of-N Multi-Sig

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs)

Require every address in a list to have signed the transaction. The host
verifies all signatures atomically.

```rust
pub fn multi_sig_action(_env: Env, signers: Vec<Address>, value: u32) -> u32 {
    for signer in signers.iter() {
        signer.require_auth();
    }
    value + signers.len()
}
```

**When to use:** Treasury operations, protocol upgrades, or any action that
must not proceed unless all named parties consent. For M-of-N thresholds,
track approvals in storage and execute once the count reaches the threshold.

---

## 7. Time-Lock

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs)

Block an action until a future ledger timestamp. The admin sets the unlock
time; the action checks it on every call.

```rust
pub fn set_time_lock(env: Env, admin: Address, unlock_time: u64)
    -> Result<(), AuthError>
{
    admin.require_auth();
    require_admin(&env, &admin)?;
    env.storage().instance().set(&DataKey::TimeLock, &unlock_time);
    Ok(())
}

pub fn time_locked_action(env: Env, caller: Address)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    let unlock: u64 = env.storage().instance()
        .get(&DataKey::TimeLock).unwrap_or(0);
    if env.ledger().timestamp() < unlock {
        return Err(AuthError::TimeLocked);
    }
    Ok(env.ledger().timestamp())
}
```

**When to use:** Vesting schedules, governance cool-off periods, delayed
withdrawals. Store the unlock timestamp in instance storage (it is
contract-wide configuration, not per-user data).

---

## 8. Per-Address Cooldown

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs)

Enforce a minimum interval between successive calls from the same address.

```rust
pub fn cooldown_action(env: Env, caller: Address)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    let period: u64 = env.storage().instance()
        .get(&DataKey::CooldownPeriod).unwrap_or(0);
    let last: u64 = env.storage().persistent()
        .get(&DataKey::LastAction(caller.clone())).unwrap_or(0);
    let now = env.ledger().timestamp();
    if last > 0 && now < last + period {
        return Err(AuthError::CooldownActive);
    }
    env.storage().persistent()
        .set(&DataKey::LastAction(caller), &now);
    Ok(now)
}
```

**When to use:** Rate-limiting claims, faucets, or any action that should
not be spammable. Store the last-action timestamp in **persistent** storage
(per-address, long-lived) and the cooldown period in **instance** storage
(contract-wide configuration).

---

## 9. Circuit-Breaker (Contract State Gate)

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs)

A global state enum lets admins pause or freeze the contract in an emergency.

```rust
#[contracttype]
#[repr(u32)]
pub enum ContractState { Active = 0, Paused = 1, Frozen = 2 }

pub fn active_only_action(env: Env, caller: Address)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    let state: ContractState = env.storage().instance()
        .get(&DataKey::State).unwrap_or(ContractState::Active);
    if state != ContractState::Active {
        return Err(AuthError::InvalidState);
    }
    Ok(env.ledger().timestamp())
}

// Admin toggles state:
pub fn set_state(env: Env, admin: Address, state: ContractState)
    -> Result<(), AuthError>
{
    admin.require_auth();
    require_admin(&env, &admin)?;
    env.storage().instance().set(&DataKey::State, &state);
    Ok(())
}
```

**When to use:** Any production contract. A circuit-breaker lets you halt
operations during an exploit or upgrade without redeploying. Default to
`Active` via `unwrap_or` so the contract works before the admin explicitly
sets state.

---

## 10. Storage Type Selection

**Source:** [`02-storage-patterns`](../examples/basics/02-storage-patterns/src/lib.rs)

Choose the storage tier that matches the data's expected lifetime and access
frequency.

```rust
// Persistent — survives indefinitely; requires TTL extension.
// Use for: user balances, role assignments, long-lived records.
env.storage().persistent().set(&key, &value);
env.storage().persistent().extend_ttl(&key, 100, 100);

// Instance — lives as long as the contract instance; cheaper than persistent.
// Use for: admin address, contract config, global flags.
env.storage().instance().set(&key, &value);
env.storage().instance().extend_ttl(100, 100);

// Temporary — cheapest; expires after a short TTL (minimum 16 ledgers).
// Use for: reentrancy guards, nonces, short-lived locks.
env.storage().temporary().set(&key, &value);
```

| Storage | Lifetime | Cost | Typical use |
|---------|----------|------|-------------|
| Persistent | Until TTL expires | Highest | Balances, roles |
| Instance | Contract instance | Medium | Config, admin |
| Temporary | Short TTL | Lowest | Guards, nonces |

**When to use:** Match the tier to the data's lifetime. Using persistent
storage for ephemeral data wastes rent fees; using temporary storage for
balances risks data loss.

---

## 11. Typed Storage Keys

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs),
[`02-storage-patterns`](../examples/basics/02-storage-patterns/src/lib.rs)

Use a `#[contracttype]` enum as the key type instead of raw symbols. This
prevents key collisions and makes the storage schema self-documenting.

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance(Address),           // per-user balance
    Allowance(Address, Address), // (owner, spender) allowance
    UserRole(Address),          // per-user role
    TimeLock,
    CooldownPeriod,
    LastAction(Address),
    State,
}

// Usage — no risk of colliding with a plain symbol_short!("admin"):
env.storage().instance().set(&DataKey::Admin, &admin);
let bal: i128 = env.storage().persistent()
    .get(&DataKey::Balance(user.clone())).unwrap_or(0);
```

**When to use:** Always. Raw `symbol_short!` keys are error-prone and
undocumented. A typed enum makes the full storage schema visible at a glance
and catches typos at compile time.

---

## 12. Structured Error Enum

**Source:** [`03-custom-errors`](../examples/basics/03-custom-errors/src/lib.rs),
[`05-error-handling`](../examples/basics/05-error-handling/src/lib.rs)

Define a `#[contracterror]` enum with sequential `u32` codes. Return
`Result<T, Error>` for expected failures; reserve `panic!` for invariant
violations.

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    InvalidAmount      = 1,
    InsufficientBalance = 2,
    Unauthorized       = 3,
    NotFound           = 4,
    AlreadyExists      = 5,
}

// Expected failure → Result
pub fn transfer(amount: u64, balance: u64) -> Result<u64, Error> {
    if amount == 0 { return Err(Error::InvalidAmount); }
    if amount > balance { return Err(Error::InsufficientBalance); }
    Ok(balance - amount)
}

// Invariant violation → panic
pub fn get_verified_state(env: Env, key: u32) -> u64 {
    let value: u64 = env.storage().instance().get(&key).unwrap_or(0);
    if value > 1_000 { panic!("invariant violated: state corrupted"); }
    value
}
```

**When to use:** `Result` for anything a caller can reasonably handle
(bad input, insufficient funds, not found). `panic!` only when the contract
has reached an impossible state that indicates a bug. See
[`05-error-handling`](../examples/basics/05-error-handling/) for a full
comparison.

---

## 13. Query-Friendly Event Topics

**Source:** [`04-events`](../examples/basics/04-events/src/lib.rs)

Place filterable identifiers in topics and rich payload in data. Use a
consistent `(namespace, action, [keys…])` layout across all events.

```rust
// Topic layout:
//   [0] contract namespace  — primary filter for all events from this contract
//   [1] action name         — secondary filter by operation type
//   [2] primary entity      — e.g. sender address
//   [3] secondary entity    — e.g. recipient address  (optional)
// Data: non-indexed payload (amounts, structs, metadata)

env.events().publish(
    (symbol_short!("mytoken"), symbol_short!("transfer"), sender, recipient),
    TransferEventData { amount, memo },
);
```

Off-chain query examples:

```
All events from this contract:   topic[0] == "mytoken"
All transfers:                   topic[0] == "mytoken" AND topic[1] == "transfer"
All sends by Alice:              … AND topic[2] == Alice
Alice → Bob only:                … AND topic[2] == Alice AND topic[3] == Bob
```

**When to use:** Every event emission. Unstructured topics make off-chain
indexing expensive. Define topic layout constants at the top of the file so
the schema is explicit and consistent.

---

## 14. Checked Arithmetic

**Source:** [`docs/best-practices`](./best-practices.md)

Use `checked_*` methods for all arithmetic on user-supplied values to prevent
silent overflow or underflow.

```rust
// Addition
let new_balance = balance.checked_add(amount)
    .expect("balance overflow");

// Subtraction
let remaining = balance.checked_sub(amount)
    .expect("balance underflow");

// Multiplication (e.g. fee calculation)
let fee = amount.checked_mul(rate)
    .expect("fee overflow")
    .checked_div(10_000)
    .expect("fee div by zero");
```

**When to use:** Any arithmetic involving values that originate from storage
or function arguments. Rust's debug builds trap overflow, but release WASM
builds wrap silently — `checked_*` catches this in both modes.

---

## 15. Balance Read / Write Helpers

**Source:** [`03-authentication`](../examples/basics/03-authentication/src/lib.rs),
[`docs/best-practices`](./best-practices.md)

Centralise balance reads and writes in private helpers to avoid scattered
storage calls and make the accounting logic easy to audit.

```rust
fn get_balance(env: &Env, addr: &Address) -> i128 {
    env.storage().persistent()
        .get(&DataKey::Balance(addr.clone()))
        .unwrap_or(0)
}

fn set_balance(env: &Env, addr: &Address, amount: i128) {
    env.storage().persistent()
        .set(&DataKey::Balance(addr.clone()), &amount);
}

// Transfer using helpers:
pub fn transfer(env: Env, from: Address, to: Address, amount: i128)
    -> Result<(), Error>
{
    from.require_auth();
    let from_bal = get_balance(&env, &from);
    if from_bal < amount { return Err(Error::InsufficientBalance); }
    set_balance(&env, &from, from_bal - amount);
    set_balance(&env, &to, get_balance(&env, &to) + amount);
    Ok(())
}
```

**When to use:** Any contract that tracks per-address balances. Helpers
eliminate copy-paste errors and make it trivial to add TTL extension or
event emission in one place.

---

## Pattern Selection Summary

| Situation | Pattern |
|-----------|---------|
| One-time setup | [Initialization Guard](#1-contract-initialization-guard) |
| Any state-mutating function | [Auth → Validate → Execute](#2-authenticate--validate--execute) |
| Privileged admin functions | [Stored-Admin Check](#3-stored-admin-check) |
| Multiple privilege tiers | [RBAC](#4-role-based-access-control-rbac) |
| Third-party spending rights | [Allowance](#5-allowance-delegated-spend) |
| All-parties-must-sign | [N-of-N Multi-Sig](#6-n-of-n-multi-sig) |
| Delayed execution | [Time-Lock](#7-time-lock) |
| Rate limiting | [Per-Address Cooldown](#8-per-address-cooldown) |
| Emergency stop | [Circuit-Breaker](#9-circuit-breaker-contract-state-gate) |
| Choosing storage | [Storage Type Selection](#10-storage-type-selection) |
| Avoiding key collisions | [Typed Storage Keys](#11-typed-storage-keys) |
| Communicating failures | [Structured Error Enum](#12-structured-error-enum) |
| Off-chain indexing | [Query-Friendly Event Topics](#13-query-friendly-event-topics) |
| Safe math | [Checked Arithmetic](#14-checked-arithmetic) |
| Token accounting | [Balance Helpers](#15-balance-read--write-helpers) |

---

## Related Resources

- [Best Practices](./best-practices.md) — security and efficiency rules
- [Quick Reference](./quick-reference.md) — syntax cheat sheet
- [03-authentication](../examples/basics/03-authentication/) — auth patterns in full
- [02-storage-patterns](../examples/basics/02-storage-patterns/) — storage tiers in full
- [04-events](../examples/basics/04-events/) — event design in full
- [05-error-handling](../examples/basics/05-error-handling/) — panic vs errors in full

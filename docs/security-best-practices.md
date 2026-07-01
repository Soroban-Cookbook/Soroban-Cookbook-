# Security Best Practices for Soroban Smart Contracts

A practical guide to writing secure Soroban contracts, covering common vulnerabilities, defensive patterns, and a pre-deployment audit checklist with real contract examples.

---

## Table of Contents

1. [Common Vulnerabilities](#1-common-vulnerabilities)
2. [Secure Patterns](#2-secure-patterns)
3. [Pre-Deployment Audit Checklist](#3-pre-deployment-audit-checklist)
4. [Real-World Examples](#4-real-world-examples)

---

## 1. Common Vulnerabilities

### 1.1 Missing Authorization Checks

The most critical vulnerability: calling a state-changing function without verifying the caller is allowed to perform it.

**Vulnerable:**

```rust
pub fn withdraw(env: Env, user: Address, amount: i128) {
    // No require_auth() — anyone can drain any account!
    let bal = read_balance(&env, &user);
    write_balance(&env, &user, bal - amount);
}
```

**Secure:**

```rust
pub fn withdraw(env: Env, user: Address, amount: i128) {
    user.require_auth(); // Caller must be `user`
    let bal = read_balance(&env, &user);
    if bal < amount {
        panic!("insufficient balance");
    }
    write_balance(&env, &user, bal - amount);
}
```

> `require_auth()` must be called before any state mutation. Soroban's host enforces this at the protocol level — if the required signature is absent the transaction is rejected.

---

### 1.2 Integer Overflow / Underflow

Rust's release profile enables `overflow-checks = true` in this workspace (see root `Cargo.toml`). However, intermediate calculations can still produce logically-wrong results if you rely on wrapping semantics. Always use **checked arithmetic** for financial values.

**Vulnerable:**

```rust
pub fn add_reward(env: Env, user: Address, amount: i128) {
    let current = read_balance(&env, &user);
    // Wraps on overflow if overflow-checks = false (e.g. a custom profile)
    write_balance(&env, &user, current + amount);
}
```

**Secure:**

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    ArithmeticOverflow = 1,
}

pub fn add_reward(env: Env, user: Address, amount: i128) -> Result<(), TokenError> {
    user.require_auth();
    let current = read_balance(&env, &user);
    let new_bal = current
        .checked_add(amount)
        .ok_or(TokenError::ArithmeticOverflow)?;
    write_balance(&env, &user, new_bal);
    Ok(())
}
```

---

### 1.3 Re-Initialization Attacks

A contract that can be initialized more than once lets an attacker overwrite the admin or reset state.

**Vulnerable:**

```rust
pub fn initialize(env: Env, admin: Address) {
    // No guard — callable multiple times!
    env.storage().instance().set(&DataKey::Admin, &admin);
}
```

**Secure:**

```rust
pub fn initialize(env: Env, admin: Address) {
    if env.storage().instance().has(&DataKey::Admin) {
        panic!("already initialized");
    }
    env.storage().instance().set(&DataKey::Admin, &admin);
}
```

---

### 1.4 Unchecked Cross-Contract Return Values

When calling another contract, a non-`Result` return type causes a panic on error. This is usually fine, but if you're passing unchecked external input into a sub-call, the caller can craft arguments that trigger panics in unexpected places.

**Pattern to avoid:**

```rust
// If `token_address` is attacker-controlled, they can pass a contract
// that panics or misbehaves.
TokenClient::new(&env, &token_address).transfer(&from, &to, &amount);
```

**Mitigation:** Validate that `token_address` is a known, trusted contract before calling it. Store the trusted address at initialization and reject callers who supply a different one.

---

### 1.5 Storage Key Collisions

Using short, generic keys increases the risk of collisions between different data items, especially after contract upgrades.

**Vulnerable:**

```rust
// Both "bal" keys might clash if multiple features share the same namespace.
env.storage().persistent().set(&symbol_short!("bal"), &amount);
```

**Secure:**

```rust
#[contracttype]
pub enum DataKey {
    Balance(Address),   // includes the address — unique per user
    Allowance(Address, Address),
    TotalSupply,
}

env.storage().persistent().set(&DataKey::Balance(user.clone()), &amount);
```

Using a `#[contracttype]` enum guarantees each variant has a unique XDR serialization, eliminating collisions.

---

### 1.6 Improper Access Control on Admin Functions

Admin operations that don't verify the caller is the stored admin allow privilege escalation.

**Vulnerable:**

```rust
pub fn set_rate(env: Env, caller: Address, new_rate: u32) {
    caller.require_auth();
    // Any authorized address can change the rate!
    env.storage().instance().set(&DataKey::Rate, &new_rate);
}
```

**Secure:**

```rust
pub fn set_rate(env: Env, caller: Address, new_rate: u32) -> Result<(), Error> {
    caller.require_auth();
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)?;
    if caller != admin {
        return Err(Error::Unauthorized);
    }
    env.storage().instance().set(&DataKey::Rate, &new_rate);
    Ok(())
}
```

---

### 1.7 Timestamp Dependence

`env.ledger().timestamp()` returns the ledger close time, which validators can influence by a small amount. Do **not** use timestamps for high-precision timing or as a source of randomness.

**Avoid:**

```rust
// Validators can slightly shift this to force a particular outcome
let won = env.ledger().timestamp() % 2 == 0;
```

**Use instead:** commit-reveal schemes, VRF oracles, or sequence-number-based logic for randomness; use ledger sequence numbers for ordering.

---

## 2. Secure Patterns

### 2.1 Checks-Effects-Interactions

Perform all authorization and state checks first, update storage next, and make external calls last. This is the Soroban equivalent of the Ethereum CEI pattern.

```rust
pub fn flash_withdraw(
    env: Env,
    caller: Address,
    amount: i128,
    callback: Address,
) -> Result<(), Error> {
    // 1. CHECKS
    caller.require_auth();
    let bal = read_balance(&env, &caller);
    if bal < amount {
        return Err(Error::InsufficientBalance);
    }

    // 2. EFFECTS — mutate state before any external call
    write_balance(&env, &caller, bal - amount);

    // 3. INTERACTIONS — external call after state is updated
    CallbackClient::new(&env, &callback).on_flash(&caller, &amount);

    Ok(())
}
```

---

### 2.2 Principle of Least Privilege

Grant only the minimum access required. Prefer function-level authorization over blanket admin powers.

```rust
#[contracttype]
pub enum Role {
    Admin,
    Minter,
    Pauser,
}

fn require_role(env: &Env, caller: &Address, role: Role) -> Result<(), Error> {
    caller.require_auth();
    let stored: Address = env
        .storage()
        .instance()
        .get(&role)
        .ok_or(Error::RoleNotSet)?;
    if *caller != stored {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

pub fn mint(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), Error> {
    require_role(&env, &minter, Role::Minter)?;
    // ...
    Ok(())
}

pub fn pause(env: Env, pauser: Address) -> Result<(), Error> {
    require_role(&env, &pauser, Role::Pauser)?;
    // ...
    Ok(())
}
```

---

### 2.3 Emergency Pause

Critical contracts should support a pause mechanism so an admin can halt operations while a vulnerability is investigated.

```rust
#[contracttype]
pub enum DataKey {
    Paused,
    Admin,
}

fn require_not_paused(env: &Env) -> Result<(), Error> {
    let paused: bool = env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

pub fn pause(env: Env, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;
    env.storage().instance().set(&DataKey::Paused, &true);
    Ok(())
}

pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
    require_not_paused(&env)?;
    from.require_auth();
    // ...
    Ok(())
}
```

See `examples/intermediate/03-pause-unpause` for a full implementation.

---

### 2.4 Input Validation at Boundaries

Validate inputs as early as possible, ideally in a dedicated validation helper.

```rust
fn validate_amount(amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    if amount > MAX_TRANSFER {
        return Err(Error::AmountExceedsLimit);
    }
    Ok(())
}

fn validate_recipient(recipient: &Address, _env: &Env) -> Result<(), Error> {
    // In a real contract, you might check a whitelist here.
    // For now this is a placeholder showing the pattern.
    let _ = recipient;
    Ok(())
}

pub fn transfer(
    env: Env,
    from: Address,
    to: Address,
    amount: i128,
) -> Result<(), Error> {
    from.require_auth();
    validate_amount(amount)?;
    validate_recipient(&to, &env)?;
    // ... proceed
    Ok(())
}
```

---

### 2.5 Event-Driven Auditability

Every significant state change should emit an event. Events are the primary mechanism for off-chain monitoring and incident detection.

```rust
const NS: Symbol = symbol_short!("token");
const EV_XFER: Symbol = symbol_short!("transfer");
const EV_PAUSE: Symbol = symbol_short!("paused");
const EV_ADMIN: Symbol = symbol_short!("admin_chg");

#[contracttype]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

fn emit_transfer(env: &Env, from: Address, to: Address, amount: i128) {
    env.events()
        .publish((NS, EV_XFER), TransferEvent { from, to, amount });
}

fn emit_paused(env: &Env, by: Address) {
    env.events().publish((NS, EV_PAUSE, by), true);
}

fn emit_admin_changed(env: &Env, old: Address, new: Address) {
    env.events().publish((NS, EV_ADMIN, old), new);
}
```

---

### 2.6 Safe Admin Transfer (Two-Step)

A one-step admin transfer can lock the contract forever if the wrong address is supplied. Use a two-step pattern.

```rust
#[contracttype]
pub enum DataKey {
    Admin,
    PendingAdmin,
}

/// Step 1: current admin nominates a successor.
pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) -> Result<(), Error> {
    current_admin.require_auth();
    require_admin(&env, &current_admin)?;
    env.storage()
        .instance()
        .set(&DataKey::PendingAdmin, &new_admin);
    Ok(())
}

/// Step 2: the successor accepts, proving they control the key.
pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), Error> {
    new_admin.require_auth();
    let pending: Address = env
        .storage()
        .instance()
        .get(&DataKey::PendingAdmin)
        .ok_or(Error::NoPendingAdmin)?;
    if new_admin != pending {
        return Err(Error::Unauthorized);
    }
    env.storage().instance().set(&DataKey::Admin, &new_admin);
    env.storage().instance().remove(&DataKey::PendingAdmin);
    Ok(())
}
```

---

## 3. Pre-Deployment Audit Checklist

Use this checklist before every production deployment.

### Authorization

- [ ] Every function that mutates state calls `require_auth()` on the appropriate signer **before** any storage write
- [ ] Admin-gated functions verify the caller equals the stored admin address, not just that they have some authorization
- [ ] There is no way to call `initialize()` (or equivalent setup function) more than once

### Arithmetic

- [ ] All balance additions use `checked_add` / `saturating_add` with explicit error handling
- [ ] All balance subtractions use `checked_sub` / `saturating_sub` with explicit error handling
- [ ] Multiplication and division on financial values use checked variants
- [ ] Division-by-zero cases are guarded explicitly

### Storage

- [ ] All storage keys use a `#[contracttype]` enum — no raw `symbol_short!` for keys that include variable data
- [ ] Storage types match data lifetime: `persistent()` for balances, `instance()` for contract config, `temporary()` for nonces/locks
- [ ] No two enum variants can produce the same serialized key
- [ ] There are no orphaned storage entries left behind after contract logic changes

### Errors

- [ ] A `#[contracterror]` enum is defined and used for all error conditions
- [ ] No `unwrap()` on `Option` or `Result` in contract code — every failure path is an intentional error variant
- [ ] Error values start at `1` (never `0`) to distinguish them from success

### Events

- [ ] Every balance transfer emits a transfer event
- [ ] Admin changes, pauses, and role assignments emit events
- [ ] Event topics follow the `(namespace, action, ...)` pattern for easy filtering

### Testing

- [ ] Unit tests cover every public function's happy path
- [ ] Unit tests cover every error variant (insufficient balance, unauthorized, invalid amount, etc.)
- [ ] Authorization tests verify the call panics without a valid auth (`env.set_auths(&[])`)
- [ ] Overflow edge cases are tested (max `i128` values)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes cleanly
- [ ] `cargo test --workspace` passes
- [ ] WASM release build succeeds: `cargo build --target wasm32-unknown-unknown --release`

### Configuration

- [ ] The crate compiles with `#![no_std]`
- [ ] `[profile.release]` has `overflow-checks = true` and `panic = "abort"`
- [ ] No `std` features are enabled in contract dependencies

---

## 4. Real-World Examples

### Example A: Secure Token Transfer

The following combines authorization, checked arithmetic, input validation, and event emission in a single function — the pattern used in `examples/tokens/01-sep41-token`.

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    NotInitialized      = 1,
    Unauthorized        = 2,
    InsufficientBalance = 3,
    InvalidAmount       = 4,
    ArithmeticOverflow  = 5,
}

pub fn transfer(
    env: Env,
    from: Address,
    to: Address,
    amount: i128,
) -> Result<(), TokenError> {
    // AUTHORIZATION
    from.require_auth();

    // INPUT VALIDATION
    ensure_initialized(&env)?;
    if amount <= 0 {
        return Err(TokenError::InvalidAmount);
    }

    // CHECKS
    let from_bal = read_balance(&env, &from);
    if from_bal < amount {
        return Err(TokenError::InsufficientBalance);
    }

    // EFFECTS (checked arithmetic)
    let to_bal = read_balance(&env, &to);
    let new_to_bal = to_bal
        .checked_add(amount)
        .ok_or(TokenError::ArithmeticOverflow)?;

    env.storage()
        .persistent()
        .set(&DataKey::Balance(from.clone()), &(from_bal - amount));
    env.storage()
        .persistent()
        .set(&DataKey::Balance(to.clone()), &new_to_bal);

    // EVENTS
    env.events().publish(
        (symbol_short!("token"), symbol_short!("transfer"), from, to),
        amount,
    );

    Ok(())
}
```

### Example B: Multi-Party Authorization

See `examples/advanced/01-multi-party-auth` for a complete N-of-N and M-of-N threshold authorization implementation with:
- Canonical auth-vector encoding
- Signer set management (add / remove / rotate)
- Threshold enforcement
- Full audit-trail events

### Example C: Time-Locked Execution

See `examples/advanced/02-timelock` for a contract demonstrating:
- Delayed execution of sensitive operations
- Queue / cancel / execute state machine
- Time-based validation using `env.ledger().timestamp()`
- Appropriate use of persistent vs. instance storage

---

## Further Reading

- [Best Practices](./best-practices.md) — general coding and performance guidelines
- [Common Pitfalls](./common-pitfalls.md) — mistakes to avoid when writing Soroban contracts
- [Security Audit Prep Checklist](./security-audit/audit-prep-checklist.md) — full pre-audit readiness guide
- [Soroban SDK Documentation](https://docs.rs/soroban-sdk)
- [Stellar Security Disclosure Policy](https://www.stellar.org/security)

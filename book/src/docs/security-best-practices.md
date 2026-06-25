# Soroban Security Best Practices

This guide covers the most common vulnerabilities in Soroban smart contracts,
the secure patterns that prevent them, a pre-audit checklist, and real examples
drawn from the cookbook.

---

## Table of Contents

1. [Common Vulnerabilities](#common-vulnerabilities)
   - [Missing Authorization](#1-missing-authorization)
   - [Re-entrancy](#2-re-entrancy)
   - [Arithmetic Overflow / Underflow](#3-arithmetic-overflow--underflow)
   - [Uninitialized Contract State](#4-uninitialized-contract-state)
   - [Integer Validation Gaps](#5-integer-validation-gaps)
   - [Unchecked Cross-Contract Calls](#6-unchecked-cross-contract-calls)
   - [Storage Key Collisions](#7-storage-key-collisions)
   - [Stale Allowances](#8-stale-allowances)
2. [Secure Patterns](#secure-patterns)
3. [Audit Checklist](#audit-checklist)
4. [Real Examples from the Cookbook](#real-examples-from-the-cookbook)

---

## Common Vulnerabilities

### 1. Missing Authorization

**Risk:** Any caller can invoke state-mutating functions.

**Vulnerable pattern:**

```rust
// DANGER: no auth — anyone can drain the contract
pub fn withdraw(env: Env, to: Address, amount: i128) {
    let balance: i128 = env.storage().instance().get(&BALANCE).unwrap_or(0);
    env.storage().instance().set(&BALANCE, &(balance - amount));
    // transfer to `to`...
}
```

**Secure pattern:**

```rust
pub fn withdraw(env: Env, admin: Address, to: Address, amount: i128) {
    // Require the caller to prove they control `admin`
    admin.require_auth();
    let stored_admin: Address = env.storage().instance().get(&ADMIN).unwrap();
    if admin != stored_admin {
        panic!("unauthorized");
    }
    // safe to proceed
}
```

**Key rules:**
- Every state-mutating entry point must call `address.require_auth()` before
  touching storage.
- Admin-only functions must additionally verify the caller matches the stored
  admin address.

---

### 2. Re-entrancy

Soroban's execution model prevents classic EVM re-entrancy, but **logic
re-entrancy** — reading state, making an external call, then using stale state
— is still possible.

**Secure pattern (checks-effects-interactions):**

```rust
pub fn claim(env: Env, user: Address) {
    user.require_auth();
    let owed: i128 = read_owed(&env, &user);
    if owed == 0 { panic!("nothing to claim"); }
    write_owed(&env, &user, 0);                  // effect first
    external_token.transfer(&env, &user, &owed); // interaction last
}
```

---

### 3. Arithmetic Overflow / Underflow

Use `checked_add` / `checked_sub` / `checked_mul` wherever user-supplied
values are involved so your contract can return a typed error instead of
panicking.

```rust
let new_total = total
    .checked_add(amount)
    .ok_or(ContractError::ArithmeticOverflow)?;
```

See `examples/advanced/05-batch-transfer/src/lib.rs` for a full example.

---

### 4. Uninitialized Contract State

Guard every entry point with an initialization check:

```rust
fn ensure_initialized(env: &Env) -> Result<(), ContractError> {
    if env.storage().instance().has(&DataKey::Initialized) {
        Ok(())
    } else {
        Err(ContractError::NotInitialized)
    }
}
```

And prevent double-initialization:

```rust
pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
    if env.storage().instance().has(&DataKey::Initialized) {
        return Err(ContractError::AlreadyInitialized);
    }
    env.storage().instance().set(&DataKey::Initialized, &true);
    // ...
}
```

---

### 5. Integer Validation Gaps

Always validate that amounts are strictly positive (or non-negative where zero
is valid) before processing them:

```rust
if amount <= 0 {
    return Err(ContractError::InvalidAmount);
}
```

---

### 6. Unchecked Cross-Contract Calls

Prefer `try_*` variants over direct invocation so you can propagate errors
gracefully:

```rust
token_client
    .try_transfer(&from, &to, &amount)
    .map_err(|_| ContractError::TransferFailed)?;
```

---

### 7. Storage Key Collisions

Use typed, parameterised `DataKey` variants to guarantee uniqueness:

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance(Address),
    Allowance(Address, Address),
}
```

---

### 8. Stale Allowances

Tie allowances to an `expiration_ledger` and honour it on every query:

```rust
fn effective_allowance(env: &Env, val: &AllowanceValue) -> i128 {
    if val.amount > 0 && val.expiration_ledger < env.ledger().sequence() {
        0
    } else {
        val.amount
    }
}
```

See `examples/tokens/allowance-pattern` for a complete implementation.

---

## Secure Patterns

| Pattern | Description |
|---------|-------------|
| Guard-then-mutate | `require_auth()` and validate before any storage write |
| Checks-Effects-Interactions | Clear state before making external calls |
| Checked arithmetic | `checked_add`/`checked_sub` on all user-supplied arithmetic |
| Initialization guard | Reject calls before `initialize()` with a typed error |
| Double-init guard | Reject a second `initialize()` call |
| Per-address storage keys | `DataKey::Balance(Address)` prevents collisions |
| Expiring allowances | Every allowance has an `expiration_ledger` |
| Typed error enum | `#[contracterror]` so callers can distinguish failures |

---

## Audit Checklist

### Authorization

- [ ] Every state-mutating entry point calls `address.require_auth()`.
- [ ] Admin operations verify the caller against the stored admin address.
- [ ] `require_auth_for_args` is used where auth must be argument-bound.

### Arithmetic

- [ ] All user-supplied arithmetic uses `checked_*` methods.
- [ ] `overflow-checks = true` is set in `[profile.release]`.

### Input validation

- [ ] Amounts are validated (positive or non-negative as required).
- [ ] Batch sizes are bounded by a constant.

### State initialization

- [ ] `initialize()` is guarded against double calls.
- [ ] All entry points call `ensure_initialized()` at the top.

### Storage

- [ ] `DataKey` variants cannot produce the same serialized key.
- [ ] Allowances carry `expiration_ledger` and queries honour expiry.

### External interactions

- [ ] Cross-contract calls use `try_*` variants.
- [ ] State changes happen before external calls.

### Build and tooling

- [ ] `cargo clippy --all-targets -- -D warnings` is clean.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo audit --deny warnings --deny unsound` passes.
- [ ] WASM release build succeeds.
- [ ] All tests pass.

---

## Real Examples from the Cookbook

| Example | Security pattern illustrated |
|---------|------------------------------|
| `examples/advanced/01-multi-party-auth` | M-of-N authorization with threshold enforcement |
| `examples/advanced/05-batch-transfer` | Checked arithmetic on every accumulation; atomic all-or-nothing |
| `examples/tokens/allowance-pattern` | Expiring allowances; `effective_allowance` expiry check |
| `examples/tokens/02-sep41-extensions` | Typed error enum; `require_auth_for_args` in permit |
| All token examples | Initialization guard; double-init prevention |

---

## Further Reading

- [Soroban authorization docs](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/auth)
- [`docs/security-audit/audit-scope.md`](../../docs/security-audit/audit-scope.md)
- [`docs/security-audit/audit-prep-checklist.md`](../../docs/security-audit/audit-prep-checklist.md)

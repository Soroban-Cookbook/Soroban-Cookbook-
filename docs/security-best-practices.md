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
- Every state-mutating entry point must call `address.require_auth()` or
  `address.require_auth_for_args(...)` before touching storage.
- Admin-only functions must additionally verify the caller matches the stored
  admin address.

---

### 2. Re-entrancy

Soroban's execution model prevents classic EVM re-entrancy: cross-contract
calls are synchronous and the host does not allow a contract to call back into
itself during the same invocation. However, **logic re-entrancy** — where state
is read, an external call is made, then stale state is used — is still
possible.

**Vulnerable pattern:**

```rust
pub fn claim(env: Env, user: Address) {
    user.require_auth();
    let owed: i128 = read_owed(&env, &user);   // (1) read
    external_token.transfer(&env, &user, &owed); // (2) external call
    write_owed(&env, &user, 0);                  // (3) clear — too late if (2) reverts partially
}
```

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

Soroban compiles with `overflow-checks = true` in `[profile.release]`, which
causes arithmetic overflows to panic rather than wrap. However, in contracts
that must distinguish overflow from other errors (and return a typed error
variant), use `checked_add` / `checked_sub` / `checked_mul`.

**Vulnerable pattern:**

```rust
// Could panic with a cryptic host error rather than your typed error
let new_total = total + amount;
```

**Secure pattern:**

```rust
let new_total = total
    .checked_add(amount)
    .ok_or(ContractError::ArithmeticOverflow)?;
```

See `examples/advanced/05-batch-transfer/src/lib.rs` for a full example where
every accumulation uses `checked_add`.

---

### 4. Uninitialized Contract State

Calling functions before `initialize()` leaves storage empty; reading defaults
(`unwrap_or(0)`, `unwrap_or_default()`) may silently allow unintended actions.

**Vulnerable pattern:**

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();
    // Reading balance returns 0 by default even before initialize()
    let bal: i128 = read_balance(&env, &from);  // silently 0
    // ...
}
```

**Secure pattern:**

```rust
fn ensure_initialized(env: &Env) -> Result<(), ContractError> {
    if env.storage().instance().has(&DataKey::Initialized) {
        Ok(())
    } else {
        Err(ContractError::NotInitialized)
    }
}

pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), ContractError> {
    from.require_auth();
    ensure_initialized(&env)?;   // guard at the top
    // ...
}
```

Also protect `initialize` itself from being called twice:

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

Amounts, counts, and indexes supplied by callers must be validated before use.
Negative amounts, zero amounts where positive is required, and out-of-range
values are common attack vectors.

**Vulnerable pattern:**

```rust
pub fn mint(env: Env, to: Address, amount: i128) {
    // amount = -1_000 drains the total supply
    let supply: i128 = read_supply(&env);
    write_supply(&env, supply + amount);  // underflow or negative supply
}
```

**Secure pattern:**

```rust
pub fn mint(env: Env, admin: Address, to: Address, amount: i128) -> Result<(), ContractError> {
    admin.require_auth();
    if amount <= 0 {
        return Err(ContractError::InvalidAmount);
    }
    let supply = read_supply(&env)
        .checked_add(amount)
        .ok_or(ContractError::ArithmeticOverflow)?;
    write_supply(&env, supply);
    // ...
}
```

---

### 6. Unchecked Cross-Contract Calls

When calling an external token contract, verify the returned result and do not
assume the call succeeded.

**Secure pattern:**

```rust
// Prefer try_transfer over transfer so you can propagate the error
token_client
    .try_transfer(&from, &to, &amount)
    .map_err(|_| ContractError::TransferFailed)?;
```

---

### 7. Storage Key Collisions

Using the same `DataKey` variant for different logical values can cause one
record to silently overwrite another.

**Vulnerable pattern:**

```rust
#[contracttype]
pub enum DataKey {
    Balance,         // used for admin AND user balances — collision!
}
```

**Secure pattern:**

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance(Address),   // per-address key; no collision possible
    Allowance(Address, Address),
}
```

---

### 8. Stale Allowances

Allowances without expiration are permanent. A user who forgets an old
approval, or whose keys are compromised, remains exposed indefinitely.

**Vulnerable pattern:**

```rust
// Bare allowance — lives forever
env.storage().persistent().set(&DataKey::Allowance(owner, spender), &amount);
```

**Secure pattern:**

```rust
// Allowance with expiration ledger
env.storage().persistent().set(
    &DataKey::Allowance(owner.clone(), spender.clone()),
    &AllowanceValue { amount, expiration_ledger },
);

// Query respects expiry
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

| Pattern | Description | Where to find it |
|---------|-------------|------------------|
| Guard-then-mutate | Call `require_auth()` and validate inputs before any storage write | All contract entry points |
| Checks-Effects-Interactions | Clear state before making external calls | `claim`, `redeem` patterns |
| Checked arithmetic | Use `checked_add`/`checked_sub` on all user-supplied arithmetic | `05-batch-transfer`, `02-sep41-extensions` |
| Initialization guard | Reject calls before `initialize()` with a typed error | All contracts |
| Double-init guard | Reject a second `initialize()` call | All contracts |
| Per-address storage keys | Use `DataKey::Balance(Address)` to prevent collisions | All token contracts |
| Expiring allowances | Tie every allowance to an `expiration_ledger` | `allowance-pattern`, `02-sep41-extensions` |
| Typed error enum | Use `#[contracterror]` so callers can distinguish failures | All contracts |

---

## Audit Checklist

Use this list before requesting an external audit or opening a PR for security-
sensitive code.

### Authorization

- [ ] Every state-mutating entry point calls `address.require_auth()` before
      modifying storage.
- [ ] Admin-only operations verify the caller matches the stored admin address
      after `require_auth()`.
- [ ] `require_auth_for_args` is used where the authorization must be bound to
      specific arguments (e.g., `permit`).

### Arithmetic

- [ ] All arithmetic on user-supplied values uses `checked_add` / `checked_sub`
      / `checked_mul`.
- [ ] Subtraction results are checked against underflow before writing to
      storage.
- [ ] `[profile.release]` in `Cargo.toml` has `overflow-checks = true`.

### Input validation

- [ ] Amounts are validated as strictly positive (or non-negative, as
      appropriate) before use.
- [ ] Batch sizes are bounded by a `MAX_BATCH_SIZE` constant.
- [ ] String or byte inputs from callers are length-bounded.

### State initialization

- [ ] `initialize()` is guarded against double calls.
- [ ] All other entry points call `ensure_initialized()` (or equivalent) at the
      top.

### Storage

- [ ] `DataKey` variants are unique per logical value; no two variants can
      produce the same serialized key.
- [ ] Allowances carry `expiration_ledger`; queries honour expiry.
- [ ] Temporary storage is used for data that does not need to survive ledger
      closes.

### Interactions with external contracts

- [ ] Cross-contract calls use `try_*` variants and propagate errors.
- [ ] State changes happen before external calls (checks-effects-interactions).

### Build and tooling

- [ ] `cargo clippy --all-targets -- -D warnings` is clean.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo audit --deny warnings --deny unsound` passes.
- [ ] WASM release build succeeds: `cargo build --target wasm32-unknown-unknown --release`.
- [ ] All tests pass: `cargo test --workspace`.

---

## Real Examples from the Cookbook

### Authorization — `examples/advanced/01-multi-party-auth`

Demonstrates requiring M-of-N signers before executing a sensitive action.
Each signer's address calls `require_auth()` and the contract enforces a
configurable threshold.

### Checked arithmetic — `examples/advanced/05-batch-transfer`

Every credit and debit uses `checked_add` / `checked_sub`, and the overall
total is accumulated with `checked_add` before any storage is written. If any
step overflows, `BatchError::TotalOverflow` or `BatchError::RecipientOverflow`
is returned without touching state.

### Initialization guard — all token examples

Every token contract in `examples/tokens/` checks for an `Initialized` storage
key at the top of every entry point and rejects calls before `initialize()`.

### Expiring allowances — `examples/tokens/allowance-pattern`

All allowances carry an `expiration_ledger`; the `allowance()` query returns
`0` for expired entries, and `approve` rejects non-zero allowances with an
already-passed expiration.

### Typed errors — `examples/tokens/02-sep41-extensions`

Uses a `#[contracterror]` enum with 8 variants, allowing callers and test
harnesses to assert on the exact failure mode with `try_*` client methods.

---

## Further Reading

- [Soroban authorization docs](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/auth)
- [Stellar security disclosure policy](https://www.stellar.org/bug-bounty-program)
- [`docs/security-audit/audit-scope.md`](./security-audit/audit-scope.md) — scope definition for external audits
- [`docs/security-audit/audit-prep-checklist.md`](./security-audit/audit-prep-checklist.md) — repo-wide pre-audit gates

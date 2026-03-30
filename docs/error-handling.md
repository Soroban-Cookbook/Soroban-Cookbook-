# Error Handling Patterns

Soroban contracts fail in two distinct ways. Choosing the right one keeps
your contract auditable, your clients resilient, and your bugs obvious.

---

## Two Failure Modes

### 1. Panic — unrecoverable abort

The transaction reverts immediately. No state is written. The caller gets a
failed simulation/transaction result but **no structured error code** unless
you use `panic_with_error!`.

**Use when:**
- A contract *invariant* is violated (double-init, impossible state).
- The call should literally never have been made.
- There is nothing the caller can do differently to succeed.

```rust
// Invariant: contract must only be initialised once.
if env.storage().instance().has(&DataKey::Admin) {
    panic!("contract already initialised");
}

// Invariant + structured code: only the admin may pause.
if caller != admin {
    panic_with_error!(env, ContractError::Unauthorized);
}
```

### 2. Typed Error — recoverable, documentable

Return `Result<T, ContractError>` where `ContractError` is a
`#[contracterror]` enum.  The `u32` discriminant travels to the caller via
XDR, so the client SDK can match on it and take corrective action (show a
message, retry with different params, etc.).

**Use when:**
- The input is predictably wrong (zero amount, too large, wrong type).
- A business rule is violated (insufficient balance, rate limit exceeded).
- The contract is in a temporary state (paused, initialising).

```rust
#[contracterror]
#[repr(u32)]
pub enum ContractError {
    ZeroAmount          = 100,
    InsufficientBalance = 101,
    ContractPaused      = 200,
    Overflow            = 202,
}

pub fn withdraw(env: Env, from: Address, amount: i128) -> Result<i128, ContractError> {
    if amount == 0 { return Err(ContractError::ZeroAmount); }
    let bal: i128 = env.storage().persistent().get(&DataKey::Balance(from.clone())).unwrap_or(0);
    if bal < amount { return Err(ContractError::InsufficientBalance); }
    // ...
    Ok(bal - amount)
}
```

---

## `panic!` vs `panic_with_error!`

| | `panic!("string")` | `panic_with_error!(env, E)` |
|---|---|---|
| Carries a typed code | ❌ | ✅ (u32 discriminant) |
| Visible off-chain | Message stripped in release | Code survives in tx result |
| Best for | Pure programmer assertions | Auth / invariant violations you want to monitor |

---

## Best Practice Guidelines

1. **Number your errors, leave gaps.**  
   Use ranges (1xx input, 2xx state) so you can add variants without
   renumbering existing ones that clients may already reference.

2. **Document every variant.**  
   Error codes are part of your public API just like function signatures.

3. **Never use `unwrap()` on caller-controlled data.**  
   Use `ok_or(ContractError::…)?` or an explicit guard instead.

4. **Let `require_auth` panic — that's correct.**  
   It is Soroban's built-in auth abort. Do not try to wrap it in a
   `Result`; the panic is the right behaviour.

5. **Test both paths.**  
   Use `try_*` client methods.  Typed errors match `Err(Ok(Variant))`;
   raw panics match `is_err()`.

---

## Performance Implications

- The ledger fee is consumed on *any* failure — panic or error.
- Typed errors allow clients to **simulate** before broadcasting; a clean
  `Err` from simulation prevents a fee-burning on-chain failure.
- `panic!` strings are stripped at compile time in release mode
  (`panic = "abort"` in `Cargo.toml`) so they add zero binary size.
- `panic_with_error!` adds a tiny `env.error()` call but this is
  negligible compared to storage reads/writes.

---

## Examples

| File | What it shows |
|------|--------------|
| [`src/lib.rs`](../examples/basics/05-error-handling/src/lib.rs) | Full contract: all three panic patterns + typed errors |
| [`src/test.rs`](../examples/basics/05-error-handling/src/test.rs) | `try_*` usage for both failure modes |
| [`ERROR_HANDLING_QUICK_REFERENCE.md`](../examples/basics/05-error-handling/ERROR_HANDLING_QUICK_REFERENCE.md) | One-page cheat sheet |

Run the tests:

```sh
cargo test -p error-handling
```

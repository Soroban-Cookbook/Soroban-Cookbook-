# ERROR_HANDLING_QUICK_REFERENCE.md

## Panic vs. Typed Errors — One-Glance Decision Table

| Question | Panic | Typed Error |
|----------|-------|-------------|
| Is this a **programming bug** / invariant? | ✅ | ❌ |
| Is this a **foreseeable user mistake**? | ❌ | ✅ |
| Can the **caller react** and retry? | No | Yes |
| Should it appear in the **public API docs**? | No | Yes |
| Does the **transaction always revert**? | Yes | Yes (for `Err`, unless caught) |

---

## When to Panic

```rust
// 1. Contract invariant (should be impossible in correct usage)
if env.storage().instance().has(&DataKey::Admin) {
    panic!("contract already initialised");
}

// 2. Privileged admin operation where any non-admin call is a bug
if caller != admin {
    panic_with_error!(env, ContractError::Unauthorized);
}

// 3. Defensive unreachable branch
_ => panic!("unknown code: this is a bug"),
```

### `panic!` vs `panic_with_error!`

| | `panic!("msg")` | `panic_with_error!(env, E)` |
|---|---|---|
| Off-chain error code | ❌ (message only) | ✅ (u32 discriminant) |
| Use when | Pure invariant with no meaningful code | Invariant that still benefits from a code |

---

## When to Return `Err`

```rust
#[contracterror]
#[repr(u32)]
pub enum ContractError {
    ZeroAmount         = 100,   // bad input — caller can fix and retry
    InsufficientBalance = 101,  // business rule — caller can top up first
    ContractPaused     = 200,   // state gate — caller can wait and retry
    Overflow           = 202,   // arithmetic safety — caller should reduce amount
}

// In your function:
pub fn deposit(env: Env, from: Address, amount: i128) -> Result<i128, ContractError> {
    if amount == 0 {
        return Err(ContractError::ZeroAmount);  // ← typed, documentable
    }
    // ...
    let new_bal = old.checked_add(amount).ok_or(ContractError::Overflow)?;  // ← ? propagates
    Ok(new_bal)
}
```

---

## Error Numbering Convention

```
1xx  — Input validation   (bad args the caller controls)
2xx  — State / auth       (contract-level gates)
3xx  — External calls     (cross-contract errors, if needed)
```

Leave gaps (100, 101, … not 1, 2, 3) so variants can be inserted without
breaking existing numeric codes that clients may already hardcode.

---

## Testing Both Modes

```rust
// Typed error — use try_* and match on Err(Ok(variant))
let result = client.try_deposit(&user, &0);
assert_eq!(result, Err(Ok(ContractError::ZeroAmount)));

// Panic — use try_* and check is_err() (no variant available)
let result = client.try_initialize(&admin); // second call
assert!(result.is_err());
```

---

## Performance Note

- **Neither path is "cheaper"** — the submitted fee is consumed either way.
- Typed errors help *clients* avoid submitting doomed transactions by
  reading the simulation result before broadcast.
- Panic messages are stripped in `profile.release` (`panic = "abort"` +
  `strip = "symbols"`), so they have zero runtime overhead in production.

---

## Related Patterns

- [`storage-patterns.md`](./storage-patterns.md) — emit events on write errors
- [`events.md`](./events-patterns.md) — audit trail events for error paths
- [`05-error-handling/src/lib.rs`](../examples/basics/05-error-handling/src/lib.rs) — full working example

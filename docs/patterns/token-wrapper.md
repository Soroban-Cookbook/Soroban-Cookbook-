# Token Wrapper Pattern

> **⚠️ UNAUDITED EXAMPLE — NOT FOR PRODUCTION USE**
>
> The contract code and patterns shown on this page have **not been audited**.
> They are provided solely as a learning resource to illustrate Soroban
> development techniques.  **Do not deploy this contract with real funds
> or in a production environment without a professional security audit.**
>
> **Reentrancy:** Soroban's execution model does not support re-entrant
> cross-contract calls within the same transaction — re-entry is a
> protocol-level impossibility on Soroban.  The reentrancy guard shown
> in this example (via `DataKey::Entered`) is therefore illustrative.
> On Soroban you **do not need** a mutex for reentrancy protection
> because the VM itself prevents interleaved re-entry.
>
> **Storage TTL / data-expiry:** Soroban instance storage entries expire
> after a ledger-defined TTL (default ~30 days on Mainnet).  A wrapper
> contract that is not called for an extended period will have its
> storage — including per-user balances and total supply — **silently
> deleted**.  Production deployments **must** extend instance (and any
> persistent) storage TTL on every call or via an off-chain keeper.
> Failure to do so will cause the backing invariant to become
> un-checkable and all wrapped balances to vanish.

---

## Overview

A Token Wrapper is a 1:1 proxy contract that accepts deposits of an
existing SEP-41 token ("underlying") and issues equal amounts of
"wrapped" shares in return.  The wrapped shares can be transferred among
users.  Wrapped shares are redeemed 1:1 for underlying tokens at any time.

**Related example crate:**
[`examples/tokens/06-token-wrapper`](../../examples/tokens/06-token-wrapper/)

The primary invariant maintained by this contract is:

```
underlying_balance(wrapper_contract) >= wrapped_total_supply
```

In normal operation both sides are exactly equal.

---

## Key Concepts

### Contract API

| Function | Auth required | Description |
|---|---|---|
| `initialize(underlying)` | — | Stores the token contract this wrapper accepts. One-time. |
| `wrap(user, amount)` | `user` | Pulls `amount` underlying tokens from `user`; mints `amount` wrapped shares. |
| `unwrap(user, amount)` | `user` | Burns `amount` wrapped shares; sends `amount` underlying tokens back to `user`. |
| `transfer(from, to, amount)` | `from` | Transfers wrapped shares without moving collateral. |
| `balance(user)` | — | Returns a user's wrapped-share balance. |
| `total_supply()` | — | Returns total wrapped shares outstanding. |
| `backing()` | — | Returns collateral balance, wrapped supply, surplus, and whether the contract is fully backed. |

### Core Wrap Flow

```rust
pub fn wrap(env: Env, user: Address, amount: i128) -> Result<i128, WrapperError> {
    user.require_auth();

    if amount <= 0 {
        return Err(WrapperError::InvalidAmount);
    }

    let underlying = read_underlying(&env)?;
    let wrapper = env.current_contract_address();

    // Transfer underlying token into this contract from the user.
    TokenClient::new(&env, &underlying).transfer(&user, &wrapper, &amount);

    // Credit wrapped shares to the user.
    let prev_balance = read_balance(&env, &user);
    let new_balance = prev_balance
        .checked_add(amount)
        .ok_or(WrapperError::ArithmeticOverflow)?;
    write_balance(&env, &user, new_balance);

    let prev_supply = read_total_supply(&env);
    let new_supply = prev_supply
        .checked_add(amount)
        .ok_or(WrapperError::ArithmeticOverflow)?;
    write_total_supply(&env, new_supply);

    Ok(new_balance)
}
```

> ⚠️ **Unaudited example.** Edge cases such as clawback-style
> undercollateralisation on the underlying token, rounding errors
> with non-standard decimals, or re-initialization are guarded at a
> basic level but have not been formally verified.

---

## Reentrancy

Soroban's VM **does not permit re-entrant calls** within a single
transaction.  The `DataKey::Entered` flag guard shown in the example
crate is illustrative; it demonstrates the *pattern* used on other
platforms but is **not required on Soroban**.

The cross-contract call to the underlying token's `transfer` inside
`wrap` / `unwrap` cannot call back into the wrapper contract
mid-execution under Soroban's execution model.

---

## Backing Invariant

The wrapper checks the invariant via `backing()`:

```rust
pub fn backing(env: Env) -> Result<BackingInfo, WrapperError> {
    let underlying = read_underlying(&env)?;
    let wrapper = env.current_contract_address();
    let collateral = TokenClient::new(&env, &underlying).balance(&wrapper);
    let supply = read_total_supply(&env);
    let surplus = collateral - supply;

    Ok(BackingInfo {
        collateral,
        supply,
        surplus,
        fully_backed: surplus >= 0,
    })
}
```

If the underlying token supports admin clawback, the wrapper can become
**undercollateralized**.  In that state `unwrap` returns
`WrapperError::NotFullyBacked` and no redemptions are possible until
the collateral is restored.

---

## Storage TTL / Data-Expiry Limits

| Storage tier | Data | Default TTL | Notes |
|---|---|---|---|
| Instance | `Underlying`, `TotalSupply`, per-user `Balance` | 30 days (refreshed on call) | Must be extended; all balances vanish on expiry. |

**Recommended pattern for production:**

```rust
// At the start of every state-mutating function:
env.storage().instance().extend_ttl(MIN_TTL_THRESHOLD, TARGET_TTL);
```

Where `MIN_TTL_THRESHOLD` is the ledger count at which you want to
start bumping (e.g. 100 000 ledgers ≈ 14 days), and `TARGET_TTL` is
the desired maximum lifetime (e.g. 200 000 ledgers ≈ 28 days).

Failure to extend TTLs means all wrapped balances become **permanently
inaccessible** when the storage entry expires — users cannot redeem
their underlying tokens.

---

## Security Considerations

- **Authorization**: `wrap`, `unwrap`, and `transfer` all call
  `user.require_auth()` / `from.require_auth()`.
- **Positive-amount guard**: Any call with `amount <= 0` is rejected.
- **Arithmetic safety**: All additions and subtractions use
  `checked_add` / `checked_sub` to surface overflows as errors.
- **Undercollateralization detection**: `backing()` surfaces the state;
  `unwrap` rejects the call when `!fully_backed`.
- **No admin upgrade path**: The example does not include a contract
  upgrade mechanism.  Add one only if needed, and document the trust
  model clearly.
- **Direct token transfers**: If someone sends underlying tokens
  directly to the wrapper address (bypassing `wrap`), the `surplus`
  field of `BackingInfo` will be positive.  This is tracked but not
  automatically distributed.

> ⚠️ **Reminder:** This example is **unaudited**.  These considerations
> are a starting checklist, not a complete security guarantee.

---

## Running the Example

```bash
# Build
cargo build --target wasm32-unknown-unknown --release -p token-wrapper

# Test
cargo test -p token-wrapper
```

---

## Related Pages

- [`examples/tokens/06-token-wrapper`](../../examples/tokens/06-token-wrapper/) — full example source
- [`examples/advanced/15-reentrancy-guard`](../../examples/advanced/15-reentrancy-guard/) — reentrancy guard pattern reference
- [`docs/patterns/token-vesting.md`](./token-vesting.md) — vesting tokens
- [`docs/security-best-practices.md`](../security-best-practices.md) — general security guide
- [`tests/integration/SECURITY_REVIEW_TOKEN_EXAMPLES.md`](../../tests/integration/SECURITY_REVIEW_TOKEN_EXAMPLES.md) — security review notes

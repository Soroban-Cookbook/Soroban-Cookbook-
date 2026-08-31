# Token Wrapper Pattern

This example shows how to wrap an existing SEP-41 token with a composable contract that issues 1:1 internal shares.

Users call `wrap(user, amount)` to deposit underlying tokens into the wrapper contract. The contract mints the same amount of wrapped shares to the user. Users call `unwrap(user, amount)` to burn wrapped shares and receive the same amount of underlying tokens back.

## What It Demonstrates

- Cross-contract calls with `soroban_sdk::token::TokenClient`
- 1:1 accounting between underlying collateral and wrapped supply
- Deposit and withdraw flows
- Backing verification with `backing()`
- Edge-case tests for invalid amounts, insufficient wrapped balance, direct token transfers, and clawback-style undercollateralization

## Contract API

| Function | Purpose |
| --- | --- |
| `initialize(underlying)` | Stores the token contract this wrapper accepts |
| `wrap(user, amount)` | Pulls underlying tokens from `user` and mints wrapped shares |
| `unwrap(user, amount)` | Burns wrapped shares and sends underlying tokens back |
| `transfer(from, to, amount)` | Transfers wrapped shares without moving collateral |
| `balance(user)` | Returns a user's wrapped-share balance |
| `total_supply()` | Returns total wrapped shares outstanding |
| `backing()` | Returns collateral balance, wrapped supply, surplus, and backing flags |

## Core Flow

```rust
pub fn wrap(env: Env, user: Address, amount: i128) -> Result<i128, WrapperError> {
    user.require_auth();

    let wrapper = env.current_contract_address();
    TokenClient::new(&env, &underlying).transfer(&user, &wrapper, &amount);

    // Mint exactly `amount` wrapped shares after validating arithmetic.
    // The full contract stores per-user balances and total supply.
    Ok(new_balance)
}
```

## Security

`wrap`, `unwrap`, and `transfer` share a reentrancy guard (`DataKey::Entered`,
following the same pattern as
[`examples/advanced/05-reentrancy-guard`](../../advanced/05-reentrancy-guard/)).
Without it, a hook-bearing or malicious `underlying` token's `transfer` (or
even its read-only `balance`, checked by `unwrap` before the guard used to be
set) could call back into `wrap` mid-flight and mint wrapped shares against
the same real deposit more than once — this contract's own bookkeeping
having already been updated doesn't stop a second, independent `wrap`
invocation from reading that updated state as its new baseline and minting
again on top of it. See
[`SECURITY_REVIEW_TOKEN_EXAMPLES.md`](../../../tests/integration/SECURITY_REVIEW_TOKEN_EXAMPLES.md)
and
[`tests/integration/tests/token_security_tests.rs`](../../../tests/integration/tests/token_security_tests.rs)
(#795) for the attack demonstration and regression tests.

## Backing Invariant

The primary invariant is:

```text
underlying token balance held by wrapper >= wrapped total supply
```

In normal operation the values are exactly equal. The example also handles surplus collateral caused by a direct token transfer to the wrapper address. If the wrapper becomes undercollateralized, for example through an administrative clawback on the underlying token, `unwrap` returns `WrapperError::NotFullyBacked`.

## Run Tests

```bash
cargo test -p token-wrapper
```

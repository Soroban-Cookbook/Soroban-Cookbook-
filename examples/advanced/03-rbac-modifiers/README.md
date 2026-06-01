# RBAC Modifiers

Role-Based Access Control (RBAC) patterns for Soroban smart contracts. This example shows how to define roles, protect functions with composable role guards, and emit a consistent event model for every role change.

## What You'll Learn

- How to define and store named roles on-chain
- How to protect functions with `only_role` (single role) and `any_role` (multi-role) guards
- How to grant, revoke, and renounce roles with proper authorization
- How to emit structured events for every role lifecycle change

## Overview

```
Admin grants MINTER role to Alice
         ↓
Alice calls protected_mint()  →  only_role(MINTER) passes  →  mint executes
         ↓
Admin revokes MINTER from Alice
         ↓
Alice calls protected_mint()  →  only_role(MINTER) fails  →  panic
```

## Built-in Roles

| Role     | Symbol    | Purpose                                  |
|----------|-----------|------------------------------------------|
| `ADMIN`  | `"ADMIN"` | Full control; can grant/revoke any role  |
| `MINTER` | `"MINTER"`| Can call mint-style operations           |
| `PAUSER` | `"PAUSER"`| Can pause and unpause the contract       |

Custom roles are supported — any `Symbol` can be used as a role name.

## API Reference

### `initialize(env, initial_admin)`

Bootstrap the contract. Grants `ADMIN` to `initial_admin`. Can only be called once.

### `grant_role(env, caller, role, account)`

Grant `role` to `account`. `caller` must hold `ADMIN`.

```rust
client.grant_role(&admin, &ROLE_MINTER, &alice);
```

### `revoke_role(env, caller, role, account)`

Revoke `role` from `account`. `caller` must hold `ADMIN`.

```rust
client.revoke_role(&admin, &ROLE_MINTER, &alice);
```

### `renounce_role(env, caller, role)`

Self-service role removal. `caller` removes their own `role` without needing an admin.

```rust
client.renounce_role(&alice, &ROLE_MINTER);
```

### `has_role(env, role, account) → bool`

Returns `true` if `account` currently holds `role`.

### `get_role_members(env, role) → Vec<Address>`

Returns all addresses that currently hold `role`.

### `is_paused(env) → bool`

Returns whether the contract is currently paused.

## Protected Operations

### `protected_mint(env, caller, to, amount)` — requires `MINTER`

```rust
// Grant role first
client.grant_role(&admin, &ROLE_MINTER, &minter);
// Then call the protected function
client.protected_mint(&minter, &recipient, &1000i128);
```

### `pause(env, caller)` / `unpause(env, caller)` — requires `PAUSER`

```rust
client.grant_role(&admin, &ROLE_PAUSER, &pauser);
client.pause(&pauser);
assert!(client.is_paused());
client.unpause(&pauser);
```

### `admin_action(env, caller)` — requires `ADMIN`

Demonstrates a single-role guard.

### `admin_or_minter_action(env, caller)` — requires `ADMIN` **or** `MINTER`

Demonstrates the `any_role` composable guard — passes if the caller holds at least one of the listed roles.

## Role Guard Patterns

### Single-role guard (`only_role`)

```rust
pub fn protected_mint(env: Env, caller: Address, to: Address, amount: i128) {
    caller.require_auth();
    Self::only_role(&env, &caller, ROLE_MINTER); // panics if caller lacks MINTER
    // ... mint logic
}
```

### Multi-role guard (`any_role`)

```rust
pub fn admin_or_minter_action(env: Env, caller: Address) {
    caller.require_auth();
    Self::any_role(&env, &caller, &[ROLE_ADMIN, ROLE_MINTER]); // passes if either role held
    // ... logic
}
```

## Events

Every role change emits a structured event so off-chain indexers can reconstruct the full role history.

| Event topic[1] | Payload type              | When emitted              |
|----------------|---------------------------|---------------------------|
| `"grant"`      | `RoleGrantedEventData`    | Role granted to account   |
| `"revoke"`     | `RoleRevokedEventData`    | Role revoked from account |
| `"call"`       | `ProtectedCallEventData`  | Protected function called |

## Security Notes

- Only `ADMIN` can grant or revoke roles — there is no self-grant.
- `renounce_role` lets accounts remove their own roles without admin involvement.
- Granting a role that is already held is a no-op (idempotent).
- Revoking a role that is not held is a no-op (safe).
- Custom roles work with any `Symbol` — no registration required.

## Running Tests

```bash
cargo test -p rbac-modifiers --target x86_64-unknown-linux-gnu
```

## Related Examples

- [01-multi-party-auth](../01-multi-party-auth/) — N-of-N and M-of-N authorization patterns
- [02-timelock](../02-timelock/) — Time-delayed execution with admin guards
- [03-authentication](../../basics/03-authentication/) — Single-party auth basics

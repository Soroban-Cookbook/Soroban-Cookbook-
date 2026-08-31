# Role-Based Access Control

This intermediate example demonstrates a full Role-Based Access Control (RBAC) implementation with composable role guards for Soroban smart contracts.

## What is RBAC?

Role-Based Access Control (RBAC) restricts system access to authorized users based on defined roles. Instead of checking account addresses directly, protected functions require callers to possess specific roles.

## Available Roles & Hierarchy

The contract defines four hierarchical roles:

- **Owner** (`4`): Highest authority, can manage all roles.
- **Admin** (`3`): Can manage `Moderator` and `User` roles.
- **Moderator** (`2`): Can execute moderation functions and user-level actions.
- **User** (`1`): Base role with standard access privileges.

Higher roles inherit the permissions of lower roles (`Owner` >= `Admin` >= `Moderator` >= `User`).

## Guard Helpers

The contract provides composable helper functions to reduce boilerplate and enforce access controls across protected functions:

- **Single-Role Guard**: `require_single_role(env, caller, required_role)`
  - Verifies caller authorization (`caller.require_auth()`) and checks if the caller holds at least `required_role`.
- **Multi-Role / OR Guard**: `require_any_role(env, caller, allowed_roles)`
  - Verifies caller authorization and checks if the caller holds any one of several allowed roles.

### Using Single-Role Guard

```rust
pub fn moderator_action(env: Env, caller: Address, value: u64) -> Result<u64, RbacError> {
    Self::require_single_role(&env, &caller, Role::Moderator)?;
    Ok(value + 10)
}
```

### Using Multi-Role / OR Guard

```rust
pub fn moderator_or_admin_action(
    env: Env,
    caller: Address,
    value: u64,
) -> Result<u64, RbacError> {
    Self::require_any_role(
        &env,
        &caller,
        Vec::from_array(&env, [Role::Moderator, Role::Admin]),
    )?;
    Ok(value + 100)
}
```

## Grant & Revoke Flows

- **Granting Roles**: An authorized role manager calls `grant_role(env, caller, account, role)`. Role changes enforce hierarchy checks (`Admin` can only manage `User` and `Moderator`).
- **Revoking Roles**: An authorized role manager calls `revoke_role(env, caller, account)`.

## Role-Change Event Model

Role modifications publish structured `RoleChangeEvent` audit events via Soroban's `#[contractevent]` macro:

```rust
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleChangeEvent {
    pub operator: Address,
    pub account: Address,
    pub old_role: Role,
    pub new_role: Role,
}
```

Events are automatically emitted during:
- Contract initialization (`initialize`)
- Granting roles (`grant_role`)
- Revoking roles (`revoke_role`)

## Security Considerations

1. **Caller Authentication**: Guard helpers enforce `caller.require_auth()` to prevent identity spoofing.
2. **Single Initialization**: Contract initialization can only be executed once.
3. **Privilege Hierarchy**: Admins cannot grant or revoke roles equal to or higher than their own level.
4. **Deterministic Side-Effect-Free Checking**: State lookups and authorization checks fail fast with typed `RbacError::Unauthorized` errors.

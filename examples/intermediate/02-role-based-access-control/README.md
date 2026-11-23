# Role-Based Access Control

> **Note:** For learning RBAC patterns, start with [**32-rbac-modifiers**](../../advanced/32-rbac-modifiers/) — the canonical RBAC example with composable guards and flexible symbol-based roles. This example demonstrates a **simpler numeric hierarchy** approach.

This intermediate example demonstrates RBAC with a **strict numeric role hierarchy** for Soroban smart contracts.

## What This Example Adds

This example differs from the canonical RBAC pattern by using a **numeric role hierarchy** where roles are compared by their ordinal values:

- **Owner (4)** — Top-level role with full control
- **Admin (3)** — Can grant/revoke Moderator and User roles
- **Moderator (2)** — Mid-level permissions
- **User (1)** — Default role

**When to use this pattern:**
- ✅ You need a strict, unchangeable role hierarchy
- ✅ You want simple numeric comparisons (`role_a >= role_b`)
- ✅ Your permission model maps naturally to a ladder (Owner > Admin > Moderator > User)

**When to use the canonical pattern instead:**
- 🔄 You need flexible, custom role names (MINTER, PAUSER, etc.)
- 🔄 You want roles that aren't strictly hierarchical
- 🔄 You need composable multi-role guards

## Features

- Persistent role storage per account
- Grant and revoke flows with hierarchy enforcement
- Role hierarchy rules (Owner, Admin, Moderator, User)
- Role checks for protected actions
- Audit event emission for role changes


## Related Examples

- **[32-rbac-modifiers](../../advanced/32-rbac-modifiers/)** — **Canonical RBAC pattern** with composable guards and flexible symbol-based roles
- [access-control](../access-control/) — Combined RBAC + Multisig + Timelock pattern
- [16-hierarchical-access-control](../../advanced/16-hierarchical-access-control/) — RBAC with dynamic permission inheritance
- [03-authentication](../../basics/03-authentication/) — Single-party auth basics

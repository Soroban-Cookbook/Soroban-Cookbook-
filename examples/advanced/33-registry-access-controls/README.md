Registry Access Controls
========================

> **Note:** For learning RBAC patterns, start with [**32-rbac-modifiers**](../32-rbac-modifiers/) — the canonical RBAC example. This example demonstrates a **domain-specific access control pattern** for registries with whitelist and fee enforcement.

Example demonstrating a registry with:

- Optional whitelist-only registration
- Configurable registration fee
- Simple dispute / removal workflow resolved by the owner

## What This Example Adds

This example shows a **specialized access control pattern** for registry use cases, distinct from general RBAC:

- **Owner-based** authorization (single admin rather than role system)
- **Whitelist management** for controlling registration access
- **Fee enforcement** with configurable amounts
- **Dispute resolution** workflow for removal requests

**When to use this pattern:**
- ✅ You're building a domain/name registry
- ✅ You need whitelist + fee gating for registrations
- ✅ You need a simple owner + dispute resolution model
- ✅ You don't need complex role hierarchies

**When to use the canonical RBAC pattern instead:**
- 🔄 You need multiple roles with different permissions
- 🔄 You need more than just an owner authorization model
- 🔄 Your access control isn't registry-specific

See `src/lib.rs` for the contract and `src/test.rs` for tests.


## Related Examples

- **[32-rbac-modifiers](../32-rbac-modifiers/)** — **Canonical RBAC pattern** with composable guards (use for general access control)
- [05-hierarchical-access-control](../05-hierarchical-access-control/) — Advanced RBAC with permission inheritance
- [02-role-based-access-control](../../intermediate/02-role-based-access-control/) — Simple RBAC with numeric hierarchy
- [29-merkle-whitelist](../29-merkle-whitelist/) — Alternative whitelist approach using Merkle proofs

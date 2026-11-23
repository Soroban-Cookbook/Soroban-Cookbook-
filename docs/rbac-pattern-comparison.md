# RBAC Pattern Comparison Guide

This guide helps you choose the right access control pattern for your Soroban smart contract.

## Quick Decision Tree

```
Do you need access control?
│
├─ YES → Continue below
└─ NO → Skip access control (use simple auth checks)

What kind of access control?
│
├─ Single admin only → Use simple admin pattern
│   └─ Example: examples/basics/03-authentication
│
├─ Multiple roles with different permissions → Choose RBAC pattern below
│   │
│   ├─ Need flexible, custom role names? → **32-rbac-modifiers** ✅ CANONICAL
│   │   └─ Roles: ADMIN, MINTER, PAUSER, or custom Symbol roles
│   │
│   ├─ Need strict numeric hierarchy? → 02-role-based-access-control
│   │   └─ Roles: Owner (4) > Admin (3) > Moderator (2) > User (1)
│   │
│   ├─ Need role inheritance + dynamic permissions? → 16-hierarchical-access-control
│   │   └─ Roles: ADMIN → MANAGER → OPERATOR with permission propagation
│   │
│   └─ Building a registry with whitelist/fees? → 33-registry-access-controls
│       └─ Owner + whitelist + registration fees + dispute resolution
│
└─ Need RBAC + Multisig + Timelock combined? → access-control
    └─ Layered governance with time delays and threshold approvals
```

## Pattern Comparison Table

| Feature | 32-rbac-modifiers<br/>**(CANONICAL)** | 02-role-based-access-control | 05-hierarchical-access-control | 33-registry-access-controls | access-control |
|---------|------------------------|------------------------------|--------------------------------|------------------------------|----------------|
| **Role Type** | Symbol-based (flexible) | Numeric enum (strict hierarchy) | Symbol-based + permissions | Owner-based | Numeric enum |
| **Custom Roles** | ✅ Yes, any Symbol | ❌ Fixed: Owner/Admin/Moderator/User | ✅ Yes, via permissions | ❌ No roles, just owner | ❌ Fixed: Admin/Auditor/Operator/User |
| **Composable Guards** | ✅ `only_role`, `any_role` | ❌ Manual checks | ❌ Permission-based only | ❌ No | ❌ Manual checks |
| **Role Hierarchy** | ❌ Flat (no hierarchy) | ✅ Numeric: 4 > 3 > 2 > 1 | ✅ ADMIN → MANAGER → OPERATOR | ❌ No hierarchy | ✅ Numeric: 3 > 2 > 1 > 0 |
| **Permission Inheritance** | ❌ No | ❌ No | ✅ Yes, dynamic | ❌ No | ❌ No |
| **Dynamic Permissions** | ❌ No | ❌ No | ✅ Yes, grant/revoke at runtime | ❌ No | ❌ No |
| **Multi-sig** | ❌ No | ❌ No | ❌ No | ❌ No | ✅ M-of-N threshold |
| **Timelock** | ❌ No | ❌ No | ❌ No | ❌ No | ✅ Delayed execution |
| **Whitelist** | ❌ No | ❌ No | ❌ No | ✅ Yes | ❌ No |
| **Registration Fees** | ❌ No | ❌ No | ❌ No | ✅ Yes | ❌ No |
| **Event Audit Trail** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ Minimal | ✅ Yes |
| **Workspace Member** | ✅ Yes | ❌ No (reference only) | ✅ Yes | ✅ Yes | ❌ No (reference only) |
| **Complexity** | Low | Low | Medium | Low | High |
| **Gas Cost** | Low | Low | Medium | Low | Medium-High |

## Pattern Details

### 1. 32-rbac-modifiers (Canonical) ✅

**Location:** `examples/advanced/32-rbac-modifiers/`

**Best for:**
- Token contracts (minter, pauser, burner roles)
- DeFi protocols with operator separation
- General-purpose access control
- Learning RBAC patterns

**Key Features:**
- Symbol-based roles: `ADMIN`, `MINTER`, `PAUSER`, or any custom `Symbol`
- Composable guards: `only_role(&env, &caller, ROLE_MINTER)`
- Multi-role guards: `any_role(&env, &caller, &[ROLE_ADMIN, ROLE_MINTER])`
- Role renunciation: accounts can remove their own roles
- Full event audit trail

**Example Use Cases:**
- Token with separate minter and pauser
- Protocol with admin and operator roles
- Contract with multiple specialized roles

**Code Example:**
```rust
// Grant role
client.grant_role(&admin, &ROLE_MINTER, &alice);

// Protected function
pub fn protected_mint(env: Env, caller: Address, to: Address, amount: i128) {
    caller.require_auth();
    Self::only_role(&env, &caller, ROLE_MINTER);
    // ... mint logic
}

// Multi-role guard
pub fn admin_or_minter_action(env: Env, caller: Address) {
    caller.require_auth();
    Self::any_role(&env, &caller, &[ROLE_ADMIN, ROLE_MINTER]);
    // ... logic
}
```

---

### 2. 02-role-based-access-control

**Location:** `examples/intermediate/02-role-based-access-control/`

**Best for:**
- Strict organizational hierarchies
- When numeric role comparison is natural
- Simpler permission model

**Key Features:**
- Numeric role hierarchy: `Owner (4) > Admin (3) > Moderator (2) > User (1)`
- Role comparison: `user_role >= required_role`
- Owner can grant any role
- Admin can grant Moderator and User only

**Example Use Cases:**
- Forum moderation system
- Content management with editors and contributors
- Simple organizational permission ladder

**When to use instead of canonical:**
- You need a fixed, unchangeable hierarchy
- Numeric comparisons simplify your logic
- Your permissions map naturally to a ladder

---

### 3. 16-hierarchical-access-control

**Location:** `examples/advanced/16-hierarchical-access-control/`

**Best for:**
- Complex organizational structures
- Dynamic permission systems
- Fine-grained access control

**Key Features:**
- Role hierarchy: `ADMIN → MANAGER → OPERATOR`
- Permission-based checks (not just role membership)
- Dynamic permission grants: add/remove permissions at runtime
- Permission inheritance: higher roles inherit lower permissions
- Fine-grained permissions: `MANAGE_ROLES`, `MANAGE_PERMS`, `MANAGE_RES`, `USE_RES`

**Example Use Cases:**
- Multi-department DAOs
- Enterprise systems with teams and departments
- Protocols where permissions evolve over time

**When to use instead of canonical:**
- You need roles to inherit permissions from other roles
- Permissions need to be added/removed without redeployment
- You have complex organizational hierarchies

**Code Example:**
```rust
// Grant role
client.grant_role(&admin, &ROLE_MANAGER, &alice);

// Grant permission to role
client.grant_permission(&admin, &PERM_MANAGE_RESOURCES, &ROLE_OPERATOR);

// Check permission (not just role)
pub fn manage_resource(env: Env, caller: Address, resource_id: Symbol) {
    caller.require_auth();
    Self::require_permission(&env, &caller, PERM_MANAGE_RESOURCES);
    // ... logic
}
```

---

### 4. 33-registry-access-controls

**Location:** `examples/advanced/33-registry-access-controls/`

**Best for:**
- Domain/name registries
- Membership systems with fees
- Registries with dispute resolution

**Key Features:**
- Owner-based authorization (single admin)
- Whitelist management for registration control
- Configurable registration fees
- Dispute resolution workflow

**Example Use Cases:**
- Domain name registry
- Membership club with admission fees
- Any registry with whitelist + fees

**When to use instead of canonical:**
- You're building a registry-specific system
- You need whitelist + fee enforcement
- Simple owner + dispute model is sufficient

---

### 5. access-control

**Location:** `examples/intermediate/access-control/`

**Best for:**
- DAO governance
- Treasury management
- High-security protocol upgrades

**Key Features:**
- Combines RBAC + Multisig + Timelock in one contract
- Layered security: roles + threshold approvals + time delays
- Proposal-based workflow
- Emergency pause mechanism

**Example Use Cases:**
- DAO treasury with time-delayed withdrawals
- Protocol governance requiring multi-party approval
- High-value asset management

**When to use instead of canonical:**
- You need all three: RBAC, multisig, and timelock
- Governance requires multiple approval gates
- Time delays are part of your security model

**Code Example:**
```rust
// Initialize with RBAC + multisig + timelock
client.initialize(&admin, &threshold, &signers, &timelock_delay);

// Grant role (RBAC)
client.grant_role(&admin, &operator, &Role::Operator);

// Create proposal (operator creates)
let proposal_id = client.create_proposal(&operator, &action);

// Approve proposal (signers approve)
client.approve(&signer1, &proposal_id);
client.approve(&signer2, &proposal_id);

// Execute after timelock
client.execute(&anyone, &proposal_id); // After delay expires
```

---

## Migration Guide

### From No Access Control → Canonical RBAC

1. Add `32-rbac-modifiers` code to your contract
2. Initialize with admin: `initialize(env, initial_admin)`
3. Add `only_role` checks to protected functions
4. Grant roles to addresses as needed

### From 02-role-based-access-control → Canonical RBAC

**Why migrate:**
- Need custom roles beyond Owner/Admin/Moderator/User
- Want composable multi-role guards
- Need more flexibility without numeric constraints

**Migration steps:**
1. Map numeric roles to symbols: `Owner → ADMIN`, `Admin → OPERATOR`
2. Replace numeric checks with `only_role` guards
3. Update role grants to use symbol-based API

### From Canonical RBAC → Hierarchical RBAC

**Why migrate:**
- Need permission inheritance
- Want to add/remove permissions dynamically
- Have complex organizational hierarchies

**Migration steps:**
1. Map roles to hierarchy: decide which roles inherit from others
2. Define permissions for each role
3. Replace role checks with permission checks
4. Add permission management functions

---

## Security Considerations

### All RBAC Patterns

✅ **Do:**
- Always call `require_auth()` before role checks
- Emit events for all role changes
- Use idempotent grant/revoke (safe to call multiple times)
- Provide role renunciation for emergency exits

❌ **Don't:**
- Mix role storage tiers (use consistent storage)
- Allow self-granting of roles
- Skip role checks on privileged functions
- Forget to initialize the contract

### Canonical Pattern Specific

✅ **Do:**
- Use well-known role symbols consistently (`ROLE_ADMIN`, `ROLE_MINTER`)
- Combine roles with pause/unpause for emergency controls
- Document which roles can call which functions

❌ **Don't:**
- Use dynamic symbols without documentation
- Bypass `only_role` guards in internal functions
- Forget that roles are flat (no automatic hierarchy)

---

## Performance Comparison

| Pattern | Storage Reads per Check | Gas Cost | Best For |
|---------|------------------------|----------|----------|
| 32-rbac-modifiers | 1 (role member list) | Low | Most use cases |
| 02-role-based-access-control | 1 (user role) | Low | Simple hierarchies |
| 05-hierarchical-access-control | 1-3 (role + permissions) | Medium | Complex hierarchies |
| 33-registry-access-controls | 1-2 (owner + whitelist) | Low | Registries |
| access-control | 3-5 (role + signers + proposals) | Medium-High | Governance |

---

## Testing Recommendations

### For All Patterns

Test these scenarios:
1. ✅ Initialization (single-time only)
2. ✅ Role grant by authorized account
3. ✅ Role grant by unauthorized account (should fail)
4. ✅ Protected function with correct role
5. ✅ Protected function without role (should fail)
6. ✅ Role revocation
7. ✅ Role renunciation
8. ✅ Event emission for all role changes

### Pattern-Specific Tests

**Canonical (32-rbac-modifiers):**
- Multi-role guard (`any_role`) with multiple roles
- Custom role symbols
- Idempotent grants

**Hierarchical (16-hierarchical-access-control):**
- Permission inheritance across hierarchy
- Dynamic permission grants
- Manager can grant OPERATOR but not ADMIN

**Combined (access-control):**
- Proposal lifecycle (create → approve → execute)
- Timelock enforcement
- Threshold requirement

---

## Recommended Learning Path

1. **Start here:** [`32-rbac-modifiers`](../examples/advanced/32-rbac-modifiers/) — Learn the canonical pattern
2. **Then explore:**
   - [`02-role-based-access-control`](../examples/intermediate/02-role-based-access-control/) — See numeric hierarchy alternative
   - [`16-hierarchical-access-control`](../examples/advanced/16-hierarchical-access-control/) — Learn permission inheritance
3. **For specialized needs:**
   - [`33-registry-access-controls`](../examples/advanced/33-registry-access-controls/) — Registry pattern
   - [`access-control`](../examples/intermediate/access-control/) — Combined governance

---

## Related Documentation

- [Governance & Authorization Patterns](./governance-rbac-multisig-timelock.md) — RBAC, multisig, timelock combined
- [Common Patterns](./common-patterns.md) — RBAC pattern reference
- [Advanced Patterns](./advanced-patterns.md) — Architectural patterns
- [Security Best Practices](./security-best-practices.md) — Access control security
- [Architecture Guide](./architecture-guide.md) — Choosing access control models

---

## Summary

| Your Need | Choose This Pattern |
|-----------|---------------------|
| 🎯 **General-purpose RBAC** | **32-rbac-modifiers (CANONICAL)** |
| 📊 Strict numeric hierarchy | 02-role-based-access-control |
| 🏢 Complex org structure | 05-hierarchical-access-control |
| 📝 Registry with whitelist | 33-registry-access-controls |
| 🏛️ DAO governance | access-control |

**When in doubt, start with the canonical pattern (`32-rbac-modifiers`) and migrate only if you hit its limitations.**

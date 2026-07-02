# Hierarchical Access Control

This example demonstrates a hierarchical role-based access control (RBAC) system with dynamic permissions for Soroban smart contracts. It implements a multi-tier role architecture, fine-grained permission checks, and secure runtime updates.

## 📖 What You'll Learn

- **Hierarchical Roles**: Implementing ADMIN → MANAGER → OPERATOR role hierarchy
- **Permission System**: Designing fine-grained, verifiable permission checks
- **Dynamic Updates**: Securely updating roles and permissions at runtime
- **Event Logging**: Emitting events for all role/permission changes for audit trails
- **Security Best Practices**: Preventing privilege escalation and ensuring secure access control

## 🔍 Contract Overview

The contract implements a hierarchical RBAC system with three core roles and four default permissions:

### Roles & Permissions

| Role | Permissions |
|------|-------------|
| **ADMIN** | MANAGE_ROLES, MANAGE_PERMS, MANAGE_RES, USE_RES |
| **MANAGER** | MANAGE_RES, USE_RES |
| **OPERATOR** | USE_RES |

### Key Functions

#### Role Management

```rust
pub fn grant_role(env: Env, caller: Address, role: Symbol, account: Address)
pub fn revoke_role(env: Env, caller: Address, role: Symbol, account: Address)
pub fn renounce_role(env: Env, caller: Address, role: Symbol)
```

#### Permission Management

```rust
pub fn grant_permission(env: Env, caller: Address, permission: Symbol, role: Symbol)
pub fn revoke_permission(env: Env, caller: Address, permission: Symbol, role: Symbol)
```

#### Queries

```rust
pub fn has_role(env: Env, role: Symbol, account: Address) -> bool
pub fn get_role_members(env: Env, role: Symbol) -> Vec<Address>
pub fn role_has_permission(env: Env, role: Symbol, permission: Symbol) -> bool
pub fn get_role_permissions(env: Env, role: Symbol) -> Vec<Symbol>
pub fn account_has_permission(env: Env, account: Address, permission: Symbol) -> bool
```

#### Protected Operations

```rust
pub fn manage_resource(env: Env, caller: Address, resource_id: Symbol) // Requires MANAGE_RES
pub fn use_resource(env: Env, caller: Address, resource_id: Symbol) // Requires USE_RES
```

## 💡 Key Concepts

### Role Hierarchy

Permissions are inherited through the role hierarchy. For example, an ADMIN automatically has all permissions granted to MANAGER and OPERATOR roles.

### Permission Checks

When checking permissions, the contract verifies all roles held by the account in order of hierarchy (ADMIN first, then MANAGER, then OPERATOR). If any role has the required permission, access is granted.

### Audit Trails

Every role or permission change emits a structured event:

```rust
#[contracttype]
pub struct RoleGrantedEventData {
    pub role: Symbol,
    pub account: Address,
    pub sender: Address,
    pub timestamp: u64,
}
```

This allows off-chain indexers to reconstruct the full history of access control changes.

## 🔒 Security Considerations

### Privilege Escalation Prevention

- Only accounts with `MANAGE_ROLES` permission can grant/revoke roles
- Only accounts with `MANAGE_PERMS` permission can modify permissions
- Roles and permissions are checked in a strict hierarchy

### Authorization

All sensitive operations require the caller to authorize using `require_auth()`, ensuring only legitimate users can make changes.

### Initialization

The contract can only be initialized once, preventing accidental or malicious reinitialization.

### Idempotent Operations

Role grants and permission grants are idempotent, meaning granting a role/permission that an account/role already has has no effect.

## 🧪 Testing

```bash
cargo test -p hierarchical-access-control
```

Tests cover:
- ✅ Initialization and default role/permission setup
- ✅ Role grant/revoke by authorized users
- ✅ Permission grant/revoke by authorized users
- ✅ Protected operation access control
- ✅ Hierarchical permission checks
- ✅ Event emission for audit trails
- ✅ Unauthorized access rejection

## 🚀 Building & Deployment

```bash
# Build
cargo build --target wasm32-unknown-unknown --release -p hierarchical-access-control

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hierarchical_access_control.wasm \
  --source alice \
  --network testnet
```

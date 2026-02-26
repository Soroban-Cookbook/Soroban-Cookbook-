# Custom Authorization Logic

This example demonstrates advanced authorization patterns in Soroban smart contracts that go beyond basic `require_auth()`. While Soroban's built-in auth verifies that a caller *is who they claim to be*, real contracts also need to verify that the caller *is allowed to do what they're trying to do*.

## Concepts Covered

- **`require_auth()`**: Core function for verifying transaction authorization
- **Role-Based Access Control (RBAC)**: Assign Admin, Moderator, or User roles and gate functions by role
- **Time-Based Restrictions**: Time-locks that prevent actions before a deadline and cooldowns that throttle repeated calls
- **State-Based Authorization**: Contract-wide state machine (Active, Paused, Frozen) that conditionally disables functionality
- **Custom Auth Conditions**: Implementing business logic authorization rules
- **Extensibility Patterns**: Building composable authorization systems

## Key Functions

### 1. Basic Authentication Pattern
```rust
pub fn basic_auth(env: Env, user: Address) -> bool {
    user.require_auth();  // Verify the user authorized this transaction
    true
}
```

### 2. Transfer Pattern
```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> bool {
    from.require_auth();  // Only 'from' address can initiate transfer
    // Transfer logic...
    true
}
```

### 3. Admin-Only Pattern
```rust
pub fn set_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), AuthError> {
    // Verify current admin status
    if admin != stored_admin {
        return Err(AuthError::AdminOnly);
    }
    admin.require_auth();  // Admin must authorize the change
    env.storage().instance().set(&ADMIN_KEY, &new_admin);
    Ok(())
}
```

### 4. Role-Based Access Control
```rust
pub fn admin_action(env: Env, caller: Address, value: u64) -> u64 {
    caller.require_auth();
    Self::require_role(&env, &caller, &[Role::Admin]);  // Check role permission
    let result = value * 2;
    env.events().publish((symbol_short!("admin"),), result);
    result
}
```

### 5. Time-Based Authorization
```rust
pub fn time_locked_action(env: Env, caller: Address) -> u64 {
    caller.require_auth();
    
    let unlock_time: u64 = env.storage().instance().get(&DataKey::TimeLock).unwrap_or(0);
    if env.ledger().timestamp() < unlock_time {
        panic!("Action is time-locked");
    }
    
    env.ledger().timestamp()
}
```

### 6. Cooldown Protection
```rust
pub fn cooldown_action(env: Env, caller: Address) -> u64 {
    caller.require_auth();
    
    let period: u64 = env.storage().instance().get(&DataKey::CooldownPeriod).unwrap_or(0);
    let last_action: u64 = env.storage().persistent()
        .get(&DataKey::LastAction(caller.clone())).unwrap_or(0);
    
    let now = env.ledger().timestamp();
    if last_action > 0 && now < last_action + period {
        panic!("Cooldown period not elapsed");
    }
    
    env.storage().persistent().set(&DataKey::LastAction(caller.clone()), &now);
    now
}
```

### 7. State-Based Authorization
```rust
pub fn active_only_action(env: Env, caller: Address) -> u64 {
    caller.require_auth();
    
    let state: ContractState = env.storage().instance()
        .get(&DataKey::State).unwrap_or(ContractState::Active);
    
    if state != ContractState::Active {
        panic!("Contract is not active");
    }
    
    env.ledger().timestamp()
}
```

## Security Considerations

### ✅ Best Practices
- **Always call `require_auth()` before state changes**
- **Place auth checks early in function**
- **Validate inputs after authentication**
- **Use custom error types for different failure scenarios**

### ❌ Common Mistakes to Avoid
- **Forgetting to call `require_auth()`**
- **Calling it after state changes**
- **Not handling auth failures properly**
- **Confusing authorization with authentication**

## How Authentication Works

The `require_auth()` function:

1. **Verifies Transaction Signatures**: Ensures the address has signed the current transaction
2. **Prevents Unauthorized Access**: Stops malicious actors from calling functions on behalf of others
3. **Enables Secure Operations**: Allows only authorized parties to perform sensitive actions
4. **Works with Both Accounts and Contracts**: Can authenticate both user accounts and smart contracts

## When to Use `require_auth()`

Use `require_auth()` whenever:
- Transferring assets or value
- Modifying user-specific data
- Changing contract configuration
- Performing privileged operations
- Accessing sensitive information

## Error Handling

The example demonstrates proper error handling with custom error types:

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AuthError {
    Unauthorized = 1,
    AdminOnly = 2,
    InvalidAddress = 3,
}
```

## Running Tests

To run the tests for this example:

```bash
cd examples/basics/03-authentication
cargo test
```

## Deployment

To build for deployment:

```bash
cd examples/basics/03-authentication
cargo build --target wasm32-unknown-unknown --release
```

The resulting WASM file will be in `target/wasm32-unknown-unknown/release/auth-patterns.wasm`.

## Additional Resources

- [Soroban Authentication Guide](https://developers.stellar.org/docs/glossary/authentication)
- [Authorization Best Practices](https://developers.stellar.org/docs/guides/security-best-practices)
- [Soroban SDK Documentation](https://docs.rs/soroban-sdk/)
# Soroban Authentication Example: Comprehensive Guide

## Introduction To Authentication
Authentication in Soroban smart contracts ensures only authorized users can interact with sensitive functions. Security mistakes can be costly, so understanding and applying best practices is critical.

## require_auth Basics
The `require_auth()` method verifies that the caller has authorized the action. Always use it before changing contract state to prevent unauthorized access.

## Multi-Party Authorization
Multi-signature and threshold patterns require multiple parties to approve actions. Use these for high-value operations, like treasury management or governance.

## Authorization Context
Soroban allows checking not just who called, but also what arguments were authorized. Use `require_auth_for_args()` to ensure the user signed for specific parameters, preventing replay attacks.

## Custom Authorization Patterns
You can implement role-based access, time-based restrictions, or combine multiple checks for advanced security. For example, restrict admin functions to a stored admin address, or require both user and admin signatures for sensitive actions.

## Security Best Practices
- Always use `require_auth()` or `require_auth_for_args()` before state changes.
- Validate all inputs and parameters; never trust external data blindly.
- Use safe arithmetic (e.g., `checked_add`) to prevent overflows.
- Restrict admin functions to authorized accounts only.
- Handle errors with clear, custom error types.

## Common Mistakes
- Forgetting to call `require_auth()` before state changes.
- Not validating input values, leading to vulnerabilities.
- Using unchecked arithmetic, causing overflows.
- Leaving admin functions unprotected.
- Using generic panics instead of descriptive errors.

## Real-World Use Cases
- **Token Transfers:** Only the token owner can approve sending tokens.
- **Multi-Sig Wallets:** Multiple parties must approve transactions.
- **DAO Governance:** Only authenticated members can vote or propose changes.
- **Admin Functions:** Only admins can update contract settings.
- **Escrow Services:** Funds are released only when all parties have authenticated.

## Testing Authentication
To test authentication patterns, use Soroban's test framework. Write unit tests to check that unauthorized calls fail and authorized calls succeed. See `src/test.rs` for examples.

## Further Reading
- [Best Practices Guide](../../../docs/best-practices.md)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Smart Contract Security Resources](https://consensys.github.io/smart-contract-best-practices/)

For more details and code examples, see the contract in `src/lib.rs`.
# Authentication & Custom Authorization

Learn how to build custom authorization logic in Soroban smart contracts, including role-based access control, time-based restrictions, and state-dependent permissions.

## 📖 What You'll Learn

- Combining `require_auth()` with custom authorization checks
- Implementing role-based access control (Admin, Moderator, User)
- Time-locked operations and cooldown periods
- State-dependent authorization gating (Active / Paused / Frozen)
- Security best practices for on-chain access control

## 🔍 Contract Overview

This example demonstrates three complementary authorization patterns that work together to form a complete access-control system:

### Role-Based Access Control (RBAC)

```rust
pub fn initialize(env: Env, admin: Address)
pub fn grant_role(env: Env, admin: Address, account: Address, role: Role)
pub fn revoke_role(env: Env, admin: Address, account: Address)
pub fn get_role(env: Env, account: Address) -> u32
pub fn has_role(env: Env, account: Address, role: Role) -> bool
pub fn admin_action(env: Env, caller: Address, value: u64) -> u64
pub fn moderator_action(env: Env, caller: Address, value: u64) -> u64
```

### Time-Based Restrictions

```rust
pub fn set_time_lock(env: Env, admin: Address, unlock_time: u64)
pub fn time_locked_action(env: Env, caller: Address) -> u64
pub fn set_cooldown(env: Env, admin: Address, period: u64)
pub fn cooldown_action(env: Env, caller: Address) -> u64
```

### State-Based Authorization

```rust
pub fn set_state(env: Env, admin: Address, state: ContractState)
pub fn get_state(env: Env) -> u32
pub fn active_only_action(env: Env, caller: Address) -> u64
```

## 💡 Key Concepts

### Role Hierarchy

Roles are defined as an enum stored in persistent storage:

```rust
#[contracttype]
pub enum Role {
    Admin = 0,
    Moderator = 1,
    User = 2,
}
```

- **Admin** — Full access; can grant/revoke roles, configure time-locks, cooldowns, and contract state.
- **Moderator** — Mid-tier access; can perform moderator-level actions but not admin-only operations.
- **User** — Basic access; cannot perform privileged actions.

Admins implicitly satisfy moderator-level checks, so `moderator_action` accepts both Admin and Moderator callers.

### Time-Lock Pattern

A global unlock timestamp prevents actions until a future ledger time:

```rust
let current_time = env.ledger().timestamp();
let unlock_time = env.storage().instance().get(&DataKey::TimeLock).unwrap();
if current_time < unlock_time {
    panic!("Action is time-locked");
}
```

Use time-locks for vesting schedules, delayed withdrawals, or governance cool-off periods.

### Cooldown Pattern

Per-address cooldowns enforce a minimum interval between successive calls:

```rust
let last = env.storage().persistent().get(&DataKey::LastAction(caller.clone()));
if let Some(last_ts) = last {
    if current_time < last_ts + cooldown_period {
        panic!("Cooldown period not elapsed");
    }
}
```

Cooldowns mitigate spam and rate-limit sensitive operations without off-chain infrastructure.

### Contract State Gating

A global state enum controls whether critical operations are allowed:

```rust
#[contracttype]
pub enum ContractState {
    Active = 0,
    Paused = 1,
    Frozen = 2,
}
```

Only the `Active` state permits normal operations. `Paused` and `Frozen` block `active_only_action`, giving admins an emergency circuit-breaker.

## 🔒 Security Best Practices

1. **Always call `require_auth()` first** — Verify the caller's cryptographic identity before any custom checks.
2. **Separate auth from business logic** — Keep role checks and time guards in distinct, auditable code paths.
3. **Use persistent storage for roles** — Instance storage risks loss on contract upgrade; persistent storage survives.
4. **Minimize admin surface** — Only expose `grant_role`, `revoke_role`, and configuration setters to the admin.
5. **Test edge cases** — Verify behavior at exact boundary timestamps (unlock time, cooldown expiry).
6. **Prefer enums over integers** — `Role` and `ContractState` enums prevent invalid values at the type level.
7. **Fail loudly** — Use `panic!` with descriptive messages so callers and auditors understand rejection reasons.

## 🧪 Testing

```bash
cargo test
```

Tests cover:

- **Initialization** — Admin is set, double-init is rejected
- **Role management** — Grant, revoke, get, and has_role checks
- **Admin actions** — Authorized admin succeeds, non-admin panics
- **Moderator actions** — Admin and Moderator succeed, User panics
- **Time-lock** — Action blocked before unlock, succeeds after
- **Cooldown** — Second call blocked within period, succeeds after
- **State gating** — Active allows action; Paused and Frozen block it

## 🚀 Building & Deployment

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/authentication.wasm \
  --source alice \
  --network testnet
```

## ✅ Implementation Summary

This example implements comprehensive custom authorization logic covering all major patterns:

### **Custom Auth Conditions**
- Business logic authorization rules beyond basic `require_auth()`
- Conditional checks based on contract state, time, and roles
- Composable authorization patterns

### **Role-Based Access Control (RBAC)**
- Three-tier role system: Admin, Moderator, User
- Role assignment and revocation functions
- Permission-gated functions with role checking
- Persistent storage of role assignments

### **Time-Based Authorization**
- **Time Locks**: Prevent actions until a future timestamp
- **Cooldowns**: Enforce minimum intervals between actions
- Ledger timestamp integration for temporal controls

### **Extensibility Patterns**
- Modular authorization helpers (`require_admin`, `require_role`)
- State-based contract controls (Active/Paused/Frozen)
- Event emission for authorization changes
- Storage tier selection (instance vs persistent) based on data lifecycle

## 🎓 Next Steps

- [Basics Index](../README.md) - Browse the full basics learning path
- [Events](../04-events/) - Emit audit-trail events alongside auth checks
- [Storage Patterns](../02-storage-patterns/) - Understand how roles are persisted
- [Intermediate Examples](../../intermediate/) - Multi-contract authorization patterns

## 📚 References

- [Soroban Authorization](https://developers.stellar.org/docs/smart-contracts/fundamentals-and-concepts/authorization)
- [Soroban SDK Auth](https://docs.rs/soroban-sdk/latest/soroban_sdk/auth/index.html)
- [Custom Account Contracts](https://developers.stellar.org/docs/smart-contracts/guides/custom-accounts)

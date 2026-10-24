---
marp: true
theme: default
paginate: true
---

# Module 3: Authentication & Access Control

## Soroban Development Workshop

---

# The `require_auth()` Pattern

Soroban's built-in authorization. Every signed call must authenticate.

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    // Require that `from` has signed this transaction
    from.require_auth();

    // Now we know the caller is authorized by `from`
    // ... perform transfer
}
```

This is the foundation of all Soroban auth patterns.

---

# How `require_auth()` Works

1. Contract calls `address.require_auth()`
2. Soroban host checks if the transaction includes a valid signature for that address
3. If no valid signature → transaction reverts
4. If valid → execution continues

Key insight: **the host enforces auth, not the contract.** This is more secure than Solidity's `msg.sender` approach.

---

# Authorization Patterns

### 1. Owner-Only

```rust
if caller != admin {
    return Err(MyError::Unauthorized);
}
caller.require_auth();
```

### 2. Role-Based (RBAC)

```rust
if !has_role(&env, &caller, Role::Admin) {
    return Err(MyError::Unauthorized);
}
caller.require_auth();
```

### 3. Multi-Signature

```rust
// Require M-of-N signers to approve
if count_valid_signers(&signers) < threshold {
    return Err(MyError::Unauthorized);
}
```

---

# Initialization Pattern

Contracts need a one-time init function (no constructor in Soroban).

```rust
pub fn initialize(env: Env, admin: Address) -> Result<(), MyError> {
    // Prevent re-initialization
    if env.storage().instance().has(&DataKey::Admin) {
        return Err(MyError::AlreadyInitialized);
    }

    env.storage().instance().set(&DataKey::Admin, &admin);
    Ok(())
}
```

Source: [`examples/basics/03-authentication/`](../../examples/basics/03-authentication/)

---

# Role-Based Access Control

```rust
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role {
    Admin = 1,
    Moderator = 2,
    User = 3,
}

pub fn grant_role(env: Env, caller: Address, user: Address, role: Role) -> Result<(), Error> {
    caller.require_auth();

    // Only admins can grant roles
    if !has_role(&env, &caller, Role::Admin) {
        return Err(Error::Unauthorized);
    }

    // Admins can't grant admin role (only owner can)
    if role == Role::Admin && !is_owner(&env, &caller) {
        return Err(Error::Unauthorized);
    }

    set_role(&env, &user, role);
    Ok(())
}
```

Source: [`examples/intermediate/02-role-based-access-control/`](../../examples/intermediate/02-role-based-access-control/)

---

# RBAC Hierarchy

A common pattern: roles cascade permissions.

```
Owner  → can do everything
  ↓
Admin  → can manage non-admin roles
  ↓
Moderator → can manage content
  ↓
User   → basic access
```

```rust
fn can_grant(granter_role: Role, target_role: Role) -> bool {
    match (granter_role, target_role) {
        (Role::Admin, Role::Moderator | Role::User) => true,
        _ => false,
    }
}
```

---

# Multi-Signature Pattern

M-of-N approval for sensitive operations.

```rust
#[contracttype]
pub struct Proposal {
    pub id: u32,
    pub data: Symbol,
    pub approvals: Vec<Address>,
    pub executed: bool,
}

pub fn execute_proposal(env: Env, proposal_id: u32) -> Result<(), Error> {
    let mut proposal = get_proposal(&env, proposal_id)?;

    // Check threshold
    if proposal.approvals.len() < threshold {
        return Err(Error::ThresholdNotMet);
    }

    // Prevent double execution
    if proposal.executed {
        return Err(Error::AlreadyExecuted);
    }

    // Execute the proposal...
    proposal.executed = true;
    Ok(())
}
```

Source: [`examples/intermediate/multi-sig-patterns/`](../../examples/intermediate/multi-sig-patterns/)

---

# Pausable Pattern

Emergency stop mechanism.

```rust
pub fn pause(env: Env, caller: Address) -> Result<(), PauseError> {
    caller.require_auth();
    require_admin(&env, &caller)?;

    env.storage().instance().set(&DataKey::Paused, &true);
    Ok(())
}

// Called at the top of any function that should be pausable
fn require_not_paused(env: &Env) -> Result<(), PauseError> {
    if env.storage().instance().get(&DataKey::Paused).unwrap_or(false) {
        return Err(PauseError::Paused);
    }
    Ok(())
}
```

Source: [`examples/intermediate/03-pause-unpause/`](../../examples/intermediate/03-pause-unpause/)

---

# Auth Context: Getting the Invoker

```rust
// Get the contract's own address
let self_address = env.current_contract_address();

// Get the invoker (the top-level caller of this transaction)
let invoker = env.invoker();

// Check if a specific address authorized
let is_authed = env.auth().has(&some_address);
```

Source: [`examples/basics/05-auth-context/`](../../examples/basics/05-auth-context/)

---

# Testing Authentication

In tests, use `env.mock_auths()` to simulate signed transactions.

```rust
#[test]
fn test_admin_action() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // Mock auth for admin
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "admin_action",
            args: (),
            sub_invokes: &[],
        },
    }]);

    // This call now passes auth
    client.admin_action(&admin);
}
```

---

# Common Auth Pitfalls

1. **Forgetting `require_auth()`** — the #1 security bug
2. **Not checking roles** — `require_auth()` only proves they signed, not that they're authorized
3. **Re-initialization** — always guard init with an "already initialized" check
4. **Not testing unauthorized paths** — always test that unauthorized callers are rejected

---

# Module 3 Summary

- `require_auth()` is Soroban's built-in authorization primitive
- RBAC: store roles in persistent storage, check before privileged operations
- Multi-sig: collect approvals, check threshold before execution
- Pausable: emergency stop with a boolean flag and guard function
- Always guard initialization against re-initialization
- Test both authorized and unauthorized paths

---

# Up Next

**Module 4: Tokens & SEP-41 Standard**
- SEP-41 token interface
- Minting, burning, allowances
- Token metadata

---

# Exercise Time!

See [Exercise 3](../exercises/module-3-exercise.md)

**Goal:** Build an access-controlled contract with RBAC and pausable functionality.

⏱️ 45 minutes

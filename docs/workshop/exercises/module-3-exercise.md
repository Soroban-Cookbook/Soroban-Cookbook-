# Module 3 Exercise: Authentication & Access Control

**Objective:** Build an access-controlled contract with RBAC and pausable functionality.

**Duration:** 45 minutes

---

## Prerequisites

- Complete Modules 1 and 2
- Understand storage patterns and error handling

---

## Reference Examples

- [`examples/basics/03-authentication/`](../../../examples/basics/03-authentication/) — Auth patterns
- [`examples/intermediate/02-role-based-access-control/`](../../../examples/intermediate/02-role-based-access-control/) — RBAC
- [`examples/intermediate/03-pause-unpause/`](../../../examples/intermediate/03-pause-unpause/) — Pausable

---

## Part 1: Build the Contract (25 min)

### Contract Requirements

Create a "Vault" contract where:

1. Admin can deposit/withdraw any amount
2. Users with the `Operator` role can deposit but not withdraw
3. Anyone can check the contract's total balance
4. Admin can pause/unpause the contract
5. While paused, only `get_balance()` works — all other functions fail
6. Admin can grant/revoke roles

### Starter Code

```rust
#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// TODO: Define Role enum (Admin = 1, Operator = 2, User = 3)

// TODO: Define DataKey enum:
//   - Admin
//   - Paused
//   - Role(Address)  // stores the role of each user
//   - Balance(Address)

// TODO: Define VaultError enum:
//   - NotInitialized = 1
//   - Unauthorized = 2
//   - ContractPaused = 3
//   - InsufficientBalance = 4
//   - InvalidAmount = 5

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    // TODO: initialize(env, admin: Address) -> Result<(), VaultError>
    //   - Store admin, set admin role, set paused = false

    // TODO: pause(env, caller: Address) -> Result<(), VaultError>
    //   - Require auth, check admin, set paused = true

    // TODO: unpause(env, caller: Address) -> Result<(), VaultError>
    //   - Require auth, check admin, set paused = false

    // TODO: grant_role(env, caller: Address, user: Address, role: Role) -> Result<(), VaultError>
    //   - Require auth, check admin
    //   - Only admin can grant roles
    //   - Cannot grant Admin role (admin is set during init only)

    // TODO: revoke_role(env, caller: Address, user: Address) -> Result<(), VaultError>
    //   - Require auth, check admin

    // TODO: deposit(env, caller: Address, amount: i128) -> Result<(), VaultError>
    //   - Require auth
    //   - Check not paused
    //   - Require role >= Operator (i.e., Admin or Operator)
    //   - Add to caller's balance

    // TODO: withdraw(env, caller: Address, amount: i128) -> Result<(), VaultError>
    //   - Require auth
    //   - Check not paused
    //   - Require Admin role
    //   - Check sufficient balance
    //   - Subtract from balance

    // TODO: get_balance(env, user: Address) -> i128
    //   - Works even when paused
    //   - Returns user's balance

    // TODO: get_role(env, user: Address) -> Option<Role>
    //   - Returns the user's role
}
```

---

## Part 2: Write Tests (15 min)

### Test Cases to Implement

```rust
#[test]
fn test_initialize_and_admin() { /* ... */ }

#[test]
fn test_deposit_operator() {
    // Grant operator role, deposit, verify balance
}

#[test]
fn test_withdraw_admin_only() {
    // Admin withdraws → succeeds
    // Operator withdraws → fails with Unauthorized
}

#[test]
fn test_pause_blocks_operations() {
    // Pause contract
    // Deposit → fails with ContractPaused
    // Withdraw → fails with ContractPaused
    // get_balance → succeeds
}

#[test]
fn test_grant_role_admin_only() {
    // Non-admin tries to grant role → fails
    // Admin grants role → succeeds
}

#[test]
fn test_cannot_grant_admin_role() {
    // Admin tries to grant Admin role to someone → fails
}

#[test]
fn test_revoke_role() {
    // Grant, verify, revoke, verify access is gone
}
```

### Auth Testing Helper

Use this pattern to mock auth in tests:

```rust
use soroban_sdk::{testutils::{MockAuth, MockAuthInvoke}, Address};

fn mock_auth(env: &Env, contract_id: &Address, address: &Address, fn_name: &str) {
    env.mock_auths(&[MockAuth {
        address,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name,
            args: (),
            sub_invokes: &[],
        },
    }]);
}
```

---

## Part 3: Build and Test (5 min)

```bash
cargo build --target wasm32-unknown-unknown --release
cargo test
```

---

## Verification Checklist

- [ ] Admin can deposit and withdraw
- [ ] Operators can deposit but not withdraw
- [ ] Users without a role can't deposit or withdraw
- [ ] Pausing blocks all operations except `get_balance`
- [ ] Unpausing restores functionality
- [ ] Only admin can grant/revoke roles
- [ ] Admin role cannot be granted (only via initialization)
- [ ] All unauthorized attempts return `VaultError::Unauthorized`
- [ ] All tests pass

---

## Solution

<details>
<summary>Click to reveal the solution</summary>

```rust
#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Role {
    User = 3,
    Operator = 2,
    Admin = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Paused,
    Role(Address),
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    NotInitialized = 1,
    Unauthorized = 2,
    ContractPaused = 3,
    InsufficientBalance = 4,
    InvalidAmount = 5,
    AlreadyInitialized = 6,
}

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), VaultError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VaultError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Role(admin.clone()), &Role::Admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    pub fn pause(env: Env, caller: Address) -> Result<(), VaultError> {
        caller.require_auth();
        require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        Ok(())
    }

    pub fn unpause(env: Env, caller: Address) -> Result<(), VaultError> {
        caller.require_auth();
        require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    pub fn grant_role(env: Env, caller: Address, user: Address, role: Role) -> Result<(), VaultError> {
        caller.require_auth();
        require_admin(&env, &caller)?;
        if role == Role::Admin {
            return Err(VaultError::Unauthorized);
        }
        env.storage().persistent().set(&DataKey::Role(user), &role);
        Ok(())
    }

    pub fn revoke_role(env: Env, caller: Address, user: Address) -> Result<(), VaultError> {
        caller.require_auth();
        require_admin(&env, &caller)?;
        env.storage().persistent().remove(&DataKey::Role(user));
        Ok(())
    }

    pub fn deposit(env: Env, caller: Address, amount: i128) -> Result<(), VaultError> {
        caller.require_auth();
        require_not_paused(&env)?;
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        let role = get_role(&env, &caller).ok_or(VaultError::Unauthorized)?;
        if role > Role::Operator {
            return Err(VaultError::Unauthorized);
        }
        let current = read_balance(&env, &caller);
        let new_balance = current.checked_add(amount).ok_or(VaultError::InvalidAmount)?;
        env.storage().persistent().set(&DataKey::Balance(caller), &new_balance);
        Ok(())
    }

    pub fn withdraw(env: Env, caller: Address, amount: i128) -> Result<(), VaultError> {
        caller.require_auth();
        require_not_paused(&env)?;
        require_admin(&env, &caller)?;
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        let current = read_balance(&env, &caller);
        if current < amount {
            return Err(VaultError::InsufficientBalance);
        }
        env.storage().persistent().set(&DataKey::Balance(caller.clone()), &(current - amount));
        Ok(())
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    pub fn get_role(env: Env, user: Address) -> Option<Role> {
        get_role(&env, &user)
    }
}

fn require_admin(env: &Env, addr: &Address) -> Result<(), VaultError> {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(VaultError::NotInitialized)?;
    if addr != &admin {
        let role = get_role(env, addr);
        if role != Some(Role::Admin) {
            return Err(VaultError::Unauthorized);
        }
    }
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), VaultError> {
    let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
    if paused {
        return Err(VaultError::ContractPaused);
    }
    Ok(())
}

fn get_role(env: &Env, user: &Address) -> Option<Role> {
    env.storage().persistent().get(&DataKey::Role(user.clone()))
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage().persistent().get(&DataKey::Balance(user.clone())).unwrap_or(0)
}
```

</details>

---

## Next Steps

Continue to [Module 4: Tokens & SEP-41 Standard](../slides/module-4-tokens.md) to build a fungible token.

# Playground Tutorial 4: Authentication & Authorization

## Overview
Learn to secure your Soroban contracts with authentication and authorization mechanisms. Control who can call what functions.

**Duration**: 30-35 minutes
**Difficulty**: Intermediate
**Prerequisites**: Tutorials 1-3 completion

## Learning Objectives
- Implement address-based authentication
- Create role-based access control
- Use the `require_auth` pattern
- Verify caller identity

## Step 1: Understanding Authentication vs Authorization

**Authentication**: Who are you?
- Verify the caller's identity
- Use the `require_auth()` function

**Authorization**: What can you do?
- Check caller's permissions/roles
- Compare against stored access rights

## Step 2: Create a Bank Contract with Authorization

Build a contract with multi-user support:

```rust
#![no_std]
use soroban_sdk::{
    contract, contractimpl, symbol_short, Symbol, Env, Address,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    InvalidAmount = 3,
    InsufficientBalance = 4,
    UserNotFound = 5,
}

impl Into<u32> for ContractError {
    fn into(self) -> u32 {
        self as u32
    }
}

#[contract]
pub struct BankContract;

const ADMIN: Symbol = symbol_short!("admin");
const BALANCE_PREFIX: &[u8] = b"bal_";

#[contractimpl]
impl BankContract {
    /// Initialize bank with admin
    pub fn init(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&ADMIN) {
            return Err(ContractError::AlreadyInitialized);
        }
        
        env.storage().instance().set(&ADMIN, &admin);
        Ok(())
    }
    
    /// Deposit funds to account
    pub fn deposit(
        env: Env,
        account: Address,
        amount: u128,
    ) -> Result<u128, ContractError> {
        // Verify caller is the account owner
        account.require_auth();
        
        // Validate amount
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        
        // Get storage key for account
        let balance_key = get_balance_key(&account);
        
        // Get current balance
        let current: u128 = env.storage()
            .instance()
            .get(&balance_key)
            .unwrap_or(Ok(0u128))
            .unwrap_or(0u128);
        
        let new_balance = current
            .checked_add(amount)
            .ok_or(ContractError::InvalidAmount)?;
        
        // Store new balance
        env.storage().instance().set(&balance_key, &new_balance);
        Ok(new_balance)
    }
    
    /// Withdraw funds from account
    pub fn withdraw(
        env: Env,
        account: Address,
        amount: u128,
    ) -> Result<u128, ContractError> {
        // Verify caller is the account owner
        account.require_auth();
        
        // Validate amount
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        
        // Get storage key
        let balance_key = get_balance_key(&account);
        
        // Get current balance
        let current: u128 = env.storage()
            .instance()
            .get(&balance_key)
            .unwrap_or(Ok(0u128))
            .unwrap_or(0u128);
        
        // Check sufficient balance
        if current < amount {
            return Err(ContractError::InsufficientBalance);
        }
        
        let new_balance = current - amount;
        env.storage().instance().set(&balance_key, &new_balance);
        Ok(new_balance)
    }
    
    /// Transfer funds between accounts
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: u128,
    ) -> Result<(), ContractError> {
        // Verify caller is the sender
        from.require_auth();
        
        // Withdraw from sender
        self.withdraw(env.clone(), from.clone(), amount)?;
        
        // Deposit to recipient
        self.deposit(env, to, amount)?;
        
        Ok(())
    }
    
    /// Get account balance
    pub fn get_balance(env: Env, account: Address) -> u128 {
        let balance_key = get_balance_key(&account);
        env.storage()
            .instance()
            .get(&balance_key)
            .unwrap_or(Ok(0u128))
            .unwrap_or(0u128)
    }
    
    /// Admin-only function: reset account balance
    pub fn admin_reset_balance(
        env: Env,
        admin: Address,
        account: Address,
    ) -> Result<(), ContractError> {
        // Verify caller is admin
        admin.require_auth();
        
        // Check if caller is the actual admin
        let stored_admin: Address = env.storage()
            .instance()
            .get(&ADMIN)
            .ok_or(ContractError::Unauthorized)?
            .map_err(|_| ContractError::Unauthorized)?;
        
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }
        
        // Reset balance
        let balance_key = get_balance_key(&account);
        env.storage().instance().set(&balance_key, &0u128);
        
        Ok(())
    }
}

/// Helper function to create balance storage key
fn get_balance_key(account: &Address) -> Symbol {
    // In production, use proper serialization
    symbol_short!("bal")
}
```

## Step 3: Build and Deploy

1. Build the contract:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Deploy:
   ```bash
   soroban contract deploy --wasm target/wasm32-unknown-unknown/release/bank_contract.wasm
   ```

## Step 4: Test Authentication Flow

### Setup Phase

**Initialize bank:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn init --arg <ADMIN_ADDRESS>
```

### User Authentication

**Deposit with authentication:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn deposit --arg <USER1> --arg 1000 --sign-with-key <USER1_KEY>
```

**Get balance:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn get_balance --arg <USER1>
```
Expected: `1000`

**Unauthorized deposit attempt (wrong account):**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn deposit --arg <USER2> --arg 500 --sign-with-key <USER1_KEY>
```
Expected error: `Unauthorized (1)`

### Transfer with Authentication

**Successful transfer:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn transfer --arg <USER1> --arg <USER2> --arg 300 --sign-with-key <USER1_KEY>
```

**Verify balances:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn get_balance --arg <USER1>
```
Expected: `700`

```bash
soroban contract invoke --id <CONTRACT_ID> --fn get_balance --arg <USER2>
```
Expected: `300`

### Admin Operations

**Admin reset (authorized):**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn admin_reset_balance --arg <ADMIN> --arg <USER1> --sign-with-key <ADMIN_KEY>
```

**Verify reset:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn get_balance --arg <USER1>
```
Expected: `0`

**Admin reset (unauthorized):**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn admin_reset_balance --arg <USER2> --arg <USER1> --sign-with-key <USER2_KEY>
```
Expected error: `Unauthorized (1)`

## Step 5: Validation Checklist

✅ Contract initializes with admin
✅ Users can only access their own accounts
✅ Deposits and withdrawals work correctly
✅ Transfers between accounts succeed
✅ Admin functions are protected
✅ Unauthorized access is rejected
✅ All balance checks are enforced

## Authentication Patterns

### Pattern 1: Caller Verification
```rust
caller.require_auth();
```

### Pattern 2: Role-Based Access
```rust
let stored_admin: Address = env.storage()
    .instance()
    .get(&ADMIN)?;

if caller != stored_admin {
    return Err(ContractError::Unauthorized);
}
```

### Pattern 3: Delegated Authority
```rust
// Function can be called by owner on behalf of user
pub fn transfer_for_user(
    env: Env,
    delegated_caller: Address,
    user: Address,
    amount: u128,
) -> Result<(), ContractError> {
    delegated_caller.require_auth();
    // ...
}
```

## Security Best Practices

1. **Always Verify**: Use `require_auth()` before sensitive operations
2. **Single Responsibility**: One check per function
3. **Explicit Permissions**: State clearly who can do what
4. **Audit Logging**: Consider logging who did what
5. **Rate Limiting**: Protect against abuse (advanced)

## Common Pitfalls

- Forgetting `require_auth()` calls
- Mixing up caller with argument address
- Not checking role/permission level
- Replay attacks (Soroban handles this)

## Troubleshooting

**Issue**: "Unauthorized" on valid calls
- **Solution**: Ensure correct account is calling, check signature

**Issue**: Admin checks not working
- **Solution**: Verify admin address in storage matches caller

**Issue**: Transfer fails unexpectedly
- **Solution**: Check both accounts have sufficient balance

## Advanced Topics

- Multi-sig authorization
- Time-locked transactions
- Delegation patterns
- Escrow mechanisms

## Next Steps

- Tutorial 5: Multi-Contract Interactions
- [Authentication Examples](../examples/basics.md)
- [Multi-Party Auth Example](../examples/advanced.md)

## Summary

You've learned to:
1. Implement caller authentication with `require_auth()`
2. Create role-based access control
3. Verify user identity before operations
4. Implement admin-only functions
5. Design multi-user contract systems

Strong authentication and authorization are essential for secure contracts!

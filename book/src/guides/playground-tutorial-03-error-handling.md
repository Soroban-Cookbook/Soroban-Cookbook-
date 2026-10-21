# Playground Tutorial 3: Error Handling & Validation

## Overview
Master error handling in Soroban contracts. Learn to validate inputs, define custom errors, and implement robust error handling patterns.

**Duration**: 25-30 minutes
**Difficulty**: Intermediate
**Prerequisites**: Tutorials 1-2 completion

## Learning Objectives
- Define custom contract errors
- Implement input validation
- Use Result types effectively
- Provide helpful error messages

## Step 1: Understanding Soroban Errors

Soroban errors are represented with custom error enums:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    InvalidAmount = 1,
    Unauthorized = 2,
    AlreadyInitialized = 3,
    NotFound = 4,
}
```

## Step 2: Create a Validated Wallet Contract

Build a contract with comprehensive error handling:

```rust
#![no_std]
use soroban_sdk::{
    contract, contractimpl, symbol_short, Symbol, Env, Address,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    InvalidAmount = 1,
    Unauthorized = 2,
    AlreadyInitialized = 3,
    InsufficientBalance = 4,
    MaxBalanceExceeded = 5,
}

impl Into<u32> for ContractError {
    fn into(self) -> u32 {
        self as u32
    }
}

#[contract]
pub struct WalletContract;

const OWNER: Symbol = symbol_short!("owner");
const BALANCE: Symbol = symbol_short!("bal");
const MAX_BALANCE: u128 = 1_000_000_000_000;

#[contractimpl]
impl WalletContract {
    /// Initialize wallet with owner
    pub fn init(env: Env, owner: Address) -> Result<(), ContractError> {
        // Check if already initialized
        if env.storage()
            .instance()
            .has(&OWNER)
        {
            return Err(ContractError::AlreadyInitialized);
        }
        
        env.storage().instance().set(&OWNER, &owner);
        env.storage().instance().set(&BALANCE, &0u128);
        Ok(())
    }
    
    /// Deposit funds
    pub fn deposit(
        env: Env,
        amount: u128,
    ) -> Result<u128, ContractError> {
        // Validate amount
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        
        // Get current balance
        let current: u128 = env.storage()
            .instance()
            .get(&BALANCE)
            .unwrap_or(Ok(0u128))
            .unwrap_or(0u128);
        
        // Check max balance
        let new_balance = current.checked_add(amount)
            .ok_or(ContractError::MaxBalanceExceeded)?;
        
        if new_balance > MAX_BALANCE {
            return Err(ContractError::MaxBalanceExceeded);
        }
        
        env.storage().instance().set(&BALANCE, &new_balance);
        Ok(new_balance)
    }
    
    /// Withdraw funds
    pub fn withdraw(
        env: Env,
        caller: Address,
        amount: u128,
    ) -> Result<u128, ContractError> {
        // Validate caller is owner
        let owner: Address = env.storage()
            .instance()
            .get(&OWNER)
            .unwrap()
            .unwrap();
        
        if caller != owner {
            return Err(ContractError::Unauthorized);
        }
        
        // Validate amount
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        
        // Get current balance
        let current: u128 = env.storage()
            .instance()
            .get(&BALANCE)
            .unwrap_or(Ok(0u128))
            .unwrap_or(0u128);
        
        // Check sufficient balance
        if current < amount {
            return Err(ContractError::InsufficientBalance);
        }
        
        let new_balance = current - amount;
        env.storage().instance().set(&BALANCE, &new_balance);
        Ok(new_balance)
    }
    
    /// Get current balance
    pub fn get_balance(env: Env) -> u128 {
        env.storage()
            .instance()
            .get(&BALANCE)
            .unwrap_or(Ok(0u128))
            .unwrap_or(0u128)
    }
}
```

## Step 3: Build and Deploy

1. Build the contract:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Deploy:
   ```bash
   soroban contract deploy --wasm target/wasm32-unknown-unknown/release/wallet_contract.wasm
   ```

## Step 4: Test Error Scenarios

### Success Path - Valid Operations

**Initialize wallet:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn init --arg <OWNER_ADDRESS>
```

**Deposit 500:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn deposit --arg 500
```
Expected: `500`

**Check balance:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn get_balance
```
Expected: `500`

### Error Path - Invalid Operations

**Try deposit with 0 amount:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn deposit --arg 0
```
Expected error: `InvalidAmount (1)`

**Try initialize again:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn init --arg <OWNER_ADDRESS>
```
Expected error: `AlreadyInitialized (3)`

**Try withdraw from non-owner account:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn withdraw --arg <DIFFERENT_OWNER> --arg 100
```
Expected error: `Unauthorized (2)`

**Try withdraw more than balance:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn withdraw --arg <OWNER> --arg 1000
```
Expected error: `InsufficientBalance (4)`

## Step 5: Validation Checklist

✅ Contract initializes successfully
✅ Valid deposits increase balance
✅ Invalid amounts are rejected with errors
✅ Unauthorized access is prevented
✅ Insufficient balance is detected
✅ Re-initialization is prevented
✅ Error codes are consistent

## Error Handling Best Practices

1. **Be Specific**: Define granular error types
2. **Validate Early**: Check inputs at function entry
3. **Use Result Types**: Return `Result<T, E>` for fallible operations
4. **Provide Context**: Error messages should explain what went wrong
5. **Test Error Paths**: Include error scenarios in tests

## Common Patterns

### Pattern 1: Checked Arithmetic
```rust
let result = amount.checked_add(current)
    .ok_or(ContractError::Overflow)?;
```

### Pattern 2: Authorization Check
```rust
if caller != expected_owner {
    return Err(ContractError::Unauthorized);
}
```

### Pattern 3: State Validation
```rust
if env.storage().instance().has(&KEY) {
    return Err(ContractError::AlreadyInitialized);
}
```

## Troubleshooting

**Issue**: Error codes not displaying correctly
- **Solution**: Ensure `#[repr(u32)]` is on your error enum

**Issue**: Validation not working as expected
- **Solution**: Check condition logic and operator precedence

**Issue**: Authorization always fails
- **Solution**: Verify address equality comparison

## Advanced Topics

- Custom error context with structured data
- Error recovery strategies
- Graceful degradation patterns
- Error logging and diagnostics

## Next Steps

- Tutorial 4: Authentication & Authorization
- Tutorial 5: Multi-Contract Interactions
- Explore [Error Handling Examples](../examples/error-handling.md)

## Summary

You now understand:
1. How to define custom contract errors
2. Implementing comprehensive input validation
3. Using Rust Result types for error handling
4. Testing both success and error paths
5. Best practices for robust contracts

Error handling is critical for security and reliability!

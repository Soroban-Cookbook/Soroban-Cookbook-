# Playground Tutorial 2: Working with Contract Storage

## Overview
Learn how to persist data in your Soroban contracts using the storage layer. This tutorial covers storing and retrieving values.

**Duration**: 20-25 minutes
**Difficulty**: Beginner to Intermediate
**Prerequisites**: Tutorial 1 completion

## Learning Objectives
- Understand contract storage mechanics
- Implement persistent state
- Read and write storage values
- Handle storage errors

## Step 1: Understanding Storage

Soroban contracts have access to:
- **Persistent Storage**: Long-term data storage on the ledger
- **Temporary Storage**: Short-lived data
- **Instance Storage**: Contract-instance specific data

## Step 2: Create a Counter Contract

Replace your previous contract with this storage-enabled example:

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Symbol, Env};

#[contract]
pub struct CounterContract;

const COUNTER: Symbol = symbol_short!("count");

#[contractimpl]
impl CounterContract {
    /// Initialize the counter with a starting value
    pub fn init(env: Env, initial_value: u32) -> u32 {
        env.storage().instance().set(&COUNTER, &initial_value);
        initial_value
    }
    
    /// Increment the counter by 1
    pub fn increment(env: Env) -> u32 {
        let current: u32 = env.storage()
            .instance()
            .get(&COUNTER)
            .unwrap_or(Ok(0u32))
            .unwrap_or(0u32);
        
        let new_value = current + 1;
        env.storage().instance().set(&COUNTER, &new_value);
        new_value
    }
    
    /// Get the current counter value
    pub fn get_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&COUNTER)
            .unwrap_or(Ok(0u32))
            .unwrap_or(0u32)
    }
    
    /// Reset the counter to zero
    pub fn reset(env: Env) {
        env.storage().instance().set(&COUNTER, &0u32);
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
   soroban contract deploy --wasm target/wasm32-unknown-unknown/release/counter_contract.wasm
   ```

3. Save the contract ID

## Step 4: Initialize the Counter

Initialize with starting value of 10:

```bash
soroban contract invoke --id <CONTRACT_ID> --fn init --arg 10
```

Expected output: `10`

## Step 5: Test Storage Operations

**Get the current value:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn get_count
```
Expected: `10`

**Increment the counter:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn increment
```
Expected: `11`

**Verify the increment persisted:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn get_count
```
Expected: `11` (not 10!)

**Increment again:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn increment
```
Expected: `12`

**Reset the counter:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn reset
```

**Verify the reset:**
```bash
soroban contract invoke --id <CONTRACT_ID> --fn get_count
```
Expected: `0`

## Step 6: Validation Checklist

✅ Contract builds successfully
✅ Contract deploys without errors
✅ `init` sets the starting value
✅ `get_count` returns the current value
✅ `increment` increases the value by 1
✅ Values persist between contract calls
✅ `reset` sets the counter to 0

## Storage Best Practices

1. **Use short symbols** for storage keys to minimize gas costs
2. **Handle missing keys gracefully** - use `.unwrap_or()` or similar
3. **Type your storage values** - use consistent types
4. **Document storage layout** in your contract comments
5. **Consider storage costs** when designing data structures

## Common Patterns

### Pattern 1: Safe Value Retrieval
```rust
let value = env.storage()
    .instance()
    .get(&KEY)
    .unwrap_or(Ok(0u32))
    .unwrap_or(0u32);
```

### Pattern 2: Conditional Update
```rust
let mut current = get_stored_value(&env);
if should_update {
    current += 1;
    env.storage().instance().set(&KEY, &current);
}
```

### Pattern 3: Multiple Key Storage
```rust
const KEY_A: Symbol = symbol_short!("key_a");
const KEY_B: Symbol = symbol_short!("key_b");

env.storage().instance().set(&KEY_A, &value_a);
env.storage().instance().set(&KEY_B, &value_b);
```

## Troubleshooting

**Issue**: "Value not found" error
- **Solution**: Check that you've initialized the key first

**Issue**: Type mismatch errors
- **Solution**: Ensure storage key types match when reading/writing

**Issue**: Storage operations are expensive
- **Solution**: Batch multiple operations in a single transaction

## Advanced Topics to Explore

- Persistent storage vs. instance storage
- Storage cost optimization
- Complex data structures
- Storage migration patterns

## Next Steps

- Tutorial 3: Error Handling and Validation
- Tutorial 4: Authentication and Authorization
- Tutorial 5: Multi-Contract Interactions

## Summary

You've learned to:
1. Use persistent storage in contracts
2. Initialize and retrieve stored values
3. Implement basic state management
4. Handle storage operations safely

This foundation is crucial for building stateful smart contracts.

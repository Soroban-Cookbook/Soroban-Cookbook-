# Enum Types in Soroban

This example demonstrates various enum patterns in Soroban smart contracts, showing how to use enums for type-safe data representation and control flow.

## Overview

Enums in Soroban provide a powerful way to represent fixed sets of values and complex state with associated data. This example covers:

- **Simple Enums**: Basic enums without associated data
- **Data Enums**: Enums that carry additional information
- **Contract Errors**: Using `#[contracterror]` for custom error types
- **Pattern Matching**: Safe and expressive control flow with enums

## What's Covered

### 1. Simple Enums

Simple enums represent discrete values without associated data:

```rust
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UserRole {
    None = 0,
    User = 1,
    Moderator = 2,
    Admin = 3,
    Owner = 4,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ContractState {
    Uninitialized = 0,
    Active = 1,
    Paused = 2,
    Frozen = 3,
    Shutdown = 4,
}
```

**Use Cases:**
- User roles and permissions
- Contract lifecycle states
- Transaction types
- Status indicators

### 2. Enums with Associated Data

Enums can carry different types of data per variant:

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    Transfer {
        from: Address,
        to: Address,
        amount: i128,
    },
    TransferWithMessage {
        from: Address,
        to: Address,
        amount: i128,
        message: String,
    },
    BatchTransfer {
        from: Address,
        recipients: Vec<Address>,
        amounts: Vec<i128>,
    },
    TimeLockedTransfer {
        from: Address,
        to: Address,
        amount: i128,
        unlock_time: u64,
    },
    ConditionalTransfer {
        from: Address,
        to: Address,
        amount: i128,
        condition: String,
        approved_by: Vec<Address>,
    },
}
```

**Use Cases:**
- Complex operations with different parameters
- State machines with data
- Event types with metadata
- Validation results with context

### 3. Contract Error Enums

Use `#[contracterror]` to create custom error types:

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    // General errors (1000-1099)
    InvalidInput = 1000,
    Unauthorized = 1001,
    InsufficientBalance = 1002,
    
    // State errors (1100-1199)
    ContractNotInitialized = 1100,
    ContractPaused = 1102,
    
    // Operation errors (1200-1299)
    OperationNotFound = 1200,
    InsufficientApprovals = 1204,
    
    // ... more error types
}
```

**Use Cases:**
- Type-safe error handling
- Result-based error propagation
- Clear error codes for debugging
- Internationalization support

### 4. Pattern Matching

Use pattern matching for safe and expressive control flow:

```rust
pub fn execute_operation(
    env: Env,
    operation: Operation,
) -> Result<ValidationResult, ContractError> {
    match operation {
        Operation::Transfer { from, to, amount } => {
            Self::validate_transfer(env, from, to, amount)
        }
        Operation::TransferWithMessage { from, to, amount, message } => {
            Self::validate_transfer_with_message(env, from, to, amount, message)
        }
        Operation::BatchTransfer { from, recipients, amounts } => {
            Self::validate_batch_transfer(env, from, recipients, amounts)
        }
        // ... more variants
    }
}
```

**Use Cases:**
- Type-safe dispatch based on enum variants
- Exhaustive pattern matching
- Data extraction from enum variants
- Conditional logic based on enum values

## Key Concepts

### Simple Enums vs Data Enums

| Feature | Simple Enums | Data Enums |
|---------|------------|-----------|
| Data | None | Associated data |
| Memory | Minimal | Variable |
| Use Case | Fixed values | Complex state |
| Pattern | Basic matching | Destructuring |

### Error Handling Patterns

1. **Simple Errors**: Direct error codes
2. **Contextual Errors**: Errors with additional information
3. **Multiple Causes**: Errors with multiple related issues

### Pattern Matching Patterns

1. **Basic Matching**: Simple variant checks
2. **Destructuring**: Extract data from variants
3. **Guard Clauses**: Conditional pattern matching
4. **Exhaustive Matching**: Compiler-enforced completeness

## Usage Examples

### Basic Enum Usage

```rust
// Check user role
let user_role = UserRole::User;
if user_role >= UserRole::Moderator {
    // User has elevated permissions
}

// Check contract state
let state = contract.get_state();
match state {
    ContractState::Active => {
        // Contract is operational
    }
    ContractState::Paused => {
        // Contract is temporarily paused
    }
    ContractState::Frozen => {
        // Contract is permanently frozen
    }
    _ => {
        // Handle other states
    }
}
```

### Data Enum Usage

```rust
// Create different operation types
let transfer = Operation::Transfer {
    from: user1,
    to: user2,
    amount: 100,
};

// Pattern match to extract data
match transfer {
    Operation::Transfer { from, to, amount } => {
        println!("Transfer {} tokens from {} to {}", amount, from, to);
    }
    Operation::TransferWithMessage { from, to, amount, message } => {
        println!("Transfer {} tokens from {} to {}: {}", amount, from, to, message);
    }
    // ... handle other variants
}
```

### Error Handling

```rust
// Return Result with custom error
pub fn validate_amount(amount: i128) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::InvalidAmount);
    }
    if amount > MAX_AMOUNT {
        return Err(ContractError::InsufficientBalance);
    }
    Ok(())
}

// Handle errors with pattern matching
match result {
    Ok(()) => {
        // Operation succeeded
    }
    Err(ContractError::InvalidAmount) => {
        // Handle invalid amount error
    }
    Err(ContractError::InsufficientBalance) => {
        // Handle insufficient balance error
    }
    Err(error) => {
        // Handle other errors
        return Err(error);
    }
}
```

### Complex Pattern Matching

```rust
// Validate asset with pattern matching
pub fn validate_asset(env: Env, asset: Asset) -> Result<(), ContractError> {
    match asset {
        Asset::Native => Ok(()),
        Asset::Token { contract_address, symbol, decimals } => {
            if contract_address.is_zero() {
                return Err(ContractError::InvalidAddress);
            }
            if symbol.len() == 0 || symbol.len() > 10 {
                return Err(ContractError::InvalidInput);
            }
            if decimals > 18 {
                return Err(ContractError::InvalidInput);
            }
            Ok(())
        }
        Asset::NFT { contract_address, token_id, metadata } => {
            if contract_address.is_zero() {
                return Err(ContractError::InvalidAddress);
            }
            if token_id == 0 {
                return Err(ContractError::InvalidInput);
            }
            if metadata.len() > 1000 {
                return Err(ContractError::InvalidInput);
            }
            Ok(())
        }
        // ... handle other asset types
    }
}
```

## Best Practices

### 1. Error Code Organization

- **1000-1099**: General errors (input, authorization, balance)
- **1100-1199**: State errors (initialization, lifecycle)
- **1200-1299**: Operation errors (not found, expired, approval)
- **1300-1399**: Asset errors (not found, invalid, balance)
- **1400-1499**: User/Role errors (not found, permissions, status)
- **1500-1599**: Validation errors (failed, pending, expired)
- **1600-1699**: System errors (internal, storage, overflow)

### 2. Enum Design Guidelines

- **Use descriptive names**: Make enum variants self-explanatory
- **Group related values**: Keep similar concepts together
- **Use consistent ordering**: Order variants logically
- **Add documentation**: Document the purpose of each enum

### 3. Pattern Matching Best Practices

- **Handle all variants**: Use exhaustive matching
- **Use guards when needed**: Add conditions to patterns
- **Destructure cleanly**: Extract data in match arms
- **Keep matches focused**: One logical concern per match

### 4. Error Handling Patterns

- **Be specific**: Use the most specific error type
- **Provide context**: Include relevant information
- **Document errors**: Explain what each error means
- **Test errors**: Ensure all error paths are tested

## Testing

The example includes comprehensive tests covering:

- **Unit Tests**: Individual enum functionality
- **Integration Tests**: Complete workflow scenarios
- **Edge Cases**: Boundary conditions and error cases
- **Pattern Matching**: All enum variants and combinations

Run tests with:

```bash
cargo test
```

## Advanced Patterns

### 1. Nested Enums

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComplexState {
    Active {
        operations_count: u64,
        last_activity: u64,
    },
    Maintenance {
        reason: String,
        estimated_completion: u64,
    },
    Error {
        error_type: ContractError,
        recovery_possible: bool,
    },
}
```

### 2. Generic Enums

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Result<T> {
    Success(T),
    Failure(ContractError),
    Pending {
        wait_until: u64,
        context: String,
    },
}
```

### 3. Enum Iteration

```rust
// Enum with iteration support
impl UserRole {
    pub fn all_values() -> Vec<UserRole> {
        vec![
            UserRole::None,
            UserRole::User,
            UserRole::Moderator,
            UserRole::Admin,
            UserRole::Owner,
        ]
    }
    
    pub fn next_role(&self) -> Option<UserRole> {
        match self {
            UserRole::None => Some(UserRole::User),
            UserRole::User => Some(UserRole::Moderator),
            UserRole::Moderator => Some(UserRole::Admin),
            UserRole::Admin => Some(UserRole::Owner),
            UserRole::Owner => None,
        }
    }
}
```

## Common Pitfalls

### 1. Forgetting Match Exhaustiveness

```rust
// ❌ Missing variants
fn handle_state(state: ContractState) {
    match state {
        ContractState::Active => println!("Active"),
        ContractState::Paused => println!("Paused"),
        // Missing other variants!
    }
}

// ✅ Exhaustive matching
fn handle_state(state: ContractState) {
    match state {
        ContractState::Active => println!("Active"),
        ContractState::Paused => println!("Paused"),
        ContractState::Frozen => println!("Frozen"),
        ContractState::Shutdown => println!("Shutdown"),
        ContractState::Uninitialized => println!("Uninitialized"),
    }
}
```

### 2. Ignoring Associated Data

```rust
// ❌ Not using available data
fn handle_operation(op: Operation) {
    match op {
        Operation::Transfer { .. } => println!("Transfer"),
        Operation::BatchTransfer { .. } => println!("BatchTransfer"),
        // Missing data extraction!
    }
}

// ✅ Extracting and using data
fn handle_operation(op: Operation) {
    match op {
        Operation::Transfer { from, to, amount } => {
            println!("Transfer {} from {} to {}", amount, from, to);
        }
        Operation::BatchTransfer { from, recipients, amounts } => {
            println!("Batch transfer from {} to {} recipients", from, recipients.len());
        }
        // ... handle other variants
    }
}
```

### 3. Using Panics Instead of Errors

```rust
// ❌ Using panic!
fn validate_user(role: UserRole) {
    if role == UserRole::None {
        panic!("User has no role!");
    }
}

// ✅ Using Result with errors
fn validate_user(role: UserRole) -> Result<(), ContractError> {
    if role == UserRole::None {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}
```

## Integration with Other Patterns

Enums work well with other Soroban patterns:

- **Storage Keys**: Use enums as storage keys for type safety
- **Events**: Use enums to represent different event types
- **State Machines**: Use enums to represent state transitions
- **Configuration**: Use enums for contract configuration options

## Performance Considerations

- **Memory Usage**: Simple enums are very efficient
- **Data Enums**: Consider the size of associated data
- **Pattern Matching**: Rust optimizes pattern matching well
- **Storage**: Store enums efficiently in contract storage

## Security Considerations

- **Type Safety**: Enums prevent invalid states
- **Exhaustive Matching**: Compiler ensures all cases are handled
- **Error Codes**: Use structured error codes for debugging
- **Input Validation**: Use enums to validate input ranges

## Conclusion

Enums in Soroban provide a powerful, type-safe way to represent complex data structures and control flow. By following the patterns demonstrated in this example, you can write more robust, maintainable, and secure smart contracts.

The key takeaways are:

1. **Use enums for type safety** - Prevent invalid states at compile time
2. **Leverage pattern matching** - Write clear, expressive control flow
3. **Structure errors properly** - Use `#[contracterror]` for custom errors
4. **Document thoroughly** - Make enum usage clear for other developers
5. **Test comprehensively** - Ensure all enum variants work as expected

By mastering these enum patterns, you'll be able to write more sophisticated and reliable Soroban smart contracts.

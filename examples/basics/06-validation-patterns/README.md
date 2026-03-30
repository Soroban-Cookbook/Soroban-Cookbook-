# Validation Patterns

This example demonstrates comprehensive input validation patterns in Soroban smart contracts. It covers parameter validation, state validation, and authorization validation with clear error messages.

## Overview

Validation is a critical aspect of smart contract development that ensures:
- **Security**: Prevents malicious inputs and unauthorized operations
- **Reliability**: Ensures contract state remains consistent
- **User Experience**: Provides clear error messages for failed operations
- **Gas Efficiency**: Fails early with minimal computation cost

## Validation Categories

### 1. Parameter Validation

Validates the format, type, and range of function inputs.

#### Examples:
- **Amount validation**: Ensures amounts are positive and within allowed ranges
- **String validation**: Checks length and content constraints
- **Address validation**: Verifies address format and validity
- **Array validation**: Enforces size limits and content requirements
- **Timestamp validation**: Ensures timestamps are within acceptable ranges

#### Error Codes: 100-199

```rust
ValidationError::InvalidAmount = 100,
ValidationError::AmountTooSmall = 101,
ValidationError::AmountTooLarge = 102,
ValidationError::InvalidAddress = 103,
ValidationError::InvalidString = 104,
ValidationError::StringTooShort = 105,
ValidationError::StringTooLong = 106,
```

### 2. State Validation

Validates the contract's current state and stored data.

#### Examples:
- **Contract state**: Checks if contract is active, paused, or frozen
- **Balance validation**: Ensures sufficient funds for operations
- **Allowance validation**: Checks spending allowances
- **Cooldown validation**: Prevents rapid repeated operations
- **Invariant validation**: Ensures critical invariants are maintained

#### Error Codes: 200-299

```rust
ValidationError::ContractNotInitialized = 200,
ValidationError::ContractPaused = 201,
ValidationError::ContractFrozen = 202,
ValidationError::InsufficientBalance = 203,
ValidationError::InsufficientAllowance = 204,
ValidationError::CooldownActive = 210,
```

### 3. Authorization Validation

Validates caller permissions and access rights.

#### Examples:
- **Role-based access**: Different roles have different permissions
- **Ownership validation**: Only owners can perform certain operations
- **Admin validation**: Admin-only functions
- **Blacklist checking**: Prevent blacklisted addresses from operations
- **Multi-signature**: Require multiple approvals for critical operations

#### Error Codes: 300-399

```rust
ValidationError::Unauthorized = 300,
ValidationError::NotAdmin = 301,
ValidationError::NotOwner = 302,
ValidationError::InsufficientRole = 303,
ValidationError::Blacklisted = 309,
```

## Key Functions

### Parameter Validation Functions

```rust
// Validate amount with min/max bounds
validate_amount_parameters(amount, min_amount, max_amount)

// Validate string length and content
validate_string_parameters(text, min_length, max_length)

// Validate address format
validate_address(address)

// Validate array size
validate_array_parameters(array, min_size, max_size)

// Validate timestamp range
validate_timestamp_parameters(env, timestamp, allow_past, max_future_seconds)
```

### State Validation Functions

```rust
// Validate contract is in required state
validate_contract_state(env, required_state)

// Validate sufficient balance
validate_balance(env, address, required_amount)

// Validate sufficient allowance
validate_allowance(env, owner, spender, required_amount)

// Validate cooldown period
validate_cooldown(env, address, cooldown_seconds)
```

### Authorization Validation Functions

```rust
// Validate user has sufficient role
validate_role(env, address, required_role)

// Validate ownership
validate_ownership(env, address)

// Validate admin permissions
validate_admin(env, address)
```

## Usage Examples

### Basic Transfer with Full Validation

```rust
let result = client.validated_transfer(
    &from_address,
    &to_address,
    100i128,
    Some(String::from_str(&env, "Payment for services"))
);

match result {
    Ok(()) => println!("Transfer successful"),
    Err(ValidationError::InsufficientBalance) => println!("Insufficient balance"),
    Err(ValidationError::CooldownActive) => println!("Please wait before transferring again"),
    Err(e) => println!("Validation failed: {:?}", e),
}
```

### Admin Operations with Authorization

```rust
// Set user role (admin only)
client.set_user_role(&admin_address, &user_address, UserRole::Moderator)?;

// Pause contract (admin only)
client.pause_contract(&admin_address)?;

// Resume contract (admin only)
client.resume_contract(&admin_address)?;
```

### Parameter Validation

```rust
// Validate amount before processing
ValidationContract::validate_amount_parameters(amount, 1, 1000000)?;

// Validate string input
ValidationContract::validate_string_parameters(&name, 1, 100)?;

// Validate timestamp
ValidationContract::validate_timestamp_parameters(&env, deadline, false, 86400 * 30)?;
```

## Best Practices

### 1. Validate Early, Fail Fast

```rust
// ✅ GOOD: Validate all parameters first
Self::validate_amount_parameters(amount, min_amount, max_amount)?;
Self::validate_address(&recipient)?;
Self::validate_contract_state(&env, ContractState::Active)?;

// Then perform the operation
// ...

// ❌ BAD: Perform operations before validation
let balance = get_balance(&sender);
if balance < amount {
    return Err(ValidationError::InsufficientBalance);
}
```

### 2. Use Specific Error Types

```rust
// ✅ GOOD: Specific, actionable errors
ValidationError::AmountTooSmall
ValidationError::InsufficientBalance
ValidationError::CooldownActive

// ❌ BAD: Generic errors
ValidationError::InvalidInput
ValidationError::OperationFailed
```

### 3. Validate Authorization Separately

```rust
// ✅ GOOD: Separate authorization from business logic
Self::validate_admin(&env, &caller)?;
caller.require_auth();

// Then perform admin operation
// ...

// ❌ BAD: Mix authorization with business logic
if caller != admin {
    return Err(ValidationError::NotAdmin);
}
// Perform operation...
```

### 4. Use Result Types for Expected Failures

```rust
// ✅ GOOD: Use Result for validation failures
pub fn transfer(amount: i128) -> Result<(), ValidationError> {
    if amount <= 0 {
        return Err(ValidationError::InvalidAmount);
    }
    // ...
}

// ❌ BAD: Use panic for expected failures
pub fn transfer(amount: i128) {
    if amount <= 0 {
        panic!("invalid amount");
    }
    // ...
}
```

## Error Handling Strategy

### Error Code Ranges

- **100-199**: Parameter validation errors
- **200-299**: State validation errors  
- **300-399**: Authorization validation errors

### Error Recovery

1. **Parameter errors**: User should correct input and retry
2. **State errors**: User should wait for state change or take corrective action
3. **Authorization errors**: User needs proper permissions or different account

### Gas Efficiency

- Validate parameters first (cheapest checks)
- Check contract state next
- Verify authorization last (most expensive)
- Fail as early as possible to save gas

## Testing

The contract includes comprehensive tests covering:

- All validation functions
- Error code uniqueness and ranges
- Edge cases and boundary conditions
- Combined validation scenarios
- Authorization and permission testing

Run tests with:

```bash
cargo test
```

## Integration with Other Contracts

This validation pattern can be integrated into any Soroban contract:

```rust
use crate::validation_patterns::{ValidationError, ValidationContract};

impl MyContract {
    pub fn my_function(env: Env, caller: Address, amount: i128) -> Result<(), ValidationError> {
        // Reuse validation functions
        ValidationContract::validate_amount_parameters(amount, 1, 1000)?;
        ValidationContract::validate_role(&env, &caller, UserRole::User)?;
        
        // Your contract logic here
        // ...
        
        Ok(())
    }
}
```

## Security Considerations

1. **Never trust input**: Always validate all external inputs
2. **Check invariants**: Ensure critical invariants are maintained
3. **Rate limiting**: Use cooldowns to prevent spam attacks
4. **Access control**: Implement proper role-based permissions
5. **State consistency**: Validate state transitions carefully

## Common Pitfalls

1. **Missing validation**: Forgetting to validate inputs leads to vulnerabilities
2. **Wrong error types**: Using generic errors makes debugging difficult
3. **Late validation**: Validating after expensive operations wastes gas
4. **Incomplete authorization**: Missing permission checks on sensitive functions
5. **Race conditions**: Not validating state can lead to inconsistent operations

## Advanced Patterns

### Custom Validation Logic

```rust
impl ValidationContract {
    pub fn validate_business_rule(
        env: &Env,
        user: &Address,
        operation: Symbol,
    ) -> Result<(), ValidationError> {
        // Custom business logic validation
        let user_level = Self::get_user_level(env, user);
        let required_level = Self::get_required_level(operation);
        
        if user_level < required_level {
            return Err(ValidationError::InsufficientRole);
        }
        
        Ok(())
    }
}
```

### Composite Validation

```rust
pub fn validate_complex_operation(
    env: &Env,
    caller: &Address,
    params: OperationParams,
) -> Result<(), ValidationError> {
    // Validate all parameters
    Self::validate_amount_parameters(params.amount, params.min, params.max)?;
    Self::validate_string_parameters(&params.description, 1, 500)?;
    Self::validate_timestamp_parameters(&env, params.deadline, false, 86400 * 30)?;
    
    // Validate state
    Self::validate_contract_state(env, ContractState::Active)?;
    Self::validate_balance(env, caller, params.amount)?;
    Self::validate_cooldown(env, caller, 300)?; // 5 minute cooldown
    
    // Validate authorization
    Self::validate_role(env, caller, UserRole::Moderator)?;
    
    Ok(())
}
```

This validation patterns example provides a comprehensive foundation for building secure, reliable, and user-friendly Soroban smart contracts.

# Custom Errors in Soroban

This example demonstrates comprehensive custom error handling in Soroban smart contracts using the `contracterror` attribute.

## Overview

Custom errors provide type-safe, descriptive error handling that improves the developer and user experience compared to generic panics. They enable proper error codes for frontend integration and clear debugging information.

## Key Features

- **Type-safe error handling** with enum variants
- **Descriptive error messages** for each failure scenario
- **Proper error codes** (1-8) for frontend integration
- **Event logging** for error tracking and debugging
- **Comprehensive test coverage** for all error scenarios

## Error Variants

| Error Code | Variant               | Description             | Use Case                                |
| ---------- | --------------------- | ----------------------- | --------------------------------------- |
| 1          | `InvalidInput`        | Input validation failed | Invalid parameters, zero values         |
| 2          | `Unauthorized`        | Unauthorized access     | Permission checks, admin-only functions |
| 3          | `NotFound`            | Resource not found      | Missing storage keys, non-existent data |
| 4          | `InsufficientBalance` | Insufficient balance    | Token transfers, balance checks         |
| 5          | `OperationNotAllowed` | Operation not allowed   | Business logic restrictions             |
| 6          | `RateLimitExceeded`   | Rate limit exceeded     | Frequency limits, anti-spam             |
| 7          | `ContractPaused`      | Contract is paused      | Maintenance mode, emergency stops       |
| 8          | `AlreadyExists`       | Duplicate entry         | Unique constraint violations            |

## Contract Functions

### Basic Error Scenarios

- `validate_input(value)` - Demonstrates input validation errors
- `check_authorization(caller, admin)` - Shows authorization errors
- `get_value(key)` - Handles not found errors
- `transfer_tokens(balance, amount)` - Balance and input validation

### Advanced Error Scenarios

- `perform_operation(is_paused, operation_type)` - Multiple error conditions
- `create_entry(key, value)` - Duplicate detection and validation
- `check_rate_limit(caller, count, max)` - Rate limiting and authorization
- `complex_operation(amount, caller, admin, is_paused)` - Multi-step validation

## Usage Examples

### Basic Error Handling

```rust
use soroban_sdk::{contracterror, contractimpl, Env};

#[contracterror]
#[repr(u32)]
pub enum ContractError {
    InvalidInput = 1,
    Unauthorized = 2,
    NotFound = 3,
}

#[contractimpl]
impl MyContract {
    pub fn my_function(env: Env, value: u64) -> Result<(), ContractError> {
        if value == 0 {
            return Err(ContractError::InvalidInput);
        }
        // ... function logic
        Ok(())
    }
}
```

### Error Propagation

```rust
pub fn complex_operation(env: Env, input: u64) -> Result<(), ContractError> {
    // Step 1: Validate input
    if input == 0 {
        return Err(ContractError::InvalidInput);
    }

    // Step 2: Check authorization
    if !is_authorized(&env) {
        return Err(ContractError::Unauthorized);
    }

    // Step 3: Perform operation
    Ok(())
}
```

### Error Testing

```rust
#[test]
fn test_error_handling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    // Test error case
    let result = client.try_my_function(&0u64);
    assert_eq!(result, Err(StellarError::from_contract_error(ContractError::InvalidInput)));

    // Test success case
    let result = client.try_my_function(&42u64);
    assert_eq!(result, Ok(()));
}
```

## Best Practices

1. **Use descriptive error names** that clearly indicate the problem
2. **Assign sequential error codes** starting from 1
3. **Document each error variant** with use cases
4. **Log events when errors occur** for debugging
5. **Test all error scenarios** comprehensively
6. **Use Result types** for functions that can fail
7. **Handle errors gracefully** in calling code

## Frontend Integration

Error codes can be used by frontends to display appropriate user messages:

```javascript
try {
  await contract.my_function(0);
} catch (error) {
  if (error.code === 1) {
    showMessage("Invalid input: value must be greater than 0");
  } else if (error.code === 2) {
    showMessage("You are not authorized to perform this action");
  }
  // ... handle other error codes
}
```

## Running the Example

```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test

# Deploy to testnet (if configured)
soroban contract deploy ...
```

## Key Takeaways

- Custom errors provide **better UX** than generic panics
- Error codes enable **frontend integration** and user-friendly messages
- Type safety ensures **compile-time error handling**
- Event logging helps with **debugging and monitoring**
- Comprehensive testing ensures **reliable error handling**

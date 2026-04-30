# Soroban Validation Library

A collection of reusable validation utilities for Soroban smart contracts. This library provides typed validation errors and helpers for parameter validation, state validation, and authorization validation.

## Overview

This library extracts common validation patterns used across Soroban contracts into reusable functions. It provides:

- **Parameter Validation**: Validate inputs like amounts, addresses, strings, arrays, and timestamps
- **State Validation**: Validate contract state, balances, allowances, and temporal constraints
- **Authorization Validation**: Validate permissions, roles, ownership, and access controls
- **Typed Errors**: Comprehensive error types for clear error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
soroban-validation = { path = "../../../shared" }
```

## Error Types

All validation functions return `Result<(), ValidationError>` with the following error codes:

### Parameter Validation Errors (100-199)
- `InvalidAmount` (100): Amount is negative or zero
- `AmountTooSmall` (101): Amount below minimum
- `AmountTooLarge` (102): Amount exceeds maximum
- `InvalidAddress` (103): Invalid address format
- `InvalidString` (104): Invalid string content
- `StringTooShort` (105): String too short
- `StringTooLong` (106): String too long
- `InvalidArray` (108): Invalid array content
- `ArrayTooSmall` (109): Array too small
- `ArrayTooLarge` (110): Array too large
- `InvalidTimestamp` (111): Invalid timestamp
- `TimestampInPast` (112): Timestamp in past when not allowed
- `TimestampInDistantFuture` (113): Timestamp too far in future

### State Validation Errors (200-299)
- `ContractNotInitialized` (200): Contract not initialized
- `ContractPaused` (201): Contract is paused
- `ContractFrozen` (202): Contract is frozen
- `InsufficientBalance` (203): Insufficient balance
- `InsufficientAllowance` (204): Insufficient allowance
- `ResourceNotFound` (205): Resource not found
- `ResourceAlreadyExists` (206): Resource already exists
- `InvalidStateTransition` (207): Invalid state transition
- `CooldownActive` (210): Cooldown period active

### Authorization Validation Errors (300-399)
- `Unauthorized` (300): Unauthorized access
- `NotAdmin` (301): Not an admin
- `NotOwner` (302): Not the owner
- `InsufficientRole` (303): Insufficient role permissions
- `Blacklisted` (309): Address is blacklisted

## Parameter Validation

### Amount Validation

```rust
use soroban_validation::validate_amount;

let amount = 100i128;
let min_amount = 1i128;
let max_amount = 1000000i128;

validate_amount(amount, min_amount, max_amount)?;
```

### String Validation

```rust
use soroban_validation::validate_string;

let text = String::from_str(&env, "hello");
validate_string(text, 1, 100)?; // min_length=1, max_length=100
```

### Address Validation

```rust
use soroban_validation::validate_address;

let address = Address::generate(&env);
validate_address(address)?;
```

### Array Validation

```rust
use soroban_validation::validate_array;

let array = Vec::from_array(&env, [1, 2, 3]);
validate_array(array, 1, 10)?; // min_size=1, max_size=10
```

### Timestamp Validation

```rust
use soroban_validation::validate_timestamp;

let timestamp = 1234567890u64;
let allow_past = false;
let max_future_seconds = 86400; // 1 day

validate_timestamp(&env, timestamp, allow_past, max_future_seconds)?;
```

## State Validation

### Balance Validation

```rust
use soroban_validation::require_sufficient_balance;

let current_balance = 1000i128;
let required_amount = 100i128;

require_sufficient_balance(current_balance, required_amount)?;
```

### Cooldown Validation

```rust
use soroban_validation::require_cooldown_expired;

let last_action_time = 1234567800u64;
let cooldown_seconds = 3600; // 1 hour

require_cooldown_expired(&env, last_action_time, cooldown_seconds)?;
```

## Authorization Validation

### Ownership Validation

```rust
use soroban_validation::require_owner;

let stored_owner = Address::generate(&env);
let claimed_owner = Address::generate(&env);

require_owner(stored_owner, claimed_owner)?;
```

### Admin Validation

```rust
use soroban_validation::require_admin;

let stored_admin = Address::generate(&env);
let claimed_admin = Address::generate(&env);

require_admin(stored_admin, claimed_admin)?;
```

### Role Validation

```rust
use soroban_validation::require_role;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum UserRole { User, Moderator, Admin }

let user_role = UserRole::Moderator;
let required_role = UserRole::Admin;

require_role(user_role, required_role)?;
```

### Blacklist Validation

```rust
use soroban_validation::require_not_blacklisted;

let is_blacklisted = false;
require_not_blacklisted(is_blacklisted)?;
```

## Complete Example

```rust
use soroban_sdk::*;
use soroban_validation::*;

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128
    ) -> Result<(), ValidationError> {
        from.require_auth();

        // Parameter validation
        validate_amount(amount, 1, i128::MAX)?;
        validate_address(from.clone())?;
        validate_address(to.clone())?;

        // State validation
        let balance = get_balance(&env, from.clone());
        require_sufficient_balance(balance, amount)?;

        // Check cooldown (example)
        let last_transfer = get_last_transfer(&env, from.clone());
        require_cooldown_expired(&env, last_transfer, 60)?; // 1 minute cooldown

        // Execute transfer
        update_balance(&env, from, balance - amount);
        update_balance(&env, to, get_balance(&env, to) + amount);
        update_last_transfer(&env, from, env.ledger().timestamp());

        Ok(())
    }
}
```

## Best Practices

1. **Validate Early**: Call validation functions at the beginning of your contract functions
2. **Use Appropriate Errors**: Choose error types that clearly indicate what went wrong
3. **Combine Validations**: Use multiple validation functions for comprehensive input checking
4. **Handle Errors Gracefully**: Provide clear error messages to users
5. **Test Thoroughly**: Test both success and failure cases for all validation scenarios

## Contributing

When adding new validation functions:

1. Add appropriate error codes in the `ValidationError` enum
2. Follow the existing naming conventions
3. Include comprehensive documentation
4. Add unit tests
5. Update this README
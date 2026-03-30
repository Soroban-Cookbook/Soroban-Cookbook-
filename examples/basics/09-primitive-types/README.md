# Primitive Types in Soroban

This example demonstrates the usage of primitive types in Soroban smart contracts, covering integer types, boolean operations, type conversions, and overflow handling.

## Overview

Primitive types are the fundamental building blocks of Soroban smart contracts. Understanding their characteristics, limitations, and proper usage patterns is essential for writing secure and efficient contracts.

## What's Covered

### 1. Integer Types

#### Unsigned Integers
- **`u32`**: 32-bit unsigned integer (0 to 4,294,967,295)
- **`u64`**: 64-bit unsigned integer (0 to 18,446,744,073,709,551,615)
- **`u128`**: 128-bit unsigned integer (0 to 340,282,366,920,938,463,463,374,607,431,768,211,455)

#### Signed Integers
- **`i32`**: 32-bit signed integer (-2,147,483,648 to 2,147,483,647)
- **`i64`**: 64-bit signed integer (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)
- **`i128`**: 128-bit signed integer (used for financial calculations)

#### Type Selection Guidelines
```rust
// Use u32 for small counters, flags, and indices
let counter: u32 = 100;
let flags: u32 = 0b1010;

// Use u64 for timestamps, large counters, and IDs
let timestamp: u64 = env.ledger().timestamp();
let user_id: u64 = 1234567890;

// Use i128 for financial calculations (balances, amounts)
let balance: i128 = 1000000; // $1M in smallest unit
let amount: i128 = -500; // Negative amount for refunds
```

### 2. Boolean Type

#### Boolean Operations
```rust
// Logical operations
let result = a && b;  // Logical AND
let result = a || b;  // Logical OR
let result = !a;     // Logical NOT

// Boolean comparisons
let is_equal = (a == b);
let is_greater = (a > b);
let is_valid = (value >= min && value <= max);
```

#### Storage and Retrieval
```rust
// Store boolean
env.storage().instance().set(&symbol_short!("flag"), &true);

// Retrieve boolean
let flag: bool = env.storage()
    .instance()
    .get(&symbol_short!("flag"))
    .unwrap_or(false);
```

### 3. Type Conversions

#### Safe Conversions
```rust
// Always safe (no data loss)
let u64_val: u64 = u32_val as u64;
let i64_val: i64 = i32_val as i64;

// Safe with checks
let u32_val: u32 = match u64_val.try_into() {
    Ok(val) => val,
    Err(_) => return Err(ConversionError),
};
```

#### Conversions with Potential Data Loss
```rust
// May overflow
let u32_val: Result<u32, _> = u64_to_u32(u64_val);

// May underflow (negative to unsigned)
let u32_val: Result<u32, _> = i32_to_u32(-1); // Error!

// May overflow (large unsigned to signed)
let i64_val: Result<i64, _> = u64_to_i64(u64::MAX); // Error!
```

#### Conversion Examples
```rust
// u32 to u64 (always safe)
let small: u32 = 100;
let large: u64 = small as u64; // 100

// u64 to u32 (may overflow)
let large: u64 = u32::MAX as u64 + 1;
let small: Result<u32, _> = u64_to_u32(large); // Error!

// i32 to u32 (may underflow)
let signed: i32 = -1;
let unsigned: Result<u32, _> = i32_to_u32(signed); // Error!

// u32 to i32 (may overflow)
let unsigned: u32 = i32::MAX as u32 + 1;
let signed: Result<i32, _> = u32_to_i32(unsigned); // Error!
```

### 4. Overflow Handling

#### Checked Arithmetic
```rust
// Returns Option<T> - None on overflow/underflow
let result = a.checked_add(b);
match result {
    Some(value) => Ok(value),
    None => Err(OverflowError),
}

// Division by zero protection
let result = a.checked_div(b);
if b == 0 {
    return Err(DivisionByZero);
}
```

#### Saturating Arithmetic
```rust
// Clamps to max/min values on overflow/underflow
let result = a.saturating_add(b); // Never overflows
let result = a.saturating_mul(b); // Never overflows
let result = a.saturating_sub(b); // Never underflows (clamps to 0)
```

#### Wrapping Arithmetic
```rust
// Wraps around on overflow/underflow
let result = a.wrapping_add(b); // u64::MAX + 1 = 0
let result = a.wrapping_sub(b); // 0 - 1 = u64::MAX
let result = a.wrapping_mul(b); // u64::MAX * 2 = u64::MAX - 2
```

## Contract Functions

### Basic Arithmetic Operations

```rust
// Unsigned operations
pub fn add_u32(env: Env, a: u32, b: u32) -> Result<u32, ContractError>
pub fn sub_u32(env: Env, a: u32, b: u32) -> Result<u32, ContractError>
pub fn mul_u32(env: Env, a: u32, b: u32) -> Result<u32, ContractError>
pub fn div_u32(env: Env, a: u32, b: u32) -> Result<u32, ContractError>

// Signed operations
pub fn add_i32(env: Env, a: i32, b: i32) -> Result<i32, ContractError>
pub fn sub_i32(env: Env, a: i32, b: i32) -> Result<i32, ContractError>
pub fn mul_i32(env: Env, a: i32, b: i32) -> Result<i32, ContractError>
pub fn div_i32(env: Env, a: i32, b: i32) -> Result<i32, ContractError>
```

### Boolean Operations

```rust
pub fn bool_and(env: Env, a: bool, b: bool) -> bool
pub fn bool_or(env: Env, a: bool, b: bool) -> bool
pub fn bool_not(env: Env, a: bool) -> bool
pub fn bool_xor(env: Env, a: bool, b: bool) -> bool
```

### Type Conversions

```rust
pub fn u32_to_u64(env: Env, value: u32) -> u64
pub fn u64_to_u32(env: Env, value: u64) -> Result<u32, ContractError>
pub fn i32_to_i64(env: Env, value: i32) -> i64
pub fn i64_to_i32(env: Env, value: i64) -> Result<i32, ContractError>
pub fn u32_to_i32(env: Env, value: u32) -> Result<i32, ContractError>
pub fn i32_to_u32(env: Env, value: i32) -> Result<u32, ContractError>
```

### Overflow Handling

```rust
// Safe operations with error handling
pub fn safe_add(env: Env, a: u64, b: u64) -> Result<u64, ContractError>
pub fn safe_sub(env: Env, a: u64, b: u64) -> Result<u64, ContractError>
pub fn safe_mul(env: Env, a: u64, b: u64) -> Result<u64, ContractError>

// Saturating operations (no errors)
pub fn saturating_add(env: Env, a: u64, b: u64) -> u64
pub fn saturating_sub(env: Env, a: u64, b: u64) -> u64
pub fn saturating_mul(env: Env, a: u64, b: u64) -> u64

// Wrapping operations (no errors)
pub fn wrapping_add(env: Env, a: u64, b: u64) -> u64
pub fn wrapping_sub(env: Env, a: u64, b: u64) -> u64
pub fn wrapping_mul(env: Env, a: u64, b: u64) -> u64
```

### Financial Calculations

```rust
// Simple interest: principal * rate * periods / 10000
pub fn calculate_interest(
    env: Env,
    principal: i128,
    rate: i32, // basis points (10000 = 100%)
    periods: u32,
) -> Result<i128, ContractError>

// Compound interest with overflow protection
pub fn compound_interest(
    env: Env,
    principal: i128,
    rate: i32, // basis points
    periods: u32,
) -> Result<i128, ContractError>

// Balance management
pub fn deposit(env: Env, amount: i128) -> Result<i128, ContractError>
pub fn transfer(env: Env, amount: i128) -> Result<i128, ContractError>
```

### Bit Operations

```rust
// Bitwise operations
pub fn bitwise_and(env: Env, a: u32, b: u32) -> u32
pub fn bitwise_or(env: Env, a: u32, b: u32) -> u32
pub fn bitwise_xor(env: Env, a: u32, b: u32) -> u32
pub fn bitwise_not(env: Env, a: u32) -> u32

// Bit manipulation
pub fn is_bit_set(env: Env, value: u32, bit: u32) -> Result<bool, ContractError>
pub fn set_bit(env: Env, value: u32, bit: u32) -> Result<u32, ContractError>
pub fn clear_bit(env: Env, value: u32, bit: u32) -> Result<u32, ContractError>
pub fn toggle_bit(env: Env, value: u32, bit: u32) -> Result<u32, ContractError>
```

## Usage Examples

### Basic Arithmetic
```rust
// Safe addition with overflow checking
let result = PrimitiveTypesContract::add_u32(env, 100, 200)?;
assert_eq!(result, 300);

// Overflow detection
let result = PrimitiveTypesContract::add_u32(env, u32::MAX, 1);
assert_eq!(result, Err(ContractError::OverflowError));

// Division by zero protection
let result = PrimitiveTypesContract::div_u32(env, 100, 0);
assert_eq!(result, Err(ContractError::DivisionByZero));
```

### Boolean Logic
```rust
// Boolean operations
let result = PrimitiveTypesContract::bool_and(env, true, false);
assert_eq!(result, false);

let result = PrimitiveTypesContract::bool_or(env, true, false);
assert_eq!(result, true);

// Store and retrieve boolean
PrimitiveTypesContract::set_bool(env, true)?;
let stored = PrimitiveTypesContract::get_bool(env)?;
assert_eq!(stored, true);
```

### Type Conversions
```rust
// Safe conversion (always succeeds)
let result = PrimitiveTypesContract::u32_to_u64(env, 100);
assert_eq!(result, 100);

// Unsafe conversion (may fail)
let result = PrimitiveTypesContract::u64_to_u32(env, u32::MAX as u64 + 1);
assert_eq!(result, Err(ContractError::ConversionError));

// Signed to unsigned (may underflow)
let result = PrimitiveTypesContract::i32_to_u32(env, -1);
assert_eq!(result, Err(ContractError::NegativeValue));
```

### Overflow Handling
```rust
// Checked arithmetic (returns error on overflow)
let result = PrimitiveTypesContract::safe_add(env, u64::MAX, 1);
assert_eq!(result, Err(ContractError::OverflowError));

// Saturating arithmetic (clamps to max value)
let result = PrimitiveTypesContract::saturating_add(env, u64::MAX, 1);
assert_eq!(result, u64::MAX);

// Wrapping arithmetic (wraps around)
let result = PrimitiveTypesContract::wrapping_add(env, u64::MAX, 1);
assert_eq!(result, 0);
```

### Financial Calculations
```rust
// Simple interest calculation
let interest = PrimitiveTypesContract::calculate_interest(
    env, 
    1000i128, // principal
    500,      // 5% rate (500 basis points)
    2         // 2 periods
)?;
assert_eq!(interest, 100); // 1000 * 0.05 * 2 = 100

// Compound interest
let interest = PrimitiveTypesContract::compound_interest(
    env,
    1000i128, // principal
    500,      // 5% rate
    2         // 2 periods
)?;
// 1000 * (1.05^2 - 1) = 1000 * (1.1025 - 1) = 102.5 -> 102 (integer division)
```

### Bit Operations
```rust
// Bitwise operations
let result = PrimitiveTypesContract::bitwise_and(env, 0b1010, 0b1100);
assert_eq!(result, 0b1000);

// Bit manipulation
let result = PrimitiveTypesContract::is_bit_set(env, 0b1010, 1)?;
assert_eq!(result, true);

let result = PrimitiveTypesContract::set_bit(env, 0b1010, 2)?;
assert_eq!(result, 0b1110);
```

## Testing

The example includes comprehensive tests covering:

- **Arithmetic Operations**: Addition, subtraction, multiplication, division
- **Overflow Handling**: Checked, saturating, and wrapping operations
- **Boolean Logic**: AND, OR, NOT, XOR operations
- **Type Conversions**: Safe and unsafe conversions between types
- **Financial Calculations**: Interest calculations and balance management
- **Bit Operations**: Bitwise operations and bit manipulation
- **Storage Operations**: Storing and retrieving primitive values

### Running Tests

```bash
cargo test
```

## Best Practices

### Type Selection
1. **Use appropriate sizes**: Choose the smallest type that fits your needs
2. **Prevent overflow**: Use checked arithmetic for critical calculations
3. **Financial calculations**: Always use `i128` for monetary values
4. **Boolean storage**: Use `bool` for flags and simple states

### Overflow Prevention
1. **Always check**: Use `checked_*` methods for critical arithmetic
2. **Use saturating**: When overflow should clamp to max/min values
3. **Use wrapping**: When overflow should wrap around (counters, hashes)
4. **Validate inputs**: Check ranges before operations

### Type Conversion
1. **Prevent data loss**: Use safe conversions when possible
2. **Handle errors**: Always handle conversion errors gracefully
3. **Document assumptions**: Comment when conversions may lose data
4. **Test edge cases**: Test with max/min values

### Performance Considerations
1. **u32 vs u64**: u32 is more efficient for small values
2. **Checked vs unchecked**: Checked operations have overhead
3. **Storage costs**: Larger types cost more to store
4. **Batch operations**: Minimize individual arithmetic operations

## Common Pitfalls

### Integer Overflow
```rust
❌ Wrong: May overflow
let result = a + b;

✅ Correct: Handle overflow
let result = a.checked_add(b).ok_or(OverflowError)?;
```

### Division by Zero
```rust
❌ Wrong: May panic
let result = a / b;

✅ Correct: Check first
if b == 0 {
    return Err(DivisionByZero);
}
let result = a / b;
```

### Type Conversion Errors
```rust
❌ Wrong: May lose data
let result = large_u64 as u32;

✅ Correct: Safe conversion
let result = large_u64.try_into()
    .map_err(|_| ConversionError)?;
```

### Boolean Storage
```rust
❌ Wrong: Using integer for boolean
let flag: u32 = 1; // 0 = false, 1 = true

✅ Correct: Use boolean type
let flag: bool = true;
```

## Advanced Patterns

### Counter Implementation
```rust
pub struct Counter {
    value: u64,
}

impl Counter {
    pub fn increment(&mut self) -> Result<u64, ContractError> {
        self.value = self.value.checked_add(1)
            .ok_or(ContractError::OverflowError)?;
        Ok(self.value)
    }
    
    pub fn decrement(&mut self) -> Result<u64, ContractError> {
        self.value = self.value.checked_sub(1)
            .ok_or(ContractError::UnderflowError)?;
        Ok(self.value)
    }
}
```

### Fixed-Point Arithmetic
```rust
// Represent 2 decimal places as integer
pub struct FixedPoint {
    value: i64, // Represents value * 100
}

impl FixedPoint {
    pub fn new(value: f64) -> Self {
        Self { value: (value * 100.0) as i64 }
    }
    
    pub fn add(&self, other: &FixedPoint) -> Result<FixedPoint, ContractError> {
        let result = self.value.checked_add(other.value)
            .ok_or(ContractError::OverflowError)?;
        Ok(FixedPoint { value: result })
    }
}
```

### Bitmask Implementation
```rust
pub struct Permissions {
    flags: u32,
}

impl Permissions {
    pub const READ: u32 = 1 << 0;
    pub const WRITE: u32 = 1 << 1;
    pub const EXECUTE: u32 = 1 << 2;
    
    pub fn has_permission(&self, permission: u32) -> bool {
        (self.flags & permission) != 0
    }
    
    pub fn grant_permission(&mut self, permission: u32) {
        self.flags |= permission;
    }
    
    pub fn revoke_permission(&mut self, permission: u32) {
        self.flags &= !permission;
    }
}
```

## Integration with Other Patterns

This example complements other Soroban patterns:

- **Authentication**: Use boolean flags for permissions
- **Validation**: Use integer ranges for validation
- **Error Handling**: Use custom error codes with integers
- **Storage**: Efficient storage of primitive values
- **Events**: Include primitive data in contract events

## Security Considerations

### Input Validation
```rust
// Always validate ranges
if value > MAX_ALLOWED {
    return Err(InvalidInput);
}

// Check for negative values when converting to unsigned
if signed_value < 0 {
    return Err(NegativeValue);
}
```

### Arithmetic Safety
```rust
// Use checked arithmetic for financial calculations
let new_balance = current_balance.checked_add(amount)
    .ok_or(OverflowError)?;

// Prevent division by zero
if divisor == 0 {
    return Err(DivisionByZero);
}
```

### Type Safety
```rust
// Use explicit types to prevent confusion
let user_id: u64 = user_address.clone();
let balance: i128 = token_balance;

// Avoid implicit conversions where possible
let result: u32 = large_value.try_into()?;
```

## Conclusion

Primitive types are fundamental to Soroban smart contracts. Understanding their characteristics, limitations, and proper usage patterns is essential for writing secure and efficient contracts.

Key takeaways:
- Choose appropriate type sizes for your use case
- Always handle overflow and division by zero
- Use safe type conversions and handle errors
- Prefer `i128` for financial calculations
- Use checked arithmetic for critical operations
- Test edge cases and boundary conditions

This example provides a comprehensive foundation for working with primitive types in Soroban contracts.

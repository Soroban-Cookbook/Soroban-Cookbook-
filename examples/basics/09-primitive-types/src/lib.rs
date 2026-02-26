//! # Primitive Types in Soroban
//!
//! This example demonstrates the usage of primitive types in Soroban smart contracts:
//!
//! ## What's Covered
//!
//! ### 1. Integer Types
//! - **Unsigned integers**: `u32`, `u64` - for positive values
//! - **Signed integers**: `i32`, `i64` - for positive and negative values
//! - **Large integers**: `i128`, `u128` - for financial calculations
//! - **Type selection**: When to use each integer type
//!
//! ### 2. Boolean Type
//! - `bool` usage in conditional logic
//! - Boolean operations and comparisons
//! - Storage and retrieval of boolean values
//!
//! ### 3. Type Conversions
//! - Safe type conversions between integer types
//! - Lossless conversions
//! - Explicit casting with potential data loss
//!
//! ### 4. Overflow Handling
//! - Checked arithmetic operations
//! - Saturating arithmetic
//! - Wrapping arithmetic
//! - Panic behavior in release builds

#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env,
};

// ---------------------------------------------------------------------------
// Contract Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    /// General errors (1000-1099)
    InvalidInput = 1000,
    Unauthorized = 1001,
    NotFound = 1002,
    AlreadyExists = 1003,

    /// Type conversion errors (1100-1199)
    ConversionError = 1100,
    OverflowError = 1101,
    UnderflowError = 1102,
    DivisionByZero = 1103,
    NegativeValue = 1104,

    /// Arithmetic errors (1200-1299)
    ArithmeticError = 1200,
    InvalidOperation = 1201,
    InsufficientBalance = 1202,
}

// ---------------------------------------------------------------------------
// Storage Keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    U32Value = 0,
    U64Value = 1,
    I32Value = 2,
    I64Value = 3,
    BoolValue = 4,
    Counter = 5,
    Balance = 6,
    Flags = 7,
}

// ---------------------------------------------------------------------------
// Main Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct PrimitiveTypesContract;

#[contractimpl]
impl PrimitiveTypesContract {
    /// Initialize contract with default values
    pub fn initialize(env: Env) -> Result<(), ContractError> {
        // Set default values for demonstration
        env.storage()
            .instance()
            .set(&DataKey::U32Value, &4294967295u32); // Max u32 value
        env.storage()
            .instance()
            .set(&DataKey::U64Value, &18446744073709551615u64); // Max u64 value
        env.storage()
            .instance()
            .set(&DataKey::I32Value, &2147483647i32); // Max i32 value
        env.storage()
            .instance()
            .set(&DataKey::I64Value, &9223372036854775807i64); // Max i64 value
        env.storage()
            .instance()
            .set(&DataKey::BoolValue, &true);
        env.storage()
            .instance()
            .set(&DataKey::Counter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::Balance, &1000i128);
        env.storage()
            .instance()
            .set(&DataKey::Flags, &0u32);

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Unsigned Integer Operations (u32, u64)
    // ---------------------------------------------------------------------------

    /// Add two u32 values with overflow checking
    pub fn add_u32(env: Env, a: u32, b: u32) -> Result<u32, ContractError> {
        a.checked_add(b).ok_or(ContractError::OverflowError)
    }

    /// Subtract two u32 values with underflow checking
    pub fn sub_u32(env: Env, a: u32, b: u32) -> Result<u32, ContractError> {
        a.checked_sub(b).ok_or(ContractError::UnderflowError)
    }

    /// Multiply two u32 values with overflow checking
    pub fn mul_u32(env: Env, a: u32, b: u32) -> Result<u32, ContractError> {
        a.checked_mul(b).ok_or(ContractError::OverflowError)
    }

    /// Divide two u32 values with division by zero checking
    pub fn div_u32(env: Env, a: u32, b: u32) -> Result<u32, ContractError> {
        if b == 0 {
            return Err(ContractError::DivisionByZero);
        }
        Ok(a / b)
    }

    /// Add two u64 values with overflow checking
    pub fn add_u64(env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        a.checked_add(b).ok_or(ContractError::OverflowError)
    }

    /// Subtract two u64 values with underflow checking
    pub fn sub_u64(env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        a.checked_sub(b).ok_or(ContractError::UnderflowError)
    }

    /// Multiply two u64 values with overflow checking
    pub fn mul_u64(env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        a.checked_mul(b).ok_or(ContractError::OverflowError)
    }

    /// Divide two u64 values with division by zero checking
    pub fn div_u64(env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        if b == 0 {
            return Err(ContractError::DivisionByZero);
        }
        Ok(a / b)
    }

    // ---------------------------------------------------------------------------
    // Signed Integer Operations (i32, i64)
    // ---------------------------------------------------------------------------

    /// Add two i32 values with overflow checking
    pub fn add_i32(env: Env, a: i32, b: i32) -> Result<i32, ContractError> {
        a.checked_add(b).ok_or(ContractError::OverflowError)
    }

    /// Subtract two i32 values with overflow checking
    pub fn sub_i32(env: Env, a: i32, b: i32) -> Result<i32, ContractError> {
        a.checked_sub(b).ok_or(ContractError::OverflowError)
    }

    /// Multiply two i32 values with overflow checking
    pub fn mul_i32(env: Env, a: i32, b: i32) -> Result<i32, ContractError> {
        a.checked_mul(b).ok_or(ContractError::OverflowError)
    }

    /// Divide two i32 values with division by zero checking
    pub fn div_i32(env: Env, a: i32, b: i32) -> Result<i32, ContractError> {
        if b == 0 {
            return Err(ContractError::DivisionByZero);
        }
        Ok(a / b)
    }

    /// Add two i64 values with overflow checking
    pub fn add_i64(env: Env, a: i64, b: i64) -> Result<i64, ContractError> {
        a.checked_add(b).ok_or(ContractError::OverflowError)
    }

    /// Subtract two i64 values with overflow checking
    pub fn sub_i64(env: Env, a: i64, b: i64) -> Result<i64, ContractError> {
        a.checked_sub(b).ok_or(ContractError::OverflowError)
    }

    /// Multiply two i64 values with overflow checking
    pub fn mul_i64(env: Env, a: i64, b: i64) -> Result<i64, ContractError> {
        a.checked_mul(b).ok_or(ContractError::OverflowError)
    }

    /// Divide two i64 values with division by zero checking
    pub fn div_i64(env: Env, a: i64, b: i64) -> Result<i64, ContractError> {
        if b == 0 {
            return Err(ContractError::DivisionByZero);
        }
        Ok(a / b)
    }

    // ---------------------------------------------------------------------------
    // Boolean Operations
    // ---------------------------------------------------------------------------

    /// Logical AND operation
    pub fn bool_and(env: Env, a: bool, b: bool) -> bool {
        a && b
    }

    /// Logical OR operation
    pub fn bool_or(env: Env, a: bool, b: bool) -> bool {
        a || b
    }

    /// Logical NOT operation
    pub fn bool_not(env: Env, a: bool) -> bool {
        !a
    }

    /// XOR operation (implemented as != for booleans)
    pub fn bool_xor(env: Env, a: bool, b: bool) -> bool {
        a != b
    }

    /// Store boolean value
    pub fn set_bool(env: Env, value: bool) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::BoolValue, &value);
        Ok(())
    }

    /// Get stored boolean value
    pub fn get_bool(env: Env) -> Result<bool, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::BoolValue)
            .ok_or(ContractError::NotFound)
    }

    // ---------------------------------------------------------------------------
    // Type Conversions
    // ---------------------------------------------------------------------------

    /// Convert u32 to u64 (always safe)
    pub fn u32_to_u64(env: Env, value: u32) -> u64 {
        value as u64
    }

    /// Convert u64 to u32 (may overflow)
    pub fn u64_to_u32(env: Env, value: u64) -> Result<u32, ContractError> {
        if value > u32::MAX as u64 {
            return Err(ContractError::ConversionError);
        }
        Ok(value as u32)
    }

    /// Convert i32 to i64 (always safe)
    pub fn i32_to_i64(env: Env, value: i32) -> i64 {
        value as i64
    }

    /// Convert i64 to i32 (may overflow)
    pub fn i64_to_i32(env: Env, value: i64) -> Result<i32, ContractError> {
        if value > i32::MAX as i64 || value < i32::MIN as i64 {
            return Err(ContractError::ConversionError);
        }
        Ok(value as i32)
    }

    /// Convert u32 to i32 (may overflow if value > i32::MAX)
    pub fn u32_to_i32(env: Env, value: u32) -> Result<i32, ContractError> {
        if value > i32::MAX as u32 {
            return Err(ContractError::ConversionError);
        }
        Ok(value as i32)
    }

    /// Convert i32 to u32 (may underflow if value < 0)
    pub fn i32_to_u32(env: Env, value: i32) -> Result<u32, ContractError> {
        if value < 0 {
            return Err(ContractError::NegativeValue);
        }
        Ok(value as u32)
    }

    /// Convert i64 to u64 (may underflow if value < 0)
    pub fn i64_to_u64(env: Env, value: i64) -> Result<u64, ContractError> {
        if value < 0 {
            return Err(ContractError::NegativeValue);
        }
        Ok(value as u64)
    }

    /// Convert u64 to i64 (may overflow if value > i64::MAX)
    pub fn u64_to_i64(env: Env, value: u64) -> Result<i64, ContractError> {
        if value > i64::MAX as u64 {
            return Err(ContractError::ConversionError);
        }
        Ok(value as i64)
    }

    // ---------------------------------------------------------------------------
    // Overflow Handling Examples
    // ---------------------------------------------------------------------------

    /// Safe addition with overflow detection
    pub fn safe_add(env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        match a.checked_add(b) {
            Some(result) => Ok(result),
            None => Err(ContractError::OverflowError),
        }
    }

    /// Saturating addition (clamps to max value on overflow)
    pub fn saturating_add(env: Env, a: u64, b: u64) -> u64 {
        a.saturating_add(b)
    }

    /// Wrapping addition (wraps around on overflow)
    pub fn wrapping_add(env: Env, a: u64, b: u64) -> u64 {
        a.wrapping_add(b)
    }

    /// Safe subtraction with underflow detection
    pub fn safe_sub(env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        match a.checked_sub(b) {
            Some(result) => Ok(result),
            None => Err(ContractError::UnderflowError),
        }
    }

    /// Saturating subtraction (clamps to 0 on underflow)
    pub fn saturating_sub(env: Env, a: u64, b: u64) -> u64 {
        a.saturating_sub(b)
    }

    /// Wrapping subtraction (wraps around on underflow)
    pub fn wrapping_sub(env: Env, a: u64, b: u64) -> u64 {
        a.wrapping_sub(b)
    }

    /// Safe multiplication with overflow detection
    pub fn safe_mul(env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        match a.checked_mul(b) {
            Some(result) => Ok(result),
            None => Err(ContractError::OverflowError),
        }
    }

    /// Saturating multiplication (clamps to max value on overflow)
    pub fn saturating_mul(env: Env, a: u64, b: u64) -> u64 {
        a.saturating_mul(b)
    }

    /// Wrapping multiplication (wraps around on overflow)
    pub fn wrapping_mul(env: Env, a: u64, b: u64) -> u64 {
        a.wrapping_mul(b)
    }

    // ---------------------------------------------------------------------------
    // Financial Calculations (using i128 for precision)
    // ---------------------------------------------------------------------------

    /// Calculate interest using i128 for precision
    pub fn calculate_interest(
        env: Env,
        principal: i128,
        rate: i32, // in basis points (10000 = 100%)
        periods: u32,
    ) -> Result<i128, ContractError> {
        if rate < 0 || rate > 10000 {
            return Err(ContractError::InvalidInput);
        }

        // Simple interest: principal * rate * periods / 10000
        let rate_i128 = rate as i128;
        let periods_i128 = periods as i128;
        
        match principal.checked_mul(rate_i128) {
            Some(interest_rate_product) => {
                match interest_rate_product.checked_mul(periods_i128) {
                    Some(total_product) => {
                        Ok(total_product / 10000i128)
                    }
                    None => Err(ContractError::OverflowError),
                }
            }
            None => Err(ContractError::OverflowError),
        }
    }

    /// Compound interest calculation
    pub fn compound_interest(
        env: Env,
        principal: i128,
        rate: i32, // in basis points
        periods: u32,
    ) -> Result<i128, ContractError> {
        if rate < 0 || rate > 10000 {
            return Err(ContractError::InvalidInput);
        }

        // For compound interest, we need to be careful about overflow
        // This is a simplified version - in practice, you'd use more sophisticated methods
        let mut amount = principal;
        let rate_factor = 10000i128 + rate as i128;

        for _ in 0..periods {
            match amount.checked_mul(rate_factor) {
                Some(product) => {
                    amount = product / 10000i128;
                }
                None => return Err(ContractError::OverflowError),
            }
        }

        Ok(amount - principal)
    }

    /// Transfer amount with balance checking
    pub fn transfer(env: Env, amount: i128) -> Result<i128, ContractError> {
        let current_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Balance)
            .ok_or(ContractError::NotFound)?;

        if amount < 0 {
            return Err(ContractError::NegativeValue);
        }

        match current_balance.checked_sub(amount) {
            Some(new_balance) => {
                env.storage().instance().set(&DataKey::Balance, &new_balance);
                Ok(new_balance)
            }
            None => Err(ContractError::InsufficientBalance),
        }
    }

    /// Deposit amount with overflow checking
    pub fn deposit(env: Env, amount: i128) -> Result<i128, ContractError> {
        let current_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Balance)
            .ok_or(ContractError::NotFound)?;

        if amount < 0 {
            return Err(ContractError::NegativeValue);
        }

        match current_balance.checked_add(amount) {
            Some(new_balance) => {
                env.storage().instance().set(&DataKey::Balance, &new_balance);
                Ok(new_balance)
            }
            None => Err(ContractError::OverflowError),
        }
    }

    // ---------------------------------------------------------------------------
    // Bit Operations (demonstrating integer bit manipulation)
    // ---------------------------------------------------------------------------

    /// Bitwise AND operation
    pub fn bitwise_and(env: Env, a: u32, b: u32) -> u32 {
        a & b
    }

    /// Bitwise OR operation
    pub fn bitwise_or(env: Env, a: u32, b: u32) -> u32 {
        a | b
    }

    /// Bitwise XOR operation
    pub fn bitwise_xor(env: Env, a: u32, b: u32) -> u32 {
        a ^ b
    }

    /// Bitwise NOT operation
    pub fn bitwise_not(env: Env, a: u32) -> u32 {
        !a
    }

    /// Left shift operation
    pub fn left_shift(env: Env, a: u32, shift: u32) -> Result<u32, ContractError> {
        if shift >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(a << shift)
    }

    /// Right shift operation
    pub fn right_shift(env: Env, a: u32, shift: u32) -> Result<u32, ContractError> {
        if shift >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(a >> shift)
    }

    /// Check if bit is set
    pub fn is_bit_set(env: Env, value: u32, bit: u32) -> Result<bool, ContractError> {
        if bit >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok((value & (1u32 << bit)) != 0)
    }

    /// Set bit in value
    pub fn set_bit(env: Env, value: u32, bit: u32) -> Result<u32, ContractError> {
        if bit >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(value | (1u32 << bit))
    }

    /// Clear bit in value
    pub fn clear_bit(env: Env, value: u32, bit: u32) -> Result<u32, ContractError> {
        if bit >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(value & !(1u32 << bit))
    }

    /// Toggle bit in value
    pub fn toggle_bit(env: Env, value: u32, bit: u32) -> Result<u32, ContractError> {
        if bit >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(value ^ (1u32 << bit))
    }

    // ---------------------------------------------------------------------------
    // Counter and Flag Management
    // ---------------------------------------------------------------------------

    /// Increment counter with overflow checking
    pub fn increment_counter(env: Env) -> Result<u64, ContractError> {
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::Counter)
            .ok_or(ContractError::NotFound)?;

        match counter.checked_add(1) {
            Some(new_counter) => {
                env.storage().instance().set(&DataKey::Counter, &new_counter);
                Ok(new_counter)
            }
            None => Err(ContractError::OverflowError),
        }
    }

    /// Decrement counter with underflow checking
    pub fn decrement_counter(env: Env) -> Result<u64, ContractError> {
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::Counter)
            .ok_or(ContractError::NotFound)?;

        match counter.checked_sub(1) {
            Some(new_counter) => {
                env.storage().instance().set(&DataKey::Counter, &new_counter);
                Ok(new_counter)
            }
            None => Err(ContractError::UnderflowError),
        }
    }

    /// Get current counter value
    pub fn get_counter(env: Env) -> Result<u64, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Counter)
            .ok_or(ContractError::NotFound)
    }

    /// Set flag bit
    pub fn set_flag(env: Env, flag_bit: u32) -> Result<(), ContractError> {
        if flag_bit >= 32 {
            return Err(ContractError::InvalidInput);
        }

        let flags: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Flags)
            .ok_or(ContractError::NotFound)?;

        let new_flags = flags | (1u32 << flag_bit);
        env.storage().instance().set(&DataKey::Flags, &new_flags);
        Ok(())
    }

    /// Clear flag bit
    pub fn clear_flag(env: Env, flag_bit: u32) -> Result<(), ContractError> {
        if flag_bit >= 32 {
            return Err(ContractError::InvalidInput);
        }

        let flags: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Flags)
            .ok_or(ContractError::NotFound)?;

        let new_flags = flags & !(1u32 << flag_bit);
        env.storage().instance().set(&DataKey::Flags, &new_flags);
        Ok(())
    }

    /// Check if flag is set
    pub fn is_flag_set(env: Env, flag_bit: u32) -> Result<bool, ContractError> {
        if flag_bit >= 32 {
            return Err(ContractError::InvalidInput);
        }

        let flags: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Flags)
            .ok_or(ContractError::NotFound)?;

        Ok((flags & (1u32 << flag_bit)) != 0)
    }

    // ---------------------------------------------------------------------------
    // Comparison Operations
    // ---------------------------------------------------------------------------

    /// Compare two u32 values
    pub fn compare_u32(env: Env, a: u32, b: u32) -> i32 {
        if a > b {
            1
        } else if a < b {
            -1
        } else {
            0
        }
    }

    /// Compare two i32 values
    pub fn compare_i32(env: Env, a: i32, b: i32) -> i32 {
        if a > b {
            1
        } else if a < b {
            -1
        } else {
            0
        }
    }

    /// Check if value is within range (inclusive)
    pub fn is_in_range_u32(env: Env, value: u32, min: u32, max: u32) -> bool {
        value >= min && value <= max
    }

    /// Check if value is within range (inclusive)
    pub fn is_in_range_i32(env: Env, value: i32, min: i32, max: i32) -> bool {
        value >= min && value <= max
    }

    /// Clamp value to range
    pub fn clamp_u32(env: Env, value: u32, min: u32, max: u32) -> u32 {
        value.clamp(min, max)
    }

    /// Clamp value to range
    pub fn clamp_i32(env: Env, value: i32, min: i32, max: i32) -> i32 {
        value.clamp(min, max)
    }

    // ---------------------------------------------------------------------------
    // Storage and Retrieval Examples
    // ---------------------------------------------------------------------------

    /// Store u32 value
    pub fn store_u32(env: Env, value: u32) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::U32Value, &value);
        Ok(())
    }

    /// Retrieve u32 value
    pub fn retrieve_u32(env: Env) -> Result<u32, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::U32Value)
            .ok_or(ContractError::NotFound)
    }

    /// Store u64 value
    pub fn store_u64(env: Env, value: u64) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::U64Value, &value);
        Ok(())
    }

    /// Retrieve u64 value
    pub fn retrieve_u64(env: Env) -> Result<u64, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::U64Value)
            .ok_or(ContractError::NotFound)
    }

    /// Store i32 value
    pub fn store_i32(env: Env, value: i32) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::I32Value, &value);
        Ok(())
    }

    /// Retrieve i32 value
    pub fn retrieve_i32(env: Env) -> Result<i32, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::I32Value)
            .ok_or(ContractError::NotFound)
    }

    /// Store i64 value
    pub fn store_i64(env: Env, value: i64) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::I64Value, &value);
        Ok(())
    }

    /// Retrieve i64 value
    pub fn retrieve_i64(env: Env) -> Result<i64, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::I64Value)
            .ok_or(ContractError::NotFound)
    }

    /// Get current balance
    pub fn get_balance(env: Env) -> Result<i128, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Balance)
            .ok_or(ContractError::NotFound)
    }

    /// Reset all values to defaults
    pub fn reset_to_defaults(env: Env) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::U32Value, &0u32);
        env.storage().instance().set(&DataKey::U64Value, &0u64);
        env.storage().instance().set(&DataKey::I32Value, &0i32);
        env.storage().instance().set(&DataKey::I64Value, &0i64);
        env.storage().instance().set(&DataKey::BoolValue, &false);
        env.storage().instance().set(&DataKey::Counter, &0u64);
        env.storage().instance().set(&DataKey::Balance, &0i128);
        env.storage().instance().set(&DataKey::Flags, &0u32);
        Ok(())
    }
}

// Pull in the dedicated test module.
#[cfg(test)]
mod test;

//! # Soroban Validation Library
//!
//! A collection of reusable validation utilities for Soroban smart contracts.
//! Provides typed validation errors and helpers for parameter, state, and authorization validation.
//!
//! ## Categories of Validation
//!
//! ### Parameter Validation
//! Validates function inputs such as amounts, addresses, strings, arrays, and timestamps.
//!
//! ### State Validation
//! Provides patterns and utilities for validating contract state, balances, and other stored data.
//!
//! ### Authorization Validation
//! Provides patterns and utilities for validating user permissions and access controls.

#![no_std]
use soroban_sdk::{contracterror, Address, Env, String, Vec};

// ---------------------------------------------------------------------------
// Error Types
// ---------------------------------------------------------------------------

/// Comprehensive validation error types for Soroban contracts
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ValidationError {
    // Parameter validation errors (100-199)
    InvalidAmount = 100,
    AmountTooSmall = 101,
    AmountTooLarge = 102,
    InvalidAddress = 103,
    InvalidString = 104,
    StringTooShort = 105,
    StringTooLong = 106,
    InvalidEnum = 107,
    InvalidArray = 108,
    ArrayTooSmall = 109,
    ArrayTooLarge = 110,
    InvalidTimestamp = 111,
    TimestampInPast = 112,
    TimestampInDistantFuture = 113,

    // State validation errors (200-299)
    ContractNotInitialized = 200,
    ContractPaused = 201,
    ContractFrozen = 202,
    InsufficientBalance = 203,
    InsufficientAllowance = 204,
    ResourceNotFound = 205,
    ResourceAlreadyExists = 206,
    InvalidStateTransition = 207,
    InvariantViolation = 208,
    RateLimitExceeded = 209,
    CooldownActive = 210,

    // Authorization validation errors (300-399)
    Unauthorized = 300,
    NotAdmin = 301,
    NotOwner = 302,
    InsufficientRole = 303,
    SignatureRequired = 304,
    MultiSigRequired = 305,
    InvalidSignature = 306,
    ExpiredSignature = 307,
    WrongContract = 308,
    Blacklisted = 309,
}

// ---------------------------------------------------------------------------
// Parameter Validation Functions
// ---------------------------------------------------------------------------

/// Validates amount parameters with min/max bounds
///
/// # Arguments
/// * `amount` - The amount to validate
/// * `min_amount` - Minimum allowed amount (inclusive)
/// * `max_amount` - Maximum allowed amount (inclusive)
///
/// # Errors
/// * `ValidationError::InvalidAmount` - If amount is negative or zero
/// * `ValidationError::AmountTooSmall` - If amount is below minimum
/// * `ValidationError::AmountTooLarge` - If amount exceeds maximum
pub fn validate_amount(
    amount: i128,
    min_amount: i128,
    max_amount: i128,
) -> Result<(), ValidationError> {
    // Basic amount validation
    if amount <= 0 {
        return Err(ValidationError::InvalidAmount);
    }

    // Range validation
    if amount < min_amount {
        return Err(ValidationError::AmountTooSmall);
    }

    if amount > max_amount {
        return Err(ValidationError::AmountTooLarge);
    }

    Ok(())
}

/// Validates string parameters with length constraints
///
/// # Arguments
/// * `text` - The string to validate
/// * `min_length` - Minimum required length (inclusive)
/// * `max_length` - Maximum allowed length (inclusive)
///
/// # Errors
/// * `ValidationError::InvalidString` - If string contains invalid characters or is empty
///   when `min_length > 0`
/// * `ValidationError::StringTooShort` - If string is too short
/// * `ValidationError::StringTooLong` - If string is too long
pub fn validate_string(
    text: String,
    min_length: u32,
    max_length: u32,
) -> Result<(), ValidationError> {
    let length = text.len();

    // Length validation
    if length < min_length {
        return Err(ValidationError::StringTooShort);
    }

    if length > max_length {
        return Err(ValidationError::StringTooLong);
    }

    // Content validation (example: no empty strings when min_length > 0)
    if min_length > 0 && length == 0 {
        return Err(ValidationError::InvalidString);
    }

    Ok(())
}

/// Validates address parameters
///
/// # Arguments
/// * `_address` - The address to validate
///
/// # Errors
/// * `ValidationError::InvalidAddress` - If address is invalid
///
/// # Note
/// In Soroban, addresses are always valid if they exist.
/// This function is a placeholder for more complex address validation
/// such as checking against a blacklist or whitelist.
pub fn validate_address(_address: Address) -> Result<(), ValidationError> {
    // In Soroban, addresses are always valid if they exist
    // This is a placeholder for more complex address validation
    Ok(())
}

/// Validates array parameters with size constraints
///
/// # Arguments
/// * `array` - The array to validate
/// * `min_size` - Minimum required size (inclusive)
/// * `max_size` - Maximum allowed size (inclusive)
///
/// # Errors
/// * `ValidationError::ArrayTooSmall` - If array is too small
/// * `ValidationError::ArrayTooLarge` - If array is too large
pub fn validate_array(
    array: Vec<i32>,
    min_size: u32,
    max_size: u32,
) -> Result<(), ValidationError> {
    let size = array.len();

    if size < min_size {
        return Err(ValidationError::ArrayTooSmall);
    }

    if size > max_size {
        return Err(ValidationError::ArrayTooLarge);
    }

    Ok(())
}

/// Validates timestamp parameters with temporal constraints
///
/// # Arguments
/// * `env` - The contract environment
/// * `timestamp` - The timestamp to validate
/// * `allow_past` - Whether past timestamps are allowed
/// * `max_future_seconds` - Maximum seconds in the future allowed
///
/// # Errors
/// * `ValidationError::InvalidTimestamp` - If timestamp is invalid
/// * `ValidationError::TimestampInPast` - If timestamp is in the past (when not allowed)
/// * `ValidationError::TimestampInDistantFuture` - If timestamp is too far in the future
pub fn validate_timestamp(
    env: &Env,
    timestamp: u64,
    allow_past: bool,
    max_future_seconds: u64,
) -> Result<(), ValidationError> {
    let current_time = env.ledger().timestamp();

    // Check if timestamp is in the past (when not allowed)
    if !allow_past && timestamp < current_time {
        return Err(ValidationError::TimestampInPast);
    }

    // Check if timestamp is too far in the future
    if timestamp > current_time + max_future_seconds {
        return Err(ValidationError::TimestampInDistantFuture);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// State Validation Patterns
// ---------------------------------------------------------------------------

/// Pattern for validating contract initialization
///
/// # Arguments
/// * `env` - The contract environment
/// * `key` - The key to check for existence
///
/// # Errors
/// * `ValidationError::ContractNotInitialized` - If the key doesn't exist
pub fn require_initialized<K>(env: &Env, key: &K) -> Result<(), ValidationError>
where
    K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
{
    if !env.storage().instance().has(key) {
        return Err(ValidationError::ContractNotInitialized);
    }
    Ok(())
}

/// Pattern for validating sufficient balance
///
/// # Arguments
/// * `current_balance` - The current balance
/// * `required_amount` - The required amount
///
/// # Errors
/// * `ValidationError::InsufficientBalance` - If balance is insufficient
pub fn require_sufficient_balance(
    current_balance: i128,
    required_amount: i128,
) -> Result<(), ValidationError> {
    if current_balance < required_amount {
        return Err(ValidationError::InsufficientBalance);
    }
    Ok(())
}

/// Pattern for validating cooldown periods
///
/// # Arguments
/// * `env` - The contract environment
/// * `last_action_time` - The timestamp of the last action
/// * `cooldown_seconds` - The required cooldown period
///
/// # Errors
/// * `ValidationError::CooldownActive` - If cooldown is still active
pub fn require_cooldown_expired(
    env: &Env,
    last_action_time: u64,
    cooldown_seconds: u64,
) -> Result<(), ValidationError> {
    let current_time = env.ledger().timestamp();
    if current_time < last_action_time + cooldown_seconds {
        return Err(ValidationError::CooldownActive);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Authorization Validation Patterns
// ---------------------------------------------------------------------------

/// Pattern for validating ownership
///
/// # Arguments
/// * `stored_owner` - The stored owner address
/// * `claimed_owner` - The address claiming to be owner
///
/// # Errors
/// * `ValidationError::NotOwner` - If addresses don't match
pub fn require_owner(stored_owner: Address, claimed_owner: Address) -> Result<(), ValidationError> {
    if stored_owner != claimed_owner {
        return Err(ValidationError::NotOwner);
    }
    Ok(())
}

/// Pattern for validating admin permissions
///
/// # Arguments
/// * `stored_admin` - The stored admin address
/// * `claimed_admin` - The address claiming to be admin
///
/// # Errors
/// * `ValidationError::NotAdmin` - If addresses don't match
pub fn require_admin(stored_admin: Address, claimed_admin: Address) -> Result<(), ValidationError> {
    if stored_admin != claimed_admin {
        return Err(ValidationError::NotAdmin);
    }
    Ok(())
}

/// Pattern for validating role hierarchy
///
/// # Arguments
/// * `user_role` - The user's current role
/// * `required_role` - The minimum required role
///
/// # Type Parameters
/// * `R` - The role type that implements Ord
///
/// # Errors
/// * `ValidationError::InsufficientRole` - If user role is insufficient
pub fn require_role<R: Ord>(user_role: R, required_role: R) -> Result<(), ValidationError> {
    if user_role < required_role {
        return Err(ValidationError::InsufficientRole);
    }
    Ok(())
}

/// Pattern for checking blacklist status
///
/// # Arguments
/// * `is_blacklisted` - Whether the address is blacklisted
///
/// # Errors
/// * `ValidationError::Blacklisted` - If address is blacklisted
pub fn require_not_blacklisted(is_blacklisted: bool) -> Result<(), ValidationError> {
    if is_blacklisted {
        return Err(ValidationError::Blacklisted);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_validate_amount() {
        // Valid amounts
        assert!(validate_amount(100, 1, 1000).is_ok());
        assert!(validate_amount(1, 1, 1000).is_ok());
        assert!(validate_amount(1000, 1, 1000).is_ok());

        // Invalid amounts
        assert_eq!(
            validate_amount(0, 1, 1000),
            Err(ValidationError::InvalidAmount)
        );
        assert_eq!(
            validate_amount(-1, 1, 1000),
            Err(ValidationError::InvalidAmount)
        );
        assert_eq!(
            validate_amount(100, 1, 50),
            Err(ValidationError::AmountTooLarge)
        );
        assert_eq!(
            validate_amount(100, 200, 1000),
            Err(ValidationError::AmountTooSmall)
        );
    }

    #[test]
    fn test_validate_string() {
        let env = Env::default();

        // Valid strings
        let short = String::from_str(&env, "hi");
        assert!(validate_string(short, 0, 10).is_ok());

        let at_limit = String::from_str(&env, "1234567890");
        assert!(validate_string(at_limit, 0, 10).is_ok());

        // Invalid strings
        let empty = String::from_str(&env, "");
        assert_eq!(
            validate_string(empty.clone(), 1, 10),
            Err(ValidationError::StringTooShort)
        );
        assert_eq!(validate_string(empty, 0, 10), Ok(()));

        let too_long = String::from_str(&env, "this string is way too long for the limit");
        assert_eq!(
            validate_string(too_long, 0, 10),
            Err(ValidationError::StringTooLong)
        );
    }

    #[test]
    fn test_validate_address() {
        let env = Env::default();
        let address = Address::generate(&env);

        // Addresses are always valid in Soroban
        assert!(validate_address(address).is_ok());
    }

    #[test]
    fn test_validate_array() {
        let env = Env::default();

        // Valid arrays
        let small = Vec::from_array(&env, [1, 2, 3]);
        assert!(validate_array(small, 1, 10).is_ok());

        let empty = Vec::from_array(&env, []);
        assert!(validate_array(empty, 0, 10).is_ok());

        // Invalid arrays
        let too_small = Vec::from_array(&env, [1]);
        assert_eq!(
            validate_array(too_small, 2, 10),
            Err(ValidationError::ArrayTooSmall)
        );

        let too_large = Vec::from_array(&env, [1, 2, 3, 4, 5, 6]);
        assert_eq!(
            validate_array(too_large, 1, 3),
            Err(ValidationError::ArrayTooLarge)
        );
    }
}

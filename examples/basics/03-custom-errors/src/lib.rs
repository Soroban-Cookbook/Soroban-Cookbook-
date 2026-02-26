//! # Custom Errors Contract
//!
//! This contract demonstrates comprehensive custom error handling in Soroban.
//! It shows how to define descriptive error variants using the `contracterror`
//! attribute, assign proper error codes, and use them in contract functions.
//!
//! Custom errors provide:
//! - Type-safe error handling
//! - Clear error messages for debugging
//! - Proper error codes for frontend integration
//! - Better user experience compared to generic panics

#![no_std]

use soroban_sdk::{ contract, contracterror, contractimpl, symbol_short, Address, Env, Symbol };

/// Custom error enum with descriptive variants
/// Each variant represents a specific error condition that can occur
/// during contract execution.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Input validation failed (Error Code: 1)
    /// Used when provided parameters don't meet requirements
    InvalidInput = 1,

    /// Unauthorized access attempt (Error Code: 2)
    /// Used when caller lacks required permissions
    Unauthorized = 2,

    /// Resource not found (Error Code: 3)
    /// Used when requested data doesn't exist in storage
    NotFound = 3,

    /// Insufficient balance (Error Code: 4)
    /// Used when account doesn't have enough tokens/balance
    InsufficientBalance = 4,

    /// Operation not allowed (Error Code: 5)
    /// Used when business logic prevents the operation
    OperationNotAllowed = 5,

    /// Rate limit exceeded (Error Code: 6)
    /// Used when caller exceeds operation frequency limits
    RateLimitExceeded = 6,

    /// Contract is paused (Error Code: 7)
    /// Used when contract is in paused state for maintenance
    ContractPaused = 7,

    /// Duplicate entry detected (Error Code: 8)
    /// Used when trying to create something that already exists
    AlreadyExists = 8,
}

/// Contract demonstrating various error scenarios
#[contract]
pub struct CustomErrorsContract;

#[contractimpl]
impl CustomErrorsContract {
    /// Validates input and returns error if invalid
    ///
    /// # Arguments
    /// * `value` - Input value to validate (must be > 0)
    ///
    /// # Errors
    /// * `InvalidInput` - If value is 0 or negative
    pub fn validate_input(env: Env, value: i64) -> Result<(), ContractError> {
        if value <= 0 {
            env.events().publish(symbol_short!("inp_err"), ("Invalid value", value));
            Err(ContractError::InvalidInput)
        } else {
            Ok(())
        }
    }

    /// Checks authorization and returns error if unauthorized
    ///
    /// # Arguments
    /// * `caller` - Address attempting the operation
    /// * `admin` - Authorized admin address
    ///
    /// # Errors
    /// * `Unauthorized` - If caller is not the admin
    pub fn check_authorization(
        env: Env,
        caller: Address,
        admin: Address
    ) -> Result<(), ContractError> {
        if caller != admin {
            env.events().publish(symbol_short!("auth_err"), ("Unauthorized access", caller));
            Err(ContractError::Unauthorized)
        } else {
            Ok(())
        }
    }

    /// Retrieves a value from storage or returns not found error
    ///
    /// # Arguments
    /// * `key` - Storage key to retrieve
    ///
    /// # Errors
    /// * `NotFound` - If key doesn't exist in storage
    pub fn get_value(env: Env, key: Symbol) -> Result<u64, ContractError> {
        let storage = env.storage().instance();

        if storage.has(&key) {
            Ok(storage.get(&key).unwrap())
        } else {
            env.events().publish(symbol_short!("not_found"), ("Key not found", key));
            Err(ContractError::NotFound)
        }
    }

    /// Transfers tokens with balance checking
    ///
    /// # Arguments
    /// * `from_balance` - Current balance of sender
    /// * `amount` - Amount to transfer
    ///
    /// # Errors
    /// * `InsufficientBalance` - If balance is too low
    /// * `InvalidInput` - If amount is zero or negative
    pub fn transfer_tokens(env: Env, from_balance: u64, amount: u64) -> Result<(), ContractError> {
        if amount == 0 {
            env.events().publish(symbol_short!("xfer_err"), ("Zero amount transfer", amount));
            Err(ContractError::InvalidInput)
        } else if from_balance < amount {
            env.events().publish(symbol_short!("bal_err"), (
                "Insufficient balance",
                from_balance,
            ));
            Err(ContractError::InsufficientBalance)
        } else {
            // Simulate successful transfer
            env.events().publish(symbol_short!("xfer_ok"), ("Amount transferred", amount));
            Ok(())
        }
    }

    /// Performs an operation that might be restricted
    ///
    /// # Arguments
    /// * `is_paused` - Whether contract is paused
    /// * `operation_type` - Type of operation being attempted
    ///
    /// # Errors
    /// * `ContractPaused` - If contract is paused
    /// * `OperationNotAllowed` - If operation is not permitted
    pub fn perform_operation(
        env: Env,
        is_paused: bool,
        operation_type: Symbol
    ) -> Result<(), ContractError> {
        if is_paused {
            env.events().publish(symbol_short!("pause_err"), (
                "Contract paused",
                operation_type,
            ));
            Err(ContractError::ContractPaused)
        } else if operation_type == symbol_short!("forbidden") {
            env.events().publish(symbol_short!("forbid"), (
                "Operation not allowed",
                operation_type,
            ));
            Err(ContractError::OperationNotAllowed)
        } else {
            env.events().publish(symbol_short!("op_ok"), (
                "Operation completed",
                operation_type,
            ));
            Ok(())
        }
    }

    /// Creates a new entry with duplicate checking
    ///
    /// # Arguments
    /// * `key` - Key for the new entry
    /// * `value` - Value to store
    ///
    /// # Errors
    /// * `AlreadyExists` - If entry already exists
    /// * `InvalidInput` - If value is zero
    pub fn create_entry(env: Env, key: Symbol, value: u64) -> Result<(), ContractError> {
        let storage = env.storage().instance();

        if value == 0 {
            env.events().publish(symbol_short!("create_er"), ("Zero value not allowed", value));
            Err(ContractError::InvalidInput)
        } else if storage.has(&key) {
            env.events().publish(symbol_short!("dup_err"), ("Entry already exists", key));
            Err(ContractError::AlreadyExists)
        } else {
            storage.set(&key, &value);
            env.events().publish(symbol_short!("create_ok"), ("Entry created", key));
            Ok(())
        }
    }

    /// Checks rate limiting for operations
    ///
    /// # Arguments
    /// * `caller` - Address of the caller
    /// * `operation_count` - Number of operations performed
    /// * `max_operations` - Maximum allowed operations
    ///
    /// # Errors
    /// * `RateLimitExceeded` - If caller exceeded rate limit
    /// * `Unauthorized` - If caller address is invalid (None equivalent)
    pub fn check_rate_limit(
        env: Env,
        caller: Address,
        operation_count: u32,
        max_operations: u32
    ) -> Result<(), ContractError> {
        // Check if caller is the contract itself (simplified invalid check)
        let contract_address = env.current_contract_address();
        if caller == contract_address {
            env.events().publish(symbol_short!("inv_call"), (
                "Contract cannot call itself",
                caller,
            ));
            Err(ContractError::Unauthorized)
        } else if operation_count >= max_operations {
            env.events().publish(symbol_short!("rate_lim"), (
                "Rate limit exceeded",
                operation_count,
            ));
            Err(ContractError::RateLimitExceeded)
        } else {
            env.events().publish(symbol_short!("rate_ok"), ("Operation allowed", operation_count));
            Ok(())
        }
    }

    /// Demonstrates multiple error scenarios in one function
    ///
    /// # Arguments
    /// * `amount` - Amount to process
    /// * `caller` - Caller address
    /// * `admin` - Admin address
    /// * `is_paused` - Contract pause status
    ///
    /// # Errors
    /// * Multiple possible errors based on conditions
    pub fn complex_operation(
        env: Env,
        amount: u64,
        caller: Address,
        admin: Address,
        is_paused: bool
    ) -> Result<(), ContractError> {
        // Step 1: Check if contract is paused
        if is_paused {
            return Err(ContractError::ContractPaused);
        }

        // Step 2: Validate input
        if amount == 0 {
            return Err(ContractError::InvalidInput);
        }

        // Step 3: Check authorization
        if caller != admin {
            return Err(ContractError::Unauthorized);
        }

        // Step 4: Simulate balance check (assuming minimum balance of 1000)
        let min_balance = 1000u64;
        if amount > min_balance {
            return Err(ContractError::InsufficientBalance);
        }

        // All checks passed
        env.events().publish(symbol_short!("cmplx_ok"), ("Operation completed", amount));
        Ok(())
    }
}

mod test;

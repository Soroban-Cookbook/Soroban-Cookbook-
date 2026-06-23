//! # Enum Patterns in Soroban
//!
//! This example demonstrates various enum patterns in Soroban smart contracts:
//!
//! ## What's Covered
//!
//! ### 1. Simple Enums
//! - Basic enum definitions without associated data
//! - Used for representing fixed sets of values
//!
//! ### 2. Contract Error Enums
//! - Using `#[contracterror]` for custom error types
//! - Proper error handling with Result types
//!
//! ### 3. Pattern Matching
//! - Using match statements with enums
//! - Exhaustive pattern matching

#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, Env, Symbol,
    Vec,
};

// ---------------------------------------------------------------------------
// Simple Enums (without associated data)
// ---------------------------------------------------------------------------

/// Simple enum for representing different user roles
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum UserRole {
    None = 0,
    User = 1,
    Moderator = 2,
    Admin = 3,
    Owner = 4,
}

/// Simple enum for representing contract states
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum ContractState {
    Uninitialized = 0,
    Active = 1,
    Paused = 2,
    Frozen = 3,
    Shutdown = 4,
}

/// Simple enum for representing transaction types
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TransactionType {
    Deposit = 0,
    Withdraw = 1,
    Transfer = 2,
    Mint = 3,
    Burn = 4,
}

/// Simple enum for representing validation results
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValidationResult {
    /// Validation passed
    Success = 0,
    /// Validation failed
    Failure = 1,
    /// Validation requires approval
    RequiresApproval = 2,
    /// Validation is pending
    Pending = 3,
}

// ---------------------------------------------------------------------------
// Enums with Associated Data
// ---------------------------------------------------------------------------

// Soroban's #[contracttype] macro does not support named fields inside enum
// variants.  The idiomatic pattern is to define a separate #[contracttype]
// struct for each variant's payload and then use a tuple variant that wraps
// that struct.

/// Payload for a token transfer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferParams {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

/// Payload for minting new tokens.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintParams {
    pub to: Address,
    pub amount: i128,
}

/// Payload for burning tokens.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BurnParams {
    pub from: Address,
    pub amount: i128,
}

/// An asset operation whose variants each carry different data structs.
///
/// Pattern matching on this enum extracts the inner struct, whose fields
/// are then accessed by name — the Soroban-idiomatic way of writing
/// enums with data.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetOperation {
    /// Transfer tokens from one address to another.
    Transfer(TransferParams),
    /// Mint new tokens to an address.
    Mint(MintParams),
    /// Burn tokens held by an address.
    Burn(BurnParams),
}

// ---------------------------------------------------------------------------
// Contract Error Enums
// ---------------------------------------------------------------------------

/// Custom error enum for contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum ContractError {
    /// General errors (1000-1099)
    InvalidInput = 1000,
    Unauthorized = 1001,
    InsufficientBalance = 1002,
    InvalidAmount = 1003,
    InvalidAddress = 1004,

    /// State errors (1100-1199)
    ContractNotInitialized = 1100,
    ContractAlreadyInitialized = 1101,
    ContractPaused = 1102,
    ContractFrozen = 1103,
    InvalidStateTransition = 1104,

    /// Operation errors (1200-1299)
    OperationNotFound = 1200,
    OperationAlreadyCompleted = 1201,
    OperationExpired = 1202,
    OperationNotApproved = 1203,
    InsufficientApprovals = 1204,

    /// Asset errors (1300-1399)
    AssetNotFound = 1300,
    InvalidAsset = 1301,
    AssetNotSupported = 1302,
    InsufficientAssetBalance = 1303,

    /// User/Role errors (1400-1499)
    UserNotFound = 1400,
    InsufficientRole = 1401,
    UserBlacklisted = 1402,
    UserSuspended = 1403,

    /// Validation errors (1500-1599)
    ValidationFailed = 1500,
    ValidationPending = 1501,
    ValidationExpired = 1502,
    ConditionNotMet = 1503,

    /// System errors (1600-1699)
    InternalError = 1600,
    StorageError = 1601,
    MathOverflow = 1602,
    TimestampError = 1603,
}

// ---------------------------------------------------------------------------
// Main Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct EnumContract;

#[contractimpl]
impl EnumContract {
    /// Initialize contract with default state
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        // Check if already initialized
        if env.storage().instance().has(&symbol_short!("state")) {
            return Err(ContractError::ContractAlreadyInitialized);
        }

        // Set initial state
        env.storage()
            .instance()
            .set(&symbol_short!("state"), &ContractState::Active);
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);

        // Set admin as Owner
        env.storage()
            .instance()
            .set(&(symbol_short!("user_role"), admin), &UserRole::Owner);

        Ok(())
    }

    /// Get current contract state
    pub fn get_state(env: Env) -> ContractState {
        env.storage()
            .instance()
            .get(&symbol_short!("state"))
            .unwrap_or(ContractState::Uninitialized)
    }

    /// Get user role
    pub fn get_user_role(env: Env, user: Address) -> UserRole {
        env.storage()
            .instance()
            .get(&(symbol_short!("user_role"), user))
            .unwrap_or(UserRole::None)
    }

    /// Set user role (admin only)
    pub fn set_user_role(
        env: Env,
        admin: Address,
        user: Address,
        role: UserRole,
    ) -> Result<(), ContractError> {
        // Validate admin role
        let admin_role = Self::get_user_role(env.clone(), admin);
        if admin_role != UserRole::Owner && admin_role != UserRole::Admin {
            return Err(ContractError::InsufficientRole);
        }

        // Cannot set owner role through this function
        if role == UserRole::Owner {
            return Err(ContractError::InvalidInput);
        }

        // Set role
        env.storage()
            .instance()
            .set(&(symbol_short!("user_role"), user), &role);

        Ok(())
    }

    /// Execute operation with enum-based pattern matching
    pub fn execute_operation(
        env: Env,
        operation: TransactionType,
        amount: i128,
        to: Address,
    ) -> Result<ValidationResult, ContractError> {
        // Pattern match on operation type
        match operation {
            TransactionType::Transfer => Self::validate_transfer(env.clone(), amount, to),
            TransactionType::Deposit => Self::validate_deposit(env.clone(), amount, to),
            TransactionType::Withdraw => Self::validate_withdraw(env.clone(), amount, to),
            TransactionType::Mint => Self::validate_mint(env.clone(), amount, to),
            TransactionType::Burn => Self::validate_burn(env, amount, to),
        }
    }

    /// Process validation result with pattern matching
    pub fn process_validation_result(
        env: Env,
        result: ValidationResult,
        operation_id: u64,
    ) -> Result<(), ContractError> {
        match result {
            ValidationResult::Success => {
                // Mark operation as completed
                env.storage()
                    .instance()
                    .set(&symbol_short!("op"), &operation_id);
                Ok(())
            }
            ValidationResult::Failure => Err(ContractError::ValidationFailed),
            ValidationResult::RequiresApproval => Err(ContractError::InsufficientApprovals),
            ValidationResult::Pending => Err(ContractError::ValidationPending),
        }
    }

    /// Demonstrate enum comparisons and operations
    pub fn compare_enums(_env: Env, role1: UserRole, role2: UserRole) -> bool {
        // Compare roles
        role1 >= role2
    }

    /// Demonstrate enum arithmetic and conversions
    pub fn enum_arithmetic(_env: Env) -> u32 {
        // Convert enum to u32 and perform arithmetic
        let admin_value = UserRole::Admin as u32;
        let user_value = UserRole::User as u32;
        admin_value + user_value
    }

    /// Demonstrate enum iteration
    pub fn get_all_roles(env: Env) -> Vec<UserRole> {
        // Return all possible roles
        vec![
            &env,
            UserRole::None,
            UserRole::User,
            UserRole::Moderator,
            UserRole::Admin,
            UserRole::Owner,
        ]
    }

    // ---------------------------------------------------------------------------
    // Enums with Associated Data
    // ---------------------------------------------------------------------------

    /// Execute an `AssetOperation`, extracting its inner struct via pattern matching.
    ///
    /// Returns the effective signed amount:
    /// - Transfer / Mint → positive amount
    /// - Burn → negative amount (signals supply reduction)
    ///
    /// Demonstrates matching on tuple variants and accessing fields by name
    /// on the extracted struct.
    pub fn process_asset_op(_env: Env, op: AssetOperation) -> Result<i128, ContractError> {
        match op {
            AssetOperation::Transfer(p) => {
                if p.amount <= 0 {
                    return Err(ContractError::InvalidAmount);
                }
                Ok(p.amount)
            }
            AssetOperation::Mint(p) => {
                if p.amount <= 0 {
                    return Err(ContractError::InvalidAmount);
                }
                Ok(p.amount)
            }
            AssetOperation::Burn(p) => {
                if p.amount <= 0 {
                    return Err(ContractError::InvalidAmount);
                }
                // Burn reduces supply — return negative to signal that.
                Ok(-p.amount)
            }
        }
    }

    /// Return a short `Symbol` label identifying the operation kind.
    ///
    /// Uses `_` to discard the inner struct — useful when only the variant
    /// type matters, not its data.
    pub fn op_kind(_env: Env, op: AssetOperation) -> Symbol {
        match op {
            AssetOperation::Transfer(_) => symbol_short!("transfer"),
            AssetOperation::Mint(_) => symbol_short!("mint"),
            AssetOperation::Burn(_) => symbol_short!("burn"),
        }
    }

    // ---------------------------------------------------------------------------
    // Helper Functions (private)
    // ---------------------------------------------------------------------------

    fn validate_transfer(
        _env: Env,
        amount: i128,
        _to: Address,
    ) -> Result<ValidationResult, ContractError> {
        if amount <= 0 {
            return Ok(ValidationResult::Failure);
        }

        // Check balance (simplified)
        if amount > 1000 {
            return Ok(ValidationResult::Failure);
        }

        Ok(ValidationResult::Success)
    }

    fn validate_deposit(
        _env: Env,
        amount: i128,
        _to: Address,
    ) -> Result<ValidationResult, ContractError> {
        if amount <= 0 {
            return Ok(ValidationResult::Failure);
        }

        // Check deposit limit (simplified)
        if amount > 5000 {
            return Ok(ValidationResult::Failure);
        }

        Ok(ValidationResult::Success)
    }

    fn validate_withdraw(
        _env: Env,
        amount: i128,
        _to: Address,
    ) -> Result<ValidationResult, ContractError> {
        if amount <= 0 {
            return Ok(ValidationResult::Failure);
        }

        // Check withdraw limit (simplified)
        if amount > 10000 {
            return Ok(ValidationResult::Failure);
        }

        Ok(ValidationResult::Success)
    }

    fn validate_mint(
        _env: Env,
        amount: i128,
        _to: Address,
    ) -> Result<ValidationResult, ContractError> {
        if amount <= 0 {
            return Ok(ValidationResult::Failure);
        }

        // Check mint limit (simplified)
        if amount > 1000000 {
            return Ok(ValidationResult::Failure);
        }

        Ok(ValidationResult::Success)
    }

    fn validate_burn(
        _env: Env,
        amount: i128,
        _to: Address,
    ) -> Result<ValidationResult, ContractError> {
        if amount <= 0 {
            return Ok(ValidationResult::Failure);
        }

        // Check burn limit (simplified)
        if amount > 500000 {
            return Ok(ValidationResult::Failure);
        }

        Ok(ValidationResult::Success)
    }
}

// Pull in the dedicated test module.
#[cfg(test)]
mod test;

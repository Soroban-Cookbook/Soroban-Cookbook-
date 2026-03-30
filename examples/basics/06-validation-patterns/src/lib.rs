//! # Validation Patterns
//!
//! This contract demonstrates comprehensive input validation patterns in Soroban contracts.
//! It covers parameter validation, state validation, and authorization validation with clear error messages.
//!
//! ## Key Validation Patterns
//!
//! ### 1. Parameter Validation
//! - Type checking and range validation
//! - Format validation for strings and addresses
//! - Business rule validation
//! - Clear error messages for invalid inputs
//!
//! ### 2. State Validation
//! - Contract state consistency checks
//! - Invariant validation
//! - Temporal validation (time-based constraints)
//! - Resource availability validation
//!
//! ### 3. Authorization Validation
//! - Role-based access control
//! - Ownership verification
//! - Multi-signature requirements
//! - Permission checks for specific operations

#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};

// ---------------------------------------------------------------------------
// Error Types
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
// Data Types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum UserRole {
    None = 0,
    User = 1,
    Moderator = 2,
    Admin = 3,
    Owner = 4,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractState {
    Uninitialized = 0,
    Active = 1,
    Paused = 2,
    Frozen = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Owner,
    State,
    UserRole(Address),
    Balance(Address),
    Allowance(Address, Address),
    LastAction(Address),
    Cooldown(Address),
    Blacklist(Address),
    Counter,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// Validation Patterns Contract
///
/// This contract demonstrates comprehensive validation patterns for Soroban smart contracts.
/// It shows how to properly validate inputs, state, and authorization with clear error messages.
#[contract]
pub struct ValidationContract;

#[contractimpl]
impl ValidationContract {
    // ==================== INITIALIZATION ====================

    /// Initialize the contract with an owner
    ///
    /// # Arguments
    /// * `owner` - The address that will own the contract
    ///
    /// # Errors
    /// * `ValidationError::InvalidAddress` - If owner address is invalid
    /// * `ValidationError::ContractNotInitialized` - If already initialized
    pub fn initialize(env: Env, owner: Address) -> Result<(), ValidationError> {
        // Parameter validation
        Self::validate_address(owner.clone())?;

        // State validation
        if env.storage().instance().has(&DataKey::Owner) {
            return Err(ValidationError::ContractNotInitialized);
        }

        // Authorization validation (anyone can initialize initially)
        owner.require_auth();

        // Set initial state
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::Admin, &owner);
        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Active);

        Ok(())
    }

    // ==================== PARAMETER VALIDATION EXAMPLES ====================

    /// Example of parameter validation with amounts
    ///
    /// # Arguments
    /// * `amount` - The amount to validate
    /// * `min_amount` - Minimum allowed amount
    /// * `max_amount` - Maximum allowed amount
    ///
    /// # Errors
    /// * `ValidationError::InvalidAmount` - If amount is negative
    /// * `ValidationError::AmountTooSmall` - If amount is below minimum
    /// * `ValidationError::AmountTooLarge` - If amount exceeds maximum
    pub fn validate_amount_parameters(
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

    /// Example of string parameter validation
    ///
    /// # Arguments
    /// * `text` - The string to validate
    /// * `min_length` - Minimum required length
    /// * `max_length` - Maximum allowed length
    ///
    /// # Errors
    /// * `ValidationError::InvalidString` - If string contains invalid characters
    /// * `ValidationError::StringTooShort` - If string is too short
    /// * `ValidationError::StringTooLong` - If string is too long
    pub fn validate_string_parameters(
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

        // Content validation (example: no empty strings)
        if length == 0 {
            return Err(ValidationError::InvalidString);
        }

        Ok(())
    }

    /// Example of address parameter validation
    ///
    /// # Arguments
    /// * `address` - The address to validate
    ///
    /// # Errors
    /// * `ValidationError::InvalidAddress` - If address is invalid
    pub fn validate_address(_address: Address) -> Result<(), ValidationError> {
        // In Soroban, addresses are always valid if they exist
        // This is a placeholder for more complex address validation
        // such as checking against a blacklist or whitelist
        Ok(())
    }

    /// Example of array parameter validation
    ///
    /// # Arguments
    /// * `array` - The array to validate
    /// * `min_size` - Minimum required size
    /// * `max_size` - Maximum allowed size
    ///
    /// # Errors
    /// * `ValidationError::ArrayTooSmall` - If array is too small
    /// * `ValidationError::ArrayTooLarge` - If array is too large
    pub fn validate_array_parameters(
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

    /// Example of timestamp parameter validation
    ///
    /// # Arguments
    /// * `timestamp` - The timestamp to validate
    /// * `allow_past` - Whether past timestamps are allowed
    /// * `max_future_seconds` - Maximum seconds in the future allowed
    ///
    /// # Errors
    /// * `ValidationError::InvalidTimestamp` - If timestamp is invalid
    /// * `ValidationError::TimestampInPast` - If timestamp is in the past (when not allowed)
    /// * `ValidationError::TimestampInDistantFuture` - If timestamp is too far in the future
    pub fn validate_timestamp_parameters(
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

    // ==================== STATE VALIDATION EXAMPLES ====================

    /// Example of contract state validation
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `required_state` - The required contract state
    ///
    /// # Errors
    /// * `ValidationError::ContractNotInitialized` - If contract is not initialized
    /// * `ValidationError::ContractPaused` - If contract is paused
    /// * `ValidationError::ContractFrozen` - If contract is frozen
    pub fn validate_contract_state(
        env: &Env,
        required_state: ContractState,
    ) -> Result<(), ValidationError> {
        // Check if contract is initialized
        if !env.storage().instance().has(&DataKey::State) {
            return Err(ValidationError::ContractNotInitialized);
        }

        let current_state: ContractState = env.storage().instance().get(&DataKey::State).unwrap();

        match current_state {
            ContractState::Uninitialized => {
                return Err(ValidationError::ContractNotInitialized);
            }
            ContractState::Paused => {
                return Err(ValidationError::ContractPaused);
            }
            ContractState::Frozen => {
                return Err(ValidationError::ContractFrozen);
            }
            ContractState::Active => {
                // Check if specific state is required
                if current_state != required_state {
                    return Err(ValidationError::InvalidStateTransition);
                }
            }
        }

        Ok(())
    }

    /// Example of balance validation
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to check balance for
    /// * `required_amount` - The required minimum balance
    ///
    /// # Errors
    /// * `ValidationError::InsufficientBalance` - If balance is insufficient
    pub fn validate_balance(
        env: &Env,
        address: Address,
        required_amount: i128,
    ) -> Result<(), ValidationError> {
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(address.clone()))
            .unwrap_or(0);

        if balance < required_amount {
            return Err(ValidationError::InsufficientBalance);
        }

        Ok(())
    }

    /// Example of allowance validation
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `owner` - The owner address
    /// * `spender` - The spender address
    /// * `required_amount` - The required minimum allowance
    ///
    /// # Errors
    /// * `ValidationError::InsufficientAllowance` - If allowance is insufficient
    pub fn validate_allowance(
        env: &Env,
        owner: Address,
        spender: Address,
        required_amount: i128,
    ) -> Result<(), ValidationError> {
        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(owner.clone(), spender.clone()))
            .unwrap_or(0);

        if allowance < required_amount {
            return Err(ValidationError::InsufficientAllowance);
        }

        Ok(())
    }

    /// Example of cooldown validation
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to check cooldown for
    /// * `cooldown_seconds` - The cooldown period in seconds
    ///
    /// # Errors
    /// * `ValidationError::CooldownActive` - If cooldown is still active
    pub fn validate_cooldown(
        env: &Env,
        address: Address,
        cooldown_seconds: u64,
    ) -> Result<(), ValidationError> {
        if let Some(last_action) = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&DataKey::LastAction(address.clone()))
        {
            let current_time = env.ledger().timestamp();

            if current_time < last_action + cooldown_seconds {
                return Err(ValidationError::CooldownActive);
            }
        }

        Ok(())
    }

    // ==================== AUTHORIZATION VALIDATION EXAMPLES ====================

    /// Example of role-based authorization validation
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to validate
    /// * `required_role` - The minimum required role
    ///
    /// # Errors
    /// * `ValidationError::NotAdmin` - If address is not admin
    /// * `ValidationError::NotOwner` - If address is not owner
    /// * `ValidationError::InsufficientRole` - If role is insufficient
    /// * `ValidationError::Blacklisted` - If address is blacklisted
    pub fn validate_role(
        env: &Env,
        address: Address,
        required_role: UserRole,
    ) -> Result<(), ValidationError> {
        // Check if address is blacklisted
        if env
            .storage()
            .instance()
            .has(&DataKey::Blacklist(address.clone()))
        {
            return Err(ValidationError::Blacklisted);
        }

        // Get user role
        let user_role: UserRole = env
            .storage()
            .instance()
            .get(&DataKey::UserRole(address.clone()))
            .unwrap_or(UserRole::None);

        // Check role hierarchy
        if user_role < required_role {
            return Err(ValidationError::InsufficientRole);
        }

        // Special checks for owner and admin
        match required_role {
            UserRole::Owner => {
                if user_role != UserRole::Owner {
                    return Err(ValidationError::NotOwner);
                }
            }
            UserRole::Admin => {
                if user_role != UserRole::Admin && user_role != UserRole::Owner {
                    return Err(ValidationError::NotAdmin);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Example of ownership validation
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address claiming to be owner
    ///
    /// # Errors
    /// * `ValidationError::NotOwner` - If address is not the owner
    pub fn validate_ownership(env: &Env, address: Address) -> Result<(), ValidationError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(ValidationError::ContractNotInitialized)?;

        if address != owner {
            return Err(ValidationError::NotOwner);
        }

        Ok(())
    }

    /// Example of admin validation
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address claiming to be admin
    ///
    /// # Errors
    /// * `ValidationError::NotAdmin` - If address is not admin
    pub fn validate_admin(env: &Env, address: Address) -> Result<(), ValidationError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ValidationError::ContractNotInitialized)?;

        if address != admin {
            return Err(ValidationError::NotAdmin);
        }

        Ok(())
    }

    // ==================== COMBINED VALIDATION EXAMPLES ====================

    /// Example of a function that combines all validation types
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `from` - The sender address
    /// * `to` - The recipient address
    /// * `amount` - The amount to transfer
    /// * `message` - Optional transfer message
    ///
    /// # Errors
    /// Various validation errors depending on the validation that fails
    pub fn validated_transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        message: Option<String>,
    ) -> Result<(), ValidationError> {
        // 1. Parameter validation
        Self::validate_address(from.clone())?;
        Self::validate_address(to.clone())?;
        Self::validate_amount_parameters(amount, 1, 1000000)?;

        if let Some(msg) = &message {
            Self::validate_string_parameters(msg.clone(), 0, 100)?;
        }

        // 2. State validation
        Self::validate_contract_state(&env, ContractState::Active)?;
        Self::validate_balance(&env, from.clone(), amount)?;

        // 3. Authorization validation
        Self::validate_role(&env, from.clone(), UserRole::User)?;
        from.require_auth();

        // 4. Business logic validation (cooldown, rate limiting, etc.)
        Self::validate_cooldown(&env, from.clone(), 60)?; // 1 minute cooldown

        // Execute the transfer
        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_balance + amount));

        // Update last action timestamp
        env.storage()
            .persistent()
            .set(&DataKey::LastAction(from), &env.ledger().timestamp());

        Ok(())
    }

    // ==================== UTILITY FUNCTIONS ====================

    /// Set user role (admin only)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address
    /// * `user` - The user to set role for
    /// * `role` - The role to assign
    ///
    /// # Errors
    /// * `ValidationError::NotAdmin` - If caller is not admin
    /// * `ValidationError::InvalidEnum` - If role is invalid
    pub fn set_user_role(
        env: Env,
        admin: Address,
        user: Address,
        role: UserRole,
    ) -> Result<(), ValidationError> {
        // Validate admin authorization
        Self::validate_admin(&env, admin.clone())?;
        admin.require_auth();

        // Validate user address
        Self::validate_address(user.clone())?;

        // Set the role
        env.storage()
            .instance()
            .set(&DataKey::UserRole(user), &role);

        Ok(())
    }

    /// Pause contract (admin only)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address
    ///
    /// # Errors
    /// * `ValidationError::NotAdmin` - If caller is not admin
    pub fn pause_contract(env: Env, admin: Address) -> Result<(), ValidationError> {
        Self::validate_admin(&env, admin.clone())?;
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Paused);

        Ok(())
    }

    /// Resume contract (admin only)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address
    ///
    /// # Errors
    /// * `ValidationError::NotAdmin` - If caller is not admin
    pub fn resume_contract(env: Env, admin: Address) -> Result<(), ValidationError> {
        Self::validate_admin(&env, admin.clone())?;
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Active);

        Ok(())
    }

    /// Get contract state
    pub fn get_contract_state(env: Env) -> ContractState {
        env.storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(ContractState::Uninitialized)
    }

    /// Get user role
    pub fn get_user_role(env: Env, user: Address) -> UserRole {
        env.storage()
            .instance()
            .get(&DataKey::UserRole(user))
            .unwrap_or(UserRole::None)
    }

    /// Get balance
    pub fn get_balance(env: Env, address: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(address))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;

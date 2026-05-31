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
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};
use soroban_validation::*;

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
        Self::validate_address(owner)?;

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
        // Use shared validation function
        validate_amount(amount, min_amount, max_amount)
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
        // Use shared validation function
        validate_string(text, min_length, max_length)
    }

    /// Example of address parameter validation
    ///
    /// # Arguments
    /// * `address` - The address to validate
    ///
    /// # Errors
    /// * `ValidationError::InvalidAddress` - If address is invalid
    pub fn validate_address(address: Address) -> Result<(), ValidationError> {
        // Use shared validation function (fully qualified to avoid recursive call)
        soroban_validation::validate_address(address)
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
        // Use shared validation function
        validate_array(array, min_size, max_size)
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
        // Use shared validation function
        validate_timestamp(env, timestamp, allow_past, max_future_seconds)
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
            .get(&DataKey::Balance(address))
            .unwrap_or(0);

        // Use shared validation pattern
        require_sufficient_balance(balance, required_amount)
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
            .get(&DataKey::Allowance(owner, spender))
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
            .get::<DataKey, u64>(&DataKey::LastAction(address))
        {
            // Use shared validation pattern
            require_cooldown_expired(env, last_action, cooldown_seconds)
        } else {
            Ok(())
        }
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
        let is_blacklisted = env
            .storage()
            .instance()
            .has(&DataKey::Blacklist(address));
        
        if is_blacklisted {
            return Err(ValidationError::Blacklisted);
        }

        // Get user role
        let user_role: UserRole = env
            .storage()
            .instance()
            .get(&DataKey::UserRole(address))
            .unwrap_or(UserRole::None);

        // Use shared validation pattern for role comparison
        require_role(user_role, required_role)?;

        // Special checks for owner and admin
        match required_role {
            UserRole::Owner if user_role != UserRole::Owner => {
                return Err(ValidationError::NotOwner);
            }
            UserRole::Admin if user_role != UserRole::Admin && user_role != UserRole::Owner => {
                return Err(ValidationError::NotAdmin);
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

        // Use shared validation pattern
        require_owner(owner, address)
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

        // Use shared validation pattern
        require_admin(admin, address)
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
        Self::validate_address(from)?;
        Self::validate_address(to)?;
        Self::validate_amount_parameters(amount, 1, 1000000)?;

        if let Some(msg) = message {
            Self::validate_string_parameters(msg, 0, 100)?;
        }

        // 2. State validation
        Self::validate_contract_state(&env, ContractState::Active)?;
        Self::validate_balance(&env, from, amount)?;

        // 3. Authorization validation
        Self::validate_role(&env, from, UserRole::User)?;
        from.require_auth();

        // 4. Business logic validation (cooldown, rate limiting, etc.)
        Self::validate_cooldown(&env, from, 60)?; // 1 minute cooldown

        // Execute the transfer
        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from))
            .unwrap_or(0);

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_balance - amount));
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
        Self::validate_admin(&env, admin)?;
        admin.require_auth();

        // Validate user address
        Self::validate_address(user)?;

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
        Self::validate_admin(&env, admin)?;
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
        Self::validate_admin(&env, admin)?;
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

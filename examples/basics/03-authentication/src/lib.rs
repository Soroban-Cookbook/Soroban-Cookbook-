#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, Env, Symbol, Vec};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Roles that can be assigned to accounts. The numeric discriminants are used
/// when returning roles as `u32` to callers that cannot decode the enum.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Admin = 0,
    Moderator = 1,
    User = 2,
}

/// Contract-wide operational state. Transitions are admin-only.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractState {
    Active = 0,
    Paused = 1,
    Frozen = 2,
}

/// Storage keys. Instance storage holds contract-wide config; persistent
/// storage holds per-account data that must survive across ledgers.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Role(Address),
    State,
    TimeLock,
    CooldownPeriod,
    LastAction(Address),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// Authentication Patterns Contract
/// 
/// This contract demonstrates various address authentication patterns using Soroban's require_auth() function.
/// 
/// # Context
/// Address authentication is the foundation of authorization in Soroban. The require_auth() function:
/// - Verifies that the caller has authorized the transaction
/// - Prevents unauthorized access to protected functions
/// - Works with both user accounts and contract addresses
/// - Is essential for security in multi-user contracts
#[contract]
pub struct AuthContract;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuthError {
    Unauthorized = 1,
    NotAdmin = 2,
    AlreadyInitialized = 3,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance(Address),
    Allowance(Address, Address),
}

#[contractimpl]
impl AuthContract {
    /// Basic authentication check
    pub fn check_auth(_env: Env, user: Address) -> bool {
        user.require_auth();
        true
    }

    /// Initialize contract with admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), AuthError> {
    /// Single-address authorization pattern
    /// 
    /// Demonstrates how to require authentication from a specific address for operations
    /// like transferring assets or modifying user-specific data.
    /// 
    /// # Parameters
    /// * `env` - The Soroban environment
    /// * `from` - The address initiating the transfer
    /// * `to` - The destination address
    /// * `amount` - The amount to transfer
    /// 
    /// # How authorization is verified:
    /// The `from.require_auth()` call ensures that the `from` address has authorized this transaction.
    /// This prevents someone else from initiating a transfer from another person's account.
    pub fn transfer(_env: Env, from: Address, _to: Address, amount: i128) -> bool {
        // Require authorization from the 'from' address
        // This prevents unauthorized transfers from someone else's account
        from.require_auth();
    
        // Validate inputs
        if amount <= 0 {
            panic!("Amount must be positive");
        }
    
        // Perform the transfer logic here (in a real contract, this would update balances)
        // For demonstration purposes, we just return true
        true
    }

    /// Admin-only function pattern
    /// 
    /// Demonstrates how to restrict function access to a specific admin address.
    /// 
    /// # Parameters
    /// * `env` - The Soroban environment
    /// * `admin` - The address claiming to be admin
    /// * `new_admin` - The address to set as new admin
    /// 
    /// # Security considerations:
    /// - Store the admin address in persistent storage
    /// - Only allow the current admin to change the admin
    /// - Always verify admin permissions before critical operations
    pub fn set_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), AuthError> {
        // First, check if there's already an admin stored
        if let Some(stored_admin) = env.storage().instance().get::<Symbol, Address>(&ADMIN_KEY) {
            // If there's a stored admin, verify that the caller is that admin
            if admin != stored_admin {
                return Err(AuthError::AdminOnly);
            }
            // Require authorization from the current admin
            admin.require_auth();
        } else {
            // If no admin is set yet, anyone can become the initial admin
            // In a real deployment, this would typically be the contract deployer
            admin.require_auth();
        }

        // Set the new admin
        env.storage().instance().set(&ADMIN_KEY, &new_admin);

        Ok(())
    }

    /// Get the current admin address
    /// 
    /// # Parameters
    /// * `env` - The Soroban environment
    /// 
    /// # Returns
    /// The current admin address, if set
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get::<Symbol, Address>(&ADMIN_KEY)
    }

    /// User-specific operations pattern
    /// 
    /// Demonstrates how to perform operations that affect only the authenticated user.
    /// 
    /// # Parameters
    /// * `env` - The Soroban environment
    /// * `user` - The user whose data will be modified
    /// * `data` - The data to store for the user
    /// 
    /// # Pattern:
    /// 1. Require auth from the user who owns the data
    /// 2. Use the authenticated address as a key for user-specific storage
    pub fn update_user_data(env: Env, user: Address, data: Symbol) -> bool {
        // Require authentication from the user
        // This ensures that only the data owner can update their own data
        user.require_auth();

        // Store data keyed by the authenticated user address
        // This creates user-specific storage isolation
        env.storage().persistent().set(&user, &data);

        true
    }

    /// Retrieve user-specific data
    /// 
    /// # Parameters
    /// * `env` - The Soroban environment
    /// * `user` - The user whose data to retrieve
    /// 
    /// # Returns
    /// The data stored for the user, if any
    pub fn get_user_data(env: Env, user: Address) -> Option<Symbol> {
        env.storage().persistent().get(&user)
    }

    /// Function demonstrating proper error handling for auth failures
    /// 
    /// # Parameters
    /// * `env` - The Soroban environment
    /// * `user` - The address that should authorize the transaction
    /// * `operation` - The operation identifier
    /// 
    /// # Returns
    /// Result indicating success or specific error type
    /// 
    /// # Proper error handling:
    /// - Clear error messages when auth fails
    /// - Meaningful error codes for different failure types
    /// - Graceful handling of authorization failures
    pub fn secure_operation(
        env: Env,
        user: Address,
        operation: Symbol,
    ) -> Result<Vec<Symbol>, AuthError> {
        // Require authentication before proceeding
        // If authentication fails, this will panic and the transaction will revert
        user.require_auth();

        // Validate operation is allowed
        if operation == symbol_short!("invalid") {
            return Err(AuthError::Unauthorized);
        }

        // Perform the secure operation
        let result = vec![&env, symbol_short!("success"), operation];

        Ok(result)
    }

    /// Demonstration of self-authorization pattern
    /// 
    /// Shows how a contract can authenticate itself when calling other contracts
    /// 
    /// # Parameters
    /// * `env` - The Soroban environment
    /// * `self_address` - The address of this contract
    /// 
    /// # Self-authorization use case:
    /// When a contract needs to authenticate itself to call other contracts
    /// or when implementing contract-to-contract authorization
    pub fn self_authenticate(_env: Env, self_address: Address) -> bool {
        // The contract authenticates itself
        // This is useful when the contract needs to prove its identity to other contracts
        self_address.require_auth();
    
        // In a real scenario, this would be used to call other contracts
        // or to prove the contract's identity for cross-contract operations
        true
    }

    /// Demonstrates basic address-based authentication.
    /// Only the 'user' can successfully call this function.
    pub fn secure_action(_env: Env, user: Address) {
        // 1. The magic line: checks signature and protects against replays.
        user.require_auth();
    }

    // ==================== INITIALIZATION ====================

    /// Initializes the contract with the given admin address.
    ///
    /// Must be called exactly once. Panics on repeated calls to prevent
    /// admin hijacking after deployment.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AuthError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Admin-only function
    pub fn admin_action(env: Env, admin: Address, value: u32) -> Result<u32, AuthError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AuthError::NotAdmin)?;
        
        if admin != stored_admin {
            return Err(AuthError::NotAdmin);
        }
        
        Ok(value * 2)
    }

    /// Transfer with authentication
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
        from.require_auth();
        
        let from_balance: i128 = env.storage().persistent().get(&DataKey::Balance(from.clone())).unwrap_or(0);
        let to_balance: i128 = env.storage().persistent().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        
        env.storage().persistent().set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage().persistent().set(&DataKey::Balance(to), &(to_balance + amount));
        
        Ok(())
    }

    /// Set balance (admin only)
    pub fn set_balance(env: Env, admin: Address, user: Address, amount: i128) -> Result<(), AuthError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AuthError::NotAdmin)?;
        
        if admin != stored_admin {
            return Err(AuthError::NotAdmin);
        }
        
        env.storage().persistent().set(&DataKey::Balance(user), &amount);
        Ok(())
    }

    /// Get balance
    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Balance(user)).unwrap_or(0)
    }

    /// Approve allowance
    pub fn approve(env: Env, from: Address, spender: Address, amount: i128) -> Result<(), AuthError> {
        from.require_auth();
        env.storage().persistent().set(&DataKey::Allowance(from, spender), &amount);
        Ok(())
    }

    /// Transfer from allowance
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
        spender.require_auth();
        
        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(0);
        
        if allowance < amount {
            return Err(AuthError::Unauthorized);
        }
        
        let from_balance: i128 = env.storage().persistent().get(&DataKey::Balance(from.clone())).unwrap_or(0);
        let to_balance: i128 = env.storage().persistent().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        
        env.storage().persistent().set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage().persistent().set(&DataKey::Balance(to), &(to_balance + amount));
        env.storage().persistent().set(&DataKey::Allowance(from, spender), &(allowance - amount));
        
        Ok(())
    }

    /// Multi-signature operation
    pub fn multi_sig_action(env: Env, signers: Vec<Address>, value: u32) -> u32 {
        for signer in signers.iter() {
            signer.require_auth();
        }
        value + signers.len()
    }

    /// Emit event with authentication
    pub fn emit_event(env: Env, user: Address, message: Symbol) {
        user.require_auth();
        env.events().publish((symbol_short!("event"), user), message);
    }
}

mod test;

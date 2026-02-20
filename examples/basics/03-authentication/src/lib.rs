#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, vec, Address, Env, Symbol, Vec,
};

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

/// Custom error types for authentication-related errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AuthError {
    /// Unauthorized access attempt
    Unauthorized = 1,
    /// Admin-only function called by non-admin
    AdminOnly = 2,
    /// Invalid address provided
    InvalidAddress = 3,
}

const ADMIN_KEY: Symbol = symbol_short!("admin");

#[contractimpl]
impl AuthContract {
    /// Basic function with address authentication
    /// 
    /// Demonstrates the fundamental pattern of requiring authentication before performing actions.
    /// 
    /// # Parameters
    /// * `env` - The Soroban environment
    /// * `user` - The address of the user calling the function
    /// 
    /// # What require_auth does:
    /// The `require_auth()` function verifies that the transaction has been signed by the given address.
    /// If the address hasn't authorized the transaction, the function will panic and the transaction will fail.
    /// 
    /// # When to use it:
    /// Use `require_auth()` whenever you need to verify that the caller has authorized a specific action,
    /// particularly when the action involves transferring assets, changing state, or accessing sensitive data.
    /// 
    /// # Security implications:
    /// Always call `require_auth()` before making any state changes to prevent unauthorized access.
    pub fn basic_auth(_env: Env, user: Address) -> bool {
        // Require authorization from the user address
        // This ensures that the transaction has been signed by the user
        user.require_auth();
    
        // After successful authentication, perform the authorized action
        // In this case, we just return true to indicate successful authentication
        true
    }

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
}

mod test;

#[cfg(test)]
mod smoke_tests {
    use super::*;
    use soroban_sdk::{symbol_short, Address, Env};
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_basic_auth() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AuthContract);
        let client = AuthContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);

        // Mock auth to simulate the user signing the transaction
        env.mock_all_auths();

        // This should succeed
        let result = client.basic_auth(&user);
        assert_eq!(result, true);

        let result = client.transfer(&user, &Address::generate(&env), &100_i128);
        assert_eq!(result, true);
    }

    #[test]
    fn test_admin_function() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AuthContract);
        let client = AuthContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        // Mock auth for the admin
        env.mock_all_auths();
        
        // Set initial admin
        client.set_admin(&admin, &new_admin);
        
        // Verify admin was set
        let stored_admin = client.get_admin();
        assert_eq!(stored_admin, Some(new_admin));
    }

    #[test]
    fn test_user_specific_operations() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AuthContract);
        let client = AuthContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let data = symbol_short!("userdata");

        // Mock auth for the user
        env.mock_all_auths();

        // Update user data
        let result = client.update_user_data(&user, &data);
        assert_eq!(result, true);

        // Retrieve user data
        let retrieved_data = client.get_user_data(&user);
        assert_eq!(retrieved_data, Some(data));
    }

    #[test]
    fn test_secure_operation_success() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AuthContract);
        let client = AuthContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let operation = symbol_short!("valid_op");

        // Mock auth for the user
        env.mock_all_auths();

        // This should succeed - the result is Vec<Symbol> since the client handles the Result
        let _result = client.secure_operation(&user, &operation);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn test_secure_operation_invalid() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AuthContract);
        let client = AuthContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let invalid_operation = symbol_short!("invalid");

        // Mock auth for the user
        env.mock_all_auths();

        // This should panic with Unauthorized error
        client.secure_operation(&user, &invalid_operation);
    }
}
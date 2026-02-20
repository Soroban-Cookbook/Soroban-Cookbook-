//! # Hello World Contract
//!
//! This is the simplest possible Soroban contract that demonstrates:
//! - Basic contract structure
//! - Function definitions
//! - Working with Soroban types (Symbol, Vec)
//! - Returning values
//!
//! The contract has one function that takes a name and returns a greeting.

// Required for all Soroban contracts - prevents standard library from being linked
#![no_std]

// Import core types and macros from the Soroban SDK
use soroban_sdk::{contract, contractimpl, vec, Env, String, Vec};

/// The contract struct. This can be empty or contain state.
/// The #[contract] macro marks this as a Soroban contract.
#[contract]
pub struct HelloContract;

/// Implementation block for the contract.
/// The #[contractimpl] macro makes these functions callable from outside.
#[contractimpl]
impl HelloContract {
    /// Says hello to the provided name.
    ///
    /// # Arguments
    /// * `env` - The contract environment, providing access to blockchain context
    /// * `to` - A Symbol representing the name to greet
    ///
    /// # Returns
    /// A Vec containing two Symbols: ["Hello", name]
    ///
    /// # Example
    /// ```
    /// hello(env, symbol_short!("World"))
    /// // Returns: vec![env, symbol_short!("Hello"), symbol_short!("World")]
    /// ```
    pub fn hello(env: Env, to: String) -> Vec<String> {
        // Create a vector containing "Hello" and the provided name
        // vec! macro creates a Soroban Vec (different from Rust's std::vec!)
        vec![&env, String::from_str(&env, "Hello"), to]
    }
}

// Tests are in a separate module
mod test;

//! # Soroban Types Demonstration
//!
//! This example demonstrates the unique types available in Soroban:
//!
//! - **Address** - Represents account identifiers in Soroban
//! - **Bytes** - Variable-length byte arrays
//! - **BytesN** - Fixed-length byte arrays  
//! - **Symbol** - Short, efficient string-like identifiers
//! - **String** - Standard string handling in Soroban contracts
//!
//! Each type has specific use cases and performance characteristics
//! that make them suitable for different scenarios in smart contracts.

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Symbol, String, Env, symbol_short};

/// Contract demonstrating Soroban-specific types
#[contract]
pub struct SorobanTypesContract;

#[contractimpl]
impl SorobanTypesContract {
    // -----------------------------------------------------------------------
    // Address Type Demonstrations
    // -----------------------------------------------------------------------

    /// Store an address in contract storage
    pub fn store_address(env: Env, owner: Address) {
        env.storage().instance().set(&symbol_short!("owner"), &owner);
    }

    /// Retrieve stored address
    pub fn get_address(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("owner"))
            .unwrap_or_else(|| Address::generate(&env))
    }

    /// Verify address equality
    pub fn verify_address(_env: Env, addr1: Address, addr2: Address) -> bool {
        addr1 == addr2
    }

    /// Generate a new random address
    pub fn generate_address(env: Env) -> Address {
        Address::generate(&env)
    }

    // -----------------------------------------------------------------------
    // Bytes Type Demonstrations
    // -----------------------------------------------------------------------

    /// Store variable-length bytes data
    pub fn store_bytes(env: Env, data: Bytes) {
        env.storage().instance().set(&symbol_short!("bytes_data"), &data);
    }

    /// Retrieve stored bytes
    pub fn get_bytes(env: Env) -> Bytes {
        env.storage()
            .instance()
            .get(&symbol_short!("bytes_data"))
            .unwrap_or_else(|| Bytes::from_slice(&env, b"default"))
    }

    /// Create bytes from slice
    pub fn create_bytes_from_slice(env: Env, input: &str) -> Bytes {
        Bytes::from_slice(&env, input.as_bytes())
    }

    /// Get bytes length
    pub fn get_bytes_length(_env: Env, bytes: Bytes) -> u32 {
        bytes.len()
    }

    // -----------------------------------------------------------------------
    // BytesN Type Demonstrations  
    // -----------------------------------------------------------------------

    /// Store fixed-length bytes (32 bytes - common for hashes)
    pub fn store_fixed_bytes(env: Env, data: BytesN<32>) {
        env.storage().instance().set(&symbol_short!("fixed_bytes"), &data);
    }

    /// Retrieve stored fixed bytes
    pub fn get_fixed_bytes(env: Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&symbol_short!("fixed_bytes"))
            .unwrap_or_else(|| BytesN::from_array(&env, &[0; 32]))
    }

    /// Create BytesN from array (simulating a hash)
    pub fn create_hash_bytes(env: Env, seed: u8) -> BytesN<32> {
        let mut hash = [0u8; 32];
        for i in 0..32 {
            hash[i] = seed.wrapping_mul(i as u8 + 1);
        }
        BytesN::from_array(&env, &hash)
    }

    /// Convert BytesN to Bytes
    pub fn fixed_to_variable_bytes(env: Env, fixed: BytesN<32>) -> Bytes {
        Bytes::from_slice(&env, fixed.to_array().as_slice())
    }

    // -----------------------------------------------------------------------
    // Symbol Type Demonstrations
    // -----------------------------------------------------------------------

    /// Store a Symbol (short, efficient identifier)
    pub fn store_symbol(env: Env, sym: Symbol) {
        env.storage().instance().set(&symbol_short!("symbol"), &sym);
    }

    /// Retrieve stored symbol
    pub fn get_symbol(env: Env) -> Symbol {
        env.storage()
            .instance()
            .get(&symbol_short!("symbol"))
            .unwrap_or(symbol_short!("default"))
    }

    /// Create symbol from string
    pub fn create_symbol(env: Env, text: &str) -> Symbol {
        Symbol::new(&env, text)
    }

    /// Compare symbols for equality
    pub fn compare_symbols(_env: Env, sym1: Symbol, sym2: Symbol) -> bool {
        sym1 == sym2
    }

    /// Get symbol as string for display
    pub fn symbol_to_string(env: Env, sym: Symbol) -> String {
        sym.to_string(&env)
    }

    // -----------------------------------------------------------------------
    // String Type Demonstrations
    // -----------------------------------------------------------------------

    /// Store a String (for longer text)
    pub fn store_string(env: Env, text: String) {
        env.storage().instance().set(&symbol_short!("string"), &text);
    }

    /// Retrieve stored string
    pub fn get_string(env: Env) -> String {
        env.storage()
            .instance()
            .get(&symbol_short!("string"))
            .unwrap_or_else(|| String::from_str(&env, "default"))
    }

    /// Create string from slice
    pub fn create_string(env: Env, text: &str) -> String {
        String::from_str(&env, text)
    }

    /// Get string length
    pub fn get_string_length(_env: Env, text: String) -> u32 {
        text.len()
    }

    /// Concatenate two strings
    pub fn concatenate_strings(env: Env, str1: String, str2: String) -> String {
        let mut result = String::from_str(&env, "");
        result.push_str(&str1);
        result.push_str(&str2);
        result
    }

    /// Convert String to Symbol (if short enough)
    pub fn string_to_symbol(env: Env, text: String) -> Symbol {
        let text_str = text.to_string();
        Symbol::new(&env, &text_str)
    }

    // -----------------------------------------------------------------------
    // Cross-Type Demonstrations
    // -----------------------------------------------------------------------

    /// Demonstrate type conversion and interoperability
    pub fn type_conversion_demo(env: Env) {
        // String to Symbol conversion
        let short_text = String::from_str(&env, "token");
        let text_str = short_text.to_string();
        let symbol = Symbol::new(&env, &text_str);
        
        env.storage().instance().set(&symbol_short!("symbol_from_string"), &symbol);
        env.storage().instance().set(&symbol_short!("original_string"), &short_text);

        // BytesN to Bytes conversion
        let hash = BytesN::from_array(&env, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
        let variable_bytes = Bytes::from_slice(&env, hash.to_array().as_slice());
        env.storage().instance().set(&symbol_short!("hash_bytes"), &hash);
        env.storage().instance().set(&symbol_short!("variable_bytes"), &variable_bytes);
    }

    /// Complex data structure using multiple types
    pub fn create_user_profile(env: Env, user: Address, username: String, bio: String, avatar_hash: BytesN<32>) -> u32 {
        // Store user information using appropriate types
        env.storage().instance().set(&symbol_short!("user_addr"), &user);
        env.storage().instance().set(&symbol_short!("username"), &username);
        env.storage().instance().set(&symbol_short!("bio"), &bio);
        env.storage().instance().set(&symbol_short!("avatar"), &avatar_hash);
        
        // Create a user status symbol
        let status = Symbol::new(&env, "active");
        env.storage().instance().set(&symbol_short!("user_status"), &status);
        
        // Return profile completion score
        username.len() + bio.len()
    }

    /// Type validation examples
    pub fn validate_types(env: Env, addr: Address, sym: Symbol, text: String) -> bool {
        // Validate symbol is not empty
        let sym_str = sym.to_string(&env);
        if sym_str.len() == 0 {
            return false;
        }
        
        // Validate string is reasonable length
        if text.len() > 1000 {
            return false;
        }
        
        true
    }
}

mod test;

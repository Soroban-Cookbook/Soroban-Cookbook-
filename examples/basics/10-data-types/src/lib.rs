//! # Data Types Example
//!
//! This contract demonstrates Soroban's comprehensive type system and how to work
//! with each type effectively. It covers:
//!
//! - **Primitive types**: integers (u32, u64, i128)
//! - **Text types**: Symbol (short identifiers) and String (longer text)
//! - **Binary types**: Bytes (variable-length) and BytesN (fixed-length)
//! - **Address type**: for account and contract identifiers
//! - **Collection types**: Vec (ordered lists) and Map (key-value pairs)
//!
//! Each type is optimized for specific use cases and has different gas costs.
//! Understanding when to use each type is crucial for writing efficient contracts.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, symbol_short, vec, Address, Bytes, BytesN, Env, Map, String, Symbol,
    Vec,
};

// Import testutils for Address::generate in tests
#[cfg(test)]
use soroban_sdk::testutils::Address as _;

/// The contract type for demonstrating Soroban data types.
#[contract]
pub struct DataTypesContract;

/// Public interface for the data types contract.
#[contractimpl]
impl DataTypesContract {
    // ============================================================================
    // PRIMITIVE TYPES (Integers)
    // ============================================================================

    /// Store a 32-bit unsigned integer.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `value` - a u32 value to store
    pub fn store_u32(_env: Env, value: u32) -> u32 {
        // In a real contract, this would be stored in persistent storage
        value
    }

    /// Store a 64-bit unsigned integer.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `value` - a u64 value to store
    pub fn store_u64(_env: Env, value: u64) -> u64 {
        value
    }

    /// Store a 128-bit signed integer (most common for financial values).
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `value` - an i128 value to store
    pub fn store_i128(_env: Env, value: i128) -> i128 {
        value
    }

    /// Demonstrate safe arithmetic with overflow checking.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `a` - first i128 value
    /// * `b` - second i128 value
    ///
    /// # Returns
    /// The sum of a and b, or panics on overflow
    pub fn safe_add(_env: Env, a: i128, b: i128) -> i128 {
        // Soroban's release profile has overflow-checks enabled
        a.checked_add(b).unwrap_or_else(|| {
            panic!("Arithmetic overflow");
        })
    }

    // ============================================================================
    // TEXT TYPES (Symbol vs String)
    // ============================================================================

    /// Store a Symbol (short, gas-efficient identifier).
    ///
    /// Symbols are ideal for:
    /// - Token symbols (e.g., "USDC", "ETH")
    /// - Enum-like values (e.g., "active", "paused")
    /// - Short identifiers (≤9 characters recommended)
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `symbol` - a Symbol to store
    pub fn store_symbol(_env: Env, symbol: Symbol) -> Symbol {
        symbol
    }

    /// Store a String (variable-length text).
    ///
    /// Strings are ideal for:
    /// - Human-readable messages
    /// - Longer text content
    /// - User-provided descriptions
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `text` - a String to store
    pub fn store_string(_env: Env, text: String) -> String {
        text
    }

    /// Demonstrate Symbol creation from a string literal.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    ///
    /// # Returns
    /// A Symbol representing "token"
    pub fn create_symbol(_env: Env) -> Symbol {
        symbol_short!("token")
    }

    /// Demonstrate String creation from a string literal.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    ///
    /// # Returns
    /// A String with a message
    pub fn create_string(env: Env) -> String {
        String::from_str(&env, "Hello, Soroban!")
    }

    // ============================================================================
    // BINARY TYPES (Bytes vs BytesN)
    // ============================================================================

    /// Store variable-length binary data.
    ///
    /// Bytes are ideal for:
    /// - Arbitrary binary data
    /// - Serialized objects
    /// - Data of unknown length
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `data` - binary data to store
    pub fn store_bytes(_env: Env, data: Bytes) -> Bytes {
        data
    }

    /// Store fixed-length binary data (32 bytes, typical for hashes).
    ///
    /// BytesN is ideal for:
    /// - Cryptographic hashes (SHA-256 = 32 bytes)
    /// - Address hashes (20 bytes)
    /// - Fixed-size identifiers
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `hash` - a 32-byte hash to store
    pub fn store_bytes32(_env: Env, hash: BytesN<32>) -> BytesN<32> {
        hash
    }

    /// Create a BytesN<32> from a slice (for testing).
    ///
    /// # Arguments
    /// * `env` - the execution environment
    ///
    /// # Returns
    /// A BytesN<32> filled with zeros
    pub fn create_bytes32(env: Env) -> BytesN<32> {
        BytesN::<32>::from_array(&env, &[0u8; 32])
    }

    // ============================================================================
    // ADDRESS TYPE
    // ============================================================================

    /// Store an Address (account or contract identifier).
    ///
    /// Addresses are essential for:
    /// - Identifying users and contracts
    /// - Authorization and access control
    /// - Cross-contract calls
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `addr` - an Address to store
    pub fn store_address(_env: Env, addr: Address) -> Address {
        addr
    }

    /// Get the current contract's address.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    ///
    /// # Returns
    /// The address of this contract
    pub fn get_contract_address(env: Env) -> Address {
        env.current_contract_address()
    }

    /// Verify if two addresses are equal.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `addr1` - first address
    /// * `addr2` - second address
    ///
    /// # Returns
    /// true if addresses are equal, false otherwise
    pub fn addresses_equal(_env: Env, addr1: Address, addr2: Address) -> bool {
        addr1 == addr2
    }

    // ============================================================================
    // COLLECTION TYPES (Vec and Map)
    // ============================================================================

    /// Store a vector of integers.
    ///
    /// Vec is ideal for:
    /// - Ordered lists of values
    /// - Dynamic-size collections
    /// - Sequences of data
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `values` - a Vec of i128 values
    pub fn store_vec(_env: Env, values: Vec<i128>) -> Vec<i128> {
        values
    }

    /// Create a vector with sample data.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    ///
    /// # Returns
    /// A Vec containing [1, 2, 3]
    pub fn create_vec(env: Env) -> Vec<i128> {
        vec![&env, 1, 2, 3]
    }

    /// Get the length of a vector.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `values` - the vector to measure
    ///
    /// # Returns
    /// The number of elements in the vector
    pub fn vec_length(_env: Env, values: Vec<i128>) -> u32 {
        values.len()
    }

    /// Get an element from a vector by index.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `values` - the vector to access
    /// * `index` - the index to retrieve
    ///
    /// # Returns
    /// The value at the given index, or panics if out of bounds
    pub fn vec_get(_env: Env, values: Vec<i128>, index: u32) -> i128 {
        values.get(index).unwrap_or_else(|| {
            panic!("Index out of bounds");
        })
    }

    /// Store a map of symbol keys to integer values.
    ///
    /// Map is ideal for:
    /// - Key-value associations
    /// - Fast lookups
    /// - Metadata storage
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `data` - a Map with Symbol keys and i128 values
    pub fn store_map(_env: Env, data: Map<Symbol, i128>) -> Map<Symbol, i128> {
        data
    }

    /// Create a map with sample data.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    ///
    /// # Returns
    /// A Map with entries: {"count" -> 42, "balance" -> 1000}
    pub fn create_map(env: Env) -> Map<Symbol, i128> {
        let mut map = Map::new(&env);
        map.set(symbol_short!("count"), 42);
        map.set(symbol_short!("balance"), 1000);
        map
    }

    /// Get a value from a map by key.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `data` - the map to access
    /// * `key` - the key to look up
    ///
    /// # Returns
    /// The value associated with the key, or panics if not found
    pub fn map_get(_env: Env, data: Map<Symbol, i128>, key: Symbol) -> i128 {
        data.get(key).unwrap_or_else(|| {
            panic!("Key not found in map");
        })
    }

    // ============================================================================
    // TYPE CONVERSIONS
    // ============================================================================

    /// Convert BytesN to Bytes.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `fixed` - a BytesN<32> to convert
    ///
    /// # Returns
    /// A Bytes object containing the same data
    pub fn bytesn_to_bytes(env: Env, fixed: BytesN<32>) -> Bytes {
        Bytes::from_slice(&env, fixed.to_array().as_slice())
    }

    /// Convert Bytes to BytesN<32> (panics if wrong size).
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `data` - a Bytes object to convert
    ///
    /// # Returns
    /// A BytesN<32> if the input is exactly 32 bytes
    pub fn bytes_to_bytesn(env: Env, data: Bytes) -> BytesN<32> {
        // Create a 32-byte array from the Bytes
        let mut array = [0u8; 32];
        if data.len() as usize != 32 {
            panic!("Bytes must be exactly 32 bytes");
        }

        // Copy bytes into the array
        for (i, byte) in array.iter_mut().enumerate() {
            *byte = data.get(i as u32).unwrap();
        }

        BytesN::<32>::from_array(&env, &array)
    }

    /// Demonstrate Symbol and String creation (simplified conversions).
    ///
    /// # Arguments
    /// * `env` - the execution environment
    ///
    /// # Returns
    /// A Symbol created from a literal
    pub fn create_symbol_from_literal(env: Env) -> Symbol {
        Symbol::new(&env, "token")
    }

    /// Demonstrate String creation from a literal.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    ///
    /// # Returns
    /// A String created from a literal
    pub fn create_string_from_literal(env: Env) -> String {
        String::from_str(&env, "Hello, Soroban!")
    }

    // ============================================================================
    // STORAGE ROUND-TRIPS
    // ============================================================================

    /// Persist a u32 value and read it back.
    pub fn put_u32(env: Env, value: u32) {
        env.storage().instance().set(&symbol_short!("u32"), &value);
    }

    pub fn get_u32(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&symbol_short!("u32"))
            .unwrap_or(0)
    }

    /// Persist an i128 value and read it back.
    pub fn put_i128(env: Env, value: i128) {
        env.storage().instance().set(&symbol_short!("i128"), &value);
    }

    pub fn get_i128(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&symbol_short!("i128"))
            .unwrap_or(0)
    }

    /// Persist a Symbol and read it back.
    pub fn put_symbol(env: Env, value: Symbol) {
        env.storage().instance().set(&symbol_short!("sym"), &value);
    }

    pub fn get_symbol(env: Env) -> Symbol {
        env.storage()
            .instance()
            .get(&symbol_short!("sym"))
            .unwrap_or_else(|| symbol_short!("none"))
    }

    /// Persist a Vec<i128> and read it back.
    pub fn put_vec(env: Env, value: Vec<i128>) {
        env.storage().instance().set(&symbol_short!("vec"), &value);
    }

    pub fn get_vec(env: Env) -> Vec<i128> {
        env.storage()
            .instance()
            .get(&symbol_short!("vec"))
            .unwrap_or_else(|| vec![&env])
    }
}

// Pull in the test module
#[cfg(test)]
mod test;

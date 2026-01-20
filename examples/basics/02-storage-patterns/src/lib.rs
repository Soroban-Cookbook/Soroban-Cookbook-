//! # Storage Patterns Contract
//!
//! Demonstrates the three types of storage available in Soroban:
//! - Persistent: Data that lives permanently (requires TTL management)
//! - Temporary: Data that only exists for the current ledger
//! - Instance: Data tied to the contract instance lifetime
//!
//! Each storage type has different cost and lifetime characteristics.

#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Symbol};

/// Storage contract demonstrating all three storage types
#[contract]
pub struct StorageContract;

#[contractimpl]
impl StorageContract {
    // ==================== PERSISTENT STORAGE ====================

    /// Stores a value in persistent storage.
    /// Persistent data remains until explicitly deleted and requires TTL extension.
    ///
    /// # Arguments
    /// * `key` - The storage key
    /// * `value` - The value to store
    ///
    /// # Cost
    /// Higher write cost, requires rent (TTL management)
    pub fn set_persistent(env: Env, key: Symbol, value: u64) {
        // Store in persistent storage
        env.storage().persistent().set(&key, &value);

        // Extend TTL to keep data alive
        // Parameters: (key, threshold_ledgers, extend_to_ledgers)
        // This extends TTL to 100 ledgers when it falls below 100
        env.storage().persistent().extend_ttl(&key, 100, 100);
    }

    /// Retrieves a value from persistent storage.
    ///
    /// # Returns
    /// The stored value, or panics if key doesn't exist
    pub fn get_persistent(env: Env, key: Symbol) -> u64 {
        env.storage().persistent().get(&key).unwrap()
    }

    /// Checks if a key exists in persistent storage.
    pub fn has_persistent(env: Env, key: Symbol) -> bool {
        env.storage().persistent().has(&key)
    }

    /// Removes a value from persistent storage.
    pub fn remove_persistent(env: Env, key: Symbol) {
        env.storage().persistent().remove(&key);
    }

    // ==================== TEMPORARY STORAGE ====================

    /// Stores a value in temporary storage.
    /// Temporary data only exists for the current ledger - cheapest option.
    ///
    /// # Arguments
    /// * `key` - The storage key
    /// * `value` - The value to store
    ///
    /// # Cost
    /// Lowest cost, no rent required
    ///
    /// # Use Cases
    /// - Intermediate calculations
    /// - Transaction-scoped flags
    /// - Temporary state within a single operation
    pub fn set_temporary(env: Env, key: Symbol, value: u64) {
        env.storage().temporary().set(&key, &value);
    }

    /// Retrieves a value from temporary storage.
    ///
    /// # Returns
    /// The stored value, or panics if key doesn't exist
    pub fn get_temporary(env: Env, key: Symbol) -> u64 {
        env.storage().temporary().get(&key).unwrap()
    }

    /// Checks if a key exists in temporary storage.
    pub fn has_temporary(env: Env, key: Symbol) -> bool {
        env.storage().temporary().has(&key)
    }

    // ==================== INSTANCE STORAGE ====================

    /// Stores a value in instance storage.
    /// Instance data lives as long as the contract instance exists.
    ///
    /// # Arguments
    /// * `key` - The storage key
    /// * `value` - The value to store
    ///
    /// # Cost
    /// Medium cost, requires rent (but cheaper than persistent)
    ///
    /// # Use Cases
    /// - Contract configuration
    /// - Admin addresses
    /// - Contract metadata
    pub fn set_instance(env: Env, key: Symbol, value: u64) {
        env.storage().instance().set(&key, &value);

        // Extend instance storage TTL
        env.storage().instance().extend_ttl(100, 100);
    }

    /// Retrieves a value from instance storage.
    ///
    /// # Returns
    /// The stored value, or panics if key doesn't exist
    pub fn get_instance(env: Env, key: Symbol) -> u64 {
        env.storage().instance().get(&key).unwrap()
    }

    /// Checks if a key exists in instance storage.
    pub fn has_instance(env: Env, key: Symbol) -> bool {
        env.storage().instance().has(&key)
    }

    /// Removes a value from instance storage.
    pub fn remove_instance(env: Env, key: Symbol) {
        env.storage().instance().remove(&key);
    }
}

mod test;

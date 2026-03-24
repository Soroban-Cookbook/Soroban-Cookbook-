//! # Storage Patterns Contract
//!
//! Demonstrates the three types of storage available in Soroban:
//! - Persistent: Data that lives permanently (requires TTL management)
//! - Temporary: Data that only exists for the current ledger
//! - Instance: Data tied to the contract instance lifetime
//!
//! Each storage type has different cost and lifetime characteristics.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Persistent(Symbol),
    Temporary(Symbol),
    Instance(Symbol),
}

/// Storage contract demonstrating all three storage types
#[contract]
pub struct StorageContract;

#[contractimpl]
impl StorageContract {
    // ==================== PERSISTENT STORAGE ====================

    /// Stores a value in persistent storage.
    /// Persistent data remains until explicitly deleted and requires TTL extension.
    pub fn set_persistent(env: Env, key: Symbol, value: u64) {
        let storage_key = DataKey::Persistent(key.clone());
        // Store in persistent storage
        env.storage().persistent().set(&storage_key, &value);

<<<<<<< HEAD
        // Extend TTL to keep data alive
        // Parameters: (key, threshold_ledgers, extend_to_ledgers)
        // This extends TTL to 100 ledgers when it falls below 100
        env.storage().persistent().extend_ttl(&key, 100, 100);

=======
        // Temporarily disabled for debugging
        // env.storage().persistent().extend_ttl(&storage_key, 1000, 10000);
        
>>>>>>> 0fee596 (new storage patterns)
        // EVENT: Persistent storage updated
        env.events().publish(
            (symbol_short!("persist"), symbol_short!("set")),
            (key, value),
        );
    }

    /// Retrieves a value from persistent storage.
    pub fn get_persistent(env: Env, key: Symbol) -> Option<u64> {
        env.storage().persistent().get(&DataKey::Persistent(key))
    }

    /// Checks if a key exists in persistent storage.
    pub fn has_persistent(env: Env, key: Symbol) -> bool {
        env.storage().persistent().has(&DataKey::Persistent(key))
    }

    /// Removes a value from persistent storage.
    pub fn remove_persistent(env: Env, key: Symbol) {
        env.storage().persistent().remove(&key);

        // EVENT: Persistent storage removed
        env.events()
            .publish((symbol_short!("persist"), symbol_short!("remove")), key);
    }

    // ==================== TEMPORARY STORAGE ====================

    /// Stores a value in temporary storage.
    pub fn set_temporary(env: Env, key: Symbol, value: u64) {
        env.storage().temporary().set(&key, &value);

        // EVENT: Temporary storage updated
        env.events()
            .publish((symbol_short!("temp"), symbol_short!("set")), (key, value));
    }

    /// Retrieves a value from temporary storage.
    pub fn get_temporary(env: Env, key: Symbol) -> Option<u64> {
        env.storage().temporary().get(&DataKey::Temporary(key))
    }

    /// Checks if a key exists in temporary storage.
    pub fn has_temporary(env: Env, key: Symbol) -> bool {
        env.storage().temporary().has(&DataKey::Temporary(key))
    }

    // ==================== INSTANCE STORAGE ====================

    /// Stores a value in instance storage.
    pub fn set_instance(env: Env, key: Symbol, value: u64) {
        let storage_key = DataKey::Instance(key.clone());
        env.storage().instance().set(&storage_key, &value);

<<<<<<< HEAD
        // Extend instance storage TTL
        env.storage().instance().extend_ttl(100, 100);

=======
        // Temporarily disabled for debugging
        // env.storage().instance().extend_ttl(1000, 10000);
        
>>>>>>> 0fee596 (new storage patterns)
        // EVENT: Instance storage updated
        env.events().publish(
            (symbol_short!("instance"), symbol_short!("set")),
            (key, value),
        );
    }

    /// Retrieves a value from instance storage.
    pub fn get_instance(env: Env, key: Symbol) -> Option<u64> {
        env.storage().instance().get(&DataKey::Instance(key))
    }

    /// Checks if a key exists in instance storage.
    pub fn has_instance(env: Env, key: Symbol) -> bool {
        env.storage().instance().has(&DataKey::Instance(key))
    }

    /// Removes a value from instance storage.
    pub fn remove_instance(env: Env, key: Symbol) {
        env.storage().instance().remove(&key);

        // EVENT: Instance storage removed
        env.events()
            .publish((symbol_short!("instance"), symbol_short!("remove")), key);
    }
}

#[cfg(test)]
mod test;

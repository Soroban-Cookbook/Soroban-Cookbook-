#![no_std]
//! # Persistent Storage Pattern
//!
//! This contract demonstrates Soroban's persistent storage type — the most durable
//! storage tier for data that must survive indefinitely.
//!
//! ## Key Features
//! - Per-key TTL management for independent data lifecycles
//! - Type-safe storage keys using enums
//! - Proper TTL extension to prevent archival
//! - Safe arithmetic with overflow protection
//!
//! ## Storage Keys
//! We use a typed enum to prevent key collisions and enable compile-time checks.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

/// Storage keys for persistent data.
///
/// Using an enum provides type safety and prevents typos in key names.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The contract administrator address
    Admin,
    /// A counter that increments with each call
    Counter,
}

/// Persistent storage demonstration contract.
///
/// This contract shows how to use persistent storage for data that must
/// survive long-term with independent TTL management per key.
#[contract]
pub struct PersistentStorageContract;

#[contractimpl]
impl PersistentStorageContract {
    /// Sets the contract administrator address.
    ///
    /// # Arguments
    /// * `address` - The address to set as admin
    ///
    /// # Storage
    /// - Uses persistent storage for durability
    /// - Extends TTL to 10,000 ledgers (~14 hours on mainnet)
    /// - Only extends if current TTL < 2,000 ledgers
    ///
    /// # Example
    /// ```ignore
    /// client.set_admin(&admin_address);
    /// ```
    pub fn set_admin(env: Env, address: Address) {
        let key = DataKey::Admin;
        env.storage().persistent().set(&key, &address);
        // Extend TTL: only if remaining < 2000 ledgers, extend to 10000
        env.storage().persistent().extend_ttl(&key, 2000, 10000);
    }

    /// Retrieves the current administrator address.
    ///
    /// # Returns
    /// `Some(Address)` if admin is set, `None` otherwise
    ///
    /// # Example
    /// ```ignore
    /// if let Some(admin) = client.get_admin() {
    ///     // admin exists
    /// }
    /// ```
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Admin)
    }

    /// Increments the counter and returns the new value.
    ///
    /// # Returns
    /// The new counter value after incrementing
    ///
    /// # Panics
    /// Panics if the counter would overflow u64::MAX
    ///
    /// # Storage
    /// - Reads current value (defaults to 0 if not set)
    /// - Increments with overflow protection
    /// - Writes new value to persistent storage
    /// - Extends TTL to keep data alive
    ///
    /// # Example
    /// ```ignore
    /// let count = client.increment(); // Returns 1
    /// let count = client.increment(); // Returns 2
    /// ```
    pub fn increment(env: Env) -> u64 {
        let key = DataKey::Counter;
        let mut count: u64 = env.storage().persistent().get(&key).unwrap_or(0);

        // Use checked_add to prevent overflow
        count = count.checked_add(1).expect("counter overflow");

        env.storage().persistent().set(&key, &count);
        env.storage().persistent().extend_ttl(&key, 2000, 10000);

        count
    }

    /// Retrieves the current counter value without modifying it.
    ///
    /// # Returns
    /// The current counter value, or 0 if not yet initialized
    ///
    /// # Example
    /// ```ignore
    /// let current = client.get_counter();
    /// ```
    pub fn get_counter(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Counter)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;

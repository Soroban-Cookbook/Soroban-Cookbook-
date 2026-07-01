//! # Implementation Contract — Version 1
//!
//! Provides basic arithmetic (`add`, `sub`) and a simple persistent counter.
//! This is the *initial* implementation registered with the Beacon.
//!
//! ## Storage
//! | Key | Type | Description |
//! |---|---|---|
//! | `"count"` | `u32` | Monotonically increasing counter |

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNT_KEY: Symbol = symbol_short!("count");

/// Version 1 implementation: addition, subtraction, and a counter.
#[contract]
pub struct ImplV1;

#[contractimpl]
impl ImplV1 {
    /// Add two numbers with overflow protection.
    ///
    /// # Arguments
    /// * `a` - first addend
    /// * `b` - second addend
    ///
    /// # Returns
    /// `a + b`
    pub fn add(_env: Env, a: i128, b: i128) -> i128 {
        a.checked_add(b).unwrap_or_else(|| panic!("Overflow"))
    }

    /// Subtract two numbers with underflow protection.
    ///
    /// # Arguments
    /// * `a` - minuend
    /// * `b` - subtrahend
    ///
    /// # Returns
    /// `a - b`
    pub fn sub(_env: Env, a: i128, b: i128) -> i128 {
        a.checked_sub(b).unwrap_or_else(|| panic!("Underflow"))
    }

    /// Increment the persistent counter and return the new value.
    ///
    /// # Returns
    /// New counter value after increment.
    pub fn increment(env: Env) -> u32 {
        let current: u32 = env.storage().persistent().get(&COUNT_KEY).unwrap_or(0);
        let next = current.checked_add(1).unwrap_or_else(|| panic!("Counter overflow"));
        env.storage().persistent().set(&COUNT_KEY, &next);
        next
    }

    /// Return the current counter value without modifying state.
    pub fn get_count(env: Env) -> u32 {
        env.storage().persistent().get(&COUNT_KEY).unwrap_or(0)
    }

    /// Return the version number for this implementation.
    pub fn version(_env: Env) -> u32 {
        1
    }
}

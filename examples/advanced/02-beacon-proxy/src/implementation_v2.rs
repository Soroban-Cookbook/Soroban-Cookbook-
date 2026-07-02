//! # Implementation Contract — Version 2
//!
//! Extends v1 with multiplication (`mul`) and a `reset` function that clears the
//! counter. This contract is used to demonstrate **upgrade propagation** in tests:
//! after the Beacon is upgraded from v1 → v2, all proxies immediately gain access
//! to `mul` and `reset` without any per-proxy state change.
//!
//! ## Storage
//! | Key | Type | Description |
//! |---|---|---|
//! | `"count"` | `u32` | Monotonically increasing counter (same key as v1 for storage continuity) |

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNT_KEY: Symbol = symbol_short!("count");

/// Version 2 implementation: adds multiplication and counter reset to v1's surface.
#[contract]
pub struct ImplV2;

#[contractimpl]
impl ImplV2 {
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

    /// Multiply two numbers with overflow protection.
    ///
    /// *New in v2.* Not available through a proxy pointing at v1.
    ///
    /// # Arguments
    /// * `a` - first factor
    /// * `b` - second factor
    ///
    /// # Returns
    /// `a * b`
    pub fn mul(_env: Env, a: i128, b: i128) -> i128 {
        a.checked_mul(b).unwrap_or_else(|| panic!("Overflow"))
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

    /// Reset the counter to zero.
    ///
    /// *New in v2.* Demonstrates that upgraded implementations can expose
    /// operations that were impossible in earlier versions.
    pub fn reset(env: Env) {
        env.storage().persistent().set(&COUNT_KEY, &0u32);
    }

    /// Return the version string for this implementation.
    pub fn version(_env: Env) -> u32 {
        2
    }
}

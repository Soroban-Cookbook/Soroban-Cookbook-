//! # Implementation Contract (v2)
//!
//! This is an upgraded implementation contract that adds multiplication.

#![no_std]

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct ImplementationV2;

#[contractimpl]
impl ImplementationV2 {
    /// Add two numbers.
    ///
    /// # Arguments
    /// * `a` - first number
    /// * `b` - second number
    ///
    /// # Returns
    /// The sum of a and b
    pub fn add(a: i128, b: i128) -> i128 {
        a.checked_add(b).unwrap_or_else(|| panic!("Overflow"))
    }

    /// Subtract two numbers.
    ///
    /// # Arguments
    /// * `a` - the minuend
    /// * `b` - the subtrahend
    ///
    /// # Returns
    /// The difference (a - b)
    pub fn sub(a: i128, b: i128) -> i128 {
        a.checked_sub(b).unwrap_or_else(|| panic!("Underflow"))
    }

    /// Multiply two numbers (new in v2).
    ///
    /// # Arguments
    /// * `a` - first number
    /// * `b` - second number
    ///
    /// # Returns
    /// The product of a and b
    pub fn mul(a: i128, b: i128) -> i128 {
        a.checked_mul(b).unwrap_or_else(|| panic!("Overflow"))
    }
}

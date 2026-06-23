//! # Implementation Contract (v1)
//!
//! This is the initial implementation contract that provides basic arithmetic operations.

#![no_std]

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct ImplementationV1;

#[contractimpl]
impl ImplementationV1 {
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
}

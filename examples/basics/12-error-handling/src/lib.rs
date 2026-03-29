//! # Error Handling Contract
//!
//! This example demonstrates foundational error handling patterns in Soroban,
//! focusing on the `Result<T, E>` pattern for recoverable errors and `panic!` 
//! for irrecoverable invariants.
//!
//! Key concepts:
//! - Defining custom error enums with `#[contracterror]`
//! - Assigning explicit `u32` codes to error variants
//! - Graceful error propagation using `Result`
//! - Intentional panics for internal state violations

#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, Env};

/// Custom error enum for the contract.
/// Each variant must have an explicit u32 representation for Soroban's error system.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Input cannot be zero (Error Code: 1)
    ZeroInput = 1,
    /// Calculation resulted in an overflow (Error Code: 2)
    Overflow = 2,
    /// Unauthorized access (Error Code: 3)
    Unauthorized = 3,
}

#[contract]
pub struct ErrorHandlingContract;

#[contractimpl]
impl ErrorHandlingContract {
    /// Demonstrates graceful error handling with `Result`.
    /// 
    /// Returns `Ok(a / b)` if `b != 0`, otherwise returns `Err(Error::ZeroInput)`.
    pub fn divide(a: i128, b: i128) -> Result<i128, Error> {
        if b == 0 {
            return Err(Error::ZeroInput);
        }
        Ok(a / b)
    }

    /// Demonstrates input validation with error propagation.
    pub fn check_positive(value: i128) -> Result<(), Error> {
        if value <= 0 {
            return Err(Error::ZeroInput);
        }
        Ok(())
    }

    /// Demonstrates an irrecoverable panic for an internal invariant violation.
    ///
    /// In Soroban, `panic!` should be reserved for states that should be impossible 
    /// if the contract is functioning correctly.
    pub fn invariant_check(env: Env, value: u32) {
        // Assume some internal state is retrieved
        // let state = env.storage().instance().get(&key).unwrap_or(0);
        
        // If state is corrupted, we panic as it's not a user error
        if value > 100 {
            panic!("internal invariant violated: state exceeds maximum allowed value");
        }
    }
}

#[cfg(test)]
mod test;

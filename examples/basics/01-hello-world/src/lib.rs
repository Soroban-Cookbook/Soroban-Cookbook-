//! # Hello World Soroban Contract
//!
//! This is the simplest possible Soroban smart contract. It demonstrates the
//! fundamental building blocks every Soroban developer needs to understand:
//!
//! - How to define a contract struct with `#[contract]`
//! - How to expose contract functions with `#[contractimpl]`
//! - How to use the `Env` parameter to access the blockchain environment
//! - How to work with Soroban SDK types (`Symbol`, `Vec`)
//!
//! ## Why `Vec<Symbol>` instead of a formatted `String`?
//!
//! `soroban_sdk::String` is an immutable host object -- there is no `format!`,
//! no `push_str`, and no string concatenation available in the `no_std` Wasm
//! sandbox.  Returning a `Vec<Symbol>` (the approach used by the official
//! Soroban examples) is idiomatic: it is cheap, composable, and easy for
//! frontends to decode.

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};

/// The contract type.
///
/// Soroban contracts are plain unit structs tagged with `#[contract]`. The
/// macro registers the type with the host so that invocations are routed to
/// the `#[contractimpl]` block below.
#[contract]
pub struct HelloContract;

/// Public interface of `HelloContract`.
#[contractimpl]
impl HelloContract {
    /// Return a greeting vector for the given name.
    ///
    /// # Arguments
    ///
    /// * `env` - the execution environment, provided automatically by the host.
    /// * `to`  - the name to greet as a `Symbol`. `Symbol` is the most
    ///           gas-efficient way to pass short identifiers across the
    ///           host-guest boundary.
    ///
    /// # Returns
    ///
    /// `Vec<Symbol>` of the form `["Hello", <to>]`.
    ///
    /// # Example
    ///
    /// ```text
    /// hello(env, symbol_short!("World")) -> ["Hello", "World"]
    /// ```
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        // `vec!` is the Soroban macro equivalent of the std `vec![]` macro.
        // It allocates the vector in host memory and is the idiomatic way to
        // return multiple values from a contract function.
        vec![&env, symbol_short!("Hello"), to]
    }
}

// Pull in the dedicated test module.
#[cfg(test)]
mod test;

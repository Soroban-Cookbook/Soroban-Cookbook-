#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};

/// The Hello World contract.
///
/// This is a minimal, beginner-friendly smart contract example for Soroban.
/// It demonstrates how to receive input, construct a vector, and return it.
#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    /// Returns a vector containing the symbol "Hello" and the provided `to` symbol.
    ///
    /// # Arguments
    ///
    /// * `env` - The execution environment provided by the Soroban host.
    /// * `to` - A short string (Symbol) representing the entity to greet.
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        // Create a new vector using the `vec!` macro provided by the Soroban SDK.
        // We use `symbol_short!` for "Hello" because it's a compile-time string of up to 9 characters.
        // The `to` parameter is already a Symbol, so we pass it directly.
        vec![&env, symbol_short!("Hello"), to]
    }
}

// Include the tests module only when running tests.
#[cfg(test)]
mod test;

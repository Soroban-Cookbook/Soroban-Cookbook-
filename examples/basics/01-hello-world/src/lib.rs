//! # Hello World Soroban Contract
//!
//! This is the simplest possible Soroban smart contract. It demonstrates the
//! fundamental building blocks every Soroban developer needs to understand:
//!
//! - How to define a contract struct with `#[contract]`
//! - How to expose contract functions with `#[contractimpl]`
//! - How to use the `Env` parameter to access the blockchain environment
//! - How to work with Soroban SDK types (`Symbol`, `String`)
//! - How to perform `no_std`-safe string manipulation on-chain

// Soroban contracts must be `no_std` – they run inside the Wasm sandbox and
// have no access to the Rust standard library.
#![no_std]

// Import core types and macros from the Soroban SDK
use soroban_sdk::{contract, contractimpl, vec, Env, String, Vec};

/// The contract type.
///
/// Soroban contracts are plain unit structs tagged with `#[contract]`.  The
/// macro registers the type with the host so that invocations are routed to
/// the `#[contractimpl]` block below.
#[contract]
pub struct HelloContract;

/// Public interface of `HelloContract`.
#[contractimpl]
impl HelloContract {
    /// Return a greeting for the given name.
    ///
    /// # Arguments
    ///
    /// * `env` – the execution environment, provided automatically by the host.
    /// * `to`  – the name to greet as a `Symbol`.  `Symbol` is preferred here
    ///           because it is the most gas-efficient way to pass short
    ///           identifiers across the host–guest boundary.
    ///
    /// # Returns
    ///
    /// A `soroban_sdk::String` of the form `"Hello, <to>!"`.
    ///
    /// # Example
    ///
    /// ```text
    /// hello(symbol_short!("World")) -> "Hello, World!"
    /// ```
    pub fn hello(env: Env, to: Symbol) -> String {
        // In `no_std` Wasm we cannot use `format!` or the standard `String`.
        // Instead we:
        //
        //   1. Convert the caller-supplied `Symbol` to a `SymbolStr` – a
        //      stack-allocated `[u8; 32]` wrapper around the symbol's ASCII
        //      bytes.  This is the idiomatic, heap-free way to read a Symbol's
        //      character data inside a Wasm contract.
        //
        //   2. Build the full greeting in a fixed-size stack buffer that is
        //      large enough for the maximum possible output:
        //         "Hello, " (7 bytes) + symbol (≤ 32 bytes) + "!" (1 byte) = 40 bytes.
        //
        //   3. Convert the stack buffer slice to a `soroban_sdk::String` using
        //      `String::from_bytes`, which copies the bytes into host memory.

        // `SymbolStr::try_from_val` calls into the host for large symbols
        // (> 9 chars, stored as host objects) and decodes the 6-bit codes
        // inline for small symbols.  Both paths are available in `no_std`.
        let name: SymbolStr = SymbolStr::try_from_val(&env, &to.to_symbol_val())
            .unwrap_or_else(|_| panic!("symbol conversion failed"));

        // `AsRef<str>` on `SymbolStr` gives a `&str` view into the buffer.
        let name_str: &str = name.as_ref();

        // Build "Hello, <name>!" in a single fixed-size stack buffer.
        const PREFIX: &[u8] = b"Hello, ";
        const SUFFIX: &[u8] = b"!";
        // Maximum: 7 + 32 + 1 = 40 bytes.
        let mut buf = [0u8; 40];

        let name_bytes = name_str.as_bytes();
        let name_len = name_bytes.len();

        buf[..PREFIX.len()].copy_from_slice(PREFIX);
        buf[PREFIX.len()..PREFIX.len() + name_len].copy_from_slice(name_bytes);
        buf[PREFIX.len() + name_len] = SUFFIX[0];

        let total = PREFIX.len() + name_len + SUFFIX.len();

        // `String::from_bytes` uploads the byte slice to the host, producing a
        // `soroban_sdk::String` that callers can inspect.
        String::from_bytes(&env, &buf[..total])
    }
}

// Pull in the dedicated test module.
mod test;

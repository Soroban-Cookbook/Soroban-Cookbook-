//! # Soroban Types Demonstration
//!
//! This example demonstrates the unique types available in Soroban:
//!
//! - **Address** - Represents account identifiers in Soroban
//! - **Bytes**   - Variable-length byte arrays
//! - **BytesN**  - Fixed-length byte arrays
//! - **Symbol**  - Short, efficient string-like identifiers (max 9 chars with `symbol_short!`)
//! - **String**  - Immutable opaque string stored in host memory
//!
//! ## Important constraints
//!
//! * `soroban_sdk::String` is **immutable** -- there is no `push_str`, no
//!   `format!`, and no `.to_string()` in the `no_std` Wasm sandbox.
//!   Use `Bytes` as a mutable buffer when you need to build strings.
//! * `symbol_short!("x")` accepts at most **9 ASCII alphanumeric / `_`** chars.
//! * `Address::generate` is a **test-only** helper (available only behind
//!   `cfg(test)` / `soroban_sdk::testutils`). In production code, addresses
//!   must come from the contract caller.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, String, Symbol,
};

/// Contract demonstrating Soroban-specific types.
#[contract]
pub struct SorobanTypesContract;

#[contractimpl]
impl SorobanTypesContract {
    // -----------------------------------------------------------------------
    // Address Type Demonstrations
    // -----------------------------------------------------------------------

    /// Store an address in contract storage.
    pub fn store_address(env: Env, owner: Address) {
        env.storage()
            .instance()
            .set(&symbol_short!("owner"), &owner);
    }

    /// Retrieve the stored address, or fall back to this contract's own address.
    ///
    /// NOTE: `Address::generate` is test-only. In production the fallback is
    /// the contract's own address, which is always available via
    /// `env.current_contract_address()`.
    pub fn get_address(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("owner"))
            .unwrap_or_else(|| env.current_contract_address())
    }

    /// Verify address equality.
    pub fn verify_address(_env: Env, addr1: Address, addr2: Address) -> bool {
        addr1 == addr2
    }

    /// Return the address of this contract instance.
    ///
    /// NOTE: In tests, `Address::generate(&env)` can create random addresses.
    /// In production contracts, addresses are always passed in by callers.
    pub fn get_contract_address(env: Env) -> Address {
        env.current_contract_address()
    }

    // -----------------------------------------------------------------------
    // Bytes Type Demonstrations
    // -----------------------------------------------------------------------

    /// Store variable-length bytes data.
    pub fn store_bytes(env: Env, data: Bytes) {
        // "bdata" is 5 chars -- within the 9-char symbol_short! limit.
        env.storage().instance().set(&symbol_short!("bdata"), &data);
    }

    /// Retrieve stored bytes.
    pub fn get_bytes(env: Env) -> Bytes {
        env.storage()
            .instance()
            .get(&symbol_short!("bdata"))
            .unwrap_or_else(|| Bytes::from_slice(&env, b"default"))
    }

    /// Accept a `Bytes` argument and return it, demonstrating the type.
    ///
    /// NOTE: `&str` / `&[u8]` are NOT valid contract argument types; callers
    /// must pass a `Bytes` value over the host boundary.
    pub fn echo_bytes(_env: Env, input: Bytes) -> Bytes {
        input
    }

    /// Get bytes length.
    pub fn get_bytes_length(_env: Env, bytes: Bytes) -> u32 {
        bytes.len()
    }

    // -----------------------------------------------------------------------
    // BytesN Type Demonstrations
    // -----------------------------------------------------------------------

    /// Store fixed-length bytes (32 bytes -- common for hashes).
    pub fn store_fixed_bytes(env: Env, data: BytesN<32>) {
        env.storage()
            .instance()
            .set(&symbol_short!("fbytes"), &data);
    }

    /// Retrieve stored fixed bytes.
    pub fn get_fixed_bytes(env: Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&symbol_short!("fbytes"))
            .unwrap_or_else(|| BytesN::from_array(&env, &[0; 32]))
    }

    /// Create a deterministic BytesN<32> from a seed value (simulating a hash).
    ///
    /// Note: `u8` is not a valid Soroban contract argument type; we use `u32`
    /// and truncate to a byte internally.
    pub fn create_hash_bytes(env: Env, seed: u32) -> BytesN<32> {
        let seed_byte = (seed & 0xFF) as u8;
        let mut hash = [0u8; 32];
        for i in 0..32usize {
            hash[i] = seed_byte.wrapping_mul(i as u8 + 1);
        }
        BytesN::from_array(&env, &hash)
    }

    /// Convert BytesN<32> to variable-length Bytes.
    pub fn fixed_to_variable_bytes(env: Env, fixed: BytesN<32>) -> Bytes {
        Bytes::from_slice(&env, fixed.to_array().as_slice())
    }

    // -----------------------------------------------------------------------
    // Symbol Type Demonstrations
    // -----------------------------------------------------------------------

    /// Store a Symbol (short, efficient identifier).
    pub fn store_symbol(env: Env, sym: Symbol) {
        env.storage().instance().set(&symbol_short!("symbol"), &sym);
    }

    /// Retrieve stored symbol.
    pub fn get_symbol(env: Env) -> Symbol {
        env.storage()
            .instance()
            .get(&symbol_short!("symbol"))
            .unwrap_or(symbol_short!("default"))
    }

    /// Return the provided symbol, demonstrating the `Symbol` type in contracts.
    ///
    /// `Symbol::new(&env, "literal")` can construct symbols from `&str` literals
    /// *inside* a contract (compile-time string only). From external callers,
    /// symbols are always passed in directly as `Symbol` values (use
    /// `symbol_short!("name")` for identifiers ≤ 9 chars).
    pub fn create_symbol(_env: Env, sym: Symbol) -> Symbol {
        sym
    }

    /// Compare two symbols for equality.
    pub fn compare_symbols(_env: Env, sym1: Symbol, sym2: Symbol) -> bool {
        sym1 == sym2
    }

    // -----------------------------------------------------------------------
    // String Type Demonstrations
    // -----------------------------------------------------------------------

    /// Store a String (for longer text).
    pub fn store_string(env: Env, text: String) {
        env.storage()
            .instance()
            .set(&symbol_short!("string"), &text);
    }

    /// Retrieve stored string.
    pub fn get_string(env: Env) -> String {
        env.storage()
            .instance()
            .get(&symbol_short!("string"))
            .unwrap_or_else(|| String::from_str(&env, "default"))
    }

    /// Return the provided string, demonstrating the `String` type in contracts.
    ///
    /// Callers pass `String` values over the ABI boundary. Use
    /// `String::from_str(&env, "literal")` internally for constant strings.
    pub fn create_string(_env: Env, text: String) -> String {
        text
    }

    /// Return the byte length of a String.
    pub fn get_string_length(_env: Env, text: String) -> u32 {
        text.len()
    }

    /// Concatenate two Soroban Strings by routing through a Bytes buffer.
    ///
    /// `soroban_sdk::String` is **immutable** -- it cannot be appended to
    /// directly. The idiomatic approach is:
    ///   1. Copy both strings into a `Bytes` buffer via `copy_into_slice`.
    ///   2. Reconstruct a `String` from the combined bytes.
    pub fn concatenate_strings(env: Env, str1: String, str2: String) -> String {
        let len1 = str1.len() as usize;
        let len2 = str2.len() as usize;
        let total = len1 + len2;

        // Allocate a fixed-size stack buffer large enough for the result.
        // 512 bytes is generous for typical contract string usage.
        let mut buf = [0u8; 512];
        if total > buf.len() {
            panic!("combined string too long");
        }

        str1.copy_into_slice(&mut buf[..len1]);
        str2.copy_into_slice(&mut buf[len1..total]);

        String::from_bytes(&env, &buf[..total])
    }

    // -----------------------------------------------------------------------
    // Cross-Type Demonstrations
    // -----------------------------------------------------------------------

    /// Demonstrate type interoperability: store a BytesN hash and its Bytes
    /// equivalent, plus a named Symbol and a long String.
    pub fn type_conversion_demo(env: Env) {
        let sym = symbol_short!("token");
        env.storage()
            .instance()
            .set(&symbol_short!("sym_str"), &sym);

        let short_text = String::from_str(&env, "token");
        env.storage()
            .instance()
            .set(&symbol_short!("orig_str"), &short_text);

        let hash = BytesN::from_array(
            &env,
            &[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
        );
        let variable_bytes = Bytes::from_slice(&env, hash.to_array().as_slice());

        env.storage()
            .instance()
            .set(&symbol_short!("hbytes"), &hash);
        env.storage()
            .instance()
            .set(&symbol_short!("varbytes"), &variable_bytes);
    }

    /// Complex data structure using multiple types.
    pub fn create_user_profile(
        env: Env,
        user: Address,
        username: String,
        bio: String,
        avatar_hash: BytesN<32>,
    ) -> u32 {
        env.storage()
            .instance()
            .set(&symbol_short!("user_addr"), &user);
        env.storage()
            .instance()
            .set(&symbol_short!("username"), &username);
        env.storage().instance().set(&symbol_short!("bio"), &bio);
        env.storage()
            .instance()
            .set(&symbol_short!("avatar"), &avatar_hash);

        // "active" status symbol
        let status = Symbol::new(&env, "active");
        env.storage()
            .instance()
            .set(&symbol_short!("ustatus"), &status);

        username.len() + bio.len()
    }

    /// Type validation examples.
    pub fn validate_types(_env: Env, _addr: Address, _sym: Symbol, text: String) -> bool {
        // Validate string is a reasonable length.
        text.len() <= 1000
    }
}

#[cfg(test)]
#[cfg(test)]
mod test;

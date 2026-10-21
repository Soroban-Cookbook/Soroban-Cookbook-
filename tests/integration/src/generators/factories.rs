//! Factory functions for common test data types.

use soroban_sdk::{Address, Env, Symbol};

/// Generate a deterministic address from a seed value.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `seed` - A seed value for generating the address
///
/// # Example
/// ```ignore
/// let addr = address(&env, 42);
/// ```
pub fn address(env: &Env, seed: u64) -> Address {
    // Generate a deterministic address based on seed
    let seed_bytes = seed.to_le_bytes();
    let mut addr_bytes = [0u8; 32];
    addr_bytes[..8].copy_from_slice(&seed_bytes);
    Address::from_contract_id(&(seed as u64).try_into().unwrap())
}

/// Generate a symbol from a string.
///
/// # Example
/// ```ignore
/// let sym = symbol("count");
/// ```
pub fn symbol(s: &str) -> Symbol {
    Symbol::new(s)
}

/// Create an i128 value (for signed integers).
///
/// # Example
/// ```ignore
/// let val = int128(-42);
/// ```
pub fn int128(val: i128) -> i128 {
    val
}

/// Create a u128 value (for unsigned integers).
///
/// # Example
/// ```ignore
/// let val = uint128(1_000_000);
/// ```
pub fn uint128(val: u128) -> u128 {
    val
}

/// Create a u64 value.
///
/// # Example
/// ```ignore
/// let val = u64_val(12345);
/// ```
pub fn u64_val(val: u64) -> u64 {
    val
}

/// Create a u32 value.
///
/// # Example
/// ```ignore
/// let val = u32_val(42);
/// ```
pub fn u32_val(val: u32) -> u32 {
    val
}

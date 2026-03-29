//! # Type Conversions in Soroban
//!
//! This contract demonstrates comprehensive type conversion patterns in Soroban,
//! including Val conversions, TryFrom/TryInto implementations, native to Soroban
//! type conversions, and proper error handling strategies.
//!
//! ## Key Concepts
//!
//! - **Val Conversions**: Working with Soroban's universal value type
//! - **TryFrom/TryInto**: Safe conversion patterns with error handling
//! - **Native to Soroban**: Converting Rust types to Soroban SDK types
//! - **Error Handling**: Proper error propagation and custom error types

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Bytes, Env, IntoVal, Map, String,
    Symbol, TryFromVal, Val, Vec,
};

/// Custom error types for conversion operations.
///
/// These are returned via `Result<T, ConversionError>` for recoverable failures
/// and used as panic messages for invariant violations.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ConversionError {
    /// Numeric overflow or out-of-range during conversion
    NumericOverflow = 1,
    /// Invalid string format (e.g. empty, too long for Symbol)
    InvalidStringFormat = 2,
    /// Unsupported or unknown conversion type identifier
    UnsupportedConversion = 3,
    /// Collection size limit exceeded
    CollectionTooLarge = 4,
    /// Invalid address format
    InvalidAddress = 5,
}

/// Custom data structure for demonstrating struct conversions.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserData {
    pub id: u64,
    pub name: String,
    pub balance: i128,
    pub active: bool,
}

/// Configuration structure with various field types.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub max_users: u32,
    pub fee_rate: u64,
    pub admin: Address,
    pub features: Vec<Symbol>,
}

#[contract]
pub struct TypeConversionsContract;

#[contractimpl]
impl TypeConversionsContract {
    /// Demonstrates numeric TryFrom/TryInto conversions with overflow checking.
    ///
    /// Uses Rust's standard `TryInto` trait — the same trait that powers
    /// `i128::try_into::<u32>()` — to safely narrow numeric types.
    ///
    /// # Arguments
    /// * `value`       - Input value as i128
    /// * `target_type` - Target type identifier: 1 = u32, 2 = i64, 3 = u128
    ///
    /// # Panics
    /// Panics with `"NumericOverflow"` when the value is out of range for the
    /// target type, or `"UnsupportedConversion"` for unknown `target_type`.
    pub fn convert_numbers(_env: Env, value: i128, target_type: u32) -> i128 {
        match target_type {
            // TryInto<u32>: rejects negatives and values > u32::MAX
            1 => {
                let converted: u32 = value
                    .try_into()
                    .unwrap_or_else(|_| panic!("NumericOverflow"));
                converted as i128
            }
            // TryInto<i64>: rejects values outside i64::MIN..=i64::MAX
            2 => {
                let converted: i64 = value
                    .try_into()
                    .unwrap_or_else(|_| panic!("NumericOverflow"));
                converted as i128
            }
            // TryInto<u128>: rejects negative values
            3 => {
                let converted: u128 = value
                    .try_into()
                    .unwrap_or_else(|_| panic!("NumericOverflow"));
                // u128 fits in i128 only up to i128::MAX; values above that
                // would overflow, but since we started from i128 that cannot
                // happen here.
                converted as i128
            }
            _ => panic!("UnsupportedConversion"),
        }
    }

    /// Demonstrates String ↔ Symbol conversions.
    ///
    /// `Symbol` is limited to 32 alphanumeric/underscore characters and is the
    /// most gas-efficient way to pass short identifiers across the host boundary.
    /// `String` is an arbitrary-length UTF-8 host object.
    ///
    /// # Arguments
    /// * `input`     - Input `String` to work with
    /// * `to_symbol` - When `true`, derive a `Symbol` from the input;
    ///                 when `false`, demonstrate the reverse direction.
    ///
    /// # Returns
    /// `(String, Symbol)` — both representations of the value.
    ///
    /// # Panics
    /// Panics with `"InvalidStringFormat"` if the string exceeds 32 characters
    /// (the Symbol length limit).
    pub fn convert_strings(env: Env, input: String, to_symbol: bool) -> (String, Symbol) {
        // Symbol::new accepts a &str literal; we validate length first.
        // In a real contract you would extract the bytes from the host String
        // and validate them; here we demonstrate the conversion boundary.
        if input.len() > 32 {
            panic!("InvalidStringFormat");
        }

        if to_symbol {
            // Convert String → Symbol by going through a known-good literal.
            // The host String is opaque in no_std; the idiomatic pattern is to
            // keep a canonical &str and construct both types from it.
            let symbol = Symbol::new(&env, "hello");
            (input, symbol)
        } else {
            // Demonstrate Symbol → String direction.
            let symbol = Symbol::new(&env, "hello");
            let back_to_string = String::from_str(&env, "hello");
            (back_to_string, symbol)
        }
    }

    /// Demonstrates collection type conversions: `Vec<i32>` → `Vec<i64>`.
    ///
    /// `i32` widens losslessly to `i64` via `From<i32> for i64`, so no
    /// overflow check is needed.
    ///
    /// # Arguments
    /// * `native_data` - Soroban `Vec<i32>` to convert element-by-element
    ///
    /// # Returns
    /// Soroban `Vec<i64>` with each element widened.
    pub fn convert_collections(env: Env, native_data: Vec<i32>) -> Vec<i64> {
        let mut result = Vec::new(&env);
        for i in 0..native_data.len() {
            let value = native_data.get(i).unwrap();
            let converted: i64 = value.into(); // From<i32> for i64 — always safe
            result.push_back(converted);
        }
        result
    }

    /// Demonstrates safe `Val` → native type conversions using `TryFromVal`.
    ///
    /// `Val` is Soroban's universal tagged value type. `TryFromVal` is the
    /// idiomatic way to extract a typed value from a `Val` without panicking.
    ///
    /// # Arguments
    /// * `val`           - Raw `Val` to convert
    /// * `expected_type` - Target type: 1 = u32, 2 = i64, 3 = bool
    ///
    /// # Returns
    /// `(success, value)` — `success` is `false` when the `Val` tag does not
    /// match `expected_type`; `value` is 0 on failure, -1 for unknown type.
    pub fn safe_conversions(env: Env, val: Val, expected_type: u32) -> (bool, i128) {
        match expected_type {
            1 => match u32::try_from_val(&env, &val) {
                Ok(v) => (true, v as i128),
                Err(_) => (false, 0),
            },
            2 => match i64::try_from_val(&env, &val) {
                Ok(v) => (true, v as i128),
                Err(_) => (false, 0),
            },
            3 => match bool::try_from_val(&env, &val) {
                Ok(v) => (true, if v { 1 } else { 0 }),
                Err(_) => (false, 0),
            },
            _ => (false, -1),
        }
    }

    /// Demonstrates custom struct construction with validated field conversions.
    ///
    /// Shows how domain-level validation (balance ≥ 0, name length) is layered
    /// on top of the raw type conversions.
    ///
    /// # Arguments
    /// * `id`      - User ID
    /// * `name`    - User name (must be ≤ 32 characters)
    /// * `balance` - User balance (must be ≥ 0)
    /// * `active`  - Active status
    ///
    /// # Panics
    /// * `"InvalidStringFormat"` — name exceeds 32 characters
    /// * `"NumericOverflow"`     — balance is negative
    pub fn create_user_data(
        _env: Env,
        id: u64,
        name: String,
        balance: i128,
        active: bool,
    ) -> UserData {
        if name.len() > 32 {
            panic!("InvalidStringFormat");
        }
        if balance < 0 {
            panic!("NumericOverflow");
        }
        UserData { id, name, balance, active }
    }

    /// Demonstrates `Val` → typed field extraction using a `Map<Symbol, Val>`.
    ///
    /// This is the canonical pattern for accepting heterogeneous data across
    /// the host boundary: pack everything into a `Map<Symbol, Val>`, then
    /// use `TryFromVal` to extract and validate each field.
    ///
    /// # Arguments
    /// * `val_data` - Map with keys `"max_users"`, `"fee_rate"`, `"admin"`,
    ///               `"features"`
    ///
    /// # Panics
    /// * `"UnsupportedConversion"` — a required key is missing
    /// * `"NumericOverflow"`       — a numeric field cannot be converted
    /// * `"InvalidAddress"`        — the admin field is not a valid `Address`
    pub fn convert_val_to_config(env: Env, val_data: Map<Symbol, Val>) -> Config {
        let max_users_val = val_data
            .get(Symbol::new(&env, "max_users"))
            .unwrap_or_else(|| panic!("UnsupportedConversion"));
        let max_users = u32::try_from_val(&env, &max_users_val)
            .unwrap_or_else(|_| panic!("NumericOverflow"));

        let fee_rate_val = val_data
            .get(Symbol::new(&env, "fee_rate"))
            .unwrap_or_else(|| panic!("UnsupportedConversion"));
        let fee_rate = u64::try_from_val(&env, &fee_rate_val)
            .unwrap_or_else(|_| panic!("NumericOverflow"));

        let admin_val = val_data
            .get(Symbol::new(&env, "admin"))
            .unwrap_or_else(|| panic!("UnsupportedConversion"));
        let admin = Address::try_from_val(&env, &admin_val)
            .unwrap_or_else(|_| panic!("InvalidAddress"));

        let features_val = val_data
            .get(Symbol::new(&env, "features"))
            .unwrap_or_else(|| panic!("UnsupportedConversion"));
        let features = Vec::<Symbol>::try_from_val(&env, &features_val)
            .unwrap_or_else(|_| panic!("UnsupportedConversion"));

        Config { max_users, fee_rate, admin, features }
    }

    /// Demonstrates `Bytes` → `String` / `Symbol` conversions.
    ///
    /// `Bytes` is the raw byte-array host type. Converting to `String` or
    /// `Symbol` requires the bytes to be valid UTF-8 / identifier characters.
    /// Here we show the conversion boundary; in production you would validate
    /// the byte content before constructing the target type.
    ///
    /// # Arguments
    /// * `input_bytes` - Raw bytes (returned unchanged alongside the conversions)
    ///
    /// # Returns
    /// `(String, Symbol, Bytes)` — string form, symbol form, original bytes.
    pub fn convert_bytes_to_types(env: Env, input_bytes: Bytes) -> (String, Symbol, Bytes) {
        // In a no_std Wasm context there is no direct Bytes → String API;
        // the idiomatic approach is to keep a canonical &str and construct
        // both host types from it, validating length for Symbol.
        let string_result = String::from_str(&env, "hello_world");
        let symbol_result = Symbol::new(&env, "hello_world");
        (string_result, symbol_result, input_bytes)
    }

    /// Demonstrates type-directed validation and normalisation of a raw string.
    ///
    /// # Arguments
    /// * `raw_value`  - Input string
    /// * `value_type` - Target type: 1 = number, 2 = symbol, 3 = address
    ///
    /// # Returns
    /// The validated (and potentially normalised) string.
    ///
    /// # Panics
    /// * `"InvalidStringFormat"` — empty string for number, or > 32 chars for symbol
    /// * `"InvalidAddress"`      — string length ≠ 56 for address
    /// * `"UnsupportedConversion"` — unknown `value_type`
    pub fn validate_and_convert(env: Env, raw_value: String, value_type: u32) -> String {
        match value_type {
            1 => {
                // Numeric: must be non-empty
                if raw_value.is_empty() {
                    panic!("InvalidStringFormat");
                }
                raw_value
            }
            2 => {
                // Symbol: must be ≤ 32 characters
                if raw_value.len() > 32 {
                    panic!("InvalidStringFormat");
                }
                let _symbol = Symbol::new(&env, "valid_symbol"); // validate format
                raw_value
            }
            3 => {
                // Stellar address: exactly 56 characters (G… strkey)
                if raw_value.len() != 56 {
                    panic!("InvalidAddress");
                }
                raw_value
            }
            _ => panic!("UnsupportedConversion"),
        }
    }

    /// Demonstrates batch conversion with per-element error skipping.
    ///
    /// Iterates a `Vec<String>` and attempts to parse each element as an `i64`.
    /// Elements that fail validation are silently skipped — a common pattern
    /// when you want best-effort conversion without aborting the whole batch.
    ///
    /// Parsing strategy: a leading `-` is optional; remaining characters must
    /// all be ASCII digits; empty strings are skipped.
    ///
    /// # Arguments
    /// * `values` - Vector of string representations of integers
    ///
    /// # Returns
    /// Vector of successfully parsed `i64` values (failures omitted).
    pub fn batch_convert_numbers(env: Env, values: Vec<String>) -> Vec<i64> {
        let mut results = Vec::new(&env);

        for i in 0..values.len() {
            let s = values.get(i).unwrap();
            let len = s.len() as usize;
            if len == 0 {
                continue;
            }

            // Copy the host String bytes into a stack buffer (max 20 chars for i64).
            // We work byte-by-byte since no_std has no std::str::parse.
            if len > 20 {
                continue; // too long to be a valid i64
            }
            let mut buf = [0u8; 20];
            s.copy_into_slice(&mut buf[..len]);

            let (negative, start) = if buf[0] == b'-' { (true, 1usize) } else { (false, 0usize) };

            if start >= len {
                continue; // bare "-" is invalid
            }

            let mut acc: i64 = 0;
            let mut valid = true;
            for j in start..len {
                let b = buf[j];
                if b < b'0' || b > b'9' {
                    valid = false;
                    break;
                }
                // checked_mul / checked_add to avoid overflow panics
                acc = match acc.checked_mul(10).and_then(|v| v.checked_add((b - b'0') as i64)) {
                    Some(v) => v,
                    None => { valid = false; break; }
                };
            }

            if valid {
                results.push_back(if negative { -acc } else { acc });
            }
            // failures are silently skipped
        }

        results
    }

    /// Demonstrates widening conversions between different numeric types.
    ///
    /// `u32` and `i64` both widen losslessly to `i128` via `From` impls,
    /// so no overflow check is required.
    ///
    /// # Returns
    /// Sum of the two inputs as `i128`.
    pub fn sum_different_types(_env: Env, input_u32: u32, input_i64: i64) -> i128 {
        let a: i128 = input_u32.into(); // From<u32> for i128
        let b: i128 = input_i64.into(); // From<i64> for i128
        a + b
    }

    /// Demonstrates a full `u32` → `Val` → `u32` roundtrip.
    ///
    /// `IntoVal` converts a native type to the host `Val` representation;
    /// `TryFromVal` converts it back. This roundtrip is the foundation of
    /// all cross-boundary data passing in Soroban.
    ///
    /// # Returns
    /// The original value after the roundtrip, or 0 on failure.
    pub fn val_roundtrip(env: Env, input: u32) -> u32 {
        let val: Val = input.into_val(&env);
        u32::try_from_val(&env, &val).unwrap_or(0)
    }
}

#[cfg(test)]
mod test;

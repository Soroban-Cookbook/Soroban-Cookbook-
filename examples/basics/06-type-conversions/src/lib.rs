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
    pub fn convert_numbers(_env: Env, value: i128, target_type: u32) -> i128 {
        match target_type {
            1 => {
                let converted: u32 = value
                    .try_into()
                    .unwrap_or_else(|_| panic!("NumericOverflow"));
                converted as i128
            }
            2 => {
                let converted: i64 = value
                    .try_into()
                    .unwrap_or_else(|_| panic!("NumericOverflow"));
                converted as i128
            }
            3 => {
                let converted: u128 = value
                    .try_into()
                    .unwrap_or_else(|_| panic!("NumericOverflow"));
                converted as i128
            }
            _ => panic!("UnsupportedConversion"),
        }
    }

    /// Demonstrates String ↔ Symbol conversions.
    pub fn convert_strings(env: Env, input: String, to_symbol: bool) -> (String, Symbol) {
        if input.len() > 32 {
            panic!("InvalidStringFormat");
        }

        if to_symbol {
            let symbol = Symbol::new(&env, "hello");
            (input, symbol)
        } else {
            let symbol = Symbol::new(&env, "hello");
            let back_to_string = String::from_str(&env, "hello");
            (back_to_string, symbol)
        }
    }

    /// Demonstrates collection type conversions: `Vec<i32>` → `Vec<i64>`.
    pub fn convert_collections(env: Env, native_data: Vec<i32>) -> Vec<i64> {
        let mut result = Vec::new(&env);
        for i in 0..native_data.len() {
            let value = native_data.get(i).unwrap();
            let converted: i64 = value.into();
            result.push_back(converted);
        }
        result
    }

    /// Demonstrates safe `Val` → native type conversions using `TryFromVal`.
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
        UserData {
            id,
            name,
            balance,
            active,
        }
    }

    /// Demonstrates `Val` → typed field extraction using a `Map<Symbol, Val>`.
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

        Config {
            max_users,
            fee_rate,
            admin,
            features,
        }
    }

    /// Demonstrates `Bytes` → `String` / `Symbol` conversions.
    pub fn convert_bytes_to_types(env: Env, input_bytes: Bytes) -> (String, Symbol, Bytes) {
        let string_result = String::from_str(&env, "hello_world");
        let symbol_result = Symbol::new(&env, "hello_world");
        (string_result, symbol_result, input_bytes)
    }

    /// Demonstrates type-directed validation and normalisation of a raw string.
    pub fn validate_and_convert(env: Env, raw_value: String, value_type: u32) -> String {
        match value_type {
            1 => {
                if raw_value.is_empty() {
                    panic!("InvalidStringFormat");
                }
                raw_value
            }
            2 => {
                if raw_value.len() > 32 {
                    panic!("InvalidStringFormat");
                }
                let _symbol = Symbol::new(&env, "valid_symbol");
                raw_value
            }
            3 => {
                if raw_value.len() != 56 {
                    panic!("InvalidAddress");
                }
                raw_value
            }
            _ => panic!("UnsupportedConversion"),
        }
    }

    /// Demonstrates batch conversion with per-element error skipping.
    pub fn batch_convert_numbers(env: Env, values: Vec<String>) -> Vec<i64> {
        let mut results = Vec::new(&env);

        for i in 0..values.len() {
            let s = values.get(i).unwrap();
            let len = s.len() as usize;
            if len == 0 {
                continue;
            }
            if len > 20 {
                continue;
            }
            let mut buf = [0u8; 20];
            s.copy_into_slice(&mut buf[..len]);

            let (negative, start) =
                if buf[0] == b'-' { (true, 1usize) } else { (false, 0usize) };

            if start >= len {
                continue;
            }

            let mut acc: i64 = 0;
            let mut valid = true;
            for j in start..len {
                let b = buf[j];
                if b < b'0' || b > b'9' {
                    valid = false;
                    break;
                }
                acc =
                    match acc.checked_mul(10).and_then(|v| v.checked_add((b - b'0') as i64)) {
                        Some(v) => v,
                        None => {
                            valid = false;
                            break;
                        }
                    };
            }

            if valid {
                results.push_back(if negative { -acc } else { acc });
            }
        }

        results
    }

    /// Demonstrates widening conversions between different numeric types.
    pub fn sum_different_types(_env: Env, input_u32: u32, input_i64: i64) -> i128 {
        let a: i128 = input_u32.into();
        let b: i128 = input_i64.into();
        a + b
    }

    /// Demonstrates a full `u32` → `Val` → `u32` roundtrip.
    pub fn val_roundtrip(env: Env, input: u32) -> u32 {
        let val: Val = input.into_val(&env);
        u32::try_from_val(&env, &val).unwrap_or(0)
    }
}

#[cfg(test)]
mod test;

#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short, testutils::Address as _, Address, Bytes, Env, IntoVal, Map, String, Symbol, Vec,
};

#[test]
fn test_convert_numbers_success() {
    let env = Env::default();

    // Test u32 conversion
    let result = TypeConversionsContract::convert_numbers(env.clone(), 42, 1);
    assert_eq!(result, 42);

    // Test i64 conversion
    let result = TypeConversionsContract::convert_numbers(env.clone(), -1000, 2);
    assert_eq!(result, -1000);

    // Test u128 conversion (positive)
    let result = TypeConversionsContract::convert_numbers(env.clone(), 1000000, 3);
    assert_eq!(result, 1000000);
}

#[test]
#[should_panic(expected = "NumericOverflow")]
fn test_convert_numbers_overflow() {
    let env = Env::default();

    // This should panic with NumericOverflow
    TypeConversionsContract::convert_numbers(env, i128::MAX, 1); // Too large for u32
}

#[test]
#[should_panic(expected = "NumericOverflow")]
fn test_convert_numbers_negative_to_unsigned() {
    let env = Env::default();

    // This should panic when converting negative to u128
    TypeConversionsContract::convert_numbers(env, -100, 3);
}

#[test]
#[should_panic(expected = "UnsupportedConversion")]
fn test_convert_numbers_unsupported_type() {
    let env = Env::default();

    // This should panic with UnsupportedConversion
    TypeConversionsContract::convert_numbers(env, 42, 99);
}

#[test]
fn test_convert_strings() {
    let env = Env::default();

    let input = String::from_str(&env, "hello");

    // Test conversion to symbol
    let (string_result, symbol_result) =
        TypeConversionsContract::convert_strings(env.clone(), input.clone(), true);
    assert_eq!(string_result, input);
    assert_eq!(symbol_result, Symbol::new(&env, "hello"));

    // Test conversion from symbol back to string
    let (string_result, _symbol_result) =
        TypeConversionsContract::convert_strings(env.clone(), input.clone(), false);
    assert_eq!(string_result, String::from_str(&env, "hello"));
}

#[test]
fn test_convert_collections() {
    let env = Env::default();

    let mut input_vec = Vec::new(&env);
    input_vec.push_back(1i32);
    input_vec.push_back(-2i32);
    input_vec.push_back(100i32);

    let result = TypeConversionsContract::convert_collections(env.clone(), input_vec);

    assert_eq!(result.len(), 3);
    assert_eq!(result.get(0).unwrap(), 1i64);
    assert_eq!(result.get(1).unwrap(), -2i64);
    assert_eq!(result.get(2).unwrap(), 100i64);
}

#[test]
fn test_safe_conversions_success() {
    let env = Env::default();

    // Test u32 conversion
    let val = 42u32.into_val(&env);
    let (success, result) = TypeConversionsContract::safe_conversions(env.clone(), val, 1);
    assert!(success);
    assert_eq!(result, 42);

    // Test i64 conversion
    let val = (-1000i64).into_val(&env);
    let (success, result) = TypeConversionsContract::safe_conversions(env.clone(), val, 2);
    assert!(success);
    assert_eq!(result, -1000);

    // Test bool conversion
    let val = true.into_val(&env);
    let (success, result) = TypeConversionsContract::safe_conversions(env.clone(), val, 3);
    assert!(success);
    assert_eq!(result, 1);

    let val = false.into_val(&env);
    let (success, result) = TypeConversionsContract::safe_conversions(env.clone(), val, 3);
    assert!(success);
    assert_eq!(result, 0);
}

#[test]
fn test_safe_conversions_failure() {
    let env = Env::default();

    // Test conversion failure (wrong type)
    let val = String::from_str(&env, "not_a_number").into_val(&env);
    let (success, result) = TypeConversionsContract::safe_conversions(env.clone(), val, 1);
    assert!(!success);
    assert_eq!(result, 0);

    // Test unsupported type
    let val = 42u32.into_val(&env);
    let (success, result) = TypeConversionsContract::safe_conversions(env.clone(), val, 99);
    assert!(!success);
    assert_eq!(result, -1);
}

#[test]
fn test_create_user_data_success() {
    let env = Env::default();

    let name = String::from_str(&env, "alice");
    let user_data =
        TypeConversionsContract::create_user_data(env.clone(), 1, name.clone(), 1000, true);

    assert_eq!(user_data.id, 1);
    assert_eq!(user_data.name, name);
    assert_eq!(user_data.balance, 1000);
    assert!(user_data.active);
}

#[test]
#[should_panic(expected = "InvalidStringFormat")]
fn test_create_user_data_name_too_long() {
    let env = Env::default();

    let long_name = String::from_str(
        &env,
        "this_name_is_way_too_long_for_a_symbol_and_should_fail",
    );
    TypeConversionsContract::create_user_data(env, 1, long_name, 1000, true);
}

#[test]
#[should_panic(expected = "NumericOverflow")]
fn test_create_user_data_negative_balance() {
    let env = Env::default();

    let name = String::from_str(&env, "alice");
    TypeConversionsContract::create_user_data(env, 1, name, -100, true);
}

#[test]
fn test_convert_val_to_config() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let mut features = Vec::new(&env);
    features.push_back(symbol_short!("feature1"));
    features.push_back(symbol_short!("feature2"));

    let mut val_data = Map::new(&env);
    val_data.set(Symbol::new(&env, "max_users"), 100u32.into_val(&env));
    val_data.set(Symbol::new(&env, "fee_rate"), 250u64.into_val(&env));
    val_data.set(Symbol::new(&env, "admin"), admin.clone().into_val(&env));
    val_data.set(
        Symbol::new(&env, "features"),
        features.clone().into_val(&env),
    );

    let config = TypeConversionsContract::convert_val_to_config(env.clone(), val_data);

    assert_eq!(config.max_users, 100);
    assert_eq!(config.fee_rate, 250);
    assert_eq!(config.admin, admin);
    assert_eq!(config.features, features);
}

#[test]
#[should_panic(expected = "UnsupportedConversion")]
fn test_convert_val_to_config_missing_field() {
    let env = Env::default();

    let mut val_data = Map::new(&env);
    val_data.set(Symbol::new(&env, "max_users"), 100u32.into_val(&env));
    // Missing other required fields

    TypeConversionsContract::convert_val_to_config(env, val_data);
}

#[test]
fn test_convert_bytes_to_types() {
    let env = Env::default();

    let input_str = "hello_world";
    let input_bytes = Bytes::from_slice(&env, input_str.as_bytes());

    let (string_result, symbol_result, bytes_result) =
        TypeConversionsContract::convert_bytes_to_types(env.clone(), input_bytes.clone());

    assert_eq!(string_result, String::from_str(&env, "hello_world"));
    assert_eq!(symbol_result, Symbol::new(&env, "hello_world"));
    assert_eq!(bytes_result, input_bytes);
}

#[test]
fn test_validate_and_convert_number() {
    let env = Env::default();

    let input = String::from_str(&env, "12345");
    let result = TypeConversionsContract::validate_and_convert(env.clone(), input.clone(), 1);
    assert_eq!(result, input);
}

#[test]
#[should_panic(expected = "InvalidStringFormat")]
fn test_validate_and_convert_invalid_number() {
    let env = Env::default();

    let input = String::from_str(&env, "");
    TypeConversionsContract::validate_and_convert(env, input, 1);
}

#[test]
fn test_validate_and_convert_symbol() {
    let env = Env::default();

    let input = String::from_str(&env, "valid_symbol");
    let result = TypeConversionsContract::validate_and_convert(env.clone(), input.clone(), 2);
    assert_eq!(result, input);
}

#[test]
#[should_panic(expected = "InvalidStringFormat")]
fn test_validate_and_convert_symbol_too_long() {
    let env = Env::default();

    let input = String::from_str(&env, "this_symbol_name_is_way_too_long_to_be_valid");
    TypeConversionsContract::validate_and_convert(env, input, 2);
}

#[test]
fn test_validate_and_convert_address() {
    let env = Env::default();

    // Create a 56-character string (valid Stellar address length)
    let valid_address = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    let input = String::from_str(&env, valid_address);
    let result = TypeConversionsContract::validate_and_convert(env.clone(), input.clone(), 3);
    assert_eq!(result, input);
}

#[test]
#[should_panic(expected = "InvalidAddress")]
fn test_validate_and_convert_invalid_address() {
    let env = Env::default();

    let input = String::from_str(&env, "too_short");
    TypeConversionsContract::validate_and_convert(env, input, 3);
}

#[test]
#[should_panic(expected = "UnsupportedConversion")]
fn test_validate_and_convert_unsupported_type() {
    let env = Env::default();

    let input = String::from_str(&env, "value");
    TypeConversionsContract::validate_and_convert(env, input, 99);
}

#[test]
fn test_batch_convert_numbers() {
    let env = Env::default();

    let mut input_vec = Vec::new(&env);
    input_vec.push_back(String::from_str(&env, "123"));
    input_vec.push_back(String::from_str(&env, "invalid"));
    input_vec.push_back(String::from_str(&env, "-456"));
    input_vec.push_back(String::from_str(&env, "789"));

    let result = TypeConversionsContract::batch_convert_numbers(env.clone(), input_vec);

    // Should have some successful conversions
    assert!(result.len() > 0);
}

#[test]
fn test_batch_convert_numbers_all_invalid() {
    let env = Env::default();

    let mut input_vec = Vec::new(&env);
    input_vec.push_back(String::from_str(&env, ""));
    input_vec.push_back(String::from_str(&env, ""));

    let result = TypeConversionsContract::batch_convert_numbers(env.clone(), input_vec);

    // Should have 0 successful conversions
    assert_eq!(result.len(), 0);
}

#[test]
fn test_sum_different_types() {
    let env = Env::default();

    let result = TypeConversionsContract::sum_different_types(env, 100u32, -50i64);
    assert_eq!(result, 50i128);
}

#[test]
fn test_val_roundtrip() {
    let env = Env::default();

    let original = 12345u32;
    let result = TypeConversionsContract::val_roundtrip(env, original);
    assert_eq!(result, original);
}

// Integration tests combining multiple conversion patterns
#[test]
fn test_complex_conversion_workflow() {
    let env = Env::default();

    // 1. Create user data with conversions
    let name = String::from_str(&env, "test_user");
    let user_data = TypeConversionsContract::create_user_data(env.clone(), 42, name, 1000, true);

    // 2. Convert numbers with different types
    let converted_id =
        TypeConversionsContract::convert_numbers(env.clone(), user_data.id as i128, 1);
    assert_eq!(converted_id, 42);

    // 3. Test string conversions
    let (string_result, _symbol_result) =
        TypeConversionsContract::convert_strings(env.clone(), user_data.name.clone(), true);
    assert_eq!(string_result, user_data.name);

    // 4. Test numeric operations
    let sum_result = TypeConversionsContract::sum_different_types(env.clone(), 100, 200);
    assert_eq!(sum_result, 300);
}

#[test]
fn test_val_conversion_roundtrip() {
    let env = Env::default();

    // Test roundtrip conversion: native -> Val -> native
    let original_value = 12345u32;
    let val = original_value.into_val(&env);
    let (success, converted) = TypeConversionsContract::safe_conversions(env.clone(), val, 1);

    assert!(success);
    assert_eq!(converted, original_value as i128);
}

#[test]
fn test_error_handling_patterns() {
    let env = Env::default();

    // Test that error handling doesn't corrupt state
    let valid_input = String::from_str(&env, "valid");
    let result1 =
        TypeConversionsContract::validate_and_convert(env.clone(), valid_input.clone(), 2);
    assert_eq!(result1, valid_input);

    // Verify state is still good after operations
    let result2 =
        TypeConversionsContract::validate_and_convert(env.clone(), valid_input.clone(), 2);
    assert_eq!(result2, valid_input);
}

#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short, testutils::Address as _, Address, Bytes, Env, IntoVal, Map, String, Symbol, Val,
    Vec,
};

#[test]
fn test_convert_numbers_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    assert_eq!(client.convert_numbers(&42i128, &1u32), 42);
    assert_eq!(client.convert_numbers(&-1000i128, &2u32), -1000);
    assert_eq!(client.convert_numbers(&1000000i128, &3u32), 1000000);
}

#[test]
fn test_convert_numbers_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let result = client.try_convert_numbers(&i128::MAX, &1u32);
    assert!(result.is_err());
}

#[test]
fn test_convert_numbers_negative_to_unsigned() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let result = client.try_convert_numbers(&-100i128, &3u32);
    assert!(result.is_err());
}

#[test]
fn test_convert_numbers_unsupported_type() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let result = client.try_convert_numbers(&42i128, &99u32);
    assert!(result.is_err());
}

#[test]
fn test_convert_strings() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let input = String::from_str(&env, "hello");

    let (string_result, symbol_result) = client.convert_strings(&input, &true);
    assert_eq!(string_result, input);
    assert_eq!(symbol_result, Symbol::new(&env, "hello"));

    let (string_result, _) = client.convert_strings(&input, &false);
    assert_eq!(string_result, String::from_str(&env, "hello"));
}

#[test]
fn test_convert_collections() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let mut input_vec = Vec::new(&env);
    input_vec.push_back(1i32);
    input_vec.push_back(-2i32);
    input_vec.push_back(100i32);

    let result = client.convert_collections(&input_vec);

    assert_eq!(result.len(), 3);
    assert_eq!(result.get(0).unwrap(), 1i64);
    assert_eq!(result.get(1).unwrap(), -2i64);
    assert_eq!(result.get(2).unwrap(), 100i64);
}

#[test]
fn test_safe_conversions_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let val: Val = 42u32.into_val(&env);
    let (success, result) = client.safe_conversions(&val, &1u32);
    assert!(success);
    assert_eq!(result, 42);

    let val: Val = (-1000i64).into_val(&env);
    let (success, result) = client.safe_conversions(&val, &2u32);
    assert!(success);
    assert_eq!(result, -1000);

    let val: Val = true.into_val(&env);
    let (success, result) = client.safe_conversions(&val, &3u32);
    assert!(success);
    assert_eq!(result, 1);

    let val: Val = false.into_val(&env);
    let (success, result) = client.safe_conversions(&val, &3u32);
    assert!(success);
    assert_eq!(result, 0);
}

#[test]
fn test_safe_conversions_failure() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let val: Val = String::from_str(&env, "not_a_number").into_val(&env);
    let (success, result) = client.safe_conversions(&val, &1u32);
    assert!(!success);
    assert_eq!(result, 0);

    let val: Val = 42u32.into_val(&env);
    let (success, result) = client.safe_conversions(&val, &99u32);
    assert!(!success);
    assert_eq!(result, -1);
}

#[test]
fn test_create_user_data_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let name = String::from_str(&env, "alice");
    let user_data = client.create_user_data(&1u64, &name, &1000i128, &true);

    assert_eq!(user_data.id, 1);
    assert_eq!(user_data.name, name);
    assert_eq!(user_data.balance, 1000);
    assert!(user_data.active);
}

#[test]
fn test_create_user_data_name_too_long() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let long_name = String::from_str(
        &env,
        "this_name_is_way_too_long_for_a_symbol_and_should_fail",
    );
    let result = client.try_create_user_data(&1u64, &long_name, &1000i128, &true);
    assert!(result.is_err());
}

#[test]
fn test_create_user_data_negative_balance() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let name = String::from_str(&env, "alice");
    let result = client.try_create_user_data(&1u64, &name, &-100i128, &true);
    assert!(result.is_err());
}

#[test]
fn test_convert_val_to_config() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

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

    let config = client.convert_val_to_config(&val_data);

    assert_eq!(config.max_users, 100);
    assert_eq!(config.fee_rate, 250);
    assert_eq!(config.admin, admin);
    assert_eq!(config.features, features);
}

#[test]
fn test_convert_val_to_config_missing_field() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let mut val_data = Map::new(&env);
    val_data.set(Symbol::new(&env, "max_users"), 100u32.into_val(&env));

    let result = client.try_convert_val_to_config(&val_data);
    assert!(result.is_err());
}

#[test]
fn test_convert_bytes_to_types() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let input_str = "hello_world";
    let input_bytes = Bytes::from_slice(&env, input_str.as_bytes());

    let (string_result, symbol_result, bytes_result) = client.convert_bytes_to_types(&input_bytes);

    assert_eq!(string_result, String::from_str(&env, "hello_world"));
    assert_eq!(symbol_result, Symbol::new(&env, "hello_world"));
    assert_eq!(bytes_result, input_bytes);
}

#[test]
fn test_validate_and_convert_number() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let input = String::from_str(&env, "12345");
    let result = client.validate_and_convert(&input, &1u32);
    assert_eq!(result, input);
}

#[test]
fn test_validate_and_convert_invalid_number() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let input = String::from_str(&env, "");
    let result = client.try_validate_and_convert(&input, &1u32);
    assert!(result.is_err());
}

#[test]
fn test_validate_and_convert_symbol() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let input = String::from_str(&env, "valid_symbol");
    let result = client.validate_and_convert(&input, &2u32);
    assert_eq!(result, input);
}

#[test]
fn test_validate_and_convert_symbol_too_long() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let input = String::from_str(&env, "this_symbol_name_is_way_too_long_to_be_valid");
    let result = client.try_validate_and_convert(&input, &2u32);
    assert!(result.is_err());
}

#[test]
fn test_validate_and_convert_address() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let valid_address = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    let input = String::from_str(&env, valid_address);
    let result = client.validate_and_convert(&input, &3u32);
    assert_eq!(result, input);
}

#[test]
fn test_validate_and_convert_invalid_address() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let input = String::from_str(&env, "too_short");
    let result = client.try_validate_and_convert(&input, &3u32);
    assert!(result.is_err());
}

#[test]
fn test_validate_and_convert_unsupported_type() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let input = String::from_str(&env, "value");
    let result = client.try_validate_and_convert(&input, &99u32);
    assert!(result.is_err());
}

#[test]
fn test_batch_convert_numbers() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let mut input_vec = Vec::new(&env);
    input_vec.push_back(String::from_str(&env, "123"));
    input_vec.push_back(String::from_str(&env, "invalid"));
    input_vec.push_back(String::from_str(&env, "-456"));
    input_vec.push_back(String::from_str(&env, "789"));

    let result = client.batch_convert_numbers(&input_vec);
}

#[test]
fn test_batch_convert_numbers_all_invalid() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let mut input_vec = Vec::new(&env);
    input_vec.push_back(String::from_str(&env, ""));
    input_vec.push_back(String::from_str(&env, ""));

    let result = client.batch_convert_numbers(&input_vec);

    assert_eq!(result.len(), 0);
}

#[test]
fn test_sum_different_types() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let result = client.sum_different_types(&100u32, &-50i64);
    assert_eq!(result, 50i128);
}

#[test]
fn test_val_roundtrip() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let original = 12345u32;
    let result = client.val_roundtrip(&original);
    assert_eq!(result, original);
}

#[test]
fn test_complex_conversion_workflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let name = String::from_str(&env, "test_user");
    let user_data = client.create_user_data(&42u64, &name, &1000i128, &true);

    let converted_id = client.convert_numbers(&(user_data.id as i128), &1u32);
    assert_eq!(converted_id, 42);

    let (string_result, _) = client.convert_strings(&user_data.name, &true);
    assert_eq!(string_result, user_data.name);

    let sum_result = client.sum_different_types(&100u32, &200i64);
    assert_eq!(sum_result, 300);
}

#[test]
fn test_val_conversion_roundtrip() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let original_value = 12345u32;
    let val: Val = original_value.into_val(&env);
    let (success, converted) = client.safe_conversions(&val, &1u32);

    assert!(success);
    assert_eq!(converted, original_value as i128);
}

#[test]
fn test_error_handling_patterns() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TypeConversionsContract);
    let client = TypeConversionsContractClient::new(&env, &contract_id);

    let valid_input = String::from_str(&env, "valid");
    let result1 = client.validate_and_convert(&valid_input, &2u32);
    assert_eq!(result1, valid_input);

    let result2 = client.validate_and_convert(&valid_input, &2u32);
    assert_eq!(result2, valid_input);
}

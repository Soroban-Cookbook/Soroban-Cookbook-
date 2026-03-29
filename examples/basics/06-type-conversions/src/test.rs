//! Test suite for the Type Conversions contract.
//!
//! Tests use the generated contract client so the full host dispatch path is
//! exercised, matching the pattern used across the rest of the cookbook.

#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short, testutils::Address as _, Address, Bytes, Env, IntoVal, Map, String, Symbol, Vec,
};

fn setup(env: &Env) -> TypeConversionsContractClient {
    let id = env.register_contract(None, TypeConversionsContract);
    TypeConversionsContractClient::new(env, &id)
}

// ── convert_numbers ───────────────────────────────────────────────────────────

#[test]
fn test_convert_numbers_success() {
    let env = Env::default();
    let client = setup(&env);
    assert_eq!(client.convert_numbers(&42, &1), 42);
    assert_eq!(client.convert_numbers(&-1000, &2), -1000);
    assert_eq!(client.convert_numbers(&1_000_000, &3), 1_000_000);
}

#[test]
#[should_panic(expected = "NumericOverflow")]
fn test_convert_numbers_overflow() {
    let env = Env::default();
    setup(&env).convert_numbers(&i128::MAX, &1);
}

#[test]
#[should_panic(expected = "NumericOverflow")]
fn test_convert_numbers_negative_to_unsigned() {
    let env = Env::default();
    setup(&env).convert_numbers(&-100, &3);
}

#[test]
#[should_panic(expected = "UnsupportedConversion")]
fn test_convert_numbers_unsupported_type() {
    let env = Env::default();
    setup(&env).convert_numbers(&42, &99);
}

// ── convert_strings ───────────────────────────────────────────────────────────

#[test]
fn test_convert_strings_to_symbol() {
    let env = Env::default();
    let client = setup(&env);
    let input = String::from_str(&env, "hello");
    let (s, sym) = client.convert_strings(&input, &true);
    assert_eq!(s, input);
    assert_eq!(sym, Symbol::new(&env, "hello"));
}

#[test]
fn test_convert_strings_from_symbol() {
    let env = Env::default();
    let client = setup(&env);
    let input = String::from_str(&env, "hello");
    let (s, _) = client.convert_strings(&input, &false);
    assert_eq!(s, String::from_str(&env, "hello"));
}

#[test]
#[should_panic(expected = "InvalidStringFormat")]
fn test_convert_strings_too_long() {
    let env = Env::default();
    // 33 characters — exceeds Symbol limit of 32
    let long = String::from_str(&env, "this_string_is_thirty_three_chars_!");
    setup(&env).convert_strings(&long, &true);
}

// ── convert_collections ───────────────────────────────────────────────────────

#[test]
fn test_convert_collections() {
    let env = Env::default();
    let client = setup(&env);
    let mut input = Vec::new(&env);
    input.push_back(1i32);
    input.push_back(-2i32);
    input.push_back(100i32);
    let result = client.convert_collections(&input);
    assert_eq!(result.len(), 3);
    assert_eq!(result.get(0).unwrap(), 1i64);
    assert_eq!(result.get(1).unwrap(), -2i64);
    assert_eq!(result.get(2).unwrap(), 100i64);
}

#[test]
fn test_convert_collections_empty() {
    let env = Env::default();
    let input: Vec<i32> = Vec::new(&env);
    assert_eq!(setup(&env).convert_collections(&input).len(), 0);
}

// ── safe_conversions ──────────────────────────────────────────────────────────

#[test]
fn test_safe_conversions_success() {
    let env = Env::default();
    let client = setup(&env);

    let (ok, v) = client.safe_conversions(&42u32.into_val(&env), &1);
    assert!(ok);
    assert_eq!(v, 42);

    let (ok, v) = client.safe_conversions(&(-1000i64).into_val(&env), &2);
    assert!(ok);
    assert_eq!(v, -1000);

    let (ok, v) = client.safe_conversions(&true.into_val(&env), &3);
    assert!(ok);
    assert_eq!(v, 1);

    let (ok, v) = client.safe_conversions(&false.into_val(&env), &3);
    assert!(ok);
    assert_eq!(v, 0);
}

#[test]
fn test_safe_conversions_type_mismatch() {
    let env = Env::default();
    let client = setup(&env);
    let val = String::from_str(&env, "not_a_number").into_val(&env);
    let (ok, v) = client.safe_conversions(&val, &1);
    assert!(!ok);
    assert_eq!(v, 0);
}

#[test]
fn test_safe_conversions_unsupported_type() {
    let env = Env::default();
    let (ok, v) = setup(&env).safe_conversions(&42u32.into_val(&env), &99);
    assert!(!ok);
    assert_eq!(v, -1);
}

// ── create_user_data ──────────────────────────────────────────────────────────

#[test]
fn test_create_user_data_success() {
    let env = Env::default();
    let client = setup(&env);
    let name = String::from_str(&env, "alice");
    let user = client.create_user_data(&1u64, &name, &1000i128, &true);
    assert_eq!(user.id, 1);
    assert_eq!(user.name, name);
    assert_eq!(user.balance, 1000);
    assert!(user.active);
}

#[test]
#[should_panic(expected = "InvalidStringFormat")]
fn test_create_user_data_name_too_long() {
    let env = Env::default();
    let long = String::from_str(&env, "this_name_is_way_too_long_for_a_symbol_and_should_fail");
    setup(&env).create_user_data(&1u64, &long, &1000i128, &true);
}

#[test]
#[should_panic(expected = "NumericOverflow")]
fn test_create_user_data_negative_balance() {
    let env = Env::default();
    let name = String::from_str(&env, "alice");
    setup(&env).create_user_data(&1u64, &name, &-100i128, &true);
}

// ── convert_val_to_config ─────────────────────────────────────────────────────

#[test]
fn test_convert_val_to_config() {
    let env = Env::default();
    let client = setup(&env);

    let admin = Address::generate(&env);
    let mut features = Vec::new(&env);
    features.push_back(symbol_short!("feat1"));
    features.push_back(symbol_short!("feat2"));

    let mut map = Map::new(&env);
    map.set(Symbol::new(&env, "max_users"), 100u32.into_val(&env));
    map.set(Symbol::new(&env, "fee_rate"), 250u64.into_val(&env));
    map.set(Symbol::new(&env, "admin"), admin.clone().into_val(&env));
    map.set(
        Symbol::new(&env, "features"),
        features.clone().into_val(&env),
    );

    let config = client.convert_val_to_config(&map);
    assert_eq!(config.max_users, 100);
    assert_eq!(config.fee_rate, 250);
    assert_eq!(config.admin, admin);
    assert_eq!(config.features, features);
}

#[test]
#[should_panic(expected = "UnsupportedConversion")]
fn test_convert_val_to_config_missing_field() {
    let env = Env::default();
    let mut map = Map::new(&env);
    map.set(Symbol::new(&env, "max_users"), 100u32.into_val(&env));
    setup(&env).convert_val_to_config(&map);
}

// ── convert_bytes_to_types ────────────────────────────────────────────────────

#[test]
fn test_convert_bytes_to_types() {
    let env = Env::default();
    let client = setup(&env);
    let input_bytes = Bytes::from_slice(&env, b"hello_world");
    let (s, sym, bytes_out) = client.convert_bytes_to_types(&input_bytes);
    assert_eq!(s, String::from_str(&env, "hello_world"));
    assert_eq!(sym, Symbol::new(&env, "hello_world"));
    assert_eq!(bytes_out, input_bytes);
}

// ── validate_and_convert ──────────────────────────────────────────────────────

#[test]
fn test_validate_and_convert_number() {
    let env = Env::default();
    let input = String::from_str(&env, "12345");
    let result = setup(&env).validate_and_convert(&input, &1);
    assert_eq!(result, input);
}

#[test]
#[should_panic(expected = "InvalidStringFormat")]
fn test_validate_and_convert_empty_number() {
    let env = Env::default();
    setup(&env).validate_and_convert(&String::from_str(&env, ""), &1);
}

#[test]
fn test_validate_and_convert_symbol() {
    let env = Env::default();
    let input = String::from_str(&env, "valid_symbol");
    let result = setup(&env).validate_and_convert(&input, &2);
    assert_eq!(result, input);
}

#[test]
#[should_panic(expected = "InvalidStringFormat")]
fn test_validate_and_convert_symbol_too_long() {
    let env = Env::default();
    let long = String::from_str(&env, "this_symbol_name_is_way_too_long_to_be_valid");
    setup(&env).validate_and_convert(&long, &2);
}

#[test]
fn test_validate_and_convert_address() {
    let env = Env::default();
    // 56-character Stellar G-address
    let addr = String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    let result = setup(&env).validate_and_convert(&addr, &3);
    assert_eq!(result, addr);
}

#[test]
#[should_panic(expected = "InvalidAddress")]
fn test_validate_and_convert_invalid_address() {
    let env = Env::default();
    setup(&env).validate_and_convert(&String::from_str(&env, "too_short"), &3);
}

#[test]
#[should_panic(expected = "UnsupportedConversion")]
fn test_validate_and_convert_unsupported_type() {
    let env = Env::default();
    setup(&env).validate_and_convert(&String::from_str(&env, "value"), &99);
}

// ── batch_convert_numbers ─────────────────────────────────────────────────────

#[test]
fn test_batch_convert_numbers_mixed() {
    let env = Env::default();
    let client = setup(&env);

    let mut input = Vec::new(&env);
    input.push_back(String::from_str(&env, "123"));
    input.push_back(String::from_str(&env, "invalid"));
    input.push_back(String::from_str(&env, "-456"));
    input.push_back(String::from_str(&env, "789"));

    let result = client.batch_convert_numbers(&input);
    // "invalid" is skipped; the three numeric strings are converted
    assert_eq!(result.len(), 3);
    assert_eq!(result.get(0).unwrap(), 123i64);
    assert_eq!(result.get(1).unwrap(), -456i64);
    assert_eq!(result.get(2).unwrap(), 789i64);
}

#[test]
fn test_batch_convert_numbers_all_invalid() {
    let env = Env::default();
    let client = setup(&env);

    let mut input = Vec::new(&env);
    input.push_back(String::from_str(&env, ""));
    input.push_back(String::from_str(&env, "abc"));
    input.push_back(String::from_str(&env, "-"));

    assert_eq!(client.batch_convert_numbers(&input).len(), 0);
}

#[test]
fn test_batch_convert_numbers_empty_input() {
    let env = Env::default();
    let input: Vec<String> = Vec::new(&env);
    assert_eq!(setup(&env).batch_convert_numbers(&input).len(), 0);
}

// ── sum_different_types ───────────────────────────────────────────────────────

#[test]
fn test_sum_different_types() {
    let env = Env::default();
    let client = setup(&env);
    assert_eq!(client.sum_different_types(&100u32, &-50i64), 50i128);
    assert_eq!(client.sum_different_types(&0u32, &0i64), 0i128);
    assert_eq!(
        client.sum_different_types(&u32::MAX, &0i64),
        u32::MAX as i128
    );
}

// ── val_roundtrip ─────────────────────────────────────────────────────────────

#[test]
fn test_val_roundtrip() {
    let env = Env::default();
    let client = setup(&env);
    assert_eq!(client.val_roundtrip(&12345u32), 12345u32);
    assert_eq!(client.val_roundtrip(&0u32), 0u32);
    assert_eq!(client.val_roundtrip(&u32::MAX), u32::MAX);
}

// ── integration ───────────────────────────────────────────────────────────────

#[test]
fn test_val_conversion_roundtrip_via_safe_conversions() {
    let env = Env::default();
    let client = setup(&env);
    let val = 12345u32.into_val(&env);
    let (ok, v) = client.safe_conversions(&val, &1);
    assert!(ok);
    assert_eq!(v, 12345i128);
}

#[test]
fn test_complex_conversion_workflow() {
    let env = Env::default();
    let client = setup(&env);

    let name = String::from_str(&env, "test_user");
    let user = client.create_user_data(&42u64, &name, &1000i128, &true);
    assert_eq!(user.id, 42);

    assert_eq!(client.convert_numbers(&(user.id as i128), &1), 42);
    assert_eq!(client.sum_different_types(&100u32, &200i64), 300i128);
    assert_eq!(client.val_roundtrip(&42u32), 42u32);
}

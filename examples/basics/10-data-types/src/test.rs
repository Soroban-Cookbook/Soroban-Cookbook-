//! Unit tests for the Data Types contract.

use super::*;
use soroban_sdk::{symbol_short, vec, Bytes, BytesN, Env, String, Symbol};

// ============================================================================
// PRIMITIVE TYPE TESTS
// ============================================================================

#[test]
fn test_store_u32() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let value = 42u32;
    let result = client.store_u32(&value);

    assert_eq!(result, value);
}

#[test]
fn test_store_u64() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let value = 1_000_000_000_000u64;
    let result = client.store_u64(&value);

    assert_eq!(result, value);
}

#[test]
fn test_store_i128() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let value = -123_456_789i128;
    let result = client.store_i128(&value);

    assert_eq!(result, value);
}

#[test]
fn test_safe_add_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let result = client.safe_add(&100i128, &200i128);

    assert_eq!(result, 300i128);
}

#[test]
#[should_panic(expected = "Arithmetic overflow")]
fn test_safe_add_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    // This should panic due to overflow
    let _ = client.safe_add(&i128::MAX, &1i128);
}

// ============================================================================
// TEXT TYPE TESTS
// ============================================================================

#[test]
fn test_store_symbol() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let symbol = symbol_short!("USDC");
    let result = client.store_symbol(&symbol);

    assert_eq!(result, symbol);
}

#[test]
fn test_store_string() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let text = String::from_str(&env, "Hello, Soroban!");
    let result = client.store_string(&text);

    assert_eq!(result, text);
}

#[test]
fn test_create_symbol() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let result = client.create_symbol();

    assert_eq!(result, symbol_short!("token"));
}

#[test]
fn test_create_string() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let result = client.create_string();
    let expected = String::from_str(&env, "Hello, Soroban!");

    assert_eq!(result, expected);
}

// ============================================================================
// BINARY TYPE TESTS
// ============================================================================

#[test]
fn test_store_bytes() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let data = Bytes::from_slice(&env, b"test data");
    let result = client.store_bytes(&data);

    assert_eq!(result, data);
}

#[test]
fn test_store_bytes32() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let hash = BytesN::<32>::from_array(&env, &[1u8; 32]);
    let result = client.store_bytes32(&hash);

    assert_eq!(result, hash);
}

#[test]
fn test_create_bytes32() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let result = client.create_bytes32();
    let expected = BytesN::<32>::from_array(&env, &[0u8; 32]);

    assert_eq!(result, expected);
}

// ============================================================================
// ADDRESS TYPE TESTS
// ============================================================================

#[test]
fn test_store_address() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let addr = Address::generate(&env);
    let result = client.store_address(&addr);

    assert_eq!(result, addr);
}

#[test]
fn test_get_contract_address() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let result = client.get_contract_address();

    assert_eq!(result, contract_id);
}

#[test]
fn test_addresses_equal_same() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let addr = Address::generate(&env);
    let result = client.addresses_equal(&addr, &addr);

    assert!(result);
}

#[test]
fn test_addresses_equal_different() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);
    let result = client.addresses_equal(&addr1, &addr2);

    assert!(!result);
}

// ============================================================================
// COLLECTION TYPE TESTS
// ============================================================================

#[test]
fn test_store_vec() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let values = vec![&env, 10i128, 20i128, 30i128];
    let result = client.store_vec(&values);

    assert_eq!(result.len(), 3);
    assert_eq!(result.get(0).unwrap(), 10i128);
    assert_eq!(result.get(1).unwrap(), 20i128);
    assert_eq!(result.get(2).unwrap(), 30i128);
}

#[test]
fn test_create_vec() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let result = client.create_vec();

    assert_eq!(result.len(), 3);
    assert_eq!(result.get(0).unwrap(), 1i128);
    assert_eq!(result.get(1).unwrap(), 2i128);
    assert_eq!(result.get(2).unwrap(), 3i128);
}

#[test]
fn test_vec_length() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let values = vec![&env, 1i128, 2i128, 3i128, 4i128, 5i128];
    let result = client.vec_length(&values);

    assert_eq!(result, 5);
}

#[test]
fn test_vec_get() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let values = vec![&env, 100i128, 200i128, 300i128];
    let result = client.vec_get(&values, &1u32);

    assert_eq!(result, 200i128);
}

#[test]
#[should_panic(expected = "Index out of bounds")]
fn test_vec_get_out_of_bounds() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let values = vec![&env, 100i128, 200i128];
    let _ = client.vec_get(&values, &10u32);
}

#[test]
fn test_create_map() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let result = client.create_map();

    assert_eq!(result.len(), 2);
    assert_eq!(result.get(symbol_short!("count")).unwrap(), 42i128);
    assert_eq!(result.get(symbol_short!("balance")).unwrap(), 1000i128);
}

#[test]
fn test_map_get() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let map = client.create_map();
    let result = client.map_get(&map, &symbol_short!("count"));

    assert_eq!(result, 42i128);
}

#[test]
#[should_panic(expected = "Key not found in map")]
fn test_map_get_missing_key() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let map = client.create_map();
    let _ = client.map_get(&map, &symbol_short!("missing"));
}

// ============================================================================
// TYPE CONVERSION TESTS
// ============================================================================

#[test]
fn test_bytesn_to_bytes() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let fixed = BytesN::<32>::from_array(&env, &[42u8; 32]);
    let result = client.bytesn_to_bytes(&fixed);

    assert_eq!(result.len(), 32);
    assert_eq!(result.get(0).unwrap(), 42u8);
}

#[test]
fn test_bytes_to_bytesn() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let data = Bytes::from_slice(&env, &[99u8; 32]);
    let result = client.bytes_to_bytesn(&data);

    assert_eq!(result, BytesN::<32>::from_array(&env, &[99u8; 32]));
}

#[test]
fn test_create_symbol_from_literal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let result = client.create_symbol_from_literal();

    assert_eq!(result, Symbol::new(&env, "token"));
}

#[test]
fn test_create_string_from_literal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &contract_id);

    let result = client.create_string_from_literal();
    let expected = String::from_str(&env, "Hello, Soroban!");

    assert_eq!(result, expected);
}

// ============================================================================
// STORAGE ROUND-TRIP TESTS
// ============================================================================

#[test]
fn test_storage_roundtrip_u32() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    client.put_u32(&99u32);
    assert_eq!(client.get_u32(), 99u32);
}

#[test]
fn test_storage_roundtrip_i128() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    client.put_i128(&-999_999i128);
    assert_eq!(client.get_i128(), -999_999i128);
}

#[test]
fn test_storage_roundtrip_symbol() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    let sym = symbol_short!("USDC");
    client.put_symbol(&sym);
    assert_eq!(client.get_symbol(), sym);
}

#[test]
fn test_storage_roundtrip_vec() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    let values = vec![&env, 10i128, 20i128, 30i128];
    client.put_vec(&values);
    assert_eq!(client.get_vec(), values);
}

#[test]
fn test_storage_default_u32_is_zero() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    // Nothing stored yet — should return default 0
    assert_eq!(client.get_u32(), 0u32);
}

#[test]
fn test_storage_overwrite_value() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    client.put_i128(&100i128);
    client.put_i128(&200i128);
    assert_eq!(client.get_i128(), 200i128);
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_store_u32_zero() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    assert_eq!(client.store_u32(&0u32), 0u32);
}

#[test]
fn test_store_u32_max() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    assert_eq!(client.store_u32(&u32::MAX), u32::MAX);
}

#[test]
fn test_store_i128_min() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    assert_eq!(client.store_i128(&i128::MIN), i128::MIN);
}

#[test]
fn test_store_i128_max() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    assert_eq!(client.store_i128(&i128::MAX), i128::MAX);
}

#[test]
fn test_vec_empty() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    let empty: soroban_sdk::Vec<i128> = vec![&env];
    assert_eq!(client.vec_length(&empty), 0u32);
}

#[test]
fn test_bytes_empty() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    let empty = soroban_sdk::Bytes::new(&env);
    let result = client.store_bytes(&empty);
    assert_eq!(result.len(), 0u32);
}

#[test]
fn test_safe_add_zero() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    assert_eq!(client.safe_add(&0i128, &0i128), 0i128);
}

#[test]
fn test_safe_add_negative() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    assert_eq!(client.safe_add(&-50i128, &-50i128), -100i128);
}

#[test]
#[should_panic(expected = "Bytes must be exactly 32 bytes")]
fn test_bytes_to_bytesn_wrong_size() {
    let env = Env::default();
    let id = env.register_contract(None, DataTypesContract);
    let client = DataTypesContractClient::new(&env, &id);

    let short = soroban_sdk::Bytes::from_slice(&env, &[1u8; 16]);
    let _ = client.bytes_to_bytesn(&short);
}

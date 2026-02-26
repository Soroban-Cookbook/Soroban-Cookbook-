//! Comprehensive tests for Soroban Types demonstration
//!
//! This test suite validates the behavior of all Soroban-specific types:
//! - Address operations and storage
//! - Bytes and BytesN manipulations  
//! - Symbol creation and comparisons
//! - String handling and conversions

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, symbol_short, BytesN};

// ---------------------------------------------------------------------------
// Address Type Tests
// ---------------------------------------------------------------------------

#[test]
fn test_address_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    
    // Store address
    client.store_address(&owner);
    
    // Retrieve and verify
    let retrieved = client.get_address();
    assert_eq!(retrieved, owner, "Stored address should match retrieved address");
}

#[test]
fn test_address_equality() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);
    
    // Test inequality
    assert!(!client.verify_address(&addr1, &addr2), "Different addresses should not be equal");
    
    // Test equality
    assert!(client.verify_address(&addr1, &addr1), "Same address should be equal");
}

#[test]
fn test_address_generation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let addr1 = client.generate_address();
    let addr2 = client.generate_address();
    
    // Generated addresses should be different
    assert_ne!(addr1, addr2, "Generated addresses should be unique");
}

// ---------------------------------------------------------------------------
// Bytes Type Tests
// ---------------------------------------------------------------------------

#[test]
fn test_bytes_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let data = Bytes::from_slice(&env, b"Hello, Soroban!");
    
    // Store bytes
    client.store_bytes(&data.clone());
    
    // Retrieve and verify
    let retrieved = client.get_bytes();
    assert_eq!(retrieved, data, "Stored bytes should match retrieved bytes");
}

#[test]
fn test_bytes_from_slice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let input = "Test data";
    let bytes = client.create_bytes_from_slice(input);
    
    // Verify bytes content
    let expected = Bytes::from_slice(&env, input.as_bytes());
    assert_eq!(bytes, expected, "Bytes from slice should match expected");
}

#[test]
fn test_bytes_length() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let data = Bytes::from_slice(&env, b"12345");
    let length = client.get_bytes_length(data);
    
    assert_eq!(length, 5, "Bytes length should be 5");
}

// ---------------------------------------------------------------------------
// BytesN Type Tests
// ---------------------------------------------------------------------------

#[test]
fn test_fixed_bytes_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let data = BytesN::from_array(&env, &[1; 32]);
    
    // Store fixed bytes
    client.store_fixed_bytes(&data);
    
    // Retrieve and verify
    let retrieved = client.get_fixed_bytes();
    assert_eq!(retrieved, data, "Stored fixed bytes should match retrieved");
}

#[test]
fn test_hash_bytes_creation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let hash1 = client.create_hash_bytes(1);
    let hash2 = client.create_hash_bytes(2);
    
    // Different seeds should produce different hashes
    assert_ne!(hash1, hash2, "Different seeds should produce different hashes");
    
    // Same seed should produce same hash
    let hash1_again = client.create_hash_bytes(1);
    assert_eq!(hash1, hash1_again, "Same seed should produce same hash");
}

#[test]
fn test_fixed_to_variable_bytes_conversion() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let fixed = BytesN::from_array(&env, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
    let variable = client.fixed_to_variable_bytes(fixed);
    
    let expected = Bytes::from_slice(&env, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
    assert_eq!(variable, expected, "Converted bytes should match expected");
}

// ---------------------------------------------------------------------------
// Symbol Type Tests
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let symbol = symbol_short!("token");
    
    // Store symbol
    client.store_symbol(&symbol);
    
    // Retrieve and verify
    let retrieved = client.get_symbol();
    assert_eq!(retrieved, symbol, "Stored symbol should match retrieved");
}

#[test]
fn test_symbol_creation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let symbol = client.create_symbol("my_token");
    let expected = symbol_short!("my_token");
    
    assert_eq!(symbol, expected, "Created symbol should match expected");
}

#[test]
fn test_symbol_comparison() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let sym1 = symbol_short!("token");
    let sym2 = symbol_short!("coin");
    
    // Test inequality
    assert!(!client.compare_symbols(sym1, sym2), "Different symbols should not be equal");
    
    // Test equality
    assert!(client.compare_symbols(sym1, sym1), "Same symbol should be equal");
}

#[test]
fn test_symbol_to_string() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let symbol = symbol_short!("hello");
    let string = client.symbol_to_string(symbol);
    let expected = String::from_str(&env, "hello");
    
    assert_eq!(string, expected, "Symbol to string conversion should work");
}

// ---------------------------------------------------------------------------
// String Type Tests
// ---------------------------------------------------------------------------

#[test]
fn test_string_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let text = String::from_str(&env, "Hello, Soroban World!");
    
    // Store string
    client.store_string(&text.clone());
    
    // Retrieve and verify
    let retrieved = client.get_string();
    assert_eq!(retrieved, text, "Stored string should match retrieved");
}

#[test]
fn test_string_creation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let string = client.create_string("test_string");
    let expected = String::from_str(&env, "test_string");
    
    assert_eq!(string, expected, "Created string should match expected");
}

#[test]
fn test_string_length() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let text = String::from_str(&env, "Hello, World!");
    let length = client.get_string_length(text);
    
    assert_eq!(length, 13, "String length should be 13");
}

#[test]
fn test_string_concatenation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let str1 = String::from_str(&env, "Hello");
    let str2 = String::from_str(&env, " Soroban");
    let concatenated = client.concatenate_strings(str1, str2);
    
    let expected = String::from_str(&env, "Hello Soroban");
    assert_eq!(concatenated, expected, "Concatenated string should match expected");
}

#[test]
fn test_string_to_symbol_conversion() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let string = String::from_str(&env, "token");
    let symbol = client.string_to_symbol(string);
    let expected = symbol_short!("token");
    
    assert_eq!(symbol, expected, "String to symbol conversion should work");
}

// ---------------------------------------------------------------------------
// Cross-Type Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_type_conversion_demo() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    // Run type conversion demo
    client.type_conversion_demo();
    
    // Verify the conversions worked
    let symbol: Symbol = env.storage()
        .instance()
        .get(&symbol_short!("symbol_from_string"))
        .unwrap();
    assert_eq!(symbol, symbol_short!("token"), "Symbol should be 'token'");
    
    let string: String = env.storage()
        .instance()
        .get(&symbol_short!("original_string"))
        .unwrap();
    assert_eq!(string, String::from_str(&env, "token"), "String should be 'token'");
}

#[test]
fn test_user_profile_creation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let username = String::from_str(&env, "alice");
    let bio = String::from_str(&env, "Blockchain developer");
    let avatar = BytesN::from_array(&env, &[1; 32]);
    
    let score = client.create_user_profile(user, username, bio, avatar);
    
    // Score should be username length + bio length
    assert_eq!(score, 5 + 19, "Profile score should be 24");
    
    // Verify stored data
    let stored_username: String = env.storage()
        .instance()
        .get(&symbol_short!("username"))
        .unwrap();
    assert_eq!(stored_username, username, "Stored username should match");
    
    let stored_status: Symbol = env.storage()
        .instance()
        .get(&symbol_short!("user_status"))
        .unwrap();
    assert_eq!(stored_status, symbol_short!("active"), "User status should be 'active'");
}

#[test]
fn test_type_validation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let valid_addr = Address::generate(&env);
    let valid_sym = symbol_short!("valid");
    let valid_text = String::from_str(&env, "valid text");
    
    // Should pass validation
    assert!(client.validate_types(valid_addr, valid_sym, valid_text), "Valid types should pass");
    
    // Test with empty symbol (this would fail in a real scenario)
    let empty_sym = Symbol::new(&env, "");
    assert!(!client.validate_types(valid_addr, empty_sym, valid_text), "Empty symbol should fail");
    
    // Test with too long string
    let long_text = String::from_str(&env, &"a".repeat(1001));
    assert!(!client.validate_types(valid_addr, valid_sym, long_text), "Too long string should fail");
}

// ---------------------------------------------------------------------------
// Performance and Edge Case Tests
// ---------------------------------------------------------------------------

#[test]
fn test_large_bytes_handling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    // Create large bytes data (1KB)
    let large_data = vec![42u8; 1024];
    let bytes = Bytes::from_slice(&env, &large_data);
    
    client.store_bytes(&bytes);
    let retrieved = client.get_bytes();
    
    assert_eq!(retrieved, bytes, "Large bytes should be stored and retrieved correctly");
    assert_eq!(client.get_bytes_length(retrieved), 1024, "Large bytes length should be correct");
}

#[test]
fn test_symbol_edge_cases() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    // Test maximum length symbol (9 characters)
    let max_symbol = client.create_symbol("123456789");
    assert_eq!(max_symbol.to_string(&env).len(), 9, "Max symbol length should be 9");
    
    // Test single character symbol
    let single_symbol = client.create_symbol("a");
    assert_eq!(single_symbol.to_string(&env).len(), 1, "Single character symbol should work");
}

#[test]
fn test_bytesn_fixed_size_constraints() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    // Test that BytesN<32> always has 32 bytes
    let hash1 = client.create_hash_bytes(1);
    let hash2 = client.create_hash_bytes(255);
    
    // Both should have the same size (32 bytes)
    let converted1 = client.fixed_to_variable_bytes(hash1);
    let converted2 = client.fixed_to_variable_bytes(hash2);
    
    assert_eq!(client.get_bytes_length(converted1), 32, "BytesN<32> should always be 32 bytes");
    assert_eq!(client.get_bytes_length(converted2), 32, "BytesN<32> should always be 32 bytes");
    
    // But different content
    assert_ne!(hash1, hash2, "Different seeds should produce different hashes");
}

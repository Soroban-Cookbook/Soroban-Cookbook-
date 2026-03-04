//! Tests for Soroban Types demonstration.

#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Bytes, BytesN, String, Symbol};

// ---------------------------------------------------------------------------
// Address Tests
// ---------------------------------------------------------------------------

#[test]
fn test_address_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.store_address(&owner);
    assert_eq!(client.get_address(), owner);
}

#[test]
fn test_address_equality() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);

    assert!(!client.verify_address(&addr1, &addr2));
    assert!(client.verify_address(&addr1, &addr1));
}

#[test]
fn test_get_contract_address_returns_address() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    // Should return the contract's own address without panicking.
    let addr = client.get_contract_address();
    assert_eq!(addr, contract_id);
}

// ---------------------------------------------------------------------------
// Bytes Tests
// ---------------------------------------------------------------------------

#[test]
fn test_bytes_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let data = Bytes::from_slice(&env, b"Hello, Soroban!");
    client.store_bytes(&data);
    assert_eq!(client.get_bytes(), data);
}

#[test]
fn test_echo_bytes() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let input = Bytes::from_slice(&env, b"Test data");
    let result = client.echo_bytes(&input);
    assert_eq!(result, input);
}

#[test]
fn test_bytes_length() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let data = Bytes::from_slice(&env, b"12345");
    assert_eq!(client.get_bytes_length(&data), 5);
}

// ---------------------------------------------------------------------------
// BytesN Tests
// ---------------------------------------------------------------------------

#[test]
fn test_fixed_bytes_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let data = BytesN::from_array(&env, &[1; 32]);
    client.store_fixed_bytes(&data);
    assert_eq!(client.get_fixed_bytes(), data);
}

#[test]
fn test_hash_bytes_determinism() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let h1 = client.create_hash_bytes(&1u32);
    let h2 = client.create_hash_bytes(&2u32);
    assert_ne!(h1, h2);
    assert_eq!(h1, client.create_hash_bytes(&1u32));
}

#[test]
fn test_fixed_to_variable_bytes_conversion() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let arr: [u8; 32] = core::array::from_fn(|i| i as u8 + 1);
    let fixed = BytesN::from_array(&env, &arr);
    let variable = client.fixed_to_variable_bytes(&fixed);
    let expected = Bytes::from_slice(&env, &arr);
    assert_eq!(variable, expected);
}

// ---------------------------------------------------------------------------
// Symbol Tests
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let sym = symbol_short!("token");
    client.store_symbol(&sym);
    assert_eq!(client.get_symbol(), sym);
}

#[test]
fn test_symbol_creation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let sym = symbol_short!("my_token");
    assert_eq!(client.create_symbol(&sym), sym);
}

#[test]
fn test_symbol_comparison() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let sym1 = symbol_short!("token");
    let sym2 = symbol_short!("coin");
    assert!(!client.compare_symbols(&sym1, &sym2));
    assert!(client.compare_symbols(&sym1, &sym1));
}

// ---------------------------------------------------------------------------
// String Tests
// ---------------------------------------------------------------------------

#[test]
fn test_string_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let text = String::from_str(&env, "Hello, Soroban World!");
    client.store_string(&text);
    assert_eq!(client.get_string(), text);
}

#[test]
fn test_string_creation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let text = String::from_str(&env, "hello");
    assert_eq!(client.create_string(&text), text);
}

#[test]
fn test_string_length() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let text = String::from_str(&env, "Hello, World!");
    assert_eq!(client.get_string_length(&text), 13);
}

#[test]
fn test_string_concatenation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let str1 = String::from_str(&env, "Hello");
    let str2 = String::from_str(&env, " Soroban");
    let result = client.concatenate_strings(&str1, &str2);
    assert_eq!(result, String::from_str(&env, "Hello Soroban"));
}

// ---------------------------------------------------------------------------
// Cross-Type Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_type_conversion_demo() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    client.type_conversion_demo();

    // Read back stored values from inside the contract context.
    env.as_contract(&contract_id, || {
        // Keys updated to stay within the 9-char symbol_short! limit.
        let sym: Symbol = env
            .storage()
            .instance()
            .get(&symbol_short!("sym_str"))
            .unwrap();
        assert_eq!(sym, symbol_short!("token"));

        let s: String = env
            .storage()
            .instance()
            .get(&symbol_short!("orig_str"))
            .unwrap();
        assert_eq!(s, String::from_str(&env, "token"));
    });
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

    let score = client.create_user_profile(&user, &username, &bio, &avatar);
    assert_eq!(score, 5 + 20); // "alice" + "Blockchain developer"

    // Read back stored values from inside the contract context.
    env.as_contract(&contract_id, || {
        let stored_username: String = env
            .storage()
            .instance()
            .get(&symbol_short!("username"))
            .unwrap();
        assert_eq!(stored_username, username);

        // "ustatus" is the key used by the contract (≤9 chars)
        let stored_status: Symbol = env
            .storage()
            .instance()
            .get(&symbol_short!("ustatus"))
            .unwrap();
        assert_eq!(stored_status, Symbol::new(&env, "active"));
    });
}

#[test]
fn test_validate_types() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let addr = Address::generate(&env);
    let sym = symbol_short!("valid");

    // Short string passes.
    let short = String::from_str(&env, "hello");
    assert!(client.validate_types(&addr, &sym, &short));

    // Validate 1000-char string passes.
    let at_limit: String = String::from_str(&env, &"a".repeat(1000));
    assert!(client.validate_types(&addr, &sym, &at_limit));
}

// ---------------------------------------------------------------------------
// BytesN fixed-size
// ---------------------------------------------------------------------------

#[test]
fn test_bytesn_always_32_bytes() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SorobanTypesContract);
    let client = SorobanTypesContractClient::new(&env, &contract_id);

    let h1 = client.fixed_to_variable_bytes(&client.create_hash_bytes(&1u32));
    let h2 = client.fixed_to_variable_bytes(&client.create_hash_bytes(&255u32));
    assert_eq!(client.get_bytes_length(&h1), 32);
    assert_eq!(client.get_bytes_length(&h2), 32);
}

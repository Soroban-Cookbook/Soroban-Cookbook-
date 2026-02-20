//! Unit tests for the Hello World contract

#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String};

/// Tests the basic functionality of the Hello World contract.
///
/// Validates that:
/// - The contract can be registered and called.
/// - The "Hello" greeting is correctly prepended to the input.
/// - The response is a Vec containing the expected strings.
#[test]
fn test_basic_invocation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let name = String::from_str(&env, "World");
    let result = client.hello(&name);

    assert_eq!(result, vec![&env, String::from_str(&env, "Hello"), name]);
}

/// Tests the contract with multiple different valid names.
///
/// Validates that the contract works consistently for various
/// standard strings.
#[test]
fn test_hello_with_different_names() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let test_names = vec![
        &env,
        String::from_str(&env, "Alice"),
        String::from_str(&env, "Bob"),
        String::from_str(&env, "Stellar"),
    ];

    for name in test_names.iter() {
        let result = client.hello(&name);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap(), String::from_str(&env, "Hello"));
        assert_eq!(result.get(1).unwrap(), name);
    }
}

/// Tests edge cases including empty strings and long strings.
///
/// Validates that:
/// - The contract handles an empty String correctly.
/// - The contract handles long strings gracefully.
#[test]
fn test_edge_cases() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    // 1. Empty string
    let empty_name = String::from_str(&env, "");
    let result_empty = client.hello(&empty_name);
    assert_eq!(result_empty.get(1).unwrap(), empty_name);

    // 2. Medium string
    let mid_string = String::from_str(&env, "123456789");
    let result_mid = client.hello(&mid_string);
    assert_eq!(result_mid.get(1).unwrap(), mid_string);

    // 3. Long string
    let long_name = String::from_str(
        &env,
        "ThisIsALongerStringThatGoesBeyondThirtyTwoCharactersIfNeeded",
    );
    let result_long = client.hello(&long_name);
    assert_eq!(result_long.get(1).unwrap(), long_name);
}

/// Tests handling of strings with special characters.
///
/// Validates that strings containing spaces or punctuation are processed correctly.
#[test]
fn test_special_characters() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    // String with space
    let name_with_space = String::from_str(&env, "Hello World");
    let result = client.hello(&name_with_space);
    assert_eq!(result.get(1).unwrap(), name_with_space);

    // String with punctuation
    let name_with_punct = String::from_str(&env, "User_123!");
    let result_punct = client.hello(&name_with_punct);
    assert_eq!(result_punct.get(1).unwrap(), name_with_punct);
}

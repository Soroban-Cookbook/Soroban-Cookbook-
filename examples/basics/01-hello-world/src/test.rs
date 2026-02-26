//! Unit tests for the Hello World contract

#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, Env, String};

/// Tests the basic functionality of the Hello World contract.
///
/// Validates that:
/// - The contract can be registered and called.
/// - The "Hello" greeting is correctly prepended to the input.
/// - The response is a Vec containing the expected strings.
#[test]
fn test_hello_returns_greeting_string() {
    // Set up the simulated blockchain environment.
    let env = Env::default();

    // Register the contract and obtain an auto-generated contract ID.
    let contract_id = env.register_contract(None, HelloContract);

    // Build a typed client so we can call contract methods directly in tests.
    let client = HelloContractClient::new(&env, &contract_id);

    // Call hello() with "World".
    let result = client.hello(&symbol_short!("World"));

    // The contract should return the full greeting string.
    assert_eq!(result, String::from_str(&env, "Hello, World!"));
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

    // Verify the greeting is correct for several different names.
    for (sym, expected) in [
        (symbol_short!("Alice"), "Hello, Alice!"),
        (symbol_short!("Bob"), "Hello, Bob!"),
        (symbol_short!("Stellar"), "Hello, Stellar!"),
    ] {
        let result = client.hello(&sym);
        assert_eq!(result, String::from_str(&env, expected));
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
fn test_hello_starts_with_hello() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    // String with space
    let name_with_space = String::from_str(&env, "Hello World");
    let result = client.hello(&name_with_space);
    assert_eq!(result.get(1).unwrap(), name_with_space);

    // Copy the response bytes into a local buffer so we can inspect them.
    let mut buf = [0u8; 40];
    let len = result.len() as usize;
    result.copy_into_slice(&mut buf[..len]);

    let result_str = core::str::from_utf8(&buf[..len]).unwrap();
    assert!(
        result_str.starts_with("Hello, "),
        "Expected greeting to begin with 'Hello, ', got: {result_str}"
    );
}

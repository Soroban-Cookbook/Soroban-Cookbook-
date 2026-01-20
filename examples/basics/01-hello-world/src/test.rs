//! Unit tests for the Hello World contract

#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, vec, Env};

#[test]
fn test_hello() {
    // Create a test environment
    // This simulates the blockchain environment for testing
    let env = Env::default();

    // Register the contract in the test environment
    // `None` means we'll get an auto-generated contract ID
    let contract_id = env.register_contract(None, HelloContract);

    // Create a client to interact with the contract
    let client = HelloContractClient::new(&env, &contract_id);

    // Call the hello function with "World" as the argument
    let words = client.hello(&symbol_short!("World"));

    // Verify the result
    // Should return a Vec containing ["Hello", "World"]
    assert_eq!(
        words,
        vec![&env, symbol_short!("Hello"), symbol_short!("World")]
    );
}

#[test]
fn test_hello_with_different_names() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    // Test with different names
    let test_names = vec![
        &env,
        symbol_short!("Alice"),
        symbol_short!("Bob"),
        symbol_short!("Stellar"),
    ];

    for name in test_names.iter() {
        let result = client.hello(&name);
        assert_eq!(result.get(0).unwrap(), symbol_short!("Hello"));
        assert_eq!(result.get(1).unwrap(), name);
    }
}

#[test]
fn test_hello_response_length() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let result = client.hello(&symbol_short!("Test"));

    // Verify the response contains exactly 2 elements
    assert_eq!(result.len(), 2);
}

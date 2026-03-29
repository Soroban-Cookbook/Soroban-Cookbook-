//! Unit tests for the Hello World contract.

use super::*;
use soroban_sdk::{symbol_short, vec, Env};

/// Basic greeting test: the returned Vec should be ["Hello", "World"].
#[test]
fn test_hello_returns_greeting_vec() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let result = client.hello(&symbol_short!("World"));

    assert_eq!(
        result,
        vec![&env, symbol_short!("Hello"), symbol_short!("World")]
    );
}

/// Verify the first element is always the literal "Hello" token.
#[test]
fn test_hello_first_element_is_hello() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let result = client.hello(&symbol_short!("Alice"));

    assert_eq!(result.get(0).unwrap(), symbol_short!("Hello"));
}

/// Verify the second element echoes back the supplied name.
#[test]
fn test_hello_second_element_is_name() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let name = symbol_short!("Bob");
    let result = client.hello(&name);

    assert_eq!(result.get(1).unwrap(), name);
}

/// Benchmark the hello function.
#[test]
fn test_hello_benchmark() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    env.budget().reset_default();
    let _result = client.hello(&symbol_short!("World"));
    env.budget().print();
}

/// The greeting with different valid symbol names.
#[test]
fn test_hello_with_different_names() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    for name in [
        symbol_short!("Alice"),
        symbol_short!("Bob"),
        symbol_short!("Dev"),
    ] {
        let result = client.hello(&name);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap(), symbol_short!("Hello"));
        assert_eq!(result.get(1).unwrap(), name);
    }
}

#![cfg(test)]
use super::*;
use soroban_sdk::{symbol_short, Env};

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ErrorHandlingContract);
    let client = ErrorHandlingContractClient::new(&env, &contract_id);

    assert_eq!(client.hello(&5), Ok(symbol_short!("Hello")));
}

#[test]
fn test_hello_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ErrorHandlingContract);
    let client = ErrorHandlingContractClient::new(&env, &contract_id);

    let result = client.try_hello(&11);
    assert_eq!(result, Err(Ok(Error::LimitExceeded)));
}

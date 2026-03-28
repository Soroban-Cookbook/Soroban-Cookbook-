//! Comprehensive Error Handling Tests
//!
//! This test suite demonstrates:
//! - Direct function calls for Result verification
//! - Client calls for success cases
//! - Client `try_` calls for error capture and comparison

use super::*;
use soroban_sdk::Env;

#[test]
fn test_divide_success() {
    assert_eq!(ErrorHandlingContract::divide(100, 10), Ok(10));
}

#[test]
fn test_divide_by_zero() {
    // Direct call returns Err
    assert_eq!(ErrorHandlingContract::divide(100, 0), Err(Error::ZeroInput));
}

#[test]
fn test_client_divide_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ErrorHandlingContract);
    let client = ErrorHandlingContractClient::new(&env, &contract_id);

    // Client call returns the success value directly
    let result = client.divide(&100, &10);
    assert_eq!(result, 10);
}

#[test]
fn test_client_try_divide_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ErrorHandlingContract);
    let client = ErrorHandlingContractClient::new(&env, &contract_id);

    // Use try_ methods to capture errors without panicking
    let result = client.try_divide(&100, &0);
    
    // The result is an InvokeError which can be compared with the custom error
    assert_eq!(result, Err(Ok(Error::ZeroInput)));
}

#[test]
fn test_check_positive_success() {
    assert_eq!(ErrorHandlingContract::check_positive(10), Ok(()));
}

#[test]
fn test_check_positive_error() {
    assert_eq!(ErrorHandlingContract::check_positive(0), Err(Error::ZeroInput));
    assert_eq!(ErrorHandlingContract::check_positive(-1), Err(Error::ZeroInput));
}

#[test]
#[should_panic(expected = "internal invariant violated: state exceeds maximum allowed value")]
fn test_invariant_check_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ErrorHandlingContract);
    let client = ErrorHandlingContractClient::new(&env, &contract_id);

    // This should panic the test runner
    client.invariant_check(&101);
}

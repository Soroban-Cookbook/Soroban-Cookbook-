//! Test suite for Custom Errors Contract
//!
//! This test file demonstrates comprehensive testing of custom error handling.
//! Each test validates specific error scenarios and ensures proper error codes
//! are returned for different failure conditions.

<<<<<<< HEAD
use soroban_sdk::{symbol_short, Address, Env, Error as StellarError, Symbol};
=======
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{symbol_short, Address, Env};
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

use crate::{ContractError, CustomErrorsContract, CustomErrorsContractClient};

#[test]
fn test_invalid_input_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    // Test with invalid input (zero value)
    let result = client.try_validate_input(&0);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::InvalidInput
        ))
    );

    // Test with negative input
    let result = client.try_validate_input(&-5);
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::InvalidInput
        ))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::InvalidInput)));

    // Test with negative input
    let result = client.try_validate_input(&-5);
    assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

    // Test with valid input succeeds
    let result = client.try_validate_input(&42);
    assert_eq!(result.unwrap(), Ok(()));
}

#[test]
fn test_unauthorized_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);

    // Test with unauthorized user
    let result = client.try_check_authorization(&unauthorized_user, &admin);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::Unauthorized
        ))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

    // Test with authorized admin
    let result = client.try_check_authorization(&admin, &admin);
    assert_eq!(result.unwrap(), Ok(()));
}

#[test]
fn test_not_found_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    let existing_key = symbol_short!("existing");
    let non_existent_key = symbol_short!("missing");

    // First, set up an existing value from inside the contract context
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&existing_key, &123u64);
    });

    // Test with non-existent key
    let result = client.try_get_value(&non_existent_key);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(ContractError::NotFound))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::NotFound)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

    // Test with existing key
    let result = client.try_get_value(&existing_key);
    assert_eq!(result.unwrap(), Ok(123u64));
}

#[test]
fn test_insufficient_balance_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    // Test with insufficient balance
    let result = client.try_transfer_tokens(&50u64, &100u64);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::InsufficientBalance
        ))
    );

    // Test with zero amount (should also return InvalidInput)
    let result = client.try_transfer_tokens(&100u64, &0u64);
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::InvalidInput
        ))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::InsufficientBalance)));

    // Test with zero amount (should also return InvalidInput)
    let result = client.try_transfer_tokens(&100u64, &0u64);
    assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

    // Test with sufficient balance
    let result = client.try_transfer_tokens(&100u64, &50u64);
    assert_eq!(result.unwrap(), Ok(()));
}

#[test]
fn test_operation_not_allowed_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    let allowed_operation = symbol_short!("allowed");
    let forbidden_operation = symbol_short!("forbidden");

    // Test with forbidden operation
    let result = client.try_perform_operation(&false, &forbidden_operation);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::OperationNotAllowed
        ))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::OperationNotAllowed)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

    // Test with allowed operation
    let result = client.try_perform_operation(&false, &allowed_operation);
    assert_eq!(result.unwrap(), Ok(()));
}

#[test]
fn test_contract_paused_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    let operation = symbol_short!("test_op");

    // Test when contract is paused
    let result = client.try_perform_operation(&true, &operation);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::ContractPaused
        ))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::ContractPaused)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

    // Test when contract is not paused
    let result = client.try_perform_operation(&false, &operation);
    assert_eq!(result.unwrap(), Ok(()));
}

#[test]
fn test_already_exists_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    // "dup_key" = 7 chars — within the 9-char symbol_short! limit
    let key = symbol_short!("dup_key");

    // Create first entry (should succeed)
    let result = client.try_create_entry(&key, &100u64);
    assert_eq!(result.unwrap(), Ok(()));

    // Try to create duplicate entry (should fail)
    let result = client.try_create_entry(&key, &200u64);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::AlreadyExists
        ))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::AlreadyExists)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

    // Test with zero value (should return InvalidInput)
    let new_key = symbol_short!("new_key");
    let result = client.try_create_entry(&new_key, &0u64);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::InvalidInput
        ))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006
}

#[test]
fn test_rate_limit_exceeded_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    let caller = Address::generate(&env);
    let max_operations = 10u32;

    // Test with rate limit exceeded
    let result = client.try_check_rate_limit(&caller, &15u32, &max_operations);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::RateLimitExceeded
        ))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::RateLimitExceeded)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

    // Test within rate limit
    let result = client.try_check_rate_limit(&caller, &5u32, &max_operations);
    assert_eq!(result.unwrap(), Ok(()));

<<<<<<< HEAD
    // Test with contract address calling itself (should return Unauthorized)
    let contract_address = env.current_contract_address();
    let result = client.try_check_rate_limit(&contract_address, &1u32, &max_operations);
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::Unauthorized
        ))
    );
=======
    // Test with the contract's own address (should return Unauthorized)
    let result = client.try_check_rate_limit(&contract_id, &1u32, &max_operations);
    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006
}

#[test]
fn test_complex_operation_multiple_errors() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Test 1: Contract paused (first check)
    let result = client.try_complex_operation(&100u64, &admin, &admin, &true);
<<<<<<< HEAD
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::ContractPaused
        ))
    );

    // Test 2: Invalid input (zero amount)
    let result = client.try_complex_operation(&0u64, &admin, &admin, &false);
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::InvalidInput
        ))
    );

    // Test 3: Unauthorized access
    let result = client.try_complex_operation(&100u64, &user, &admin, &false);
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::Unauthorized
        ))
    );

    // Test 4: Insufficient balance
    let result = client.try_complex_operation(&2000u64, &admin, &admin, &false);
    assert_eq!(
        result,
        Err(StellarError::from_contract_error(
            ContractError::InsufficientBalance
        ))
    );
=======
    assert_eq!(result, Err(Ok(ContractError::ContractPaused)));

    // Test 2: Invalid input (zero amount)
    let result = client.try_complex_operation(&0u64, &admin, &admin, &false);
    assert_eq!(result, Err(Ok(ContractError::InvalidInput)));

    // Test 3: Unauthorized access
    let result = client.try_complex_operation(&100u64, &user, &admin, &false);
    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));

    // Test 4: Insufficient balance
    let result = client.try_complex_operation(&2000u64, &admin, &admin, &false);
    assert_eq!(result, Err(Ok(ContractError::InsufficientBalance)));
>>>>>>> 3800da3163342990d44570d05ec3e367ee657006

    // Test 5: Successful operation
    let result = client.try_complex_operation(&500u64, &admin, &admin, &false);
    assert_eq!(result.unwrap(), Ok(()));
}

#[test]
fn test_error_codes() {
    // Verify that error codes are correctly assigned
    assert_eq!(ContractError::InvalidInput as u32, 1);
    assert_eq!(ContractError::Unauthorized as u32, 2);
    assert_eq!(ContractError::NotFound as u32, 3);
    assert_eq!(ContractError::InsufficientBalance as u32, 4);
    assert_eq!(ContractError::OperationNotAllowed as u32, 5);
    assert_eq!(ContractError::RateLimitExceeded as u32, 6);
    assert_eq!(ContractError::ContractPaused as u32, 7);
    assert_eq!(ContractError::AlreadyExists as u32, 8);
}

#[test]
fn test_error_display_and_debug() {
    // Test that errors can be compared
    let error = ContractError::InvalidInput;
    assert_eq!(error, ContractError::InvalidInput);
    assert_ne!(error, ContractError::Unauthorized);

    // Test error ordering (ContractError derives PartialOrd/Ord)
    assert!(ContractError::InvalidInput < ContractError::Unauthorized);
    assert!(ContractError::AlreadyExists > ContractError::ContractPaused);
}

#[test]
fn test_event_logging_with_errors() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomErrorsContract);
    let client = CustomErrorsContractClient::new(&env, &contract_id);

    // Trigger an error that should log an event
    let _ = client.try_validate_input(&0);

    // Verify that at least one event was emitted by the contract
    let events = env.events().all();
    assert!(
        !events.is_empty(),
        "Expected at least one event to be emitted"
    );
}

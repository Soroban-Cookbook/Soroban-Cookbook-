#![cfg(test)]
use super::*;
use soroban_sdk::Env;

// ========== Result<T, Error> Tests (Recommended Pattern) ==========

#[test]
fn test_transfer_success() {
    assert_eq!(ErrorHandlingContract::transfer(50, 100), Ok(50));
}

#[test]
fn test_transfer_invalid_amount() {
    assert_eq!(
        ErrorHandlingContract::transfer(0, 100),
        Err(Error::InvalidAmount)
    );
}

#[test]
fn test_transfer_insufficient_balance() {
    assert_eq!(
        ErrorHandlingContract::transfer(150, 100),
        Err(Error::InsufficientBalance)
    );
}

#[test]
fn test_divide_success() {
    assert_eq!(ErrorHandlingContract::divide(10, 2), Ok(5));
}

#[test]
fn test_divide_by_zero() {
    assert_eq!(
        ErrorHandlingContract::divide(10, 0),
        Err(Error::InvalidAmount)
    );
}

// ========== Panic Tests (Anti-pattern for validation) ==========

#[test]
#[should_panic(expected = "invalid amount")]
fn test_transfer_panic_invalid() {
    ErrorHandlingContract::transfer_panic(0, 100);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn test_transfer_panic_insufficient() {
    ErrorHandlingContract::transfer_panic(150, 100);
}

// ========== Invariant Violation Tests (Appropriate panic use) ==========

#[test]
fn test_get_verified_state_valid() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ErrorHandlingContract);
    let client = ErrorHandlingContractClient::new(&env, &contract_id);

    // Valid state (0 when not set)
    let value = client.get_verified_state(&1);
    assert_eq!(value, 0);
}

#[test]
#[should_panic(expected = "invariant violated")]
fn test_get_verified_state_corrupted() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ErrorHandlingContract);

    // Simulate corrupted state by setting invalid value in contract context
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&1u32, &2000u64);
    });

    let client = ErrorHandlingContractClient::new(&env, &contract_id);
    client.get_verified_state(&1); // Should panic
}

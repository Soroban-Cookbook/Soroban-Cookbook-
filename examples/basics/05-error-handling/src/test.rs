//! Comprehensive Error Handling Tests
//!
//! This test suite covers all aspects of error handling in Soroban contracts:
//! - Happy path tests (successful operations)
//! - Error case tests (expected failures)
//! - Error type verification (correct error types)
//! - Recovery tests (error handling and graceful degradation)

#![cfg(test)]
use super::*;
use soroban_sdk::Env;

// =========================================================================
// HAPPY PATH TESTS (Successful Operations)
// =========================================================================

#[test]
fn test_transfer_success() {
    assert_eq!(ErrorHandlingContract::transfer(50, 100), Ok(50));
}

#[test]
fn test_transfer_full_amount() {
    assert_eq!(ErrorHandlingContract::transfer(100, 100), Ok(0));
}

#[test]
fn test_transfer_minimum_valid_amount() {
    assert_eq!(ErrorHandlingContract::transfer(1, 100), Ok(99));
}

#[test]
fn test_divide_success() {
    assert_eq!(ErrorHandlingContract::divide(10, 2), Ok(5));
}

#[test]
fn test_divide_negative_numbers() {
    assert_eq!(ErrorHandlingContract::divide(-10, 2), Ok(-5));
}

#[test]
fn test_divide_large_numbers() {
    assert_eq!(ErrorHandlingContract::divide(1000000, 1000), Ok(1000));
}

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
fn test_get_verified_state_boundary_value() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ErrorHandlingContract);
    let client = ErrorHandlingContractClient::new(&env, &contract_id);

    // Set boundary value (1000 is the maximum allowed)
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&1u32, &1000u64);
    });

    let value = client.get_verified_state(&1);
    assert_eq!(value, 1000);
}

// =========================================================================
// ERROR CASE TESTS (Expected Failures)
// =========================================================================

#[test]
fn test_transfer_invalid_amount_zero() {
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
fn test_transfer_exact_insufficient() {
    assert_eq!(
        ErrorHandlingContract::transfer(101, 100),
        Err(Error::InsufficientBalance)
    );
}

#[test]
fn test_divide_by_zero() {
    assert_eq!(
        ErrorHandlingContract::divide(10, 0),
        Err(Error::InvalidAmount)
    );
}

#[test]
fn test_divide_zero_by_zero() {
    assert_eq!(
        ErrorHandlingContract::divide(0, 0),
        Err(Error::InvalidAmount)
    );
}

// =========================================================================
// ERROR TYPE VERIFICATION TESTS
// =========================================================================

#[test]
fn test_error_type_invalid_amount() {
    let result = ErrorHandlingContract::transfer(0, 100);
    assert!(result.is_err());
    
    match result {
        Err(Error::InvalidAmount) => {
            // Correct error type
            assert_eq!(Error::InvalidAmount as u32, 1);
        }
        _ => panic!("Expected InvalidAmount error"),
    }
}

#[test]
fn test_error_type_insufficient_balance() {
    let result = ErrorHandlingContract::transfer(150, 100);
    assert!(result.is_err());
    
    match result {
        Err(Error::InsufficientBalance) => {
            // Correct error type
            assert_eq!(Error::InsufficientBalance as u32, 2);
        }
        _ => panic!("Expected InsufficientBalance error"),
    }
}

#[test]
fn test_error_type_unauthorized() {
    // Test that we can create and compare Unauthorized error
    let error = Error::Unauthorized;
    assert_eq!(error as u32, 3);
    assert_eq!(error, Error::Unauthorized);
}

#[test]
fn test_error_equality() {
    assert_eq!(Error::InvalidAmount, Error::InvalidAmount);
    assert_eq!(Error::InsufficientBalance, Error::InsufficientBalance);
    assert_eq!(Error::Unauthorized, Error::Unauthorized);
    
    assert_ne!(Error::InvalidAmount, Error::InsufficientBalance);
    assert_ne!(Error::InsufficientBalance, Error::Unauthorized);
    assert_ne!(Error::Unauthorized, Error::InvalidAmount);
}

#[test]
fn test_error_debug_format() {
    let error = Error::InvalidAmount;
    // In no_std environment, we can't use format!, but we can still test the error
    assert_eq!(error, Error::InvalidAmount);
}

// =========================================================================
// RECOVERY TESTS (Error Handling and Graceful Degradation)
// =========================================================================

#[test]
fn test_error_handling_with_match() {
    let result = ErrorHandlingContract::transfer(0, 100);
    
    let handled_result = match result {
        Ok(new_balance) => new_balance,
        Err(Error::InvalidAmount) => 100, // Keep original balance
        Err(Error::InsufficientBalance) => 0, // Set to zero
        Err(_) => 50, // Default fallback
    };
    
    assert_eq!(handled_result, 100);
}

#[test]
fn test_error_handling_with_if_let() {
    let result = ErrorHandlingContract::transfer(150, 100);
    
    if let Err(Error::InsufficientBalance) = result {
        // Handle insufficient balance gracefully
        assert!(true); // Test passes if we get here
    } else {
        panic!("Expected InsufficientBalance error");
    }
}

#[test]
fn test_error_handling_with_unwrap_or() {
    let result = ErrorHandlingContract::transfer(150, 100);
    let fallback_balance = result.unwrap_or(0);
    assert_eq!(fallback_balance, 0);
}

#[test]
fn test_error_handling_with_unwrap_or_else() {
    let result = ErrorHandlingContract::transfer(150, 100);
    let fallback_balance = result.unwrap_or_else(|_| 999);
    assert_eq!(fallback_balance, 999);
}

#[test]
fn test_cascading_error_handling() {
    // Test handling multiple operations that can fail
    let transfer_result = ErrorHandlingContract::transfer(50, 100);
    
    let final_result = match transfer_result {
        Ok(balance) => {
            // Continue with next operation
            ErrorHandlingContract::divide(balance as i128, 2)
        }
        Err(_) => {
            // Handle transfer error and provide fallback
            Ok(25) // Fallback value
        }
    };
    
    assert_eq!(final_result, Ok(25));
}

#[test]
fn test_error_recovery_with_validation() {
    // Test validation before calling function to avoid errors
    fn safe_transfer(amount: u64, balance: u64) -> Result<u64, Error> {
        // Pre-validation
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }
        if amount > balance {
            return Err(Error::InsufficientBalance);
        }
        
        // Safe to call the actual function
        ErrorHandlingContract::transfer(amount, balance)
    }
    
    assert_eq!(safe_transfer(50, 100), Ok(50));
    assert_eq!(safe_transfer(0, 100), Err(Error::InvalidAmount));
    assert_eq!(safe_transfer(150, 100), Err(Error::InsufficientBalance));
}

// =========================================================================
// PANIC TESTS (Anti-pattern and Appropriate Use)
// =========================================================================

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

// =========================================================================
// EDGE CASE AND BOUNDARY TESTS
// =========================================================================

#[test]
fn test_maximum_values() {
    // Test with maximum u64 values
    let max_u64 = u64::MAX;
    let result = ErrorHandlingContract::transfer(1, max_u64);
    assert_eq!(result, Ok(max_u64 - 1));
}

#[test]
fn test_minimum_values() {
    // Test with minimum valid values
    let result = ErrorHandlingContract::transfer(1, 1);
    assert_eq!(result, Ok(0));
}

#[test]
fn test_large_number_division() {
    // Test division with large numbers
    let large_num = i128::MAX / 2;
    let result = ErrorHandlingContract::divide(large_num, 2);
    assert!(result.is_ok());
}

#[test]
fn test_error_consistency() {
    // Test that the same input always produces the same error
    for _ in 0..10 {
        let result1 = ErrorHandlingContract::transfer(0, 100);
        let result2 = ErrorHandlingContract::transfer(0, 100);
        assert_eq!(result1, result2);
        assert_eq!(result1, Err(Error::InvalidAmount));
    }
}

// =========================================================================
// PERFORMANCE AND GAS EFFICIENCY TESTS
// =========================================================================

#[test]
fn test_result_vs_panic_efficiency() {
    // This test demonstrates that Result is more efficient than panic
    // for expected error conditions
    
    // Result-based approach (should be efficient)
    for _ in 0..100 {
        let _ = ErrorHandlingContract::transfer(0, 100);
    }
    
    // Panic-based approach (should be less efficient)
    for i in 0..100 {
        // Only test valid cases to avoid actual panics
        let _ = ErrorHandlingContract::transfer_panic(i + 1, 1000);
    }
    
    // In no_std environment, we can't measure time, but we can verify
    // that both approaches complete without panicking for valid cases
    assert!(true); // Test passes if we get here
}

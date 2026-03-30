#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup_test() -> (Env, Address, Address, ErrorRecoveryContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ErrorRecoveryContract);
    let client = ErrorRecoveryContractClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    (env, user1, user2, client)
}

// Try-Catch Pattern Tests

#[test]
fn test_try_transfer_success() {
    let (env, user1, user2, client) = setup_test();

    // Set initial balance
    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let result = client.try_transfer(&user1, &user2, &500);

    assert_eq!(
        result,
        Ok(TransferResult {
            success: true,
            amount_transferred: 500,
            fallback_used: false,
        })
    );

    // Verify balances
    let balance1 = client.get_balance_or_default(&user1);
    let balance2 = client.get_balance_or_default(&user2);
    assert_eq!(balance1, 500);
    assert_eq!(balance2, 500);
}

#[test]
fn test_try_transfer_insufficient_balance() {
    let (env, user1, user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 100);
    });

    let result = client.try_transfer(&user1, &user2, &500);

    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_try_transfer_invalid_amount() {
    let (_env, user1, user2, client) = setup_test();

    let result = client.try_transfer(&user1, &user2, &0);

    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn test_try_transfer_negative_amount() {
    let (_env, user1, user2, client) = setup_test();

    let result = client.try_transfer(&user1, &user2, &-100);

    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

// Fallback Logic Tests

#[test]
fn test_fallback_primary_succeeds() {
    let (env, user1, user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let result = client.transfer_with_fallback(&user1, &user2, &500, &100);

    assert_eq!(
        result,
        Ok(TransferResult {
            success: true,
            amount_transferred: 500,
            fallback_used: false,
        })
    );
}

#[test]
fn test_fallback_uses_fallback_amount() {
    let (env, user1, user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 200);
    });

    let result = client.transfer_with_fallback(&user1, &user2, &500, &100);

    assert_eq!(
        result,
        Ok(TransferResult {
            success: true,
            amount_transferred: 100,
            fallback_used: true,
        })
    );

    let balance2 = client.get_balance_or_default(&user2);
    assert_eq!(balance2, 100);
}

#[test]
fn test_fallback_fails_both() {
    let (env, user1, user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 50);
    });

    let result = client.transfer_with_fallback(&user1, &user2, &500, &100);

    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_fallback_invalid_fallback_amount() {
    let (env, user1, user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 200);
    });

    // Fallback amount greater than primary - should fail
    let result = client.transfer_with_fallback(&user1, &user2, &500, &600);

    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

// Graceful Degradation Tests

#[test]
fn test_batch_transfer_all_succeed() {
    let (env, user1, _user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);

    let mut transfers = Vec::new(&env);
    transfers.push_back((recipient1.clone(), 100));
    transfers.push_back((recipient2.clone(), 200));
    transfers.push_back((recipient3.clone(), 300));

    let results = client.batch_transfer(&user1, &transfers);

    assert_eq!(results.len(), 3);
    assert_eq!(results.get(0).unwrap(), Ok(100));
    assert_eq!(results.get(1).unwrap(), Ok(200));
    assert_eq!(results.get(2).unwrap(), Ok(300));

    // Verify balances
    assert_eq!(client.get_balance_or_default(&recipient1), 100);
    assert_eq!(client.get_balance_or_default(&recipient2), 200);
    assert_eq!(client.get_balance_or_default(&recipient3), 300);
    assert_eq!(client.get_balance_or_default(&user1), 400);
}

#[test]
fn test_batch_transfer_partial_success() {
    let (env, user1, _user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 250);
    });

    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);

    let mut transfers = Vec::new(&env);
    transfers.push_back((recipient1.clone(), 100));
    transfers.push_back((recipient2.clone(), 200)); // This will fail
    transfers.push_back((recipient3.clone(), 50)); // This will also fail

    let results = client.batch_transfer(&user1, &transfers);

    assert_eq!(results.len(), 3);
    assert_eq!(results.get(0).unwrap(), Ok(100));
    assert_eq!(results.get(1).unwrap(), Err(Error::InsufficientBalance));
    assert_eq!(results.get(2).unwrap(), Err(Error::InsufficientBalance));

    // Only first transfer succeeded
    assert_eq!(client.get_balance_or_default(&recipient1), 100);
    assert_eq!(client.get_balance_or_default(&recipient2), 0);
    assert_eq!(client.get_balance_or_default(&recipient3), 0);
}

#[test]
fn test_batch_transfer_with_invalid_amounts() {
    let (env, user1, _user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);

    let mut transfers = Vec::new(&env);
    transfers.push_back((recipient1.clone(), 100));
    transfers.push_back((recipient2.clone(), -50)); // Invalid amount

    let results = client.batch_transfer(&user1, &transfers);

    assert_eq!(results.len(), 2);
    assert_eq!(results.get(0).unwrap(), Ok(100));
    assert_eq!(results.get(1).unwrap(), Err(Error::InvalidAmount));
}

// Transaction Rollback Tests

#[test]
fn test_atomic_batch_transfer_success() {
    let (env, user1, _user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);

    let mut transfers = Vec::new(&env);
    transfers.push_back((recipient1.clone(), 100));
    transfers.push_back((recipient2.clone(), 200));
    transfers.push_back((recipient3.clone(), 300));

    let result = client.atomic_batch_transfer(&user1, &transfers);

    assert_eq!(result, Ok(600));

    // Verify all transfers completed
    assert_eq!(client.get_balance_or_default(&recipient1), 100);
    assert_eq!(client.get_balance_or_default(&recipient2), 200);
    assert_eq!(client.get_balance_or_default(&recipient3), 300);
    assert_eq!(client.get_balance_or_default(&user1), 400);
}

#[test]
fn test_atomic_batch_transfer_insufficient_balance_rollback() {
    let (env, user1, _user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 250);
    });

    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);

    let mut transfers = Vec::new(&env);
    transfers.push_back((recipient1.clone(), 100));
    transfers.push_back((recipient2.clone(), 200));
    transfers.push_back((recipient3.clone(), 300)); // Total exceeds balance

    let result = client.atomic_batch_transfer(&user1, &transfers);

    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));

    // Verify NO transfers completed (atomic rollback)
    assert_eq!(client.get_balance_or_default(&recipient1), 0);
    assert_eq!(client.get_balance_or_default(&recipient2), 0);
    assert_eq!(client.get_balance_or_default(&recipient3), 0);
    assert_eq!(client.get_balance_or_default(&user1), 250); // Original balance unchanged
}

#[test]
fn test_atomic_batch_transfer_invalid_amount_rollback() {
    let (env, user1, _user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);

    let mut transfers = Vec::new(&env);
    transfers.push_back((recipient1.clone(), 100));
    transfers.push_back((recipient2.clone(), 0)); // Invalid amount

    let result = client.atomic_batch_transfer(&user1, &transfers);

    assert_eq!(result, Err(Ok(Error::InvalidAmount)));

    // Verify NO transfers completed
    assert_eq!(client.get_balance_or_default(&recipient1), 0);
    assert_eq!(client.get_balance_or_default(&recipient2), 0);
    assert_eq!(client.get_balance_or_default(&user1), 1000);
}

// Validation Tests

#[test]
fn test_validate_transfer_success() {
    let (env, user1, user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let result = client.validate_transfer(&user1, &user2, &500);

    assert_eq!(result, Ok(true));
}

#[test]
fn test_validate_transfer_invalid_amount() {
    let (_env, user1, user2, client) = setup_test();

    let result = client.validate_transfer(&user1, &user2, &0);

    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn test_validate_transfer_insufficient_balance() {
    let (env, user1, user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 100);
    });

    let result = client.validate_transfer(&user1, &user2, &500);

    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_validate_transfer_same_address() {
    let (env, user1, _user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let result = client.validate_transfer(&user1, &user1, &500);

    assert_eq!(result, Err(Ok(Error::ValidationFailed)));
}

// Safe Transfer Tests

#[test]
fn test_safe_transfer_success() {
    let (env, user1, user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let result = client.safe_transfer(&user1, &user2, &500);

    assert!(result.is_ok());
    assert_eq!(client.get_balance_or_default(&user2), 500);
}

#[test]
fn test_safe_transfer_rate_limit() {
    let (env, user1, user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    // First transfer should succeed
    let result1 = client.safe_transfer(&user1, &user2, &100);
    assert!(result1.is_ok());

    // Second transfer immediately should fail due to rate limit
    let result2 = client.safe_transfer(&user1, &user2, &100);
    assert_eq!(result2, Err(Ok(Error::RateLimitExceeded)));

    // Advance time by 11 seconds
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 11;
    });

    // Third transfer should succeed after cooldown
    let result3 = client.safe_transfer(&user1, &user2, &100);
    assert!(result3.is_ok());
}

// Recovery Tests

#[test]
fn test_get_balance_or_default_with_balance() {
    let (env, user1, _user2, client) = setup_test();

    env.as_contract(&client.address, || {
        ErrorRecoveryContract::set_balance(env.clone(), user1.clone(), 1000);
    });

    let balance = client.get_balance_or_default(&user1);
    assert_eq!(balance, 1000);
}

#[test]
fn test_get_balance_or_default_no_balance() {
    let (_env, user1, _user2, client) = setup_test();

    let balance = client.get_balance_or_default(&user1);
    assert_eq!(balance, 0); // Returns default value instead of panicking
}

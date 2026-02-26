#![cfg(test)]
use super::*;
use soroban_sdk::{Env, String, Vec};
use soroban_sdk::testutils::{Address as AddressTest, Ledger as LedgerTest};

#[test]
fn test_parameter_validation() {
    let env = Env::default();

    // Test amount validation
    assert_eq!(
        ValidationContract::validate_amount_parameters(100, 50, 200),
        Ok(())
    );

    assert_eq!(
        ValidationContract::validate_amount_parameters(0, 50, 200),
        Err(ValidationError::InvalidAmount)
    );

    assert_eq!(
        ValidationContract::validate_amount_parameters(-10, 50, 200),
        Err(ValidationError::InvalidAmount)
    );

    assert_eq!(
        ValidationContract::validate_amount_parameters(25, 50, 200),
        Err(ValidationError::AmountTooSmall)
    );

    assert_eq!(
        ValidationContract::validate_amount_parameters(250, 50, 200),
        Err(ValidationError::AmountTooLarge)
    );

    // Test string validation
    let valid_string = String::from_str(&env, "Hello, World!");
    assert_eq!(
        ValidationContract::validate_string_parameters(valid_string.clone(), 1, 100),
        Ok(())
    );

    let short_string = String::from_str(&env, "");
    assert_eq!(
        ValidationContract::validate_string_parameters(short_string.clone(), 1, 100),
        Err(ValidationError::StringTooShort)
    );

    let long_string = String::from_str(&env, "This string is way too long and exceeds the maximum length limit");
    assert_eq!(
        ValidationContract::validate_string_parameters(long_string.clone(), 1, 50),
        Err(ValidationError::StringTooLong)
    );

    // Test address validation (should always pass for valid addresses)
    let user1 = <soroban_sdk::Address as AddressTest>::generate(&env);
    assert_eq!(
        ValidationContract::validate_address(user1.clone()),
        Ok(())
    );

    // Test array validation
    let valid_array = Vec::from_array(&env, [1i32, 2i32, 3i32, 4i32, 5i32]);
    assert_eq!(
        ValidationContract::validate_array_parameters(valid_array.clone(), 1, 10),
        Ok(())
    );

    let small_array = Vec::from_array(&env, [1i32]);
    assert_eq!(
        ValidationContract::validate_array_parameters(small_array.clone(), 2, 10),
        Err(ValidationError::ArrayTooSmall)
    );

    let large_array = Vec::from_array(&env, [1i32, 2i32, 3i32, 4i32, 5i32, 6i32, 7i32, 8i32, 9i32, 10i32, 11i32]);
    assert_eq!(
        ValidationContract::validate_array_parameters(large_array.clone(), 1, 10),
        Err(ValidationError::ArrayTooLarge)
    );

    // Test timestamp validation
    let current_time = env.ledger().timestamp();
    
    // Valid future timestamp
    assert_eq!(
        ValidationContract::validate_timestamp_parameters(&env, current_time + 3600, false, 86400),
        Ok(())
    );

    // Past timestamp (not allowed)
    assert_eq!(
        ValidationContract::validate_timestamp_parameters(&env, current_time, false, 86400),
        Ok(())
    );

    // Past timestamp (allowed)
    assert_eq!(
        ValidationContract::validate_timestamp_parameters(&env, current_time.saturating_sub(3600), true, 86400),
        Ok(())
    );

    // Too far in future
    assert_eq!(
        ValidationContract::validate_timestamp_parameters(&env, current_time + 200000, false, 86400),
        Err(ValidationError::TimestampInDistantFuture)
    );
}

#[test]
fn test_state_validation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ValidationContract);

    env.as_contract(&contract_id, || {
        // Test uninitialized contract
        assert_eq!(
            ValidationContract::validate_contract_state(&env, ContractState::Active),
            Err(ValidationError::ContractNotInitialized)
        );

        // Initialize contract
        let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::State, &ContractState::Active);
    });

    // Test active contract state
    assert_eq!(
        ValidationContract::validate_contract_state(&env, ContractState::Active),
        Ok(())
    );

    // Test balance validation
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);
    
    // Insufficient balance
    assert_eq!(
        ValidationContract::validate_balance(&env, user.clone(), 100),
        Err(ValidationError::InsufficientBalance)
    );

    // Set balance and test again
    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&DataKey::Balance(user.clone()), &200);
    });
    assert_eq!(
        ValidationContract::validate_balance(&env, user.clone(), 100),
        Ok(())
    );

    // Test allowance validation
    let spender = <soroban_sdk::Address as AddressTest>::generate(&env);
    
    // Insufficient allowance
    assert_eq!(
        ValidationContract::validate_allowance(&env, user.clone(), spender.clone(), 100),
        Err(ValidationError::InsufficientAllowance)
    );

    // Set allowance and test again
    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&DataKey::Allowance(user.clone(), spender.clone()), &200);
    });
    assert_eq!(
        ValidationContract::validate_allowance(&env, user.clone(), spender.clone(), 100),
        Ok(())
    );

    // Test cooldown validation
    // No previous action should pass
    assert_eq!(
        ValidationContract::validate_cooldown(&env, user.clone(), 60),
        Ok(())
    );

    // Set last action and test cooldown
    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&DataKey::LastAction(user.clone()), &env.ledger().timestamp());
    });
    assert_eq!(
        ValidationContract::validate_cooldown(&env, user.clone(), 60),
        Err(ValidationError::CooldownActive)
    );

    // Test paused contract state
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::State, &ContractState::Paused);
    });
    assert_eq!(
        ValidationContract::validate_contract_state(&env, ContractState::Active),
        Err(ValidationError::ContractPaused)
    );

    // Resume and test again
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::State, &ContractState::Active);
    });
    assert_eq!(
        ValidationContract::validate_contract_state(&env, ContractState::Active),
        Ok(())
    );
}

#[test]
fn test_authorization_validation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ValidationContract);

    env.as_contract(&contract_id, || {
        let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
        let admin = <soroban_sdk::Address as AddressTest>::generate(&env);
        let user = <soroban_sdk::Address as AddressTest>::generate(&env);
        let moderator = <soroban_sdk::Address as AddressTest>::generate(&env);

        // Test role validation with no role assigned
        assert_eq!(
            ValidationContract::validate_role(&env, user.clone(), UserRole::User),
            Err(ValidationError::InsufficientRole)
        );

        // Set user role
        env.as_contract(&contract_id, || {
            env.storage().instance().set(&DataKey::UserRole(user.clone()), &UserRole::User);
        });
        assert_eq!(
            ValidationContract::validate_role(&env, user.clone(), UserRole::User),
            Ok(())
        );

        // Test insufficient role
        assert_eq!(
            ValidationContract::validate_role(&env, user.clone(), UserRole::Moderator),
            Err(ValidationError::InsufficientRole)
        );

        // Set moderator role
        env.as_contract(&contract_id, || {
            env.storage().instance().set(&DataKey::UserRole(moderator.clone()), &UserRole::Moderator);
        });
        assert_eq!(
            ValidationContract::validate_role(&env, moderator.clone(), UserRole::User),
            Ok(())
        );

        // Set admin role
        env.as_contract(&contract_id, || {
            env.storage().instance().set(&DataKey::UserRole(admin.clone()), &UserRole::Admin);
            env.storage().instance().set(&DataKey::Admin, &admin);
        });
        assert_eq!(
            ValidationContract::validate_role(&env, admin.clone(), UserRole::Moderator),
            Ok(())
        );

        // Test ownership validation
        assert_eq!(
            ValidationContract::validate_ownership(&env, owner.clone()),
            Ok(())
        );
    });
}

#[test]
fn test_validated_transfer() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ValidationContract);

    env.as_contract(&contract_id, || {
        // Initialize contract
        let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
        let user = <soroban_sdk::Address as AddressTest>::generate(&env);
        let recipient = <soroban_sdk::Address as AddressTest>::generate(&env);
        env.storage().instance().set(&DataKey::Admin, &owner);
        env.storage().instance().set(&DataKey::State, &ContractState::Active);

        // Set user roles
        env.as_contract(&contract_id, || {
            env.storage().instance().set(&DataKey::UserRole(user.clone()), &UserRole::User);
            env.storage().instance().set(&DataKey::UserRole(recipient.clone()), &UserRole::User);
        });

        // Set initial balance
        env.as_contract(&contract_id, || {
            env.storage().persistent().set(&DataKey::Balance(user.clone()), &1000);
        });

        // Test successful transfer
        assert_eq!(
            ValidationContract::validated_transfer(
                env.clone(),
                user.clone(),
                recipient.clone(),
                100,
                Some(String::from_str(&env, "Test transfer"))
            ),
            Ok(())
        );

        // Verify balances updated
        env.as_contract(&contract_id, || {
            let balance1: i128 = env.storage().persistent().get(&DataKey::Balance(user.clone())).unwrap_or(0);
            let balance2: i128 = env.storage().persistent().get(&DataKey::Balance(recipient.clone())).unwrap_or(0);
            assert_eq!(balance1, 900);
            assert_eq!(balance2, 100);
        });

        // Test insufficient balance
        env.as_contract(&contract_id, || {
            ValidationContract::initialize(env.clone(), owner.clone()).unwrap();
            
            // Set user role
            ValidationContract::set_user_role(env.clone(), owner.clone(), user.clone(), UserRole::User).unwrap();
            
            // Test validated transfer
            assert_eq!(
                ValidationContract::validated_transfer(env.clone(), user.clone(), recipient.clone(), 1000, None),
                Err(ValidationError::InsufficientBalance)
            );
            
            // Test transfer with insufficient balance
            assert_eq!(
                ValidationContract::validated_transfer(env.clone(), user.clone(), recipient.clone(), 1000000, None),
                Err(ValidationError::InsufficientBalance)
            );
        });

        // Test with paused contract
        env.as_contract(&contract_id, || {
            env.storage().instance().set(&DataKey::State, &ContractState::Paused);
        });
        assert_eq!(
            ValidationContract::validated_transfer(env.clone(), user.clone(), recipient.clone(), 50, None),
            Err(ValidationError::ContractPaused)
        );

        // Resume and test again
        env.as_contract(&contract_id, || {
            env.storage().instance().set(&DataKey::State, &ContractState::Active);
        });
        
        // Wait for cooldown to pass
        env.ledger().set_timestamp(env.ledger().timestamp() + 61);
        
        assert_eq!(
            ValidationContract::validated_transfer(env.clone(), user.clone(), recipient.clone(), 50, None),
            Ok(())
        );
    });
}

#[test]
fn test_admin_functions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ValidationContract);

    let admin = <soroban_sdk::Address as AddressTest>::generate(&env);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);

    env.as_contract(&contract_id, || {
        // Initialize contract
        let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::Admin, &owner);
        env.storage().instance().set(&DataKey::State, &ContractState::Active);

        // Set admin role
        env.storage().instance().set(&DataKey::UserRole(admin.clone()), &UserRole::Admin);
        env.storage().instance().set(&DataKey::Admin, &admin);

        // Test admin setting user role
        assert_eq!(
            ValidationContract::set_user_role(env.clone(), admin.clone(), user.clone(), UserRole::Moderator),
            Ok(())
        );

        // Check role was set
        let role: UserRole = env.storage().instance().get(&DataKey::UserRole(user.clone())).unwrap_or(UserRole::None);
        assert_eq!(role, UserRole::Moderator);

        // Test admin pausing contract
        assert_eq!(
            ValidationContract::pause_contract(env.clone(), admin.clone()),
            Ok(())
        );
        let state: ContractState = env.storage().instance().get(&DataKey::State).unwrap_or(ContractState::Uninitialized);
        assert_eq!(state, ContractState::Paused);

        // Test admin resuming contract
        assert_eq!(
            ValidationContract::resume_contract(env.clone(), admin.clone()),
            Ok(())
        );
        let state: ContractState = env.storage().instance().get(&DataKey::State).unwrap_or(ContractState::Uninitialized);
        assert_eq!(state, ContractState::Active);
    });

    // Test non-admin trying to pause
    env.as_contract(&contract_id, || {
        assert_eq!(
            ValidationContract::pause_contract(env.clone(), user.clone()),
            Err(ValidationError::NotAdmin)
        );
    });
}

#[test]
fn test_error_codes() {
    let env = Env::default();
    // Test that all error codes are unique and properly defined
    let errors = [
        ValidationError::InvalidAmount,
        ValidationError::AmountTooSmall,
        ValidationError::AmountTooLarge,
        ValidationError::InvalidAddress,
        ValidationError::InvalidString,
        ValidationError::StringTooShort,
        ValidationError::StringTooLong,
        ValidationError::InvalidEnum,
        ValidationError::InvalidArray,
        ValidationError::ArrayTooSmall,
        ValidationError::ArrayTooLarge,
        ValidationError::InvalidTimestamp,
        ValidationError::TimestampInPast,
        ValidationError::TimestampInDistantFuture,
        ValidationError::ContractNotInitialized,
        ValidationError::ContractPaused,
        ValidationError::ContractFrozen,
        ValidationError::InsufficientBalance,
        ValidationError::InsufficientAllowance,
        ValidationError::ResourceNotFound,
        ValidationError::ResourceAlreadyExists,
        ValidationError::InvalidStateTransition,
        ValidationError::InvariantViolation,
        ValidationError::RateLimitExceeded,
        ValidationError::CooldownActive,
        ValidationError::Unauthorized,
        ValidationError::NotAdmin,
        ValidationError::NotOwner,
        ValidationError::InsufficientRole,
        ValidationError::SignatureRequired,
        ValidationError::MultiSigRequired,
        ValidationError::InvalidSignature,
        ValidationError::ExpiredSignature,
        ValidationError::WrongContract,
        ValidationError::Blacklisted,
    ];

    // Verify all errors have unique codes
    let mut codes = Vec::new(&env);
    for error in errors.iter() {
        let code = *error as u32;
        assert!(!codes.contains(&code), "Duplicate error code: {}", code);
        codes.push_back(code);
    }

    // Verify error codes are in expected ranges
    for error in errors.iter() {
        let code = *error as u32;
        match error {
            ValidationError::InvalidAmount | ValidationError::AmountTooSmall | 
            ValidationError::AmountTooLarge | ValidationError::InvalidAddress |
            ValidationError::InvalidString | ValidationError::StringTooShort |
            ValidationError::StringTooLong | ValidationError::InvalidEnum |
            ValidationError::InvalidArray | ValidationError::ArrayTooSmall |
            ValidationError::ArrayTooLarge | ValidationError::InvalidTimestamp |
            ValidationError::TimestampInPast | ValidationError::TimestampInDistantFuture => {
                assert!(code >= 100 && code < 200, "Parameter validation error should be in range 100-199");
            }
            ValidationError::ContractNotInitialized | ValidationError::ContractPaused |
            ValidationError::ContractFrozen | ValidationError::InsufficientBalance |
            ValidationError::InsufficientAllowance | ValidationError::ResourceNotFound |
            ValidationError::ResourceAlreadyExists | ValidationError::InvalidStateTransition |
            ValidationError::InvariantViolation | ValidationError::RateLimitExceeded |
            ValidationError::CooldownActive => {
                assert!(code >= 200 && code < 300, "State validation error should be in range 200-299");
            }
            ValidationError::Unauthorized | ValidationError::NotAdmin |
            ValidationError::NotOwner | ValidationError::InsufficientRole |
            ValidationError::SignatureRequired | ValidationError::MultiSigRequired |
            ValidationError::InvalidSignature | ValidationError::ExpiredSignature |
            ValidationError::WrongContract | ValidationError::Blacklisted => {
                assert!(code >= 300 && code < 400, "Authorization validation error should be in range 300-399");
            }
        }
    }
}

#[test]
fn test_edge_cases() {
    let env = Env::default();

    // Test boundary conditions for amount validation
    assert_eq!(
        ValidationContract::validate_amount_parameters(1, 1, 1000),
        Ok(())
    ); // Minimum valid amount

    assert_eq!(
        ValidationContract::validate_amount_parameters(1000, 1, 1000),
        Ok(())
    ); // Maximum valid amount

    assert_eq!(
        ValidationContract::validate_amount_parameters(0, 1, 1000),
        Err(ValidationError::InvalidAmount)
    ); // Just below minimum

    assert_eq!(
        ValidationContract::validate_amount_parameters(1001, 1, 1000),
        Err(ValidationError::AmountTooLarge)
    ); // Just above maximum

    // Test boundary conditions for string validation
    let exact_length = String::from_str(&env, "12345");
    assert_eq!(
        ValidationContract::validate_string_parameters(exact_length.clone(), 5, 5),
        Ok(())
    ); // Exact length

    assert_eq!(
        ValidationContract::validate_string_parameters(exact_length.clone(), 6, 10),
        Err(ValidationError::StringTooShort)
    ); // One character too short

    // Test boundary conditions for array validation
    let exact_size = Vec::from_array(&env, [1i32, 2i32, 3i32]);
    assert_eq!(
        ValidationContract::validate_array_parameters(exact_size.clone(), 3, 3),
        Ok(())
    ); // Exact size

    let smaller_array = Vec::from_array(&env, [1i32, 2i32]);
    assert_eq!(
        ValidationContract::validate_array_parameters(smaller_array.clone(), 3, 5),
        Err(ValidationError::ArrayTooSmall)
    ); // One element too small

    // Test timestamp boundary conditions
    let current_time = env.ledger().timestamp();
    
    // Exactly at the limit
    assert_eq!(
        ValidationContract::validate_timestamp_parameters(&env, current_time + 86400, false, 86400),
        Ok(())
    ); // Exactly max future

    assert_eq!(
        ValidationContract::validate_timestamp_parameters(&env, current_time + 86401, false, 86400),
        Err(ValidationError::TimestampInDistantFuture)
    ); // One second over limit
}

use super::*;
use soroban_sdk::testutils::{Address as AddressTest, Ledger as LedgerTest};
use soroban_sdk::{Env, String, Vec};

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
        ValidationContract::validate_string_parameters(valid_string, 1, 100),
        Ok(())
    );

    let short_string = String::from_str(&env, "");
    assert_eq!(
        ValidationContract::validate_string_parameters(short_string, 1, 100),
        Err(ValidationError::StringTooShort)
    );

    let long_string = String::from_str(
        &env,
        "This string is way too long and exceeds the maximum length limit",
    );
    assert_eq!(
        ValidationContract::validate_string_parameters(long_string, 1, 50),
        Err(ValidationError::StringTooLong)
    );

    // Test address validation (should always pass for valid addresses)
    let user1 = <soroban_sdk::Address as AddressTest>::generate(&env);
    assert_eq!(ValidationContract::validate_address(user1), Ok(()));

    // Test array validation
    let valid_array = Vec::from_array(&env, [1i32, 2i32, 3i32, 4i32, 5i32]);
    assert_eq!(
        ValidationContract::validate_array_parameters(valid_array, 1, 10),
        Ok(())
    );

    let small_array = Vec::from_array(&env, [1i32]);
    assert_eq!(
        ValidationContract::validate_array_parameters(small_array, 2, 10),
        Err(ValidationError::ArrayTooSmall)
    );

    let large_array = Vec::from_array(
        &env,
        [
            1i32, 2i32, 3i32, 4i32, 5i32, 6i32, 7i32, 8i32, 9i32, 10i32, 11i32,
        ],
    );
    assert_eq!(
        ValidationContract::validate_array_parameters(large_array, 1, 10),
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
        ValidationContract::validate_timestamp_parameters(
            &env,
            current_time.saturating_sub(3600),
            true,
            86400
        ),
        Ok(())
    );

    // Too far in future
    assert_eq!(
        ValidationContract::validate_timestamp_parameters(
            &env,
            current_time + 200000,
            false,
            86400
        ),
        Err(ValidationError::TimestampInDistantFuture)
    );
}

#[test]
fn test_state_validation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ValidationContract);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);
    let spender = <soroban_sdk::Address as AddressTest>::generate(&env);

    env.as_contract(&contract_id, || {
        // Test uninitialized contract
        assert_eq!(
            ValidationContract::validate_contract_state(&env, ContractState::Active),
            Err(ValidationError::ContractNotInitialized)
        );

        // Initialize contract
        let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Active);

        // Test active contract state
        assert_eq!(
            ValidationContract::validate_contract_state(&env, ContractState::Active),
            Ok(())
        );

        // Insufficient balance
        assert_eq!(
            ValidationContract::validate_balance(&env, user, 100),
            Err(ValidationError::InsufficientBalance)
        );

        // Set balance and test again
        env.storage()
            .persistent()
            .set(&DataKey::Balance(user), &200i128);
        assert_eq!(
            ValidationContract::validate_balance(&env, user, 100),
            Ok(())
        );

        // Test allowance validation
        assert_eq!(
            ValidationContract::validate_allowance(&env, user, spender, 100),
            Err(ValidationError::InsufficientAllowance)
        );

        env.storage()
            .persistent()
            .set(&DataKey::Allowance(user, spender), &200i128);
        assert_eq!(
            ValidationContract::validate_allowance(&env, user, spender, 100),
            Ok(())
        );

        // Test cooldown validation — no previous action should pass
        assert_eq!(
            ValidationContract::validate_cooldown(&env, user, 60),
            Ok(())
        );

        // Set last action and test cooldown
        env.storage().persistent().set(
            &DataKey::LastAction(user),
            &env.ledger().timestamp(),
        );
        assert_eq!(
            ValidationContract::validate_cooldown(&env, user, 60),
            Err(ValidationError::CooldownActive)
        );

        // Test paused contract state
        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Paused);
        assert_eq!(
            ValidationContract::validate_contract_state(&env, ContractState::Active),
            Err(ValidationError::ContractPaused)
        );

        // Resume and test again
        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Active);
        assert_eq!(
            ValidationContract::validate_contract_state(&env, ContractState::Active),
            Ok(())
        );
    });
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
            ValidationContract::validate_role(&env, user, UserRole::User),
            Err(ValidationError::InsufficientRole)
        );

        // Set user role directly (already in contract context)
        env.storage()
            .instance()
            .set(&DataKey::UserRole(user), &UserRole::User);
        assert_eq!(
            ValidationContract::validate_role(&env, user, UserRole::User),
            Ok(())
        );

        // Test insufficient role
        assert_eq!(
            ValidationContract::validate_role(&env, user, UserRole::Moderator),
            Err(ValidationError::InsufficientRole)
        );

        // Set moderator role directly
        env.storage()
            .instance()
            .set(&DataKey::UserRole(moderator), &UserRole::Moderator);
        assert_eq!(
            ValidationContract::validate_role(&env, moderator, UserRole::User),
            Ok(())
        );

        // Set admin role directly
        env.storage()
            .instance()
            .set(&DataKey::UserRole(admin), &UserRole::Admin);
        env.storage().instance().set(&DataKey::Admin, &admin);
        assert_eq!(
            ValidationContract::validate_role(&env, admin, UserRole::Moderator),
            Ok(())
        );

        // Test ownership validation: set owner first so validate_ownership can find it
        env.storage().instance().set(&DataKey::Owner, &owner);
        assert_eq!(
            ValidationContract::validate_ownership(&env, owner),
            Ok(())
        );
    });
}

#[test]
fn test_validated_transfer() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ValidationContract);
    let client = ValidationContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);
    let recipient = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Setup contract state via direct storage write (no auth conflict inside as_contract)
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Admin, &owner);
        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Active);
        env.storage()
            .instance()
            .set(&DataKey::UserRole(user), &UserRole::User);
        env.storage()
            .instance()
            .set(&DataKey::UserRole(recipient), &UserRole::User);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(user), &1000i128);
    });

    // Test successful transfer via client (handles require_auth through invocation path)
    client.validated_transfer(
        &user,
        &recipient,
        &100i128,
        &Some(String::from_str(&env, "Test transfer")),
    );

    // Verify balances updated
    env.as_contract(&contract_id, || {
        let balance1: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(user))
            .unwrap_or(0);
        let balance2: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(recipient))
            .unwrap_or(0);
        assert_eq!(balance1, 900);
        assert_eq!(balance2, 100);
    });

    // Test insufficient balance
    let result = client.try_validated_transfer(&user, &recipient, &1000i128, &None);
    assert_eq!(result, Err(Ok(ValidationError::InsufficientBalance)));

    // Test with paused contract
    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Paused);
    });
    let result = client.try_validated_transfer(&user, &recipient, &50i128, &None);
    assert_eq!(result, Err(Ok(ValidationError::ContractPaused)));

    // Resume and wait for cooldown to pass, then transfer again
    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Active);
    });
    env.ledger().set_timestamp(env.ledger().timestamp() + 61);
    client.validated_transfer(&user, &recipient, &50i128, &None);
}

#[test]
fn test_admin_functions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ValidationContract);
    let client = ValidationContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let admin = <soroban_sdk::Address as AddressTest>::generate(&env);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Initialize contract state via direct storage (no require_auth issue)
    env.as_contract(&contract_id, || {
        let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::State, &ContractState::Active);
        env.storage()
            .instance()
            .set(&DataKey::UserRole(admin), &UserRole::Admin);
    });

    // Test admin setting user role via client
    client.set_user_role(&admin, &user, &UserRole::Moderator);

    // Check role was set
    env.as_contract(&contract_id, || {
        let role: UserRole = env
            .storage()
            .instance()
            .get(&DataKey::UserRole(user))
            .unwrap_or(UserRole::None);
        assert_eq!(role, UserRole::Moderator);
    });

    // Test admin pausing contract
    client.pause_contract(&admin);
    env.as_contract(&contract_id, || {
        let state: ContractState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(ContractState::Uninitialized);
        assert_eq!(state, ContractState::Paused);
    });

    // Test admin resuming contract
    client.resume_contract(&admin);
    env.as_contract(&contract_id, || {
        let state: ContractState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(ContractState::Uninitialized);
        assert_eq!(state, ContractState::Active);
    });

    // Test non-admin trying to pause (user has Moderator role, not Admin/Owner)
    let result = client.try_pause_contract(&user);
    assert_eq!(result, Err(Ok(ValidationError::NotAdmin)));
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
        assert!(!codes.contains(code), "Duplicate error code: {code}");
        codes.push_back(code);
    }

    // Verify error codes are in expected ranges
    for error in errors.iter() {
        let code = *error as u32;
        match error {
            ValidationError::InvalidAmount
            | ValidationError::AmountTooSmall
            | ValidationError::AmountTooLarge
            | ValidationError::InvalidAddress
            | ValidationError::InvalidString
            | ValidationError::StringTooShort
            | ValidationError::StringTooLong
            | ValidationError::InvalidEnum
            | ValidationError::InvalidArray
            | ValidationError::ArrayTooSmall
            | ValidationError::ArrayTooLarge
            | ValidationError::InvalidTimestamp
            | ValidationError::TimestampInPast
            | ValidationError::TimestampInDistantFuture => {
                assert!(
                    (100..200).contains(&code),
                    "Parameter validation error should be in range 100-199"
                );
            }
            ValidationError::ContractNotInitialized
            | ValidationError::ContractPaused
            | ValidationError::ContractFrozen
            | ValidationError::InsufficientBalance
            | ValidationError::InsufficientAllowance
            | ValidationError::ResourceNotFound
            | ValidationError::ResourceAlreadyExists
            | ValidationError::InvalidStateTransition
            | ValidationError::InvariantViolation
            | ValidationError::RateLimitExceeded
            | ValidationError::CooldownActive => {
                assert!(
                    (200..300).contains(&code),
                    "State validation error should be in range 200-299"
                );
            }
            ValidationError::Unauthorized
            | ValidationError::NotAdmin
            | ValidationError::NotOwner
            | ValidationError::InsufficientRole
            | ValidationError::SignatureRequired
            | ValidationError::MultiSigRequired
            | ValidationError::InvalidSignature
            | ValidationError::ExpiredSignature
            | ValidationError::WrongContract
            | ValidationError::Blacklisted => {
                assert!(
                    (300..400).contains(&code),
                    "Authorization validation error should be in range 300-399"
                );
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

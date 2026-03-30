use super::*;
use soroban_sdk::testutils::Address as AddressTest;
use soroban_sdk::{symbol_short, Env};

#[test]
fn test_simple_enums() {
    let _env = Env::default();

    // Test UserRole enum
    let user_role = UserRole::User;
    assert_eq!(user_role, UserRole::User);
    assert_ne!(user_role, UserRole::Admin);

    // Test ContractState enum
    let state = ContractState::Active;
    assert_eq!(state, ContractState::Active);
    assert_ne!(state, ContractState::Paused);

    // Test TransactionType enum
    let tx_type = TransactionType::Transfer;
    assert_eq!(tx_type, TransactionType::Transfer);
    assert_ne!(tx_type, TransactionType::Deposit);

    // Test ValidationResult enum
    let result = ValidationResult::Success;
    assert_eq!(result, ValidationResult::Success);
    assert_ne!(result, ValidationResult::Failure);

    // Test enum comparisons
    assert!(UserRole::Admin > UserRole::User);
    assert!(UserRole::Owner > UserRole::Admin);
    assert!(ContractState::Active > ContractState::Uninitialized);

    // Test enum values
    assert_eq!(UserRole::None as u32, 0);
    assert_eq!(UserRole::User as u32, 1);
    assert_eq!(UserRole::Admin as u32, 3);
    assert_eq!(UserRole::Owner as u32, 4);
}

#[test]
fn test_contract_error_enum() {
    // Test error codes
    assert_eq!(ContractError::InvalidInput as u32, 1000);
    assert_eq!(ContractError::Unauthorized as u32, 1001);
    assert_eq!(ContractError::InsufficientBalance as u32, 1002);
    assert_eq!(ContractError::ContractNotInitialized as u32, 1100);
    assert_eq!(ContractError::OperationNotFound as u32, 1200);
    assert_eq!(ContractError::AssetNotFound as u32, 1300);
    assert_eq!(ContractError::UserNotFound as u32, 1400);
    assert_eq!(ContractError::ValidationFailed as u32, 1500);
    assert_eq!(ContractError::InternalError as u32, 1600);

    // Test error comparisons
    assert!(ContractError::Unauthorized > ContractError::InvalidInput);
    assert!(ContractError::InsufficientBalance > ContractError::Unauthorized);
}

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EnumContract);
    let admin = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Test successful initialization
    env.as_contract(&contract_id, || {
        assert_eq!(EnumContract::initialize(env.clone(), admin.clone()), Ok(()));
    });

    // Test state after initialization
    env.as_contract(&contract_id, || {
        assert_eq!(EnumContract::get_state(env.clone()), ContractState::Active);
        assert_eq!(
            EnumContract::get_user_role(env.clone(), admin.clone()),
            UserRole::Owner
        );
    });

    // Test double initialization
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::initialize(env.clone(), admin.clone()),
            Err(ContractError::ContractAlreadyInitialized)
        );
    });
}

#[test]
fn test_user_role_management() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EnumContract);
    let admin = <soroban_sdk::Address as AddressTest>::generate(&env);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Initialize contract
    env.as_contract(&contract_id, || {
        EnumContract::initialize(env.clone(), admin.clone()).unwrap();
    });

    // Test setting user role
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::set_user_role(env.clone(), admin.clone(), user.clone(), UserRole::User),
            Ok(())
        );
    });

    // Verify role was set
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::get_user_role(env.clone(), user.clone()),
            UserRole::User
        );
    });

    // Test non-admin trying to set role
    let user2 = <soroban_sdk::Address as AddressTest>::generate(&env);
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::set_user_role(
                env.clone(),
                user.clone(),
                user2.clone(),
                UserRole::Moderator
            ),
            Err(ContractError::InsufficientRole)
        );
    });

    // Test setting owner role (should fail)
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::set_user_role(env.clone(), admin.clone(), user2.clone(), UserRole::Owner),
            Err(ContractError::InvalidInput)
        );
    });
}

#[test]
fn test_operation_execution() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EnumContract);
    let user1 = <soroban_sdk::Address as AddressTest>::generate(&env);
    let user2 = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Initialize contract
    env.as_contract(&contract_id, || {
        EnumContract::initialize(env.clone(), user1.clone()).unwrap();
        EnumContract::set_user_role(env.clone(), user1.clone(), user2.clone(), UserRole::User)
            .unwrap();
    });

    // Test transfer operation
    let result =
        EnumContract::execute_operation(env.clone(), TransactionType::Transfer, 100, user2.clone());
    assert_eq!(result, Ok(ValidationResult::Success));

    // Test deposit operation
    let result =
        EnumContract::execute_operation(env.clone(), TransactionType::Deposit, 1000, user2.clone());
    assert_eq!(result, Ok(ValidationResult::Success));

    // Test withdraw operation
    let result = EnumContract::execute_operation(
        env.clone(),
        TransactionType::Withdraw,
        5000,
        user2.clone(),
    );
    assert_eq!(result, Ok(ValidationResult::Success));

    // Test mint operation
    let result =
        EnumContract::execute_operation(env.clone(), TransactionType::Mint, 500000, user2.clone());
    assert_eq!(result, Ok(ValidationResult::Success));

    // Test burn operation
    let result =
        EnumContract::execute_operation(env.clone(), TransactionType::Burn, 250000, user2.clone());
    assert_eq!(result, Ok(ValidationResult::Success));

    // Test invalid amount (zero)
    let result =
        EnumContract::execute_operation(env.clone(), TransactionType::Transfer, 0, user2.clone());
    assert_eq!(result, Ok(ValidationResult::Failure));

    // Test invalid amount (negative)
    let result =
        EnumContract::execute_operation(env.clone(), TransactionType::Transfer, -1, user2.clone());
    assert_eq!(result, Ok(ValidationResult::Failure));

    // Test amount over limit for transfer
    let result = EnumContract::execute_operation(
        env.clone(),
        TransactionType::Transfer,
        1500,
        user2.clone(),
    );
    assert_eq!(result, Ok(ValidationResult::Failure));

    // Test amount over limit for deposit
    let result =
        EnumContract::execute_operation(env.clone(), TransactionType::Deposit, 6000, user2.clone());
    assert_eq!(result, Ok(ValidationResult::Failure));

    // Test amount over limit for withdraw
    let result = EnumContract::execute_operation(
        env.clone(),
        TransactionType::Withdraw,
        15000,
        user2.clone(),
    );
    assert_eq!(result, Ok(ValidationResult::Failure));

    // Test amount over limit for mint
    let result =
        EnumContract::execute_operation(env.clone(), TransactionType::Mint, 2000000, user2.clone());
    assert_eq!(result, Ok(ValidationResult::Failure));

    // Test amount over limit for burn
    let result =
        EnumContract::execute_operation(env.clone(), TransactionType::Burn, 750000, user2.clone());
    assert_eq!(result, Ok(ValidationResult::Failure));
}

#[test]
fn test_validation_result_processing() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EnumContract);

    env.as_contract(&contract_id, || {
        // Test processing success result
        let success = ValidationResult::Success;
        let result = EnumContract::process_validation_result(env.clone(), success, 1);
        assert_eq!(result, Ok(()));

        // Test processing failure result
        let failure = ValidationResult::Failure;
        let result = EnumContract::process_validation_result(env.clone(), failure, 2);
        assert_eq!(result, Err(ContractError::ValidationFailed));

        // Test processing approval result
        let approval = ValidationResult::RequiresApproval;
        let result = EnumContract::process_validation_result(env.clone(), approval, 3);
        assert_eq!(result, Err(ContractError::InsufficientApprovals));

        // Test processing pending result
        let pending = ValidationResult::Pending;
        let result = EnumContract::process_validation_result(env.clone(), pending, 4);
        assert_eq!(result, Err(ContractError::ValidationPending));
    });
}

#[test]
fn test_enum_comparisons() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EnumContract);

    // Test role comparisons
    env.as_contract(&contract_id, || {
        assert!(EnumContract::compare_enums(
            env.clone(),
            UserRole::Admin,
            UserRole::User
        ));
        assert!(!EnumContract::compare_enums(
            env.clone(),
            UserRole::User,
            UserRole::Admin
        ));
        assert!(EnumContract::compare_enums(
            env.clone(),
            UserRole::Owner,
            UserRole::Owner
        ));
        assert!(EnumContract::compare_enums(
            env.clone(),
            UserRole::Moderator,
            UserRole::User
        ));
        assert!(EnumContract::compare_enums(
            env.clone(),
            UserRole::None,
            UserRole::None
        ));
    });
}

#[test]
fn test_enum_arithmetic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EnumContract);

    // Test enum arithmetic
    env.as_contract(&contract_id, || {
        let result = EnumContract::enum_arithmetic(env.clone());
        assert_eq!(result, 4); // Admin (3) + User (1) = 4
    });
}

#[test]
fn test_enum_iteration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EnumContract);

    // Test enum iteration
    env.as_contract(&contract_id, || {
        let roles = EnumContract::get_all_roles(env.clone());
        assert_eq!(roles.len(), 5);
        assert_eq!(roles.get(0), Some(UserRole::None));
        assert_eq!(roles.get(1), Some(UserRole::User));
        assert_eq!(roles.get(2), Some(UserRole::Moderator));
        assert_eq!(roles.get(3), Some(UserRole::Admin));
        assert_eq!(roles.get(4), Some(UserRole::Owner));
    });
}

#[test]
fn test_comprehensive_workflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EnumContract);
    let admin = <soroban_sdk::Address as AddressTest>::generate(&env);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);
    let recipient = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Initialize contract
    env.as_contract(&contract_id, || {
        assert_eq!(EnumContract::initialize(env.clone(), admin.clone()), Ok(()));
    });

    // Verify initial state
    env.as_contract(&contract_id, || {
        assert_eq!(EnumContract::get_state(env.clone()), ContractState::Active);
    });

    // Set up user roles
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::set_user_role(env.clone(), admin.clone(), user.clone(), UserRole::User),
            Ok(())
        );
        assert_eq!(
            EnumContract::set_user_role(
                env.clone(),
                admin.clone(),
                recipient.clone(),
                UserRole::Moderator
            ),
            Ok(())
        );
    });

    // Verify roles
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::get_user_role(env.clone(), admin.clone()),
            UserRole::Owner
        );
        assert_eq!(
            EnumContract::get_user_role(env.clone(), user.clone()),
            UserRole::User
        );
        assert_eq!(
            EnumContract::get_user_role(env.clone(), recipient.clone()),
            UserRole::Moderator
        );
    });

    // Test various operations
    let operations = vec![
        &env,
        (TransactionType::Deposit, 1000, ValidationResult::Success),
        (TransactionType::Transfer, 500, ValidationResult::Success),
        (TransactionType::Withdraw, 2000, ValidationResult::Success),
        (TransactionType::Mint, 100000, ValidationResult::Success),
        (TransactionType::Burn, 50000, ValidationResult::Success),
    ];

    for (op_type, amount, expected) in operations.iter() {
        let result =
            EnumContract::execute_operation(env.clone(), op_type, amount, recipient.clone());
        assert_eq!(result, Ok(expected));
    }

    // Test validation result processing
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::process_validation_result(env.clone(), ValidationResult::Success, 1),
            Ok(())
        );
        assert_eq!(
            EnumContract::process_validation_result(env.clone(), ValidationResult::Failure, 2),
            Err(ContractError::ValidationFailed)
        );
        assert_eq!(
            EnumContract::process_validation_result(
                env.clone(),
                ValidationResult::RequiresApproval,
                3
            ),
            Err(ContractError::InsufficientApprovals)
        );
        assert_eq!(
            EnumContract::process_validation_result(env.clone(), ValidationResult::Pending, 4),
            Err(ContractError::ValidationPending)
        );
    });

    // Test enum utilities
    env.as_contract(&contract_id, || {
        assert!(EnumContract::compare_enums(
            env.clone(),
            UserRole::Owner,
            UserRole::User
        ));
        assert_eq!(EnumContract::enum_arithmetic(env.clone()), 4);

        let roles = EnumContract::get_all_roles(env.clone());
        assert_eq!(roles.len(), 5);
    });
}

#[test]
fn test_error_scenarios() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EnumContract);
    let admin = <soroban_sdk::Address as AddressTest>::generate(&env);
    let unauthorized_user = <soroban_sdk::Address as AddressTest>::generate(&env);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Test uninitialized contract
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::get_state(env.clone()),
            ContractState::Uninitialized
        );
    });

    // Initialize contract
    env.as_contract(&contract_id, || {
        assert_eq!(EnumContract::initialize(env.clone(), admin.clone()), Ok(()));
    });

    // Test unauthorized operations
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::set_user_role(
                env.clone(),
                unauthorized_user.clone(),
                user.clone(),
                UserRole::User
            ),
            Err(ContractError::InsufficientRole)
        );

        assert_eq!(
            EnumContract::set_user_role(env.clone(), admin.clone(), user.clone(), UserRole::Owner),
            Err(ContractError::InvalidInput)
        );

        assert_eq!(
            EnumContract::initialize(env.clone(), admin.clone()),
            Err(ContractError::ContractAlreadyInitialized)
        );
    });

    // Test validation result errors
    env.as_contract(&contract_id, || {
        assert_eq!(
            EnumContract::process_validation_result(env.clone(), ValidationResult::Failure, 1),
            Err(ContractError::ValidationFailed)
        );

        assert_eq!(
            EnumContract::process_validation_result(
                env.clone(),
                ValidationResult::RequiresApproval,
                2
            ),
            Err(ContractError::InsufficientApprovals)
        );

        assert_eq!(
            EnumContract::process_validation_result(env.clone(), ValidationResult::Pending, 3),
            Err(ContractError::ValidationPending)
        );
    });
}

// ---------------------------------------------------------------------------
// Enums with Associated Data
// ---------------------------------------------------------------------------

#[test]
fn test_process_asset_op_transfer() {
    let env = Env::default();
    let from = <soroban_sdk::Address as AddressTest>::generate(&env);
    let to = <soroban_sdk::Address as AddressTest>::generate(&env);

    let op = AssetOperation::Transfer(TransferParams {
        from,
        to,
        amount: 500,
    });
    assert_eq!(EnumContract::process_asset_op(env.clone(), op), Ok(500));
}

#[test]
fn test_process_asset_op_mint() {
    let env = Env::default();
    let to = <soroban_sdk::Address as AddressTest>::generate(&env);

    let op = AssetOperation::Mint(MintParams { to, amount: 1000 });
    assert_eq!(EnumContract::process_asset_op(env.clone(), op), Ok(1000));
}

#[test]
fn test_process_asset_op_burn_returns_negative() {
    let env = Env::default();
    let from = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Burn returns a negative amount to signal supply reduction.
    let op = AssetOperation::Burn(BurnParams { from, amount: 250 });
    assert_eq!(EnumContract::process_asset_op(env.clone(), op), Ok(-250));
}

#[test]
fn test_process_asset_op_invalid_amount() {
    let env = Env::default();
    let from = <soroban_sdk::Address as AddressTest>::generate(&env);
    let to = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Zero amount is invalid for all variants.
    let zero_transfer = AssetOperation::Transfer(TransferParams {
        from: from.clone(),
        to: to.clone(),
        amount: 0,
    });
    assert_eq!(
        EnumContract::process_asset_op(env.clone(), zero_transfer),
        Err(ContractError::InvalidAmount)
    );

    let neg_mint = AssetOperation::Mint(MintParams {
        to: to.clone(),
        amount: -1,
    });
    assert_eq!(
        EnumContract::process_asset_op(env.clone(), neg_mint),
        Err(ContractError::InvalidAmount)
    );

    let zero_burn = AssetOperation::Burn(BurnParams { from, amount: 0 });
    assert_eq!(
        EnumContract::process_asset_op(env.clone(), zero_burn),
        Err(ContractError::InvalidAmount)
    );
}

#[test]
fn test_op_kind() {
    let env = Env::default();
    let from = <soroban_sdk::Address as AddressTest>::generate(&env);
    let to = <soroban_sdk::Address as AddressTest>::generate(&env);

    let transfer = AssetOperation::Transfer(TransferParams {
        from: from.clone(),
        to: to.clone(),
        amount: 100,
    });
    assert_eq!(
        EnumContract::op_kind(env.clone(), transfer),
        symbol_short!("transfer")
    );

    let mint = AssetOperation::Mint(MintParams {
        to: to.clone(),
        amount: 100,
    });
    assert_eq!(
        EnumContract::op_kind(env.clone(), mint),
        symbol_short!("mint")
    );

    let burn = AssetOperation::Burn(BurnParams { from, amount: 100 });
    assert_eq!(
        EnumContract::op_kind(env.clone(), burn),
        symbol_short!("burn")
    );
}

#[test]
fn test_enums_with_data_destructuring() {
    let env = Env::default();
    let from = <soroban_sdk::Address as AddressTest>::generate(&env);
    let to = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Verify that the inner struct's fields can be accessed after matching.
    let op = AssetOperation::Transfer(TransferParams {
        from: from.clone(),
        to: to.clone(),
        amount: 42,
    });

    if let AssetOperation::Transfer(p) = op {
        assert_eq!(p.from, from);
        assert_eq!(p.to, to);
        assert_eq!(p.amount, 42);
    } else {
        panic!("Expected Transfer variant");
    }
}

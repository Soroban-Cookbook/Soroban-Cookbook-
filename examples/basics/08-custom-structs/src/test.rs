#![cfg(test)]
use super::*;
use soroban_sdk::{Env, String, Vec};
use soroban_sdk::testutils::Address as AddressTest;

#[test]
fn test_basic_struct_creation() {
    let env = Env::default();
        let user = <soroban_sdk::Address as AddressTest>::generate(&env);
    let name = String::from_str(&env, "Alice");
    let email = String::from_str(&env, "alice@example.com");

    // Test UserProfile creation
    let profile = UserProfile {
        address: user.clone(),
        name: name.clone(),
        email: Some(email.clone()),
        avatar_hash: None,
        reputation: 100,
        verified: false,
        created_at: 1234567890,
    };

    assert_eq!(profile.address, user);
    assert_eq!(profile.name, name);
    assert_eq!(profile.email, Some(email));
    assert_eq!(profile.reputation, 100);
    assert_eq!(profile.verified, false);
}

#[test]
fn test_asset_info_struct() {
    let env = Env::default();
    let asset_contract = <soroban_sdk::Address as AddressTest>::generate(&env);
    let symbol = String::from_str(&env, "BTC");
    let name = String::from_str(&env, "Bitcoin");

    let asset = AssetInfo {
        contract_address: asset_contract.clone(),
        symbol: symbol.clone(),
        name: name.clone(),
        decimals: 8,
        total_supply: Some(21000000),
        native: false,
    };

    assert_eq!(asset.contract_address, asset_contract);
    assert_eq!(asset.symbol, symbol);
    assert_eq!(asset.name, name);
    assert_eq!(asset.decimals, 8);
    assert_eq!(asset.total_supply, Some(21000000));
    assert_eq!(asset.native, false);
}

#[test]
fn test_transaction_struct() {
    let env = Env::default();
    let from = <soroban_sdk::Address as AddressTest>::generate(&env);
    let to = <soroban_sdk::Address as AddressTest>::generate(&env);
    let asset_contract = <soroban_sdk::Address as AddressTest>::generate(&env);
    let memo = String::from_str(&env, "Payment for services");

    let transaction = Transaction {
        id: 12345,
        from: from.clone(),
        to: to.clone(),
        asset: AssetInfo {
            contract_address: asset_contract.clone(),
            symbol: String::from_str(&env, "USD"),
            name: String::from_str(&env, "US Dollar"),
            decimals: 2,
            total_supply: None,
            native: true,
        },
        amount: 1000,
        timestamp: env.ledger().timestamp(),
        memo: Some(memo.clone()),
        status: TransactionStatus::Completed,
    };

    assert_eq!(transaction.id, 12345);
    assert_eq!(transaction.from, from);
    assert_eq!(transaction.to, to);
    assert_eq!(transaction.amount, 1000);
    assert_eq!(transaction.status, TransactionStatus::Completed);
    assert_eq!(transaction.memo, Some(memo));
}

#[test]
fn test_nested_structs() {
    let env = Env::default();
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);

    let extended_profile = ExtendedUserProfile {
        profile: UserProfile {
            address: user.clone(),
            name: String::from_str(&env, "Bob"),
            email: Some(String::from_str(&env, "bob@example.com")),
            avatar_hash: None,
            reputation: 500,
            verified: true,
            created_at: 1234567890,
        },
        preferences: UserPreferences {
            language: String::from_str(&env, "en"),
            theme: Theme::Dark,
            notifications: NotificationSettings {
                email_enabled: true,
                push_enabled: false,
                transaction_notifications: true,
                marketing_notifications: false,
            },
            privacy: PrivacySettings {
                profile_visibility: Visibility::Friends,
                show_online_status: false,
                allow_direct_messages: true,
            },
        },
        statistics: UserStatistics {
            total_transactions: 100,
            total_volume: 50000,
            successful_transactions: 95,
            failed_transactions: 5,
            avg_transaction_size: 500,
            last_activity: env.ledger().timestamp(),
        },
        security: SecuritySettings {
            two_factor_enabled: true,
            session_timeout: 7200,
            daily_transaction_limit: 100000,
            large_transaction_threshold: 5000,
            trusted_devices: Vec::new(&env),
        },
    };

    // Test nested access
    assert_eq!(extended_profile.profile.name, String::from_str(&env, "Bob"));
    assert_eq!(extended_profile.preferences.theme, Theme::Dark);
    assert_eq!(extended_profile.preferences.notifications.email_enabled, true);
    assert_eq!(extended_profile.statistics.total_transactions, 100);
    assert_eq!(extended_profile.security.two_factor_enabled, true);
}

#[test]
fn test_portfolio_struct() {
    let env = Env::default();
    let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
    let asset_contract = <soroban_sdk::Address as AddressTest>::generate(&env);

    let portfolio = Portfolio {
        owner: owner.clone(),
        name: String::from_str(&env, "My Portfolio"),
        description: Some(String::from_str(&env, "A diversified investment portfolio")),
        holdings: vec![
            &env,
            AssetHolding {
                asset: AssetInfo {
                    contract_address: asset_contract.clone(),
                    symbol: String::from_str(&env, "ETH"),
                    name: String::from_str(&env, "Ethereum"),
                    decimals: 18,
                    total_supply: None,
                    native: false,
                },
                quantity: 1000000000000000000, // 1 ETH in wei
                avg_purchase_price: 2000,
                current_value: Some(2500),
                purchase_history: vec![
                    &env,
                    PurchaseRecord {
                        timestamp: 1234567890,
                        quantity: 1000000000000000000,
                        price: 2000,
                        fee: 10,
                    },
                ],
            },
        ],
        metadata: PortfolioMetadata {
            portfolio_type: PortfolioType::Balanced,
            risk_level: RiskLevel::Medium,
            strategy: String::from_str(&env, "Buy and hold"),
            target_allocations: vec![
                &env,
                AssetAllocation {
                    asset: AssetInfo {
                        contract_address: asset_contract.clone(),
                        symbol: String::from_str(&env, "ETH"),
                        name: String::from_str(&env, "Ethereum"),
                        decimals: 18,
                        total_supply: None,
                        native: false,
                    },
                    target_percentage: 60,
                    current_percentage: 60,
                },
            ],
            performance: PerformanceMetrics {
                total_return: 25,
                annual_return: 12,
                sharpe_ratio: Some(150),
                max_drawdown: -15,
                volatility: 20,
            },
        },
        last_updated: env.ledger().timestamp(),
    };

    assert_eq!(portfolio.owner, owner);
    assert_eq!(portfolio.name, String::from_str(&env, "My Portfolio"));
    assert_eq!(portfolio.holdings.len(), 1);
    assert_eq!(portfolio.metadata.portfolio_type, PortfolioType::Balanced);
    assert_eq!(portfolio.metadata.risk_level, RiskLevel::Medium);
}

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomStructsContract);
    let admin = <soroban_sdk::Address as AddressTest>::generate(&env);

    env.as_contract(&contract_id, || {
        assert_eq!(CustomStructsContract::initialize(env.clone(), admin.clone()), Ok(()));
        
        // Test double initialization
        assert_eq!(
            CustomStructsContract::initialize(env.clone(), admin.clone()),
            Err(ContractError::AlreadyExists)
        );
    });
}

#[test]
fn test_user_profile_management() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomStructsContract);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);
    let name = String::from_str(&env, "Charlie");
    let email = String::from_str(&env, "charlie@example.com");

    env.as_contract(&contract_id, || {
        // Initialize contract
        CustomStructsContract::initialize(env.clone(), user.clone()).unwrap();

        // Create user profile
        let profile = CustomStructsContract::create_user_profile(
            env.clone(),
            user.clone(),
            name.clone(),
            Some(email.clone()),
        ).unwrap();

        assert_eq!(profile.address, user);
        assert_eq!(profile.name, name);
        assert_eq!(profile.email, Some(email.clone()));
        assert_eq!(profile.reputation, 0);
        assert_eq!(profile.verified, false);

        // Get user profile
        let retrieved_profile = CustomStructsContract::get_user_profile(env.clone(), user.clone()).unwrap();
        assert_eq!(profile, retrieved_profile);

        // Update user profile
        let new_name = String::from_str(&env, "Charlie Updated");
        let updated_profile = CustomStructsContract::update_user_profile(
            env.clone(),
            user.clone(),
            Some(new_name.clone()),
            None,
            None,
        ).unwrap();

        assert_eq!(updated_profile.name, new_name);
        assert_eq!(updated_profile.email, Some(email.clone())); // Should remain unchanged
    });
}

#[test]
fn test_portfolio_management() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomStructsContract);
    let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
    let asset_contract = <soroban_sdk::Address as AddressTest>::generate(&env);

    env.as_contract(&contract_id, || {
        // Initialize contract
        CustomStructsContract::initialize(env.clone(), owner.clone()).unwrap();

        // Create portfolio
        let portfolio_name = String::from_str(&env, "Test Portfolio");
        let portfolio = CustomStructsContract::create_portfolio(
            env.clone(),
            owner.clone(),
            portfolio_name.clone(),
            Some(String::from_str(&env, "A test portfolio")),
            PortfolioType::Balanced,
        ).unwrap();

        assert_eq!(portfolio.owner, owner);
        assert_eq!(portfolio.name, portfolio_name);
        assert_eq!(portfolio.holdings.len(), 0);

        // Get portfolio
        let retrieved_portfolio = CustomStructsContract::get_portfolio(
            env.clone(),
            owner.clone(),
            portfolio_name.clone(),
        ).unwrap();

        assert_eq!(portfolio, retrieved_portfolio);

        // Add asset to portfolio
        let asset = AssetInfo {
            contract_address: asset_contract.clone(),
            symbol: String::from_str(&env, "BTC"),
            name: String::from_str(&env, "Bitcoin"),
            decimals: 8,
            total_supply: None,
            native: false,
        };

        CustomStructsContract::add_asset_to_portfolio(
            env.clone(),
            owner.clone(),
            portfolio_name.clone(),
            asset,
            100000000, // 1 BTC in satoshis
            50000,    // $50,000
        ).unwrap();

        // Verify asset was added
        let updated_portfolio = CustomStructsContract::get_portfolio(
            env.clone(),
            owner.clone(),
            portfolio_name.clone(),
        ).unwrap();

        assert_eq!(updated_portfolio.holdings.len(), 1);
    });
}

#[test]
fn test_extended_profile() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomStructsContract);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);
    let name = String::from_str(&env, "Diana");
    let language = String::from_str(&env, "en");

    env.as_contract(&contract_id, || {
        // Initialize contract
        CustomStructsContract::initialize(env.clone(), user.clone()).unwrap();

        // Create extended profile
        let extended_profile = CustomStructsContract::create_extended_profile(
            env.clone(),
            user.clone(),
            name.clone(),
            language.clone(),
        ).unwrap();

        assert_eq!(extended_profile.profile.name, name);
        assert_eq!(extended_profile.preferences.language, language);
        assert_eq!(extended_profile.preferences.theme, Theme::Auto);
        assert_eq!(extended_profile.preferences.notifications.email_enabled, true);
        assert_eq!(extended_profile.statistics.total_transactions, 0);
        assert_eq!(extended_profile.security.two_factor_enabled, false);

        // Get extended profile
        let retrieved_profile = CustomStructsContract::get_extended_profile(env.clone(), user.clone()).unwrap();
        assert_eq!(extended_profile, retrieved_profile);
    });
}

#[test]
fn test_struct_validation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomStructsContract);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);

    env.as_contract(&contract_id, || {
        // Initialize contract
        CustomStructsContract::initialize(env.clone(), user.clone()).unwrap();

        // Test valid profile
        let valid_profile = UserProfile {
            address: user.clone(),
            name: String::from_str(&env, "Valid Name"),
            email: Some(String::from_str(&env, "valid@example.com")),
            avatar_hash: None,
            reputation: 500,
            verified: false,
            created_at: env.ledger().timestamp(),
        };

        assert_eq!(CustomStructsContract::validate_struct(env.clone(), valid_profile), Ok(true));

        // Test invalid profile (empty name)
        let invalid_profile = UserProfile {
            address: user.clone(),
            name: String::from_str(&env, ""), // Empty name
            email: Some(String::from_str(&env, "valid@example.com")),
            avatar_hash: None,
            reputation: 500,
            verified: false,
            created_at: env.ledger().timestamp(),
        };

        assert_eq!(
            CustomStructsContract::validate_struct(env.clone(), invalid_profile),
            Err(ContractError::InvalidFieldValue)
        );

        // Test invalid profile (reputation too high)
        let invalid_profile2 = UserProfile {
            address: user.clone(),
            name: String::from_str(&env, "Valid Name"),
            email: Some(String::from_str(&env, "valid@example.com")),
            avatar_hash: None,
            reputation: 2000, // Too high
            verified: false,
            created_at: env.ledger().timestamp(),
        };

        assert_eq!(
            CustomStructsContract::validate_struct(env.clone(), invalid_profile2),
            Err(ContractError::InvalidFieldValue)
        );
    });
}

#[test]
fn test_serialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomStructsContract);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);

    env.as_contract(&contract_id, || {
        // Initialize contract
        CustomStructsContract::initialize(env.clone(), user.clone()).unwrap();

        let profile = UserProfile {
            address: user.clone(),
            name: String::from_str(&env, "Test User"),
            email: Some(String::from_str(&env, "test@example.com")),
            avatar_hash: None,
            reputation: 100,
            verified: false,
            created_at: env.ledger().timestamp(),
        };

        // Test serialization (conceptual)
        let serialized = CustomStructsContract::serialize_struct(env.clone(), profile).unwrap();
        assert_eq!(serialized, 12345); // Placeholder check
    });
}

#[test]
fn test_portfolio_value_calculation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomStructsContract);
    let owner = <soroban_sdk::Address as AddressTest>::generate(&env);
    let asset_contract = <soroban_sdk::Address as AddressTest>::generate(&env);

    env.as_contract(&contract_id, || {
        // Initialize contract
        CustomStructsContract::initialize(env.clone(), owner.clone()).unwrap();

        // Create portfolio
        let portfolio_name = String::from_str(&env, "Value Test Portfolio");
        CustomStructsContract::create_portfolio(
            env.clone(),
            owner.clone(),
            portfolio_name.clone(),
            None,
            PortfolioType::Balanced,
        ).unwrap();

        // Add assets with current values
        let asset1 = AssetInfo {
            contract_address: asset_contract.clone(),
            symbol: String::from_str(&env, "BTC"),
            name: String::from_str(&env, "Bitcoin"),
            decimals: 8,
            total_supply: None,
            native: false,
        };

        CustomStructsContract::add_asset_to_portfolio(
            env.clone(),
            owner.clone(),
            portfolio_name.clone(),
            asset1,
            100000000, // 1 BTC
            50000,    // $50,000 purchase price
        ).unwrap();

        // Calculate portfolio value
        let value = CustomStructsContract::calculate_portfolio_value(
            env.clone(),
            owner.clone(),
            portfolio_name.clone(),
        ).unwrap();

        // Should equal purchase price since no current value is set
        // 1 BTC * 50000 = 5000000000000 (in satoshis)
        assert_eq!(value, 5000000000000);
    });
}

#[test]
fn test_error_handling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CustomStructsContract);
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);
    let _unauthorized_user = <soroban_sdk::Address as AddressTest>::generate(&env);

    env.as_contract(&contract_id, || {
        // Test getting non-existent profile
        assert_eq!(
            CustomStructsContract::get_user_profile(env.clone(), user.clone()),
            Err(ContractError::UserNotFound)
        );

        // Test getting non-existent portfolio
        assert_eq!(
            CustomStructsContract::get_portfolio(
                env.clone(),
                user.clone(),
                String::from_str(&env, "Non-existent")
            ),
            Err(ContractError::PortfolioNotFound)
        );

        // Test getting non-existent extended profile
        assert_eq!(
            CustomStructsContract::get_extended_profile(env.clone(), user.clone()),
            Err(ContractError::UserNotFound)
        );
    });
}

#[test]
fn test_complex_nested_structures() {
    let env = Env::default();
    let user = <soroban_sdk::Address as AddressTest>::generate(&env);
    let asset_contract = <soroban_sdk::Address as AddressTest>::generate(&env);

    // Create deeply nested structure
    let complex_portfolio = Portfolio {
        owner: user.clone(),
        name: String::from_str(&env, "Complex Portfolio"),
        description: Some(String::from_str(&env, "A complex nested portfolio")),
        holdings: vec![
            &env,
            AssetHolding {
                asset: AssetInfo {
                    contract_address: asset_contract.clone(),
                    symbol: String::from_str(&env, "ETH"),
                    name: String::from_str(&env, "Ethereum"),
                    decimals: 18,
                    total_supply: None,
                    native: false,
                },
                quantity: 2000000000000000000, // 2 ETH
                avg_purchase_price: 1500,
                current_value: Some(3000),
                purchase_history: vec![
                    &env,
                    PurchaseRecord {
                        timestamp: 1234567890,
                        quantity: 1000000000000000000,
                        price: 1500,
                        fee: 5,
                    },
                    PurchaseRecord {
                        timestamp: 1234567900,
                        quantity: 1000000000000000000,
                        price: 1600,
                        fee: 5,
                    },
                ],
            },
        ],
        metadata: PortfolioMetadata {
            portfolio_type: PortfolioType::Aggressive,
            risk_level: RiskLevel::High,
            strategy: String::from_str(&env, "Growth focused"),
            target_allocations: vec![
                &env,
                AssetAllocation {
                    asset: AssetInfo {
                        contract_address: asset_contract.clone(),
                        symbol: String::from_str(&env, "ETH"),
                        name: String::from_str(&env, "Ethereum"),
                        decimals: 18,
                        total_supply: None,
                        native: false,
                    },
                    target_percentage: 80,
                    current_percentage: 100,
                },
            ],
            performance: PerformanceMetrics {
                total_return: 100,
                annual_return: 50,
                sharpe_ratio: Some(200),
                max_drawdown: -25,
                volatility: 30,
            },
        },
        last_updated: env.ledger().timestamp(),
    };

    // Test deep nesting access
    assert_eq!(complex_portfolio.holdings.len(), 1);
    assert_eq!(complex_portfolio.holdings.get(0).unwrap().quantity, 2000000000000000000);
    assert_eq!(complex_portfolio.holdings.get(0).unwrap().purchase_history.len(), 2);
    assert_eq!(complex_portfolio.metadata.portfolio_type, PortfolioType::Aggressive);
    assert_eq!(complex_portfolio.metadata.target_allocations.len(), 1);
    assert_eq!(complex_portfolio.metadata.performance.total_return, 100);
}

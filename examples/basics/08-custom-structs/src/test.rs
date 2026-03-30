use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String, Vec};

fn setup(env: &Env) -> CustomStructsContractClient {
    let id = env.register_contract(None, CustomStructsContract);
    CustomStructsContractClient::new(env, &id)
}

#[test]
fn test_basic_struct_creation() {
    let env = Env::default();
    let user = Address::generate(&env);
    let name = String::from_str(&env, "Alice");
    let email = String::from_str(&env, "alice@example.com");

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
    assert!(!profile.verified);
}

#[test]
fn test_asset_info_struct() {
    let env = Env::default();
    let asset_contract = Address::generate(&env);
    let symbol = String::from_str(&env, "BTC");
    let name = String::from_str(&env, "Bitcoin");

    let asset = AssetInfo {
        contract_address: asset_contract.clone(),
        symbol: symbol.clone(),
        name: name.clone(),
        decimals: 8,
        total_supply: Some(21_000_000),
        native: false,
    };

    assert_eq!(asset.contract_address, asset_contract);
    assert_eq!(asset.symbol, symbol);
    assert_eq!(asset.decimals, 8);
    assert_eq!(asset.total_supply, Some(21_000_000));
    assert!(!asset.native);
}

#[test]
fn test_transaction_struct() {
    let env = Env::default();
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let asset_contract = Address::generate(&env);
    let memo = String::from_str(&env, "Payment for services");

    let tx = Transaction {
        id: 12345,
        from: from.clone(),
        to: to.clone(),
        asset: AssetInfo {
            contract_address: asset_contract,
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

    assert_eq!(tx.id, 12345);
    assert_eq!(tx.from, from);
    assert_eq!(tx.to, to);
    assert_eq!(tx.amount, 1000);
    assert_eq!(tx.status, TransactionStatus::Completed);
    assert_eq!(tx.memo, Some(memo));
}

#[test]
fn test_nested_structs() {
    let env = Env::default();
    let user = Address::generate(&env);

    let extended = ExtendedUserProfile {
        profile: UserProfile {
            address: user,
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
            total_volume: 50_000,
            successful_transactions: 95,
            failed_transactions: 5,
            avg_transaction_size: 500,
            last_activity: env.ledger().timestamp(),
        },
        security: SecuritySettings {
            two_factor_enabled: true,
            session_timeout: 7200,
            daily_transaction_limit: 100_000,
            large_transaction_threshold: 5000,
            trusted_devices: Vec::new(&env),
        },
    };

    assert_eq!(extended.profile.name, String::from_str(&env, "Bob"));
    assert_eq!(extended.preferences.theme, Theme::Dark);
    assert!(extended.preferences.notifications.email_enabled);
    assert_eq!(extended.statistics.total_transactions, 100);
    assert!(extended.security.two_factor_enabled);
}

#[test]
fn test_portfolio_struct() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let asset_contract = Address::generate(&env);

    let portfolio = Portfolio {
        owner: owner.clone(),
        name: String::from_str(&env, "My Portfolio"),
        description: Some(String::from_str(&env, "A diversified portfolio")),
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
                quantity: 1_000_000_000_000_000_000,
                avg_purchase_price: 2000,
                current_value: Some(2500),
                purchase_history: vec![
                    &env,
                    PurchaseRecord {
                        timestamp: 1234567890,
                        quantity: 1_000_000_000_000_000_000,
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
                        contract_address: asset_contract,
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
    assert_eq!(portfolio.holdings.len(), 1);
    assert_eq!(portfolio.metadata.portfolio_type, PortfolioType::Balanced);
}

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    let client = setup(&env);
    let admin = Address::generate(&env);

    assert_eq!(client.try_initialize(&admin), Ok(Ok(())));
    assert_eq!(
        client.try_initialize(&admin),
        Err(Ok(ContractError::AlreadyExists))
    );
}

#[test]
fn test_user_profile_management() {
    let env = Env::default();
    let client = setup(&env);
    let user = Address::generate(&env);
    let name = String::from_str(&env, "Charlie");
    let email = String::from_str(&env, "charlie@example.com");

    client.initialize(&user);

    let profile = client
        .create_user_profile(&user, &name, &Some(email.clone()))
        .unwrap();
    assert_eq!(profile.address, user);
    assert_eq!(profile.name, name);
    assert_eq!(profile.email, Some(email.clone()));
    assert_eq!(profile.reputation, 0);
    assert!(!profile.verified);

    let retrieved = client.get_user_profile(&user).unwrap();
    assert_eq!(profile, retrieved);

    let new_name = String::from_str(&env, "Charlie Updated");
    let updated = client
        .update_user_profile(&user, &Some(new_name.clone()), &None, &None)
        .unwrap();
    assert_eq!(updated.name, new_name);
    assert_eq!(updated.email, Some(email));
}

#[test]
fn test_portfolio_management() {
    let env = Env::default();
    let client = setup(&env);
    let owner = Address::generate(&env);
    let asset_contract = Address::generate(&env);

    client.initialize(&owner);

    let portfolio_name = String::from_str(&env, "Test Portfolio");
    let portfolio = client
        .create_portfolio(
            &owner,
            &portfolio_name,
            &Some(String::from_str(&env, "A test portfolio")),
            &PortfolioType::Balanced,
        )
        .unwrap();

    assert_eq!(portfolio.owner, owner);
    assert_eq!(portfolio.holdings.len(), 0);

    let retrieved = client.get_portfolio(&owner, &portfolio_name).unwrap();
    assert_eq!(portfolio, retrieved);

    let asset = AssetInfo {
        contract_address: asset_contract,
        symbol: String::from_str(&env, "BTC"),
        name: String::from_str(&env, "Bitcoin"),
        decimals: 8,
        total_supply: None,
        native: false,
    };

    client
        .add_asset_to_portfolio(&owner, &portfolio_name, &asset, &100_000_000, &50_000)
        .unwrap();

    let updated = client.get_portfolio(&owner, &portfolio_name).unwrap();
    assert_eq!(updated.holdings.len(), 1);
}

#[test]
fn test_extended_profile() {
    let env = Env::default();
    let client = setup(&env);
    let user = Address::generate(&env);
    let name = String::from_str(&env, "Diana");
    let language = String::from_str(&env, "en");

    client.initialize(&user);

    let extended = client
        .create_extended_profile(&user, &name, &language)
        .unwrap();
    assert_eq!(extended.profile.name, name);
    assert_eq!(extended.preferences.language, language);
    assert_eq!(extended.preferences.theme, Theme::Auto);
    assert!(extended.preferences.notifications.email_enabled);
    assert_eq!(extended.statistics.total_transactions, 0);
    assert!(!extended.security.two_factor_enabled);

    let retrieved = client.get_extended_profile(&user).unwrap();
    assert_eq!(extended, retrieved);
}

#[test]
fn test_struct_validation() {
    let env = Env::default();
    let client = setup(&env);
    let user = Address::generate(&env);

    client.initialize(&user);

    let valid = UserProfile {
        address: user.clone(),
        name: String::from_str(&env, "Valid Name"),
        email: Some(String::from_str(&env, "valid@example.com")),
        avatar_hash: None,
        reputation: 500,
        verified: false,
        created_at: env.ledger().timestamp(),
    };
    assert_eq!(client.try_validate_struct(&valid), Ok(Ok(true)));

    let empty_name = UserProfile {
        address: user.clone(),
        name: String::from_str(&env, ""),
        email: None,
        avatar_hash: None,
        reputation: 0,
        verified: false,
        created_at: 0,
    };
    assert_eq!(
        client.try_validate_struct(&empty_name),
        Err(Ok(ContractError::InvalidFieldValue))
    );

    let high_rep = UserProfile {
        address: user,
        name: String::from_str(&env, "Valid Name"),
        email: None,
        avatar_hash: None,
        reputation: 2000,
        verified: false,
        created_at: 0,
    };
    assert_eq!(
        client.try_validate_struct(&high_rep),
        Err(Ok(ContractError::InvalidFieldValue))
    );
}

#[test]
fn test_serialization() {
    let env = Env::default();
    let client = setup(&env);
    let user = Address::generate(&env);

    client.initialize(&user);

    let profile = UserProfile {
        address: user,
        name: String::from_str(&env, "Test User"),
        email: Some(String::from_str(&env, "test@example.com")),
        avatar_hash: None,
        reputation: 100,
        verified: false,
        created_at: env.ledger().timestamp(),
    };

    assert_eq!(client.serialize_struct(&profile).unwrap(), 12345);
}

#[test]
fn test_portfolio_value_calculation() {
    let env = Env::default();
    let client = setup(&env);
    let owner = Address::generate(&env);
    let asset_contract = Address::generate(&env);

    client.initialize(&owner);

    let portfolio_name = String::from_str(&env, "Value Test Portfolio");
    client
        .create_portfolio(&owner, &portfolio_name, &None, &PortfolioType::Balanced)
        .unwrap();

    let asset = AssetInfo {
        contract_address: asset_contract,
        symbol: String::from_str(&env, "BTC"),
        name: String::from_str(&env, "Bitcoin"),
        decimals: 8,
        total_supply: None,
        native: false,
    };

    client
        .add_asset_to_portfolio(&owner, &portfolio_name, &asset, &100_000_000, &50_000)
        .unwrap();

    let value = client
        .calculate_portfolio_value(&owner, &portfolio_name)
        .unwrap();
    assert_eq!(value, 5_000_000_000_000);
}

#[test]
fn test_error_handling() {
    let env = Env::default();
    let client = setup(&env);
    let user = Address::generate(&env);

    assert_eq!(
        client.try_get_user_profile(&user),
        Err(Ok(ContractError::UserNotFound))
    );
    assert_eq!(
        client.try_get_portfolio(&user, &String::from_str(&env, "Non-existent")),
        Err(Ok(ContractError::PortfolioNotFound))
    );
    assert_eq!(
        client.try_get_extended_profile(&user),
        Err(Ok(ContractError::UserNotFound))
    );
}

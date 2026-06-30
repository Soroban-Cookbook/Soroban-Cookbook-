use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Vec, symbol_short,
};

fn setup() -> (Env, Address, Address, PriceOracleContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PriceOracleContract, ());
    let client = PriceOracleContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let updater = Address::generate(&env);
    client.initialize(&admin);
    client.add_updater(&admin, &updater);
    (env, admin, updater, client)
}

#[test]
fn test_initialize() {
    let (_env, admin, _updater, client) = setup();
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(OracleError::AlreadyInitialized)));
}

#[test]
fn test_add_remove_updater() {
    let (env, admin, _updater, client) = setup();
    let updater2 = Address::generate(&env);

    client.add_updater(&admin, &updater2);

    let prices = Vec::from_array(&env, [(symbol_short!("BTC"), 50000i128)]);

    // Config asset first
    let config = AssetConfig { max_age: 300, twap_window: 3600 };
    client.set_asset_config(&admin, &symbol_short!("BTC"), &config);

    client.submit_prices(&updater2, &prices);

    client.remove_updater(&admin, &updater2);
    let result = client.try_submit_prices(&updater2, &prices);
    assert_eq!(result, Err(Ok(OracleError::Unauthorized)));
}

#[test]
fn test_submit_prices_unauthorized() {
    let (env, _admin, _updater, client) = setup();
    let stranger = Address::generate(&env);
    let prices = Vec::from_array(&env, [(symbol_short!("BTC"), 50000i128)]);
    let result = client.try_submit_prices(&stranger, &prices);
    assert_eq!(result, Err(Ok(OracleError::Unauthorized)));
}

#[test]
fn test_submit_prices_invalid_asset() {
    let (env, _admin, updater, client) = setup();
    let prices = Vec::from_array(&env, [(symbol_short!("BTC"), 50000i128)]);
    let result = client.try_submit_prices(&updater, &prices);
    assert_eq!(result, Err(Ok(OracleError::InvalidAsset)));
}

#[test]
fn test_submit_prices_invalid_price() {
    let (env, admin, updater, client) = setup();
    client.set_asset_config(&admin, &symbol_short!("BTC"), &AssetConfig { max_age: 300, twap_window: 3600 });
    let prices = Vec::from_array(&env, [(symbol_short!("BTC"), 0i128)]);
    let result = client.try_submit_prices(&updater, &prices);
    assert_eq!(result, Err(Ok(OracleError::InvalidPrice)));
}

#[test]
fn test_aggregation_median() {
    let (env, admin, updater1, client) = setup();
    let updater2 = Address::generate(&env);
    let updater3 = Address::generate(&env);
    client.add_updater(&admin, &updater2);
    client.add_updater(&admin, &updater3);

    let asset = symbol_short!("BTC");
    client.set_asset_config(&admin, &asset, &AssetConfig { max_age: 300, twap_window: 3600 });

    // 1. Single price
    client.submit_prices(&updater1, &Vec::from_array(&env, [(asset.clone(), 100i128)]));
    assert_eq!(client.get_price(&asset).price, 100);

    // 2. Two prices (Average of 100 and 200 = 150)
    client.submit_prices(&updater2, &Vec::from_array(&env, [(asset.clone(), 200i128)]));
    assert_eq!(client.get_price(&asset).price, 150);

    // 3. Three prices (Median of 100, 200, 150 = 150)
    client.submit_prices(&updater3, &Vec::from_array(&env, [(asset.clone(), 150i128)]));
    assert_eq!(client.get_price(&asset).price, 150);

    // 4. Four prices (Median of 100, 200, 150, 400. Sorted: 100, 150, 200, 400. Average of 150 and 200 = 175)
    let updater4 = Address::generate(&env);
    client.add_updater(&admin, &updater4);
    client.submit_prices(&updater4, &Vec::from_array(&env, [(asset.clone(), 400i128)]));
    assert_eq!(client.get_price(&asset).price, 175);
}

#[test]
fn test_stale_price_detection() {
    let (env, admin, updater, client) = setup();
    let asset = symbol_short!("BTC");
    client.set_asset_config(&admin, &asset, &AssetConfig { max_age: 300, twap_window: 3600 });

    client.submit_prices(&updater, &Vec::from_array(&env, [(asset.clone(), 100i128)]));
    assert_eq!(client.get_price_strict(&asset), 100);

    // Advance time past max_age
    env.ledger().with_mut(|l| l.timestamp += 301);
    assert_eq!(client.try_get_price_strict(&asset), Err(Ok(OracleError::StaleData)));
}

#[test]
fn test_twap_calculation() {
    let (env, admin, updater, client) = setup();
    let asset = symbol_short!("BTC");
    client.set_asset_config(&admin, &asset, &AssetConfig { max_age: 300, twap_window: 3600 });

    // T=0: Price=100
    client.submit_prices(&updater, &Vec::from_array(&env, [(asset.clone(), 100i128)]));

    // T=100: Price=200
    env.ledger().with_mut(|l| l.timestamp += 100);
    client.submit_prices(&updater, &Vec::from_array(&env, [(asset.clone(), 200i128)]));

    // TWAP at T=100:
    // (100 * 100) / 100 = 100
    assert_eq!(client.get_twap(&asset), 100);

    // T=200:
    env.ledger().with_mut(|l| l.timestamp += 100);
    // (100 * 100 + 200 * 100) / 200 = 30000 / 200 = 150
    assert_eq!(client.get_twap(&asset), 150);
}

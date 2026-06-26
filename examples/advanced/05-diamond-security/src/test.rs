#![allow(deprecated)]

use super::*;
use facet_adder::{FacetAdderContract, FacetAdderContractClient};
use facet_multiplier::{FacetMultiplierContract, FacetMultiplierContractClient};
use soroban_sdk::{testutils::Address as _, Env, IntoVal, TryIntoVal};

#[test]
fn test_proxy_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy_client = DiamondProxyContractClient::new(&env, &proxy_id);

    proxy_client.init(&admin);

    // Re-initialization should fail
    let res = proxy_client.try_init(&admin);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        SecurityError::AlreadyInitialized
    );
}

#[test]
fn test_add_facet_requires_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);

    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy_client = DiamondProxyContractClient::new(&env, &proxy_id);
    proxy_client.init(&admin);

    let adder_id = env.register_contract(None, FacetAdderContract);
    let adder_client = FacetAdderContractClient::new(&env, &adder_id);
    adder_client.init_adder(&proxy_id);

    env.mock_all_auths();
    let res = proxy_client.try_add_facet(
        &non_admin,
        &adder_id,
        &soroban_sdk::vec![&env, symbol_short!("add")],
    );
    assert_eq!(res.err().unwrap().ok().unwrap(), SecurityError::NotAdmin);
}

#[test]
fn test_add_facet_interface_verification() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy_client = DiamondProxyContractClient::new(&env, &proxy_id);
    proxy_client.init(&admin);

    let adder_id = env.register_contract(None, FacetAdderContract);
    let adder_client = FacetAdderContractClient::new(&env, &adder_id);
    adder_client.init_adder(&proxy_id);

    env.mock_all_auths();

    // Trying to register a method that Adder does NOT support (e.g. "subtract") should fail
    let res = proxy_client.try_add_facet(
        &admin,
        &adder_id,
        &soroban_sdk::vec![&env, symbol_short!("subtract")],
    );
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        SecurityError::InterfaceMismatch
    );
}

#[test]
fn test_execute_routes_correctly() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy_client = DiamondProxyContractClient::new(&env, &proxy_id);
    proxy_client.init(&admin);

    let adder_id = env.register_contract(None, FacetAdderContract);
    let adder_client = FacetAdderContractClient::new(&env, &adder_id);
    adder_client.init_adder(&proxy_id);

    env.mock_all_auths();
    proxy_client.add_facet(
        &admin,
        &adder_id,
        &soroban_sdk::vec![&env, symbol_short!("add")],
    );

    // Execute add function via Diamond Proxy
    let args = soroban_sdk::vec![&env, 10i128.into_val(&env), 20i128.into_val(&env)];
    let result = proxy_client.execute(&symbol_short!("add"), &args);
    let sum: i128 = result.try_into_val(&env).unwrap();
    assert_eq!(sum, 30);
}

#[test]
fn test_facet_access_control() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let random_user = Address::generate(&env);

    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy_client = DiamondProxyContractClient::new(&env, &proxy_id);
    proxy_client.init(&admin);

    let adder_id = env.register_contract(None, FacetAdderContract);
    let adder_client = FacetAdderContractClient::new(&env, &adder_id);
    adder_client.init_adder(&proxy_id);

    env.mock_all_auths();

    // Calling the facet contract directly should fail with InvalidCaller
    let res = adder_client.try_add(&random_user, &10, &20);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        facet_adder::SecurityError::InvalidCaller
    );
}

#[test]
fn test_namespaced_storage_isolation() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy_client = DiamondProxyContractClient::new(&env, &proxy_id);
    proxy_client.init(&admin);

    let adder_id = env.register_contract(None, FacetAdderContract);
    let adder_client = FacetAdderContractClient::new(&env, &adder_id);
    adder_client.init_adder(&proxy_id);

    let multiplier_id = env.register_contract(None, FacetMultiplierContract);
    let multiplier_client = FacetMultiplierContractClient::new(&env, &multiplier_id);
    multiplier_client.init_multiplier(&proxy_id);

    env.mock_all_auths();
    proxy_client.add_facet(
        &admin,
        &adder_id,
        &soroban_sdk::vec![&env, symbol_short!("add")],
    );
    proxy_client.add_facet(
        &admin,
        &multiplier_id,
        &soroban_sdk::vec![&env, symbol_short!("multiply")],
    );

    // Call add multiple times
    let add_args = soroban_sdk::vec![&env, 5i128.into_val(&env), 5i128.into_val(&env)];
    proxy_client.execute(&symbol_short!("add"), &add_args);
    proxy_client.execute(&symbol_short!("add"), &add_args);

    // Call multiply once
    let mult_args = soroban_sdk::vec![&env, 2i128.into_val(&env), 3i128.into_val(&env)];
    proxy_client.execute(&symbol_short!("multiply"), &mult_args);

    // Verify storage isolation of "count" for Adder (querying the Adder contract directly)
    let adder_count: i128 = env.as_contract(&adder_id, || {
        env.storage()
            .persistent()
            .get(&symbol_short!("count"))
            .unwrap_or(0)
    });
    assert_eq!(adder_count, 2);

    // Verify storage isolation of "count" for Multiplier (querying the Multiplier contract directly)
    let multiplier_count: i128 = env.as_contract(&multiplier_id, || {
        env.storage()
            .persistent()
            .get(&symbol_short!("count"))
            .unwrap_or(0)
    });
    assert_eq!(multiplier_count, 1);
}

#[test]
fn test_remove_facet() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy_client = DiamondProxyContractClient::new(&env, &proxy_id);
    proxy_client.init(&admin);

    let adder_id = env.register_contract(None, FacetAdderContract);
    let adder_client = FacetAdderContractClient::new(&env, &adder_id);
    adder_client.init_adder(&proxy_id);

    env.mock_all_auths();
    proxy_client.add_facet(
        &admin,
        &adder_id,
        &soroban_sdk::vec![&env, symbol_short!("add")],
    );

    // Remove the facet
    proxy_client.remove_facet(&admin, &adder_id);

    // Call should now fail with FacetNotFound
    let args = soroban_sdk::vec![&env, 10i128.into_val(&env), 20i128.into_val(&env)];
    let res = proxy_client.try_execute(&symbol_short!("add"), &args);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        SecurityError::FacetNotFound
    );
}

#[test]
fn test_upgrade_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy_client = DiamondProxyContractClient::new(&env, &proxy_id);
    proxy_client.init(&admin);

    env.mock_all_auths();

    // Upgrade admin
    proxy_client.upgrade_admin(&admin, &new_admin);

    // Old admin trying to perform admin actions should fail
    let adder_id = env.register_contract(None, FacetAdderContract);
    let res = proxy_client.try_add_facet(
        &admin,
        &adder_id,
        &soroban_sdk::vec![&env, symbol_short!("add")],
    );
    assert_eq!(res.err().unwrap().ok().unwrap(), SecurityError::NotAdmin);
}

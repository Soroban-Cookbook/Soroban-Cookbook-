//! Fuzz / Property-Based Tests for Advanced Security Patterns
//!
//! Exercises the Diamond multi-facet proxy, the bridge security contract,
//! and the price oracle under randomized and adversarial inputs. Contracts
//! are registered natively, following the same patterns as fuzz_tests.rs
//! and access_control_fuzz.rs.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use bridge_security::{BridgeError, BridgeSecurityContract, BridgeSecurityContractClient};
use diamond_security::{DiamondProxyContract, DiamondProxyContractClient, SecurityError};
use facet_adder::{FacetAdderContract, FacetAdderContractClient};
use facet_multiplier::{FacetMultiplierContract, FacetMultiplierContractClient};
use price_oracle::{AssetConfig, OracleError, PriceOracleContract, PriceOracleContractClient};
use proptest::prelude::*;
use soroban_sdk::{
    symbol_short, testutils::Address as _, testutils::Ledger, Address, Bytes, Env, IntoVal, Symbol,
    TryIntoVal, Vec,
};

// ---------------------------------------------------------------------------
// Diamond pattern fuzzing
// ---------------------------------------------------------------------------

fn setup_diamond_with_adder() -> (
    DiamondProxyContractClient<'static>,
    FacetAdderContractClient<'static>,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy = DiamondProxyContractClient::new(&env, &proxy_id);
    proxy.init(&admin);

    let adder_id = env.register_contract(None, FacetAdderContract);
    let adder = FacetAdderContractClient::new(&env, &adder_id);
    adder.init_adder(&proxy_id);

    (proxy, adder, admin, adder_id)
}

proptest! {
    #[test]
    fn fuzz_diamond_add_execute_matches_operands(
        a in -1_000_000i128..1_000_000i128,
        b in -1_000_000i128..1_000_000i128,
    ) {
        let (proxy, _adder, admin, adder_id) = setup_diamond_with_adder();
        proxy.add_facet(&admin, &adder_id, &soroban_sdk::vec![&proxy.env, symbol_short!("add")]);

        let args = soroban_sdk::vec![&proxy.env, a.into_val(&proxy.env), b.into_val(&proxy.env)];
        let result = proxy.execute(&symbol_short!("add"), &args);
        let sum: i128 = result.try_into_val(&proxy.env).unwrap();
        prop_assert_eq!(sum, a + b);
    }
}

#[test]
fn fuzz_diamond_multiply_execute_matches_operands() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proxy_id = env.register_contract(None, DiamondProxyContract);
    let proxy = DiamondProxyContractClient::new(&env, &proxy_id);
    proxy.init(&admin);

    let multiplier_id = env.register_contract(None, FacetMultiplierContract);
    let multiplier = FacetMultiplierContractClient::new(&env, &multiplier_id);
    multiplier.init_multiplier(&proxy_id);

    proxy.add_facet(
        &admin,
        &multiplier_id,
        &soroban_sdk::vec![&env, symbol_short!("multiply")],
    );

    for a in [-999i128, -13, 0, 1, 250, 999] {
        for b in [-999i128, -7, 0, 2, 100, 999] {
            let args = soroban_sdk::vec![&env, a.into_val(&env), b.into_val(&env)];
            let result = proxy.execute(&symbol_short!("multiply"), &args);
            let product: i128 = result.try_into_val(&env).unwrap();
            assert_eq!(product, a * b);
        }
    }
}

#[test]
fn fuzz_diamond_duplicate_function_registration_rejected() {
    let (proxy, _adder, admin, adder_id) = setup_diamond_with_adder();
    proxy.add_facet(
        &admin,
        &adder_id,
        &soroban_sdk::vec![&proxy.env, symbol_short!("add")],
    );

    let res = proxy.try_add_facet(
        &admin,
        &adder_id,
        &soroban_sdk::vec![&proxy.env, symbol_short!("add")],
    );
    assert_eq!(res, Err(Ok(SecurityError::DuplicateFunction)));
}

#[test]
fn fuzz_diamond_random_non_admins_cannot_add_facet() {
    let (proxy, _adder, _admin, adder_id) = setup_diamond_with_adder();

    for _ in 0..10 {
        let intruder = Address::generate(&proxy.env);
        let res = proxy.try_add_facet(
            &intruder,
            &adder_id,
            &soroban_sdk::vec![&proxy.env, symbol_short!("add")],
        );
        assert_eq!(res, Err(Ok(SecurityError::NotAdmin)));
    }
}

#[test]
fn fuzz_diamond_random_attackers_cannot_call_facet_directly() {
    let (_proxy, adder, _admin, _adder_id) = setup_diamond_with_adder();

    for _ in 0..10 {
        let attacker = Address::generate(&adder.env);
        let res = adder.try_add(&attacker, &1, &1);
        assert_eq!(res, Err(Ok(facet_adder::SecurityError::InvalidCaller)));
    }
}

proptest! {
    #[test]
    fn fuzz_diamond_storage_isolation_random_call_counts(
        add_calls in 0u32..20,
        mul_calls in 0u32..20,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let proxy_id = env.register_contract(None, DiamondProxyContract);
        let proxy = DiamondProxyContractClient::new(&env, &proxy_id);
        proxy.init(&admin);

        let adder_id = env.register_contract(None, FacetAdderContract);
        FacetAdderContractClient::new(&env, &adder_id).init_adder(&proxy_id);
        let multiplier_id = env.register_contract(None, FacetMultiplierContract);
        FacetMultiplierContractClient::new(&env, &multiplier_id).init_multiplier(&proxy_id);

        proxy.add_facet(&admin, &adder_id, &soroban_sdk::vec![&env, symbol_short!("add")]);
        proxy.add_facet(&admin, &multiplier_id, &soroban_sdk::vec![&env, symbol_short!("multiply")]);

        let add_args = soroban_sdk::vec![&env, 1i128.into_val(&env), 1i128.into_val(&env)];
        for _ in 0..add_calls {
            proxy.execute(&symbol_short!("add"), &add_args);
        }
        let mul_args = soroban_sdk::vec![&env, 2i128.into_val(&env), 2i128.into_val(&env)];
        for _ in 0..mul_calls {
            proxy.execute(&symbol_short!("multiply"), &mul_args);
        }

        let adder_count: i128 = env.as_contract(&adder_id, || {
            env.storage().persistent().get(&symbol_short!("count")).unwrap_or(0)
        });
        let multiplier_count: i128 = env.as_contract(&multiplier_id, || {
            env.storage().persistent().get(&symbol_short!("count")).unwrap_or(0)
        });
        prop_assert_eq!(adder_count, add_calls as i128);
        prop_assert_eq!(multiplier_count, mul_calls as i128);
    }
}

// ---------------------------------------------------------------------------
// Bridge security fuzzing
// ---------------------------------------------------------------------------

fn setup_bridge(
    rate_limit_amount: i128,
    rate_limit_window: u64,
    challenge_period: u64,
) -> (BridgeSecurityContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let bridge_id = env.register_contract(None, BridgeSecurityContract);
    let bridge = BridgeSecurityContractClient::new(&env, &bridge_id);
    bridge.initialize(
        &admin,
        &rate_limit_amount,
        &rate_limit_window,
        &challenge_period,
    );

    (bridge, admin)
}

proptest! {
    #[test]
    fn fuzz_bridge_rate_limit_boundary(amount in 1i128..2_000i128) {
        let (bridge, _admin) = setup_bridge(1_000, 1_000, 100);
        let operator = Address::generate(&bridge.env);
        let recipient = Address::generate(&bridge.env);
        let evidence = Bytes::from_array(&bridge.env, &[7u8; 32]);

        let res = bridge.try_submit_transfer(&operator, &recipient, &amount, &1u32, &evidence);
        if amount <= 1_000 {
            prop_assert!(res.is_ok());
        } else {
            prop_assert_eq!(res, Err(Ok(BridgeError::RateLimitExceeded)));
        }
    }

    #[test]
    fn fuzz_bridge_challenge_window_boundary(elapsed in 0u64..300u64) {
        let (bridge, _admin) = setup_bridge(1_000_000, 1_000_000, 100);
        let operator = Address::generate(&bridge.env);
        let recipient = Address::generate(&bridge.env);
        let evidence = Bytes::from_array(&bridge.env, &[1u8; 32]);

        let transfer_id = bridge.submit_transfer(&operator, &recipient, &500i128, &1u32, &evidence);
        bridge.env.ledger().set_timestamp(elapsed);

        let res = bridge.try_finalize_transfer(&operator, &transfer_id);
        if elapsed >= 100 {
            prop_assert!(res.is_ok());
        } else {
            prop_assert_eq!(res, Err(Ok(BridgeError::ChallengeWindowOpen)));
        }
    }

    #[test]
    fn fuzz_bridge_non_positive_amount_rejected(amount in -1_000i128..=0i128) {
        let (bridge, _admin) = setup_bridge(1_000_000, 1_000_000, 100);
        let operator = Address::generate(&bridge.env);
        let recipient = Address::generate(&bridge.env);
        let evidence = Bytes::from_array(&bridge.env, &[5u8; 32]);

        let res = bridge.try_submit_transfer(&operator, &recipient, &amount, &1u32, &evidence);
        prop_assert_eq!(res, Err(Ok(BridgeError::InvalidAmount)));
    }
}

#[test]
fn fuzz_bridge_double_challenge_rejected() {
    let (bridge, _admin) = setup_bridge(1_000_000, 1_000_000, 1_000);
    let operator = Address::generate(&bridge.env);
    let recipient = Address::generate(&bridge.env);
    let challenger = Address::generate(&bridge.env);
    let evidence = Bytes::from_array(&bridge.env, &[2u8; 32]);

    let transfer_id = bridge.submit_transfer(&operator, &recipient, &500i128, &1u32, &evidence);
    bridge.challenge_transfer(&challenger, &transfer_id);

    let res = bridge.try_challenge_transfer(&challenger, &transfer_id);
    assert_eq!(res, Err(Ok(BridgeError::TransferChallenged)));
}

#[test]
fn fuzz_bridge_finalize_after_challenge_rejected() {
    let (bridge, _admin) = setup_bridge(1_000_000, 1_000_000, 50);
    let operator = Address::generate(&bridge.env);
    let recipient = Address::generate(&bridge.env);
    let challenger = Address::generate(&bridge.env);
    let evidence = Bytes::from_array(&bridge.env, &[3u8; 32]);

    let transfer_id = bridge.submit_transfer(&operator, &recipient, &500i128, &1u32, &evidence);
    bridge.challenge_transfer(&challenger, &transfer_id);
    bridge.env.ledger().set_timestamp(1_000);

    let res = bridge.try_finalize_transfer(&operator, &transfer_id);
    assert_eq!(res, Err(Ok(BridgeError::TransferChallenged)));
}

#[test]
fn fuzz_bridge_unauthorized_operator_finalize_rejected() {
    let (bridge, _admin) = setup_bridge(1_000_000, 1_000_000, 10);
    let operator = Address::generate(&bridge.env);
    let impostor = Address::generate(&bridge.env);
    let recipient = Address::generate(&bridge.env);
    let evidence = Bytes::from_array(&bridge.env, &[4u8; 32]);

    let transfer_id = bridge.submit_transfer(&operator, &recipient, &500i128, &1u32, &evidence);
    bridge.env.ledger().set_timestamp(100);

    let res = bridge.try_finalize_transfer(&impostor, &transfer_id);
    assert_eq!(res, Err(Ok(BridgeError::Unauthorized)));
}

#[test]
fn fuzz_bridge_paused_blocks_submit() {
    let (bridge, admin) = setup_bridge(1_000_000, 1_000_000, 100);
    bridge.pause(&admin);
    assert!(bridge.is_paused());

    let operator = Address::generate(&bridge.env);
    let recipient = Address::generate(&bridge.env);
    let evidence = Bytes::from_array(&bridge.env, &[6u8; 32]);

    let res = bridge.try_submit_transfer(&operator, &recipient, &100i128, &1u32, &evidence);
    assert_eq!(res, Err(Ok(BridgeError::ContractPaused)));
}

// ---------------------------------------------------------------------------
// Oracle manipulation fuzzing
// ---------------------------------------------------------------------------

fn setup_oracle(
    max_age: u64,
    twap_window: u64,
) -> (PriceOracleContractClient<'static>, Address, Symbol) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_id = env.register_contract(None, PriceOracleContract);
    let oracle = PriceOracleContractClient::new(&env, &oracle_id);
    oracle.initialize(&admin);

    let asset = symbol_short!("XLM");
    oracle.set_asset_config(
        &admin,
        &asset,
        &AssetConfig {
            max_age,
            twap_window,
        },
    );

    (oracle, admin, asset)
}

proptest! {
    #[test]
    fn fuzz_oracle_median_resists_single_outlier(outlier in 1_000_000i128..10_000_000i128) {
        let (oracle, admin, asset) = setup_oracle(1_000, 1_000);

        let honest1 = Address::generate(&oracle.env);
        let honest2 = Address::generate(&oracle.env);
        let attacker = Address::generate(&oracle.env);
        oracle.add_updater(&admin, &honest1);
        oracle.add_updater(&admin, &honest2);
        oracle.add_updater(&admin, &attacker);

        oracle.submit_prices(&honest1, &Vec::from_array(&oracle.env, [(asset.clone(), 100i128)]));
        oracle.submit_prices(&honest2, &Vec::from_array(&oracle.env, [(asset.clone(), 100i128)]));
        oracle.submit_prices(&attacker, &Vec::from_array(&oracle.env, [(asset.clone(), outlier)]));

        let price = oracle.get_price(&asset);
        prop_assert_eq!(price.price, 100);
    }

    #[test]
    fn fuzz_oracle_stale_price_rejected(extra_age in 1u64..500u64) {
        let (oracle, admin, asset) = setup_oracle(100, 1_000);
        let updater = Address::generate(&oracle.env);
        oracle.add_updater(&admin, &updater);
        oracle.submit_prices(&updater, &Vec::from_array(&oracle.env, [(asset.clone(), 50i128)]));

        oracle.env.ledger().set_timestamp(100 + extra_age);
        let res = oracle.try_get_price_strict(&asset);
        prop_assert_eq!(res, Err(Ok(OracleError::StaleData)));
    }

    #[test]
    fn fuzz_oracle_fresh_price_accepted(age in 0u64..=100u64) {
        let (oracle, admin, asset) = setup_oracle(100, 1_000);
        let updater = Address::generate(&oracle.env);
        oracle.add_updater(&admin, &updater);
        oracle.submit_prices(&updater, &Vec::from_array(&oracle.env, [(asset.clone(), 50i128)]));

        oracle.env.ledger().set_timestamp(age);
        let price = oracle.get_price_strict(&asset);
        prop_assert_eq!(price, 50);
    }

    #[test]
    fn fuzz_oracle_non_positive_price_rejected(price in -1_000i128..=0i128) {
        let (oracle, admin, asset) = setup_oracle(100, 1_000);
        let updater = Address::generate(&oracle.env);
        oracle.add_updater(&admin, &updater);

        let res = oracle.try_submit_prices(&updater, &Vec::from_array(&oracle.env, [(asset.clone(), price)]));
        prop_assert_eq!(res, Err(Ok(OracleError::InvalidPrice)));
    }

    #[test]
    fn fuzz_oracle_twap_within_submitted_price_range(
        p1 in 10i128..1_000i128,
        p2 in 10i128..1_000i128,
        dt in 1u64..500u64,
    ) {
        let (oracle, admin, asset) = setup_oracle(10_000, 10_000);
        let updater = Address::generate(&oracle.env);
        oracle.add_updater(&admin, &updater);

        oracle.submit_prices(&updater, &Vec::from_array(&oracle.env, [(asset.clone(), p1)]));
        oracle.env.ledger().set_timestamp(dt);
        oracle.submit_prices(&updater, &Vec::from_array(&oracle.env, [(asset.clone(), p2)]));

        let twap = oracle.get_twap(&asset);
        let lo = p1.min(p2);
        let hi = p1.max(p2);
        prop_assert!((lo..=hi).contains(&twap));
    }
}

#[test]
fn fuzz_oracle_random_unauthorized_updaters_rejected() {
    let (oracle, _admin, asset) = setup_oracle(100, 1_000);

    for _ in 0..10 {
        let intruder = Address::generate(&oracle.env);
        let res = oracle.try_submit_prices(
            &intruder,
            &Vec::from_array(&oracle.env, [(asset.clone(), 10i128)]),
        );
        assert_eq!(res, Err(Ok(OracleError::Unauthorized)));
    }
}

#[test]
fn fuzz_oracle_removed_updater_excluded_from_median() {
    let (oracle, admin, asset) = setup_oracle(1_000, 1_000);
    let honest = Address::generate(&oracle.env);
    let removed = Address::generate(&oracle.env);
    oracle.add_updater(&admin, &honest);
    oracle.add_updater(&admin, &removed);

    oracle.submit_prices(
        &honest,
        &Vec::from_array(&oracle.env, [(asset.clone(), 100i128)]),
    );
    oracle.submit_prices(
        &removed,
        &Vec::from_array(&oracle.env, [(asset.clone(), 9_999_999i128)]),
    );

    // Removing the manipulative updater must exclude its price from all
    // future aggregations, even though its stale entry is still in storage.
    oracle.remove_updater(&admin, &removed);
    oracle.submit_prices(
        &honest,
        &Vec::from_array(&oracle.env, [(asset.clone(), 100i128)]),
    );

    let price = oracle.get_price(&asset);
    assert_eq!(price.price, 100);
}

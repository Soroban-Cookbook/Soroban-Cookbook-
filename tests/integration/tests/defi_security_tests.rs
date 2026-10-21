#![allow(deprecated)]
//! DeFi Security and Economic Exploit Prevention Integration Tests
//!
//! Verifies fixes for:
//! 1. Lending pool collateral check on withdraw (preventing draining)
//! 2. Staking pool, swap liquidity, and AMM price oracle re-initialization protection
//! 3. Farming pool reward rate limits (overflow/DoS prevention)
//! 4. AMM router reserve safety checks
//! 5. Collateralized lending liquidation cap and underflow prevention

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use amm_price_oracle::{
    AmmOracleContract, AmmOracleContractClient, AmmPoolContract, AmmPoolContractClient,
};
use amm_router::{AMMRouter, AMMRouterClient, Pool as RouterPool};
use collateralized_lending::{LendingContract, LendingContractClient};
use farming_pool::{FarmingPoolContract, FarmingPoolContractClient};
use lending_pool::{LendingPool, LendingPoolClient};
use staking_pool::{StakingPoolContract, StakingPoolContractClient};
use swap_liquidity_management::{SwapLiquidityContract, SwapLiquidityContractClient};

use soroban_sdk::{testutils::Address as _, token, Address, Env, Vec};

#[allow(dead_code)]
#[allow(deprecated)]
fn create_token<'a>(env: &'a Env, admin: &Address) -> (Address, token::Client<'a>) {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let addr = token_id.address();
    let client = token::Client::new(env, &addr);
    (addr, client)
}

// ---------------------------------------------------------------------------
// 1. Lending pool collateral check on withdraw
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "insufficient collateral remaining")]
fn test_lending_pool_collateral_check_on_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);

    let pool_id = env.register_contract(None, LendingPool);
    let pool = LendingPoolClient::new(&env, &pool_id);
    let (asset, _) = create_token(&env, &Address::generate(&env));
    token::StellarAssetClient::new(&env, &asset).mint(&user, &1000);
    pool.initialize(&asset, &5, &10, &80);

    // User deposits 1000
    pool.deposit(&user, &1000);
    assert_eq!(pool.get_user_position(&user).deposit, 1000);

    // User borrows 800 (80% borrow limit)
    pool.borrow(&user, &800);
    assert_eq!(pool.get_user_position(&user).borrow, 800);

    // User attempts to withdraw 200, which would leave 800 deposit.
    // 80% borrow limit of 800 deposit is 640.
    // Since borrowed is 800 > 640, this must panic!
    pool.withdraw(&user, &200);
}

// ---------------------------------------------------------------------------
// 2. Re-initialization protection
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "already initialized")]
fn test_prevent_reinitialization_staking_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    let contract_id = env.register_contract(None, StakingPoolContract);
    let client = StakingPoolContractClient::new(&env, &contract_id);
    client.initialize(&owner, &token_a, &token_b, &100);

    // Attempting second initialize must fail
    client.initialize(&owner, &token_a, &token_b, &100);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_prevent_reinitialization_swap_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let lp_token = Address::generate(&env);

    let contract_id = env.register_contract(None, SwapLiquidityContract);
    let client = SwapLiquidityContractClient::new(&env, &contract_id);
    client.initialize(&owner, &token_a, &token_b, &lp_token);

    // Attempting second initialize must fail
    client.initialize(&owner, &token_a, &token_b, &lp_token);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_prevent_reinitialization_amm_pool_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    let pool_id = env.register_contract(None, AmmPoolContract);
    let pool_client = AmmPoolContractClient::new(&env, &pool_id);
    pool_client.initialize(&owner, &token_a, &token_b);

    // Attempting second initialize must fail
    pool_client.initialize(&owner, &token_a, &token_b);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_prevent_reinitialization_amm_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let pool_id = Address::generate(&env);

    let oracle_id = env.register_contract(None, AmmOracleContract);
    let oracle_client = AmmOracleContractClient::new(&env, &oracle_id);
    oracle_client.initialize_oracle(&owner, &pool_id);

    // Attempting second initialize must fail
    oracle_client.initialize_oracle(&owner, &pool_id);
}

// ---------------------------------------------------------------------------
// 3. Farming pool reward rate limits
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Invalid reward rate")]
fn test_farming_pool_invalid_reward_rate_add() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let staking_token = Address::generate(&env);
    let reward_token = Address::generate(&env);

    let contract_id = env.register_contract(None, FarmingPoolContract);
    let client = FarmingPoolContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    // 0 rate is invalid
    client.add_pool(&admin, &staking_token, &reward_token, &0, &100);
}

#[test]
#[should_panic(expected = "Invalid reward rate")]
fn test_farming_pool_excessive_reward_rate_add() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let staking_token = Address::generate(&env);
    let reward_token = Address::generate(&env);

    let contract_id = env.register_contract(None, FarmingPoolContract);
    let client = FarmingPoolContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    // Rates larger than 1e15 are invalid
    client.add_pool(
        &admin,
        &staking_token,
        &reward_token,
        &2_000_000_000_000_000,
        &100,
    );
}

#[test]
#[should_panic(expected = "Invalid reward rate")]
fn test_farming_pool_invalid_reward_rate_update() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let staking_token = Address::generate(&env);
    let reward_token = Address::generate(&env);

    let contract_id = env.register_contract(None, FarmingPoolContract);
    let client = FarmingPoolContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let pool_id = client.add_pool(&admin, &staking_token, &reward_token, &1000, &100);

    // Negative rate update must fail/panic
    client.set_reward_rate(&admin, &pool_id, &-1);
}

// ---------------------------------------------------------------------------
// 4. AMM router reserve safety checks
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "insufficient liquidity in pool")]
fn test_amm_router_robustness_zero_reserves() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    let router_id = env.register_contract(None, AMMRouter);
    let router = AMMRouterClient::new(&env, &router_id);
    router.initialize();

    // Add pool with 0 reserves
    router.add_pool(&RouterPool {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        reserve_a: 0,
        reserve_b: 0,
    });

    let mut path = Vec::new(&env);
    path.push_back(token_a);
    path.push_back(token_b);

    // Attempt to swap should fail/panic on zero reserves
    router.swap_exact_tokens_for_tokens(&user, &100, &10, &path, &user, &99999999);
}

// ---------------------------------------------------------------------------
// 5. Collateralized lending liquidation cap and underflow prevention
// ---------------------------------------------------------------------------

#[test]
fn test_collateralized_lending_liquidation_cap_and_adjust() {
    let env = Env::default();
    env.mock_all_auths();
    let borrower = Address::generate(&env);
    let liquidator = Address::generate(&env);

    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);

    // Initializing with:
    // ltv_ratio = 80
    // liquidation_threshold = 75
    // liquidation_incentive = 10 (10%)
    // partial_liquidation_ratio = 50 (50%)
    let (collateral, _) = create_token(&env, &Address::generate(&env));
    let (loan, _) = create_token(&env, &Address::generate(&env));
    token::StellarAssetClient::new(&env, &collateral).mint(&borrower, &1000);
    token::StellarAssetClient::new(&env, &loan).mint(&contract_id, &800);
    token::StellarAssetClient::new(&env, &loan).mint(&liquidator, &1000);
    let admin = Address::generate(&env);
    client.initialize(&admin, &collateral, &loan, &80, &75, &10, &50);

    // Deposit collateral & borrow max
    client.deposit_collateral(&borrower, &1000);
    client.borrow(&borrower, &800);

    // Liquidate with a very large repay amount
    // Liquidating is allowed because liquidation_threshold (75%) < LTV (80%).
    // Borrower position: collateral = 1000, debt = 800.
    // 75% threshold means 1000 * 75% = 750 < 800 debt (unhealthy).
    // Let's call liquidate
    client.liquidate(&liquidator, &borrower, &1000);

    let borrower_pos = client.get_position(&borrower);
    let liquidator_pos = client.get_position(&liquidator);

    // Verify underflow was prevented and borrower collateral is non-negative
    assert!(borrower_pos.collateral >= 0);
    assert!(borrower_pos.debt >= 0);

    // Verify liquidator received collateral
    assert!(liquidator_pos.collateral > 0);
}

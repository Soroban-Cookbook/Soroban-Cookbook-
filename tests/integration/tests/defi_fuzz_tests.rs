//! DeFi Protocol Fuzz / Property-Based Integration Tests
//!
//! Exercises AMM invariants, lending pool accounting, and liquidation logic
//! across randomized inputs using proptest.  Contracts are registered natively
//! following the same cross-contract patterns as `integration_tests.rs`.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use collateralized_lending::{LendingContract, LendingContractClient};
use constant_product_amm::{AmmError, ConstantProductAmm, ConstantProductAmmClient};
use lending_pool::{LendingPool, LendingPoolClient};
use proptest::prelude::*;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, testutils::Address as _, Address, Env,
};

// ---------------------------------------------------------------------------
// Minimal SEP-41-compatible test token for AMM cross-contract tests
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
enum TokenDataKey {
    Initialized,
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
enum TokenError {
    AlreadyInitialized = 1,
    InvalidAmount = 2,
    InsufficientBalance = 3,
}

#[contract]
struct TestToken;

#[contractimpl]
impl TestToken {
    pub fn initialize(env: Env, owner: Address, balance: i128) -> Result<(), TokenError> {
        if env.storage().instance().has(&TokenDataKey::Initialized) {
            return Err(TokenError::AlreadyInitialized);
        }
        if balance < 0 {
            return Err(TokenError::InvalidAmount);
        }
        env.storage()
            .instance()
            .set(&TokenDataKey::Initialized, &true);
        env.storage()
            .persistent()
            .set(&TokenDataKey::Balance(owner), &balance);
        Ok(())
    }

    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), TokenError> {
        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }
        let current = env
            .storage()
            .persistent()
            .get(&TokenDataKey::Balance(to.clone()))
            .unwrap_or(0i128);
        let next = current
            .checked_add(amount)
            .ok_or(TokenError::InvalidAmount)?;
        env.storage()
            .persistent()
            .set(&TokenDataKey::Balance(to), &next);
        Ok(())
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), TokenError> {
        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }
        from.require_auth();
        let current = env
            .storage()
            .persistent()
            .get(&TokenDataKey::Balance(from.clone()))
            .unwrap_or(0i128);
        if current < amount {
            return Err(TokenError::InsufficientBalance);
        }
        let next_from = current - amount;
        let next_to = env
            .storage()
            .persistent()
            .get(&TokenDataKey::Balance(to.clone()))
            .unwrap_or(0i128)
            .checked_add(amount)
            .ok_or(TokenError::InvalidAmount)?;
        env.storage()
            .persistent()
            .set(&TokenDataKey::Balance(from), &next_from);
        env.storage()
            .persistent()
            .set(&TokenDataKey::Balance(to), &next_to);
        Ok(())
    }

    pub fn balance(env: Env, who: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&TokenDataKey::Balance(who))
            .unwrap_or(0i128)
    }
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

struct AmmFixture {
    amm_id: Address,
    token_x_id: Address,
    token_y_id: Address,
    token_x: TestTokenClient<'static>,
    token_y: TestTokenClient<'static>,
    amm: ConstantProductAmmClient<'static>,
    provider: Address,
    trader: Address,
}

fn setup_amm(initial_liquidity_x: i128, initial_liquidity_y: i128) -> AmmFixture {
    let env = Env::default();
    env.mock_all_auths();

    let provider = Address::generate(&env);
    let trader = Address::generate(&env);

    let token_x_id = env.register_contract(None, TestToken);
    let token_y_id = env.register_contract(None, TestToken);
    let token_x = TestTokenClient::new(&env, &token_x_id);
    let token_y = TestTokenClient::new(&env, &token_y_id);

    let fund = initial_liquidity_x.max(initial_liquidity_y) * 10;
    token_x.try_initialize(&provider, &fund).unwrap();
    token_x.try_mint(&trader, &fund).unwrap();
    token_y.try_initialize(&provider, &fund).unwrap();
    token_y.try_mint(&trader, &fund).unwrap();

    let amm_id = env.register_contract(None, ConstantProductAmm);
    let amm = ConstantProductAmmClient::new(&env, &amm_id);
    amm.try_initialize(&token_x_id, &token_y_id).unwrap();
    amm.try_add_liquidity(&provider, &initial_liquidity_x, &initial_liquidity_y)
        .unwrap();

    AmmFixture {
        amm_id,
        token_x_id,
        token_y_id,
        token_x,
        token_y,
        amm,
        provider,
        trader,
    }
}

fn setup_lending_pool() -> (LendingPoolClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let pool_id = env.register_contract(None, LendingPool);
    let pool = LendingPoolClient::new(&env, &pool_id);
    pool.initialize(&5, &10, &80);
    (pool, user_a, user_b)
}

fn setup_lending_contract() -> (LendingContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let borrower = Address::generate(&env);
    let liquidator = Address::generate(&env);
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    client.initialize(&80, &75, &10, &50);
    (client, borrower, liquidator, admin)
}

fn make_underwater_borrower(client: &LendingContractClient, borrower: &Address, collateral: i128) {
    client.deposit_collateral(borrower, &collateral);
    // Borrow at max LTV (80%) — underwater when liquidation_threshold < 80.
    let max_borrow = collateral * 80 / 100;
    client.borrow(borrower, &max_borrow);
}

// ---------------------------------------------------------------------------
// AMM invariant fuzz tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fuzz_amm_swap_increases_k(
        sell_amount in 1i128..5_000i128,
    ) {
        let f = setup_amm(50_000, 50_000);
        let (rx_before, ry_before) = f.amm.try_reserves().unwrap().unwrap();
        let k_before = rx_before * ry_before;

        let out = f.amm.try_swap(
            &f.trader,
            &f.token_x_id,
            &sell_amount,
            &0,
        ).unwrap().unwrap();
        prop_assert!(out > 0);

        let (rx_after, ry_after) = f.amm.try_reserves().unwrap().unwrap();
        let k_after = rx_after * ry_after;
        prop_assert!(k_after >= k_before, "k must not decrease after swap");
    }

    #[test]
    fn fuzz_amm_swap_output_less_than_reserve(
        sell_amount in 1i128..10_000i128,
    ) {
        let f = setup_amm(100_000, 100_000);
        let (_, ry_before) = f.amm.try_reserves().unwrap().unwrap();
        let out = f.amm.try_swap(
            &f.trader,
            &f.token_x_id,
            &sell_amount,
            &0,
        ).unwrap().unwrap();
        prop_assert!(out < ry_before);
    }

    #[test]
    fn fuzz_amm_swap_rejects_excessive_min_out(
        sell_amount in 100i128..5_000i128,
    ) {
        let f = setup_amm(50_000, 50_000);
        let err = f.amm.try_swap(
            &f.trader,
            &f.token_x_id,
            &sell_amount,
            &(i128::MAX / 2),
        );
        prop_assert_eq!(err, Err(Ok(AmmError::InsufficientOutputAmount)));
    }

    #[test]
    fn fuzz_amm_initial_liquidity_mint_is_sqrt(
        amount in 100i128..50_000i128,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let provider = Address::generate(&env);
        let token_x_id = env.register_contract(None, TestToken);
        let token_y_id = env.register_contract(None, TestToken);
        let token_x = TestTokenClient::new(&env, &token_x_id);
        let token_y = TestTokenClient::new(&env, &token_y_id);
        token_x.try_initialize(&provider, &(amount * 2)).unwrap();
        token_y.try_initialize(&provider, &(amount * 2)).unwrap();
        let amm_id = env.register_contract(None, ConstantProductAmm);
        let amm = ConstantProductAmmClient::new(&env, &amm_id);
        amm.try_initialize(&token_x_id, &token_y_id).unwrap();
        let minted = amm.try_add_liquidity(&provider, &amount, &amount).unwrap().unwrap();
        let expected = integer_sqrt(amount * amount);
        prop_assert_eq!(minted, expected);
        prop_assert_eq!(amm.total_supply(), expected);
    }

    #[test]
    fn fuzz_amm_add_liquidity_rejects_ratio_mismatch(
        base in 1_000i128..20_000i128,
        skew in 1i128..500i128,
    ) {
        let f = setup_amm(10_000, 10_000);
        let err = f.amm.try_add_liquidity(&f.provider, &base, &(base + skew));
        prop_assert_eq!(err, Err(Ok(AmmError::RatioMismatch)));
    }

    #[test]
    fn fuzz_amm_remove_liquidity_proportional(
        lp_burn in 100i128..2_000i128,
    ) {
        let f = setup_amm(20_000, 20_000);
        let total = f.amm.total_supply();
        let (rx, ry) = f.amm.try_reserves().unwrap().unwrap();
        let (out_x, out_y) = f.amm.try_remove_liquidity(&f.provider, &lp_burn).unwrap().unwrap();
        let expected_x = rx * lp_burn / total;
        let expected_y = ry * lp_burn / total;
        prop_assert_eq!(out_x, expected_x);
        prop_assert_eq!(out_y, expected_y);
    }

    #[test]
    fn fuzz_amm_lp_supply_matches_balances(
        extra in 500i128..5_000i128,
    ) {
        let f = setup_amm(10_000, 10_000);
        f.amm.try_add_liquidity(&f.provider, &extra, &extra).unwrap().unwrap();
        let supply = f.amm.total_supply();
        let provider_lp = f.amm.lp_balance(&f.provider);
        prop_assert_eq!(supply, provider_lp);
    }

    #[test]
    fn fuzz_amm_sequential_swaps_preserve_reserves(
        amounts in prop::collection::vec(100i128..3_000i128, 1..5),
    ) {
        let f = setup_amm(100_000, 100_000);
        for amount in amounts {
            let _ = f.amm.try_swap(
                &f.trader,
                &f.token_x_id,
                &amount,
                &0,
            ).unwrap().unwrap();
            let (rx, ry) = f.amm.try_reserves().unwrap().unwrap();
            prop_assert!(rx > 0 && ry > 0);
        }
    }

    #[test]
    fn fuzz_amm_swap_fee_reduces_output(
        sell_amount in 500i128..10_000i128,
    ) {
        let f = setup_amm(100_000, 100_000);
        let (rx, ry) = f.amm.try_reserves().unwrap().unwrap();
        let no_fee_out = sell_amount * ry / (rx + sell_amount);
        let actual_out = f.amm.try_swap(
            &f.trader,
            &f.token_x_id,
            &sell_amount,
            &0,
        ).unwrap().unwrap();
        prop_assert!(actual_out <= no_fee_out);
    }

    #[test]
    fn fuzz_amm_reserves_match_token_balances(
        sell_amount in 100i128..5_000i128,
    ) {
        let f = setup_amm(50_000, 50_000);
        f.amm.try_swap(&f.trader, &f.token_x_id, &sell_amount, &0).unwrap().unwrap();
        let (rx, ry) = f.amm.try_reserves().unwrap().unwrap();
        let contract = f.amm_id;
        prop_assert_eq!(rx, f.token_x.balance(&contract));
        prop_assert_eq!(ry, f.token_y.balance(&contract));
    }

    #[test]
    fn fuzz_amm_bidirectional_swaps_maintain_liquidity(
        amount_x in 100i128..3_000i128,
        amount_y in 100i128..3_000i128,
    ) {
        let f = setup_amm(80_000, 80_000);
        f.amm.try_swap(&f.trader, &f.token_x_id, &amount_x, &0).unwrap().unwrap();
        f.amm.try_swap(&f.trader, &f.token_y_id, &amount_y, &0).unwrap().unwrap();
        let (rx, ry) = f.amm.try_reserves().unwrap().unwrap();
        prop_assert!(rx > 0 && ry > 0);
        prop_assert!(f.amm.total_supply() > 0);
    }
}

// ---------------------------------------------------------------------------
// Lending pool fuzz tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fuzz_lending_deposit_increases_total(
        amount in 100i128..100_000i128,
    ) {
        let (pool, user, _) = setup_lending_pool();
        pool.deposit(&user, &amount);
        let pos = pool.get_user_position(&user);
        prop_assert_eq!(pos.deposit, amount);
    }

    #[test]
    fn fuzz_lending_withdraw_reduces_deposit(
        deposit in 1_000i128..50_000i128,
        withdraw in 100i128..500i128,
    ) {
        let (pool, user, _) = setup_lending_pool();
        pool.deposit(&user, &deposit);
        let withdraw_amt = withdraw.min(deposit / 2);
        pool.withdraw(&user, &withdraw_amt);
        let pos = pool.get_user_position(&user);
        prop_assert_eq!(pos.deposit, deposit - withdraw_amt);
    }

    #[test]
    fn fuzz_lending_borrow_within_80pct_limit(
        deposit in 1_000i128..50_000i128,
        borrow_pct in 1i128..80i128,
    ) {
        let (pool, user, _) = setup_lending_pool();
        pool.deposit(&user, &deposit);
        let borrow_amt = deposit * borrow_pct / 100;
        pool.borrow(&user, &borrow_amt);
        let pos = pool.get_user_position(&user);
        prop_assert_eq!(pos.borrow, borrow_amt);
    }

    #[test]
    fn fuzz_lending_repay_reduces_debt(
        deposit in 2_000i128..50_000i128,
        borrow_amt in 100i128..1_000i128,
        repay_amt in 50i128..500i128,
    ) {
        let (pool, user, _) = setup_lending_pool();
        pool.deposit(&user, &deposit);
        let borrow = borrow_amt.min(deposit * 80 / 100);
        pool.borrow(&user, &borrow);
        let repay = repay_amt.min(borrow);
        pool.repay(&user, &repay);
        let pos = pool.get_user_position(&user);
        prop_assert_eq!(pos.borrow, borrow - repay);
    }

    #[test]
    fn fuzz_lending_utilization_bounded(
        deposit in 1_000i128..100_000i128,
        borrow_pct in 1i128..80i128,
    ) {
        let (pool, user, _) = setup_lending_pool();
        pool.deposit(&user, &deposit);
        pool.borrow(&user, &(deposit * borrow_pct / 100));
        let util = pool.get_utilization();
        prop_assert!(util >= 0 && util <= 100);
    }

    #[test]
    fn fuzz_lending_borrow_rate_increases_with_utilization(
        low_pct in 10i128..40i128,
        high_pct in 50i128..80i128,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let user = Address::generate(&env);
        let pool_id = env.register_contract(None, LendingPool);
        let pool = LendingPoolClient::new(&env, &pool_id);
        pool.initialize(&5, &10, &80);
        pool.deposit(&user, &10_000);
        pool.borrow(&user, &(10_000 * low_pct / 100));
        let rate_low = pool.get_borrow_rate();
        pool.borrow(&user, &(10_000 * (high_pct - low_pct) / 100));
        let rate_high = pool.get_borrow_rate();
        prop_assert!(rate_high >= rate_low);
    }

    #[test]
    fn fuzz_lending_multi_user_isolated_positions(
        deposit_a in 1_000i128..20_000i128,
        deposit_b in 1_000i128..20_000i128,
    ) {
        let (pool, user_a, user_b) = setup_lending_pool();
        pool.deposit(&user_a, &deposit_a);
        pool.deposit(&user_b, &deposit_b);
        let pos_a = pool.get_user_position(&user_a);
        let pos_b = pool.get_user_position(&user_b);
        prop_assert_eq!(pos_a.deposit, deposit_a);
        prop_assert_eq!(pos_b.deposit, deposit_b);
        prop_assert_eq!(pos_a.borrow, 0);
        prop_assert_eq!(pos_b.borrow, 0);
    }

    #[test]
    fn fuzz_lending_deposit_withdraw_roundtrip(
        amount in 500i128..20_000i128,
    ) {
        let (pool, user, _) = setup_lending_pool();
        pool.deposit(&user, &amount);
        pool.withdraw(&user, &amount);
        let pos = pool.get_user_position(&user);
        prop_assert_eq!(pos.deposit, 0);
        prop_assert_eq!(pos.borrow, 0);
    }

    #[test]
    fn fuzz_lending_full_repay_zeros_debt(
        deposit in 2_000i128..50_000i128,
        borrow_amt in 100i128..1_000i128,
    ) {
        let (pool, user, _) = setup_lending_pool();
        pool.deposit(&user, &deposit);
        let borrow = borrow_amt.min(deposit * 80 / 100);
        pool.borrow(&user, &borrow);
        pool.repay(&user, &borrow);
        let pos = pool.get_user_position(&user);
        prop_assert_eq!(pos.borrow, 0);
    }
}

// ---------------------------------------------------------------------------
// Liquidation fuzz tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fuzz_liquidation_reduces_borrower_debt(
        collateral in 2_000i128..10_000i128,
        repay in 100i128..500i128,
    ) {
        let (client, borrower, liquidator, _) = setup_lending_contract();
        make_underwater_borrower(&client, &borrower, collateral);
        let debt_before = client.get_position(&borrower).debt;
        client.liquidate(&liquidator, &borrower, &repay);
        let debt_after = client.get_position(&borrower).debt;
        prop_assert!(debt_after < debt_before);
    }

    #[test]
    fn fuzz_liquidation_transfers_collateral_to_liquidator(
        collateral in 2_000i128..10_000i128,
        repay in 100i128..400i128,
    ) {
        let (client, borrower, liquidator, _) = setup_lending_contract();
        make_underwater_borrower(&client, &borrower, collateral);
        client.liquidate(&liquidator, &borrower, &repay);
        let liq_pos = client.get_position(&liquidator);
        prop_assert!(liq_pos.collateral > 0);
    }

    #[test]
    fn fuzz_liquidation_rejects_healthy_position(
        collateral in 2_000i128..10_000i128,
        borrow_amt in 100i128..500i128,
    ) {
        let (client, borrower, liquidator, _) = setup_lending_contract();
        client.deposit_collateral(&borrower, &collateral);
        client.borrow(&borrower, &borrow_amt);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.liquidate(&liquidator, &borrower, &100);
        }));
        prop_assert!(result.is_err());
    }

    #[test]
    fn fuzz_health_factor_decreases_with_borrow(
        collateral in 1_000i128..10_000i128,
        borrow_amt in 100i128..500i128,
    ) {
        let (client, borrower, _, _) = setup_lending_contract();
        client.deposit_collateral(&borrower, &collateral);
        let hf_before = client.get_health_factor(&borrower);
        client.borrow(&borrower, &borrow_amt);
        let hf_after = client.get_health_factor(&borrower);
        prop_assert!(hf_after < hf_before);
    }

    #[test]
    fn fuzz_partial_liquidation_capped_at_ratio(
        collateral in 2_000i128..10_000i128,
        repay in 1_000i128..10_000i128,
    ) {
        let (client, borrower, liquidator, _) = setup_lending_contract();
        make_underwater_borrower(&client, &borrower, collateral);
        let debt_before = client.get_position(&borrower).debt;
        let max_repay = debt_before * 50 / 100;
        client.liquidate(&liquidator, &borrower, &repay);
        let debt_after = client.get_position(&borrower).debt;
        let actual_repaid = debt_before - debt_after;
        prop_assert!(actual_repaid <= max_repay);
    }

    #[test]
    fn fuzz_emergency_liquidation_clears_position(
        collateral in 2_000i128..10_000i128,
    ) {
        let (client, borrower, _, admin) = setup_lending_contract();
        make_underwater_borrower(&client, &borrower, collateral);
        client.set_emergency_pause(&admin, &true);
        client.emergency_liquidate(&admin, &borrower);
        let pos = client.get_position(&borrower);
        prop_assert_eq!(pos.collateral, 0);
        prop_assert_eq!(pos.debt, 0);
    }

    #[test]
    fn fuzz_repay_improves_health_factor(
        collateral in 2_000i128..10_000i128,
        borrow_amt in 200i128..800i128,
        repay_amt in 50i128..300i128,
    ) {
        let (client, borrower, _, _) = setup_lending_contract();
        client.deposit_collateral(&borrower, &collateral);
        client.borrow(&borrower, &borrow_amt);
        let hf_before = client.get_health_factor(&borrower);
        let repay = repay_amt.min(borrow_amt);
        client.repay(&borrower, &repay);
        let hf_after = client.get_health_factor(&borrower);
        prop_assert!(hf_after >= hf_before);
    }

    #[test]
    fn fuzz_liquidation_incentive_seizure_amount(
        collateral in 2_000i128..10_000i128,
        repay in 100i128..400i128,
    ) {
        let (client, borrower, liquidator, _) = setup_lending_contract();
        make_underwater_borrower(&client, &borrower, collateral);
        let borrower_coll_before = client.get_position(&borrower).collateral;
        client.liquidate(&liquidator, &borrower, &repay);
        let borrower_coll_after = client.get_position(&borrower).collateral;
        let liquidator_coll = client.get_position(&liquidator).collateral;
        let seized = borrower_coll_before - borrower_coll_after;
        prop_assert_eq!(liquidator_coll, seized);
        prop_assert!(seized >= repay);
    }

    #[test]
    fn fuzz_collateral_deposit_increases_position(
        amount in 100i128..50_000i128,
    ) {
        let (client, borrower, _, _) = setup_lending_contract();
        client.deposit_collateral(&borrower, &amount);
        let pos = client.get_position(&borrower);
        prop_assert_eq!(pos.collateral, amount);
        prop_assert_eq!(pos.debt, 0);
    }
}

fn integer_sqrt(value: i128) -> i128 {
    if value <= 0 {
        return 0;
    }
    let mut n = value as u128;
    let mut x = n;
    let mut y = (x + 1) >> 1;
    while y < x {
        x = y;
        y = (x + n / x) >> 1;
    }
    x as i128
}

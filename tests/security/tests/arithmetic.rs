//! Arithmetic overflow / underflow security tests (#1486).
//!
//! DeFi contracts that handle large token amounts must never produce
//! silent integer wrapping or unexpected panics for extreme inputs. These
//! tests target the constant-product AMM and the staking pool with values
//! close to `i128::MAX`, zero, or negative — the typical overflow / underflow
//! boundary inputs that production fuzz targets would generate.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _,
    testutils::Ledger as _, Address, Env,
};

// ---------------------------------------------------------------------------
// Minimal test token (identical across security test modules)
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
enum TokKey {
    Bal(Address),
}

#[contract]
struct Tok;

#[contractimpl]
impl Tok {
    pub fn init(env: Env, owner: Address, bal: i128) {
        env.storage().persistent().set(&TokKey::Bal(owner), &bal);
    }
    pub fn mint(env: Env, to: Address, amount: i128) {
        let cur: i128 = env
            .storage()
            .persistent()
            .get(&TokKey::Bal(to.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(&TokKey::Bal(to), &(cur + amount));
    }
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let f: i128 = env
            .storage()
            .persistent()
            .get(&TokKey::Bal(from.clone()))
            .unwrap_or(0);
        let t: i128 = env
            .storage()
            .persistent()
            .get(&TokKey::Bal(to.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(&TokKey::Bal(from), &(f - amount));
        env.storage().persistent().set(&TokKey::Bal(to), &(t + amount));
    }
    pub fn balance(env: Env, who: Address) -> i128 {
        env.storage().persistent().get(&TokKey::Bal(who)).unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// AMM overflow tests
// ---------------------------------------------------------------------------

fn amm_env_with_reserves(rx: i128, ry: i128) -> (Env, Address, Address, constant_product_amm::ConstantProductAmmClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);

    let tx_id = env.register(Tok, ());
    let ty_id = env.register(Tok, ());
    let tx = TokClient::new(&env, &tx_id);
    let ty = TokClient::new(&env, &ty_id);

    // Give Alice enough tokens for the initial deposit.
    tx.init(&alice, &(rx + 1_000_000));
    ty.init(&alice, &(ry + 1_000_000));

    let amm_id = env.register(constant_product_amm::ConstantProductAmm, ());
    let amm = constant_product_amm::ConstantProductAmmClient::new(&env, &amm_id);
    amm.initialize(&tx_id, &ty_id);
    amm.add_liquidity(&alice, &rx, &ry);

    (env, tx_id, alice, amm)
}

/// Swapping `i128::MAX` tokens must return an error (ArithmeticOverflow),
/// not a panic or silent wrapping.
#[test]
fn amm_swap_i128_max_returns_overflow_error() {
    let (env, tx_id, alice, amm) = amm_env_with_reserves(1_000_000, 1_000_000);

    // The AMM uses checked_mul; overflow should produce ArithmeticOverflow.
    let res = amm.try_swap(&alice, &tx_id, &i128::MAX, &0);
    assert!(
        res.is_err(),
        "swap of i128::MAX must fail, got Ok"
    );
}

/// Swapping a negative amount must be rejected as InvalidAmount.
#[test]
fn amm_swap_negative_amount_rejected() {
    let (env, tx_id, alice, amm) = amm_env_with_reserves(1_000_000, 1_000_000);

    let res = amm.try_swap(&alice, &tx_id, &-1, &0);
    assert_eq!(
        res,
        Err(Ok(constant_product_amm::AmmError::InvalidAmount)),
        "negative swap amount must return InvalidAmount"
    );
}

/// Swapping zero tokens must be rejected as InvalidAmount.
#[test]
fn amm_swap_zero_amount_rejected() {
    let (env, tx_id, alice, amm) = amm_env_with_reserves(1_000_000, 1_000_000);

    let res = amm.try_swap(&alice, &tx_id, &0, &0);
    assert_eq!(
        res,
        Err(Ok(constant_product_amm::AmmError::InvalidAmount)),
        "zero swap amount must return InvalidAmount"
    );
}

/// Adding liquidity with amounts whose product overflows i128 must be rejected.
#[test]
fn amm_add_liquidity_product_overflow_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let tx_id = env.register(Tok, ());
    let ty_id = env.register(Tok, ());
    TokClient::new(&env, &tx_id).init(&alice, &i128::MAX);
    TokClient::new(&env, &ty_id).init(&alice, &i128::MAX);

    let amm_id = env.register(constant_product_amm::ConstantProductAmm, ());
    let amm = constant_product_amm::ConstantProductAmmClient::new(&env, &amm_id);
    amm.initialize(&tx_id, &ty_id);

    // i128::MAX * i128::MAX overflows; the initial sqrt computation must fail.
    let res = amm.try_add_liquidity(&alice, &i128::MAX, &i128::MAX);
    assert!(
        res.is_err(),
        "add_liquidity with i128::MAX amounts must fail"
    );
}

/// Removing more LP tokens than the user holds must return an error.
#[test]
fn amm_remove_liquidity_exceeds_balance_rejected() {
    let (env, _tx_id, alice, amm) = amm_env_with_reserves(1_000_000, 1_000_000);
    let lp = amm.lp_balance(&alice);

    let res = amm.try_remove_liquidity(&alice, &(lp + 1));
    assert!(
        res.is_err(),
        "removing more LP than held must fail"
    );
}

// ---------------------------------------------------------------------------
// Staking pool overflow / boundary tests
// ---------------------------------------------------------------------------

fn staking_env() -> (Env, Address, Address, staking_pool::StakingPoolContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000);

    let owner = Address::generate(&env);
    let stake_id = env.register(Tok, ());
    let reward_id = env.register(Tok, ());
    let pool_id = env.register(staking_pool::StakingPoolContract, ());

    TokClient::new(&env, &reward_id).init(&pool_id, &1_000_000_000i128);

    let pool = staking_pool::StakingPoolContractClient::new(&env, &pool_id);
    pool.initialize(&owner, &stake_id, &reward_id, &0i128);

    (env, stake_id, owner, pool)
}

/// Staking zero tokens must panic / fail.
#[test]
#[should_panic]
fn staking_zero_amount_rejected() {
    let (env, stake_id, _owner, pool) = staking_env();
    let staker = Address::generate(&env);
    TokClient::new(&env, &stake_id).init(&staker, &1_000);
    pool.stake(&staker, &0);
}

/// Staking a negative amount must panic / fail.
#[test]
#[should_panic]
fn staking_negative_amount_rejected() {
    let (env, stake_id, _owner, pool) = staking_env();
    let staker = Address::generate(&env);
    TokClient::new(&env, &stake_id).init(&staker, &1_000);
    pool.stake(&staker, &-1);
}

/// Unstaking more than the staked balance must panic / fail.
#[test]
#[should_panic]
fn unstake_more_than_balance_rejected() {
    let (env, stake_id, _owner, pool) = staking_env();
    let staker = Address::generate(&env);
    TokClient::new(&env, &stake_id).init(&staker, &1_001);
    pool.stake(&staker, &1_000);
    pool.unstake(&staker, &1_001);
}

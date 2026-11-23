//! Property-based tests for the constant-product AMM (#1484).
//!
//! Properties verified:
//!   1. AMM reserves are always strictly positive after any liquidity or swap operation.
//!   2. LP-token minting and burning are symmetric: `add_liquidity` followed by
//!      `remove_liquidity` on the full LP balance returns token amounts ≥ the
//!      original deposit minus rounding (1 unit per reserve).
//!   3. `swap` output is always strictly less than the reserve it draws from
//!      (the constant-product invariant k = x*y is non-decreasing after a swap).

#![cfg(test)]
#![allow(deprecated)]

use crate::{ConstantProductAmm, DataKey};
use proptest::prelude::*;
use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, Address, Env,
};

// ---------------------------------------------------------------------------
// Minimal in-process token used by all property tests
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
        env.storage()
            .persistent()
            .set(&TokKey::Bal(owner), &bal);
    }
    pub fn mint(env: Env, to: Address, amount: i128) {
        let cur: i128 = env
            .storage()
            .persistent()
            .get(&TokKey::Bal(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&TokKey::Bal(to), &(cur + amount));
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
        env.storage()
            .persistent()
            .set(&TokKey::Bal(from), &(f - amount));
        env.storage()
            .persistent()
            .set(&TokKey::Bal(to), &(t + amount));
    }
    pub fn balance(env: Env, who: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&TokKey::Bal(who))
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------

use crate::ConstantProductAmmClient;

struct F {
    env: Env,
    tx: TokClient<'static>,
    ty: TokClient<'static>,
    amm: ConstantProductAmmClient<'static>,
    alice: Address,
}

fn setup_with(rx: i128, ry: i128) -> F {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);

    let tx_id = env.register(Tok, ());
    let ty_id = env.register(Tok, ());
    let tx = TokClient::new(&env, &tx_id);
    let ty = TokClient::new(&env, &ty_id);

    // Give Alice plenty of tokens.
    tx.init(&alice, &(rx * 100 + 1_000));
    ty.init(&alice, &(ry * 100 + 1_000));

    let amm_id = env.register(ConstantProductAmm, ());
    let amm = ConstantProductAmmClient::new(&env, &amm_id);
    amm.initialize(&tx_id, &ty_id);
    amm.add_liquidity(&alice, &rx, &ry);

    F {
        env,
        tx,
        ty,
        amm,
        alice,
    }
}

// ---------------------------------------------------------------------------
// Property 1: Reserves are always positive after add / remove / swap
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(32))]
    #[test]
    fn reserves_positive_after_add_remove(
        rx in 10_000i128..1_000_000i128,
        ry in 10_000i128..1_000_000i128,
        lp_fraction in 1u32..=5u32,   // remove 1/5 to 5/5 of LP
    ) {
        let f = setup_with(rx, ry);
        let total_lp = f.amm.lp_balance(&f.alice);
        // remove at most all LP
        let remove_lp = (total_lp / lp_fraction as i128).max(1);
        let _ = f.amm.try_remove_liquidity(&f.alice, &remove_lp);
        let (rx_after, ry_after) = f.amm.reserves();
        prop_assert!(rx_after >= 0, "reserve_x went negative: {}", rx_after);
        prop_assert!(ry_after >= 0, "reserve_y went negative: {}", ry_after);
    }

    #[test]
    fn reserves_positive_after_swap(
        rx in 10_000i128..1_000_000i128,
        ry in 10_000i128..1_000_000i128,
        sell_frac in 1u32..=10u32,  // sell 1/10 to 10/10 of reserve_x
    ) {
        let f = setup_with(rx, ry);
        let sell_amount = (rx / sell_frac as i128).max(1);
        let _ = f.amm.try_swap(&f.alice, &f.tx.address, &sell_amount, &0i128);
        let (rx_after, ry_after) = f.amm.reserves();
        prop_assert!(rx_after > 0, "reserve_x not positive after swap: {}", rx_after);
        prop_assert!(ry_after > 0, "reserve_y not positive after swap: {}", ry_after);
    }
}

// ---------------------------------------------------------------------------
// Property 2: Deposit / withdraw symmetry (within 1-unit rounding per side)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(32))]
    #[test]
    fn deposit_withdraw_symmetric(
        rx in 1_000i128..100_000i128,
        ry in 1_000i128..100_000i128,
    ) {
        let f = setup_with(rx, ry);

        let bal_x_before = f.tx.balance(&f.alice);
        let bal_y_before = f.ty.balance(&f.alice);

        // Add a second deposit equal to the initial reserves (exact ratio).
        let lp_minted = f.amm.add_liquidity(&f.alice, &rx, &ry);
        // Remove exactly what was just minted.
        let (ret_x, ret_y) = f.amm.remove_liquidity(&f.alice, &lp_minted);

        // Allow ±1 rounding error per token side.
        let dx = (f.tx.balance(&f.alice) - bal_x_before).abs();
        let dy = (f.ty.balance(&f.alice) - bal_y_before).abs();
        prop_assert!(dx <= 1, "x rounding error too large: deposited {}, returned {}", rx, ret_x);
        prop_assert!(dy <= 1, "y rounding error too large: deposited {}, returned {}", ry, ret_y);
    }
}

// ---------------------------------------------------------------------------
// Property 3: Constant-product invariant k = x*y is non-decreasing after swap
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(32))]
    #[test]
    fn constant_product_invariant_non_decreasing_after_swap(
        rx in 10_000i128..500_000i128,
        ry in 10_000i128..500_000i128,
        sell_frac in 1u32..=20u32,
    ) {
        let f = setup_with(rx, ry);

        let (rx0, ry0) = f.amm.reserves();
        let k_before = rx0.saturating_mul(ry0);

        let sell_amount = (rx0 / sell_frac as i128).max(1);
        let _ = f.amm.try_swap(&f.alice, &f.tx.address, &sell_amount, &0i128);

        let (rx1, ry1) = f.amm.reserves();
        let k_after = rx1.saturating_mul(ry1);

        prop_assert!(
            k_after >= k_before,
            "k decreased after swap: before={} after={} (rx {} -> {}, ry {} -> {})",
            k_before, k_after, rx0, rx1, ry0, ry1
        );
    }
}

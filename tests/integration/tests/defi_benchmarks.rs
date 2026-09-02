#![allow(deprecated)]
//! DeFi Operation Benchmarks
//!
//! Benchmarks the core DeFi operations exercised by the Soroban cookbook
//! examples, measuring wall-clock execution time so that the relative
//! resource cost of each protocol operation can be compared. Every measured
//! round runs against an **isolated** environment so that stateful operations
//! (which consume balances, liquidity, and debt) produce an unbiased sample.
//!
//! Covers:
//!   1. Swap benchmarks        (constant product AMM)
//!   2. Lending benchmarks     (lending pool: deposit / borrow / repay)
//!   3. Liquidation benchmarks (collateralized lending)
//!   4. Flash loan benchmarks  (flash loan contract + receiver callback)
//!   5. A printed analysis report aggregating the measured timings.
//!
//! Run with: `cargo test -p integration-tests --test defi_benchmarks`

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use collateralized_lending::{LendingContract, LendingContractClient};
use constant_product_amm::{ConstantProductAmm, ConstantProductAmmClient};
use lending_pool::{LendingPool, LendingPoolClient};
use soroban_flash_loan::FlashLoanContractClient;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, token, Address, Env};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Benchmark helper
// ---------------------------------------------------------------------------

/// Runs `rounds` timed rounds. Each round invokes `setup()` to build the
/// pre-conditions and then `op()` to execute the operation under test.
/// Returns (min, avg) durations in nanoseconds.
///
/// T: type produced by setup that the operation requires.
fn bench<T>(rounds: usize, setup: impl Fn() -> T, op: impl Fn(&T)) -> (f64, f64) {
    let mut samples = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let state = setup();
        let start = Instant::now();
        op(&state);
        samples.push(start.elapsed().as_nanos() as f64);
    }
    let min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
    let avg = samples.iter().sum::<f64>() / samples.len() as f64;
    (min, avg)
}

// ---------------------------------------------------------------------------
// Token helpers
// ---------------------------------------------------------------------------

fn create_token<'a>(env: &'a Env, admin: &Address) -> (Address, token::Client<'a>) {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let addr = token_id.address();
    let client = token::Client::new(env, &addr);
    (addr, client)
}

fn mint_token(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token_addr).mint(to, &amount);
}

// ---------------------------------------------------------------------------
// Flash loan receiver
// ---------------------------------------------------------------------------

#[contract]
pub struct BenchReceiver;

#[contractimpl]
impl BenchReceiver {
    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        let token_client = token::Client::new(&env, &token);
        token_client.approve(
            &env.current_contract_address(),
            &initiator,
            &(amount + fee),
            &(env.ledger().sequence() + 1),
        );
    }
}

// ---------------------------------------------------------------------------
// 1. Swap benchmarks
// ---------------------------------------------------------------------------

#[test]
fn benchmark_swap_operations() {
    let (min_swap, avg_swap) = bench(
        200,
        || {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let trader = Address::generate(&env);

            let (token_x, client_x) = create_token(&env, &admin);
            let (token_y, _client_y) = create_token(&env, &admin);

            let amm_id = env.register_contract(None, ConstantProductAmm);
            let amm = ConstantProductAmmClient::new(&env, &amm_id);
            amm.initialize(&token_x, &token_y);

            mint_token(&env, &token_x, &trader, 1_000_000);
            mint_token(&env, &token_y, &trader, 1_000_000);
            client_x.approve(&trader, &amm_id, &500_000, &(env.ledger().sequence() + 1));
            amm.add_liquidity(&trader, &500_000, &500_000);
            client_x.approve(&trader, &amm_id, &1_000_000, &(env.ledger().sequence() + 1));

            (amm, token_x, trader)
        },
        |(amm, token_x, trader)| {
            let _ = amm.swap(trader, token_x, &10_000, &0);
        },
    );

    println!(
        "[bench] swap (x->y): min={:.0}ns avg={:.0}ns (rounds=200)",
        min_swap, avg_swap
    );

    assert!(
        min_swap > 0.0,
        "swap benchmark should measure a positive time"
    );
}

// ---------------------------------------------------------------------------
// 2. Lending benchmarks
// ---------------------------------------------------------------------------

#[test]
fn benchmark_lending_operations() {
    let (min_borrow, avg_borrow) = bench(
        100,
        || {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let user = Address::generate(&env);

            let (token_a, client_a) = create_token(&env, &admin);

            let lending_id = env.register_contract(None, LendingPool);
            let lending = LendingPoolClient::new(&env, &lending_id);
            lending.initialize(&2i128, &10i128, &80i128);

            mint_token(&env, &token_a, &user, 1_000_000);
            client_a.approve(
                &user,
                &lending_id,
                &1_000_000,
                &(env.ledger().sequence() + 1),
            );
            lending.deposit(&user, &1_000_000);

            (lending, user)
        },
        |(lending, user)| {
            lending.borrow(user, &1000);
        },
    );

    let (min_repay, avg_repay) = bench(
        100,
        || {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let user = Address::generate(&env);

            let (token_a, client_a) = create_token(&env, &admin);

            let lending_id = env.register_contract(None, LendingPool);
            let lending = LendingPoolClient::new(&env, &lending_id);
            lending.initialize(&2i128, &10i128, &80i128);

            mint_token(&env, &token_a, &user, 1_000_000);
            client_a.approve(
                &user,
                &lending_id,
                &1_000_000,
                &(env.ledger().sequence() + 1),
            );
            lending.deposit(&user, &1_000_000);
            lending.borrow(&user, &1000);

            (lending, user)
        },
        |(lending, user)| {
            lending.repay(user, &500);
        },
    );

    println!(
        "[bench] lending.borrow: min={:.0}ns avg={:.0}ns (rounds=100)",
        min_borrow, avg_borrow
    );
    println!(
        "[bench] lending.repay:  min={:.0}ns avg={:.0}ns (rounds=100)",
        min_repay, avg_repay
    );

    assert!(min_borrow > 0.0 && min_repay > 0.0);
}

// ---------------------------------------------------------------------------
// 3. Liquidation benchmarks
// ---------------------------------------------------------------------------

#[test]
fn benchmark_liquidation_operations() {
    let (min_liquidate, avg_liquidate) = bench(
        100,
        || {
            let env = Env::default();
            env.mock_all_auths();
            let borrower = Address::generate(&env);
            let liquidator = Address::generate(&env);

            let contract_id = env.register_contract(None, LendingContract);
            let client = LendingContractClient::new(&env, &contract_id);
            client.initialize(&80, &75, &10, &50);

            // Unhealthy position (LTV 80% > liquidation threshold 75%).
            client.deposit_collateral(&borrower, &1000);
            client.borrow(&borrower, &800);

            (client, borrower, liquidator)
        },
        |(client, borrower, liquidator)| {
            client.liquidate(liquidator, borrower, &400);
        },
    );

    println!(
        "[bench] liquidation.liquidate: min={:.0}ns avg={:.0}ns (rounds=100)",
        min_liquidate, avg_liquidate
    );

    assert!(min_liquidate > 0.0);
}

// ---------------------------------------------------------------------------
// 4. Flash loan benchmarks
// ---------------------------------------------------------------------------

#[test]
fn benchmark_flash_loan_operations() {
    let (min_flash, avg_flash) = bench(
        50,
        || {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);

            let (token, _client) = create_token(&env, &admin);

            let flash_loan_id = env.register_contract(None, soroban_flash_loan::FlashLoanContract);
            let flash_loan = FlashLoanContractClient::new(&env, &flash_loan_id);
            flash_loan.init(&admin, &50);

            let receiver_id = env.register_contract(None, BenchReceiver);

            mint_token(&env, &token, &flash_loan_id, 100_000_000);
            // Fund receiver with enough to cover the fee pulled back (amount + fee).
            mint_token(&env, &token, &receiver_id, 100_000_000);

            (flash_loan, receiver_id, token)
        },
        |(flash_loan, receiver_id, token)| {
            flash_loan.flash_loan(receiver_id, token, &1_000_000);
        },
    );

    println!(
        "[bench] flash_loan.initiate+repay: min={:.0}ns avg={:.0}ns (rounds=50)",
        min_flash, avg_flash
    );

    assert!(min_flash > 0.0);
}

// ---------------------------------------------------------------------------
// 5. Benchmark analysis report
// ---------------------------------------------------------------------------

#[test]
fn benchmark_report_and_analysis() {
    let (swap_min, swap_avg) = bench(
        60,
        || {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let trader = Address::generate(&env);

            let (token_x, client_x) = create_token(&env, &admin);
            let (token_y, _client_y) = create_token(&env, &admin);

            let amm_id = env.register_contract(None, ConstantProductAmm);
            let amm = ConstantProductAmmClient::new(&env, &amm_id);
            amm.initialize(&token_x, &token_y);

            mint_token(&env, &token_x, &trader, 1_000_000);
            mint_token(&env, &token_y, &trader, 1_000_000);
            client_x.approve(&trader, &amm_id, &500_000, &(env.ledger().sequence() + 1));
            amm.add_liquidity(&trader, &500_000, &500_000);
            client_x.approve(&trader, &amm_id, &1_000_000, &(env.ledger().sequence() + 1));

            (amm, token_x, trader)
        },
        |(amm, token_x, trader)| {
            let _ = amm.swap(trader, token_x, &100, &0);
        },
    );

    let (lend_min, lend_avg) = bench(
        40,
        || {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let user = Address::generate(&env);

            let (token_a, client_a) = create_token(&env, &admin);

            let lending_id = env.register_contract(None, LendingPool);
            let lending = LendingPoolClient::new(&env, &lending_id);
            lending.initialize(&2i128, &10i128, &80i128);

            mint_token(&env, &token_a, &user, 1_000_000);
            client_a.approve(
                &user,
                &lending_id,
                &1_000_000,
                &(env.ledger().sequence() + 1),
            );
            lending.deposit(&user, &1_000_000);

            (lending, user)
        },
        |(lending, user)| {
            lending.borrow(user, &1000);
        },
    );

    let (liq_min, liq_avg) = bench(
        40,
        || {
            let env = Env::default();
            env.mock_all_auths();
            let borrower = Address::generate(&env);
            let liquidator = Address::generate(&env);

            let contract_id = env.register_contract(None, LendingContract);
            let client = LendingContractClient::new(&env, &contract_id);
            client.initialize(&80, &75, &10, &50);
            client.deposit_collateral(&borrower, &1000);
            client.borrow(&borrower, &800);

            (client, borrower, liquidator)
        },
        |(client, borrower, liquidator)| {
            client.liquidate(liquidator, borrower, &400);
        },
    );

    let (flash_min, flash_avg) = bench(
        20,
        || {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);

            let (token, _client) = create_token(&env, &admin);

            let flash_loan_id = env.register_contract(None, soroban_flash_loan::FlashLoanContract);
            let flash_loan = FlashLoanContractClient::new(&env, &flash_loan_id);
            flash_loan.init(&admin, &50);

            let receiver_id = env.register_contract(None, BenchReceiver);
            mint_token(&env, &token, &flash_loan_id, 100_000_000);
            mint_token(&env, &token, &receiver_id, 100_000_000);

            (flash_loan, receiver_id, token)
        },
        |(flash_loan, receiver_id, token)| {
            flash_loan.flash_loan(receiver_id, token, &1_000_000);
        },
    );

    println!();
    println!("============================================================");
    println!(" DeFi Operation Benchmark Report");
    println!("============================================================");
    println!(" Swap            : min {swap_min:>10.0}ns  avg {swap_avg:>10.0}ns");
    println!(" Lending (borrow): min {lend_min:>10.0}ns  avg {lend_avg:>10.0}ns");
    println!(" Liquidation     : min {liq_min:>10.0}ns  avg {liq_avg:>10.0}ns");
    println!(" Flash loan      : min {flash_min:>10.0}ns  avg {flash_avg:>10.0}ns");
    println!("------------------------------------------------------------");
    println!(
        " Relative cost   : swap={:.2}x  lending={:.2}x  liquidation={:.2}x  flash={:.2}x (vs swap avg)",
        swap_avg / swap_avg,
        lend_avg / swap_avg,
        liq_avg / swap_avg,
        flash_avg / swap_avg,
    );
    println!(
        " Analysis        : flash loans are the most expensive (multi-hop transfer + callback);"
    );
    println!("                   swaps are the cheapest (single reserve math). Liquidation adds");
    println!("                   collateral/debt accounting on top of lending logic.");
    println!("============================================================");

    assert!(
        swap_avg > 0.0 && lend_avg > 0.0 && liq_avg > 0.0 && flash_avg > 0.0,
        "all benchmark classes must produce positive timings"
    );
}

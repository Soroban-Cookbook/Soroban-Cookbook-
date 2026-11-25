#![allow(deprecated)]
//! # Smoke Test Suite
//!
//! Fast sanity checks for the cookbook's critical paths. Each test registers a
//! contract natively (no WASM build), drives one happy path, and checks one
//! guard rail. The whole suite is meant to finish in seconds so it can run as
//! a pre-merge gate before the full integration, fuzz, and coverage jobs.
//!
//! Run locally:
//!
//! ```bash
//! cargo test -p integration-tests --test smoke_tests
//! ```
//!
//! Rules for adding a smoke test (see `tests/integration/SMOKE_TESTS.md`):
//! - One contract (or one tight cross-contract pair) per test.
//! - No loops over large inputs, no proptest, no benchmarks.
//! - Cover the happy path plus the single most important failure path.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    token, Address, Env, String,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn create_token<'a>(env: &'a Env, admin: &Address) -> (Address, token::Client<'a>) {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let addr = token_id.address();
    let client = token::Client::new(env, &addr);
    (addr, client)
}

fn mint_token(env: &Env, token: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token).mint(to, &amount);
}

// ---------------------------------------------------------------------------
// 1. Basics: the canonical intro contract responds
// ---------------------------------------------------------------------------

#[test]
fn smoke_hello_world_responds() {
    let env = setup_env();

    let id = env.register_contract(None, hello_world::HelloContract);
    let client = hello_world::HelloContractClient::new(&env, &id);

    let words = client.hello(&symbol_short!("Dev"));
    assert_eq!(words.len(), 2);
    assert_eq!(words.get(0).unwrap(), symbol_short!("Hello"));
    assert_eq!(words.get(1).unwrap(), symbol_short!("Dev"));
}

// ---------------------------------------------------------------------------
// 2. Storage: persistent read/write round-trips
// ---------------------------------------------------------------------------

#[test]
fn smoke_storage_round_trip() {
    let env = setup_env();

    let id = env.register_contract(None, storage_patterns::StorageContract);
    let client = storage_patterns::StorageContractClient::new(&env, &id);

    let key = symbol_short!("smoke");
    assert!(!client.has_persistent(&key));

    client.set_persistent(&key, &42u64);
    assert!(client.has_persistent(&key));
    assert_eq!(client.get_persistent(&key), Some(42u64));
}

// ---------------------------------------------------------------------------
// 3. Auth + errors: transfers move balances, bad input is rejected
// ---------------------------------------------------------------------------

#[test]
fn smoke_auth_transfer_and_custom_errors() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let auth_id = env.register_contract(None, authentication::AuthContract);
    let auth = authentication::AuthContractClient::new(&env, &auth_id);
    auth.initialize(&admin);
    auth.set_balance(&admin, &alice, &100);

    auth.transfer(&alice, &bob, &40);
    assert_eq!(auth.get_balance(&alice), 60);
    assert_eq!(auth.get_balance(&bob), 40);
    assert!(auth.try_transfer(&alice, &bob, &0).is_err());

    let err_id = env.register_contract(None, custom_errors::CustomErrorsContract);
    let errors = custom_errors::CustomErrorsContractClient::new(&env, &err_id);
    assert_eq!(errors.try_validate_input(&1i64), Ok(Ok(())));
    assert_eq!(
        errors.try_validate_input(&0i64),
        Err(Ok(custom_errors::ContractError::InvalidInput))
    );
}

// ---------------------------------------------------------------------------
// 4. Tokens: SEP-41 mint, transfer, allowance
// ---------------------------------------------------------------------------

#[test]
fn smoke_sep41_token_core_flow() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let spender = Address::generate(&env);

    let id = env.register_contract(None, sep41_token::Sep41Token);
    let token = sep41_token::Sep41TokenClient::new(&env, &id);
    token.initialize(
        &admin,
        &String::from_str(&env, "Smoke Token"),
        &symbol_short!("SMK"),
        &7u32,
        &0i128,
    );

    token.mint(&admin, &alice, &1_000);
    token.transfer(&alice, &bob, &250);
    assert_eq!(token.balance(&alice), 750);
    assert_eq!(token.balance(&bob), 250);

    token.approve(&alice, &spender, &100);
    token.transfer_from(&spender, &alice, &bob, &100);
    assert_eq!(token.allowance(&alice, &spender), 0);
    assert!(token.try_transfer_from(&spender, &alice, &bob, &1).is_err());
}

#[test]
fn smoke_mint_burn_respects_cap() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);

    let id = env.register_contract(None, mint_burn_token::MintBurnToken);
    let token = mint_burn_token::MintBurnTokenClient::new(&env, &id);
    token.initialize(&admin, &1_000);

    token.mint(&alice, &1_000);
    assert_eq!(token.balance(&alice), 1_000);
    assert!(token.try_mint(&alice, &1).is_err());
}

// ---------------------------------------------------------------------------
// 5. Access control: pause guard blocks writes, reads still work
// ---------------------------------------------------------------------------

#[test]
fn smoke_pause_guard() {
    let env = setup_env();
    let admin = Address::generate(&env);

    let id = env.register_contract(None, pause_unpause::PausableContract);
    let client = pause_unpause::PausableContractClient::new(&env, &id);
    client.initialize(&admin);

    assert_eq!(client.increment(), 1);

    client.pause();
    assert!(client.is_paused());
    assert!(client.try_increment().is_err());
    assert_eq!(client.get_counter(), 1);

    client.unpause();
    assert!(!client.is_paused());
    assert_eq!(client.increment(), 2);
}

// ---------------------------------------------------------------------------
// 6. Governance: proposal -> votes -> execution
// ---------------------------------------------------------------------------

#[test]
fn smoke_simple_voting_lifecycle() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let yes = Address::generate(&env);
    let no = Address::generate(&env);

    let id = env.register_contract(None, simple_voting::VotingContract);
    let client = simple_voting::VotingContractClient::new(&env, &id);
    client.initialize(&admin);

    env.ledger().set_timestamp(100);
    let prop_id = client.create_prop(&admin, &String::from_str(&env, "smoke"), &200u64);

    client.cast_vote(&yes, &prop_id, &simple_voting::VoteChoice::For);
    client.cast_vote(&no, &prop_id, &simple_voting::VoteChoice::Against);
    client.cast_vote(&admin, &prop_id, &simple_voting::VoteChoice::For);
    assert_eq!(client.tally(&prop_id), (2, 1, 0));

    env.ledger().set_timestamp(201);
    assert_eq!(
        client.execute(&prop_id),
        simple_voting::ProposalStatus::Passed
    );
}

// ---------------------------------------------------------------------------
// 7. DeFi: AMM accepts liquidity and executes a swap against real tokens
// ---------------------------------------------------------------------------

#[test]
fn smoke_amm_liquidity_and_swap() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let trader = Address::generate(&env);

    let (token_x, client_x) = create_token(&env, &admin);
    let (token_y, _client_y) = create_token(&env, &admin);

    let amm_id = env.register_contract(None, constant_product_amm::ConstantProductAmm);
    let amm = constant_product_amm::ConstantProductAmmClient::new(&env, &amm_id);
    amm.initialize(&token_x, &token_y);

    mint_token(&env, &token_x, &trader, 1_000_000);
    mint_token(&env, &token_y, &trader, 1_000_000);
    client_x.approve(&trader, &amm_id, &1_000_000, &(env.ledger().sequence() + 1));

    let lp = amm.add_liquidity(&trader, &500_000, &500_000);
    assert!(lp > 0);
    assert_eq!(amm.lp_balance(&trader), lp);

    let (rx_before, ry_before) = amm.reserves();
    let bought = amm.swap(&trader, &token_x, &10_000, &0);
    assert!(bought > 0 && bought < 10_000);

    let (rx_after, ry_after) = amm.reserves();
    assert_eq!(rx_after, rx_before + 10_000);
    assert_eq!(ry_after, ry_before - bought);

    // Slippage guard: demanding more than the pool can give must fail.
    assert!(amm.try_swap(&trader, &token_x, &10_000, &1_000_000).is_err());
}

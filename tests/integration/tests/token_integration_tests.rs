//! Multi-token integration tests (Issue #119).
//!
//! Spans several token example crates in one Env to cover multi-user flows,
//! cross-contract calls, and edge cases that mimic real usage.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, IssuerFlags, Ledger as _},
    token, Address, Env, String, Symbol,
};

#[test]
fn test_sep41_mint_burn_pausable_multi_user() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    // --- SEP-41 token: mint + transfer + allowance ---
    let sep_id = env.register_contract(None, sep41_token::Sep41Token);
    let sep = sep41_token::Sep41TokenClient::new(&env, &sep_id);
    sep.initialize(
        &admin,
        &String::from_str(&env, "Cookbook USD"),
        &symbol_short!("CUSD"),
        &7u32,
        &0i128,
    );

    sep.mint(&admin, &alice, &1_000);
    sep.mint(&admin, &bob, &500);
    sep.transfer(&alice, &carol, &200);
    assert_eq!(sep.balance(&alice), 800);
    assert_eq!(sep.balance(&carol), 200);

    sep.approve(&alice, &bob, &150);
    sep.transfer_from(&bob, &alice, &carol, &100);
    assert_eq!(sep.balance(&alice), 700);
    assert_eq!(sep.balance(&carol), 300);
    assert_eq!(sep.allowance(&alice, &bob), 50);

    assert_eq!(
        sep.try_transfer_from(&bob, &alice, &carol, &999),
        Err(Ok(sep41_token::TokenError::AllowanceExceeded))
    );

    // --- Mint/burn token: capped issuance ---
    let mb_id = env.register_contract(None, mint_burn_token::MintBurnToken);
    let mb = mint_burn_token::MintBurnTokenClient::new(&env, &mb_id);
    mb.initialize(&admin, &2_000);
    mb.mint(&alice, &1_500);
    assert_eq!(mb.balance(&alice), 1_500);
    assert_eq!(
        mb.try_mint(&bob, &600),
        Err(Ok(mint_burn_token::TokenError::SupplyCapExceeded))
    );
    mb.burn(&alice, &500);
    assert_eq!(mb.total_supply(), 1_000);
    mb.mint(&bob, &600);
    assert_eq!(mb.balance(&bob), 600);

    // --- Pausable token: transfers blocked while paused ---
    let pause_id = env.register_contract(None, pausable_token::PausableToken);
    let pause = pausable_token::PausableTokenClient::new(&env, &pause_id);
    pause.initialize(
        &admin,
        &String::from_str(&env, "Pause Coin"),
        &symbol_short!("PAUSE"),
        &7u32,
        &1_000i128,
    );
    pause.transfer(&admin, &alice, &400);
    pause.pause();
    assert!(pause.is_paused());
    assert_eq!(
        pause.try_transfer(&alice, &bob, &50),
        Err(Ok(pausable_token::TokenError::ContractPaused))
    );
    pause.unpause();
    pause.transfer(&alice, &bob, &50);
    assert_eq!(pause.balance(&bob), 50);
}

#[test]
fn test_wrapper_and_vesting_cross_contract_flow() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000);

    let asset_admin = Address::generate(&env);
    let vesting_admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let asset = env.register_stellar_asset_contract_v2(asset_admin);
    asset.issuer().set_flag(IssuerFlags::ClawbackEnabledFlag);
    let underlying_id = asset.address();
    let underlying = token::Client::new(&env, &underlying_id);
    let underlying_admin = token::StellarAssetClient::new(&env, &underlying_id);

    underlying_admin.mint(&alice, &10_000);
    underlying_admin.mint(&bob, &2_000);
    underlying_admin.mint(&vesting_admin, &5_000);

    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&underlying_id);

    assert_eq!(wrapper.wrap(&alice, &1_000), 1_000);
    assert_eq!(wrapper.wrap(&bob, &500), 500);
    wrapper.transfer(&alice, &bob, &200);
    assert_eq!(wrapper.balance(&alice), 800);
    assert_eq!(wrapper.balance(&bob), 700);
    assert_eq!(underlying.balance(&wrapper_id), 1_500);

    assert_eq!(wrapper.unwrap(&bob, &300), 400);
    assert_eq!(underlying.balance(&bob), 1_800);
    let backing = wrapper.backing();
    assert!(backing.fully_backed);
    assert_eq!(backing.wrapped_supply, 1_200);

    let vesting_id = env.register_contract(None, vesting_contract::VestingContract);
    let vesting = vesting_contract::VestingContractClient::new(&env, &vesting_id);
    vesting.initialize(&vesting_admin, &underlying_id);

    underlying.transfer(&vesting_admin, &vesting_id, &3_000);

    vesting.create_schedule(&vesting_admin, &alice, &1_000, &1_000, &100, &1_000);
    vesting.create_schedule(&vesting_admin, &bob, &2_000, &1_000, &0, &500);

    assert_eq!(
        vesting.try_claim(&alice),
        Err(Ok(vesting_contract::VestingError::ClaimBeforeCliff))
    );

    env.ledger().with_mut(|l| l.timestamp = 1_600);
    let alice_claimed = vesting.claim(&alice);
    assert_eq!(alice_claimed, 600);
    // alice: 10000 - 1000 wrap + 600 claim
    assert_eq!(underlying.balance(&alice), 9_600);

    let bob_claimed = vesting.claim(&bob);
    assert_eq!(bob_claimed, 2_000);

    assert_eq!(
        vesting.try_claim(&bob),
        Err(Ok(vesting_contract::VestingError::NothingToClaim))
    );

    assert_eq!(wrapper.balance(&alice), 800);
    assert!(wrapper.backing().exactly_backed);
}

#[test]
fn test_token_edge_cases_across_contracts() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let sep_id = env.register_contract(None, sep41_token::Sep41Token);
    let sep = sep41_token::Sep41TokenClient::new(&env, &sep_id);
    sep.initialize(
        &admin,
        &String::from_str(&env, "Edge"),
        &Symbol::new(&env, "EDGE"),
        &0u32,
        &100i128,
    );
    assert_eq!(
        sep.try_transfer(&admin, &alice, &0),
        Err(Ok(sep41_token::TokenError::InvalidAmount))
    );
    assert_eq!(
        sep.try_transfer(&admin, &alice, &10_000),
        Err(Ok(sep41_token::TokenError::InsufficientBalance))
    );

    let mb_id = env.register_contract(None, mint_burn_token::MintBurnToken);
    let mb = mint_burn_token::MintBurnTokenClient::new(&env, &mb_id);
    mb.initialize(&admin, &0);
    mb.mint(&alice, &50);
    assert_eq!(
        mb.try_burn(&alice, &51),
        Err(Ok(mint_burn_token::TokenError::InsufficientBalance))
    );

    let pause_id = env.register_contract(None, pausable_token::PausableToken);
    let pause = pausable_token::PausableTokenClient::new(&env, &pause_id);
    pause.initialize(
        &admin,
        &String::from_str(&env, "P"),
        &symbol_short!("P"),
        &0u32,
        &10i128,
    );
    pause.pause();
    assert_eq!(
        pause.try_pause(),
        Err(Ok(pausable_token::TokenError::AlreadyInState))
    );
    pause.unpause();
    pause.transfer(&admin, &bob, &5);
    assert_eq!(pause.balance(&bob), 5);
}

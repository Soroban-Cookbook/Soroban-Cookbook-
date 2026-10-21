#![allow(deprecated)]
//! # Comprehensive Regression Test Suite (Issue #946 / Phase 6 #305)
//!
//! Validates critical system invariants, state consistency, backwards compatibility,
//! and error handling across cookbook examples (Basics, Tokens, Governance, NFTs,
//! Intermediate, and DeFi).
//!
//! Ensures that changes to SDK versions, common dependencies, or contract interfaces
//! do not cause breaking regressions in existing contracts.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::Address as _,
    Address, BytesN, Env, String, Symbol,
};

// ---------------------------------------------------------------------------
// 1. Basics Regression: Authentication, Storage Patterns & Custom Errors
// ---------------------------------------------------------------------------

#[test]
fn test_regression_basics_auth_storage_and_custom_errors() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Register contracts
    let auth_id = env.register_contract(None, authentication::AuthContract);
    let auth_client = authentication::AuthContractClient::new(&env, &auth_id);

    let storage_id = env.register_contract(None, storage_patterns::StorageContract);
    let storage_client = storage_patterns::StorageContractClient::new(&env, &storage_id);

    let error_id = env.register_contract(None, custom_errors::CustomErrorsContract);
    let error_client = custom_errors::CustomErrorsContractClient::new(&env, &error_id);

    // Initialize auth
    auth_client.initialize(&admin);
    auth_client.set_balance(&admin, &user1, &1000);
    auth_client.set_balance(&admin, &user2, &500);

    // Invariant: Non-negative balances and valid transfers
    auth_client.transfer(&user1, &user2, &250);
    assert_eq!(auth_client.get_balance(&user1), 750);
    assert_eq!(auth_client.get_balance(&user2), 750);

    // Regression check: Negative amounts are blocked
    assert!(auth_client.try_transfer(&user1, &user2, &-50).is_err());
    assert!(auth_client.try_transfer(&user1, &user2, &0).is_err());

    // Storage invariants: Persistent storage retrieval & updates
    let key = symbol_short!("usr1_bal");
    storage_client.set_persistent(&key, &750u64);
    assert_eq!(storage_client.get_persistent(&key), Some(750u64));
    assert!(storage_client.has_persistent(&key));

    // Custom errors: Ensure specific error variants are emitted as expected
    let res = error_client.try_validate_input(&0i64);
    assert_eq!(res, Err(Ok(custom_errors::ContractError::InvalidInput)));

    let res_neg = error_client.try_validate_input(&-10i64);
    assert_eq!(res_neg, Err(Ok(custom_errors::ContractError::InvalidInput)));

    let res_valid = error_client.try_validate_input(&100i64);
    assert_eq!(res_valid, Ok(Ok(())));
}

// ---------------------------------------------------------------------------
// 2. Token Invariants: SEP-41 Supply Conservation, Allowances & Mint/Burn
// ---------------------------------------------------------------------------

#[test]
fn test_regression_token_supply_and_allowance_invariants() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    // Register SEP-41 token
    let sep_id = env.register_contract(None, sep41_token::Sep41Token);
    let sep = sep41_token::Sep41TokenClient::new(&env, &sep_id);
    sep.initialize(
        &admin,
        &String::from_str(&env, "Regression USD"),
        &symbol_short!("RUSD"),
        &7u32,
        &0i128,
    );

    // Register Capped Mint-Burn token
    let mb_id = env.register_contract(None, mint_burn_token::MintBurnToken);
    let mb = mint_burn_token::MintBurnTokenClient::new(&env, &mb_id);
    mb.initialize(&admin, &5000);

    // Invariant 1: Mint increases balance & total supply equally
    sep.mint(&admin, &alice, &2000);
    assert_eq!(sep.balance(&alice), 2000);

    // Invariant 2: Transfer preserves sum of balances
    sep.transfer(&alice, &bob, &600);
    assert_eq!(sep.balance(&alice), 1400);
    assert_eq!(sep.balance(&bob), 600);

    // Invariant 3: Allowance lifecycle: approve -> transfer_from -> allowance decrements
    sep.approve(&alice, &charlie, &500);
    assert_eq!(sep.allowance(&alice, &charlie), 500);

    sep.transfer_from(&charlie, &alice, &bob, &300);
    assert_eq!(sep.balance(&alice), 1100);
    assert_eq!(sep.balance(&bob), 900);
    assert_eq!(sep.allowance(&alice, &charlie), 200);

    // Invariant 4: Overdraft on allowance reverts
    assert!(sep.try_transfer_from(&charlie, &alice, &bob, &300).is_err());

    // Invariant 5: Capped issuance cannot exceed max supply
    mb.mint(&alice, &3000);
    assert_eq!(mb.balance(&alice), 3000);
    // Attempting to exceed cap of 5000 with 2500 more should fail
    assert!(mb.try_mint(&alice, &2500).is_err());
    // Within cap succeeds
    assert!(mb.try_mint(&alice, &2000).is_ok());
    assert_eq!(mb.balance(&alice), 5000);
}

// ---------------------------------------------------------------------------
// 3. Pausable Token Invariants: Pause/Unpause State Regression
// ---------------------------------------------------------------------------

#[test]
fn test_regression_pausable_token_invariants() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let pt_id = env.register_contract(None, pausable_token::PausableToken);
    let pt = pausable_token::PausableTokenClient::new(&env, &pt_id);
    pt.initialize(
        &admin,
        &String::from_str(&env, "Pausable Token"),
        &symbol_short!("PT"),
        &7u32,
        &0i128,
    );

    pt.mint(&admin, &user1, &1000);
    assert_eq!(pt.balance(&user1), 1000);

    // Pause contract
    pt.pause();

    // Invariant: Operations are blocked while paused
    assert!(pt.try_transfer(&user1, &user2, &100).is_err());
    assert!(pt.try_mint(&admin, &user2, &100).is_err());

    // Unpause contract
    pt.unpause();

    // Invariant: Operations resume cleanly after unpause
    assert!(pt.try_transfer(&user1, &user2, &100).is_ok());
    assert_eq!(pt.balance(&user1), 900);
    assert_eq!(pt.balance(&user2), 100);
}

// ---------------------------------------------------------------------------
// 4. Governance Regression: Proposal Lifecycle & Voting Invariants
// ---------------------------------------------------------------------------

#[test]
fn test_regression_governance_proposal_and_voting_invariants() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    let prop_id = env.register_contract(None, simple_voting::VotingContract);
    let client = simple_voting::VotingContractClient::new(&env, &prop_id);

    client.initialize(&admin);

    // Invariant: Proposals can be submitted, voted on, and resolved deterministically
    let deadline = env.ledger().timestamp() + 100;
    let p_id = client.create_prop(
        &admin,
        &String::from_str(&env, "Regression Governance Proposal"),
        &deadline,
    );

    assert_eq!(p_id, 1);
    let info = client.get_prop(&p_id);
    assert_eq!(info.status, simple_voting::ProposalStatus::Active);

    // Cast votes
    client.cast_vote(&voter1, &p_id, &simple_voting::VoteChoice::For);
    client.cast_vote(&voter2, &p_id, &simple_voting::VoteChoice::Against);

    let updated = client.get_prop(&p_id);
    assert_eq!(updated.votes_for, 1);
    assert_eq!(updated.votes_against, 1);

    // Invariant: Double voting from same address is rejected
    assert!(client
        .try_cast_vote(&voter1, &p_id, &simple_voting::VoteChoice::For)
        .is_err());
}

// ---------------------------------------------------------------------------
// 5. NFT Regression: Ownership, Approval Clearing & Mint Invariants
// ---------------------------------------------------------------------------

#[test]
fn test_regression_nft_ownership_and_transfer_invariants() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let nft_id = env.register_contract(None, basic_nft::BasicNftContract);
    let nft = basic_nft::BasicNftContractClient::new(&env, &nft_id);

    nft.initialize(
        &admin,
        &String::from_str(&env, "Regression NFT"),
        &String::from_str(&env, "RNFT"),
    );

    // Invariant 1: Mint assigns unique token ID to recipient
    let token_id = 1u32;
    nft.mint(&admin, &alice, &token_id);
    assert_eq!(nft.owner_of(&token_id), alice);
    assert_eq!(nft.balance_of(&alice), 1);
    assert_eq!(nft.balance_of(&bob), 0);

    // Invariant 2: Transfer changes ownership and balances
    nft.transfer(&alice, &bob, &token_id);
    assert_eq!(nft.owner_of(&token_id), bob);
    assert_eq!(nft.balance_of(&alice), 0);
    assert_eq!(nft.balance_of(&bob), 1);

    // Invariant 3: Previous owner cannot transfer token anymore
    assert!(nft.try_transfer(&alice, &admin, &token_id).is_err());
}

// ---------------------------------------------------------------------------
// 6. Intermediate & Discovery: Factory & Registry Invariants
// ---------------------------------------------------------------------------

#[test]
fn test_regression_registry_and_factory_invariants() {
    let env = Env::default();
    env.mock_all_auths();

    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let reg = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    let factory_id = env.register_contract(None, ajo_factory::AjoFactory);
    let factory = ajo_factory::AjoFactoryClient::new(&env, &factory_id);

    let wasm_hash = BytesN::from_array(&env, &[0xAAu8; 32]);
    factory.initialize(&wasm_hash);

    // Invariant: Registering metadata makes contracts discoverable
    let contract_name = Symbol::new(&env, "regression_factory");
    reg.register(
        &contract_name,
        &symbol_short!("factory"),
        &symbol_short!("v1"),
        &factory_id,
    );

    let entry = reg.try_get_by_name(&contract_name).unwrap().unwrap();
    assert_eq!(entry.address, factory_id);
    assert_eq!(entry.category, symbol_short!("factory"));
    assert_eq!(entry.version, symbol_short!("v1"));

    // Invariant: Non-existent name returns NotFound error
    let unknown_name = Symbol::new(&env, "non_existent");
    assert_eq!(
        reg.try_get_by_name(&unknown_name),
        Err(Ok(contract_registry::RegistryError::NotFound))
    );
}

// ---------------------------------------------------------------------------
// 7. DeFi Regression: Simple Swap Invariants
// ---------------------------------------------------------------------------

#[test]
fn test_regression_defi_swap_invariants() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let trader = Address::generate(&env);

    // Create two mock tokens via stellar asset contract v2
    let token_a_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_b_id = env.register_stellar_asset_contract_v2(admin.clone());

    let swap_id = env.register_contract(None, simple_swap::SimpleSwapContract);
    let swap = simple_swap::SimpleSwapContractClient::new(&env, &swap_id);

    swap.initialize(
        &admin,
        &token_a_id.address(),
        &token_b_id.address(),
        &1i128,
        &1i128,
    );

    // Mint tokens to trader
    soroban_sdk::token::StellarAssetClient::new(&env, &token_a_id.address()).mint(&trader, &10_000);
    soroban_sdk::token::StellarAssetClient::new(&env, &token_b_id.address()).mint(&trader, &10_000);

    // Invariant: Zero amount swap fails
    assert!(swap
        .try_swap(&trader, &token_a_id.address(), &0, &1, &trader)
        .is_err());
}

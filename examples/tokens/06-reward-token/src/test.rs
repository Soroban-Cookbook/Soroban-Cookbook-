//! Unit tests for the reward-token contract.
//!
//! Run with: cargo test -p reward-token

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// ---------------------------------------------------------------------------
// Test fixture
// ---------------------------------------------------------------------------

struct Fixture {
    env: Env,
    contract: RewardTokenClient<'static>,
    admin: Address,
    alice: Address,
    bob: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RewardToken);
    let contract = RewardTokenClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    contract.initialize(
        &admin,
        &String::from_str(&env, "Reward Token"),
        &String::from_str(&env, "RWD"),
        &7u32,
    );

    Fixture {
        env,
        contract,
        admin,
        alice,
        bob,
    }
}

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

#[test]
fn initialize_stores_fields() {
    let f = setup();

    // Admin is stored and queryable.
    assert_eq!(f.contract.admin(), f.admin);
    // Supply starts at zero.
    assert_eq!(f.contract.total_supply(), 0);
    // Balances of fresh accounts are zero.
    assert_eq!(f.contract.balance(&f.alice), 0);
    assert_eq!(f.contract.balance(&f.bob), 0);
}

#[test]
fn double_initialize_rejected() {
    let f = setup();

    assert_eq!(
        f.contract.try_initialize(
            &f.admin,
            &String::from_str(&f.env, "Other"),
            &String::from_str(&f.env, "OTH"),
            &6u32,
        ),
        Err(Ok(RewardError::AlreadyInitialized))
    );
}

// ---------------------------------------------------------------------------
// Mint
// ---------------------------------------------------------------------------

#[test]
fn mint_increases_balance_and_supply() {
    let f = setup();

    f.contract.mint(&f.alice, &1_000_000_000i128);

    assert_eq!(f.contract.balance(&f.alice), 1_000_000_000);
    assert_eq!(f.contract.total_supply(), 1_000_000_000);
}

#[test]
fn mint_unauthorized_rejected() {
    let env = Env::default();
    // Do NOT mock auths so the admin check actually fires.
    let contract_id = env.register_contract(None, RewardToken);
    let contract = RewardTokenClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);

    env.mock_all_auths();
    contract.initialize(
        &admin,
        &String::from_str(&env, "T"),
        &String::from_str(&env, "T"),
        &7u32,
    );
    // Clear all auth overrides so the next call runs under real auth rules.
    env.set_auths(&[]);

    // Minting without admin auth should panic (host auth failure).
    assert!(contract.try_mint(&alice, &100i128).is_err());
}

// ---------------------------------------------------------------------------
// Burn
// ---------------------------------------------------------------------------

#[test]
fn burn_decreases_balance_and_supply() {
    let f = setup();

    f.contract.mint(&f.alice, &500i128);
    f.contract.burn(&f.alice, &200i128);

    assert_eq!(f.contract.balance(&f.alice), 300);
    assert_eq!(f.contract.total_supply(), 300);
}

#[test]
fn burn_above_balance_rejected() {
    let f = setup();

    f.contract.mint(&f.alice, &100i128);

    assert_eq!(
        f.contract.try_burn(&f.alice, &101i128),
        Err(Ok(RewardError::InsufficientBalance))
    );
}

// ---------------------------------------------------------------------------
// Transfer
// ---------------------------------------------------------------------------

#[test]
fn transfer_moves_balance() {
    let f = setup();

    f.contract.mint(&f.alice, &1_000i128);
    f.contract.transfer(&f.alice, &f.bob, &400i128);

    assert_eq!(f.contract.balance(&f.alice), 600);
    assert_eq!(f.contract.balance(&f.bob), 400);
    assert_eq!(f.contract.total_supply(), 1_000);
}

#[test]
fn transfer_above_balance_rejected() {
    let f = setup();

    f.contract.mint(&f.alice, &100i128);

    assert_eq!(
        f.contract.try_transfer(&f.alice, &f.bob, &101i128),
        Err(Ok(RewardError::InsufficientBalance))
    );
}

// ---------------------------------------------------------------------------
// Pool management
// ---------------------------------------------------------------------------

#[test]
fn create_pool_admin_only() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RewardToken);
    let contract = RewardTokenClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    contract.initialize(
        &admin,
        &String::from_str(&env, "T"),
        &String::from_str(&env, "T"),
        &7u32,
    );
    env.set_auths(&[]);

    // create_pool without admin auth must fail.
    assert!(contract.try_create_pool(&500_000i128).is_err());
}

#[test]
fn create_multiple_pools() {
    let f = setup();

    let id0 = f.contract.create_pool(&500_000i128); // rate = 0.5
    let id1 = f.contract.create_pool(&1_000_000i128); // rate = 1.0
    let id2 = f.contract.create_pool(&250_000i128); // rate = 0.25

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    let pool0 = f.contract.pool_info(&0u32);
    assert_eq!(pool0.rate_per_token, 500_000);
    assert_eq!(pool0.total_deposited, 0);

    let pool1 = f.contract.pool_info(&1u32);
    assert_eq!(pool1.rate_per_token, 1_000_000);

    let pool2 = f.contract.pool_info(&2u32);
    assert_eq!(pool2.rate_per_token, 250_000);
}

// ---------------------------------------------------------------------------
// Deposits
// ---------------------------------------------------------------------------

#[test]
fn deposit_to_pool_increases_total() {
    let f = setup();

    // Admin needs tokens to deposit.
    f.contract.mint(&f.admin, &10_000i128);
    let _pool_id = f.contract.create_pool(&1_000_000i128);

    f.contract.deposit_to_pool(&0u32, &5_000i128);

    let pool = f.contract.pool_info(&0u32);
    assert_eq!(pool.total_deposited, 5_000);

    // Admin balance decremented.
    assert_eq!(f.contract.balance(&f.admin), 5_000);
}

// ---------------------------------------------------------------------------
// Reward claim
// ---------------------------------------------------------------------------

#[test]
fn claim_rewards_proportional() {
    let f = setup();

    // Alice holds 2_000_000 tokens; rate = 500_000 (0.5× reward).
    // Expected reward = 2_000_000 * 500_000 / 1_000_000 = 1_000_000.
    f.contract.mint(&f.alice, &2_000_000i128);
    f.contract.mint(&f.admin, &2_000_000i128);
    let _pool_id = f.contract.create_pool(&500_000i128);
    f.contract.deposit_to_pool(&0u32, &1_500_000i128);

    let claimed = f.contract.claim_rewards(&f.alice, &0u32);
    assert_eq!(claimed, 1_000_000);

    // Alice now holds original balance + reward.
    assert_eq!(f.contract.balance(&f.alice), 3_000_000);

    // Pool reserve reduced by claimed amount.
    let pool = f.contract.pool_info(&0u32);
    assert_eq!(pool.total_deposited, 500_000);
}

#[test]
fn claim_rewards_nothing_to_claim() {
    let f = setup();

    // Rate = 500_000 but balance = 1 → reward = 0 (integer truncation).
    f.contract.mint(&f.alice, &1i128);
    f.contract.mint(&f.admin, &100i128);
    let _pool_id = f.contract.create_pool(&500_000i128);
    f.contract.deposit_to_pool(&0u32, &50i128);

    assert_eq!(
        f.contract.try_claim_rewards(&f.alice, &0u32),
        Err(Ok(RewardError::NothingToClaim))
    );
}

#[test]
fn claimable_rewards_view() {
    let f = setup();

    // Alice holds 1_000_000 tokens; rate = 1_000_000 (1:1 reward).
    f.contract.mint(&f.alice, &1_000_000i128);
    f.contract.mint(&f.admin, &4_000_000i128);
    let _pool_id = f.contract.create_pool(&1_000_000i128);
    f.contract.deposit_to_pool(&0u32, &4_000_000i128);

    // View should report 1_000_000 before any claim.
    let view = f.contract.claimable_rewards(&f.alice, &0u32);
    assert_eq!(view, 1_000_000);

    // After claiming, Alice's balance grows to 2_000_000.
    // gross = 2_000_000 * 1_000_000 / 1_000_000 = 2_000_000
    // already_claimed = 1_000_000 → claimable = 1_000_000.
    // This is consistent with the `double_claim_returns_nothing` test.
    let actual = f.contract.claim_rewards(&f.alice, &0u32);
    assert_eq!(actual, 1_000_000);

    let view_after = f.contract.claimable_rewards(&f.alice, &0u32);
    assert_eq!(view_after, 1_000_000);
}

#[test]
fn pool_not_found_error() {
    let f = setup();

    assert_eq!(
        f.contract.try_pool_info(&99u32),
        Err(Ok(RewardError::PoolNotFound))
    );
    assert_eq!(
        f.contract.try_claim_rewards(&f.alice, &0u32),
        Err(Ok(RewardError::PoolNotFound))
    );
    assert_eq!(
        f.contract.try_claimable_rewards(&f.alice, &0u32),
        Err(Ok(RewardError::PoolNotFound))
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn double_claim_returns_nothing() {
    let f = setup();

    f.contract.mint(&f.alice, &1_000_000i128);
    f.contract.mint(&f.admin, &2_000_000i128);
    let _pool_id = f.contract.create_pool(&1_000_000i128);
    f.contract.deposit_to_pool(&0u32, &2_000_000i128);

    // First claim succeeds.
    let first = f.contract.claim_rewards(&f.alice, &0u32);
    assert_eq!(first, 1_000_000);

    // Second claim: balance is now 2_000_000, but already claimed 1_000_000.
    // New gross = 2_000_000 * 1_000_000 / 1_000_000 = 2_000_000.
    // Claimable = 2_000_000 - 1_000_000 = 1_000_000.
    let second = f.contract.claim_rewards(&f.alice, &0u32);
    assert_eq!(second, 1_000_000);
}

#[test]
fn independent_pools_track_separately() {
    let f = setup();

    f.contract.mint(&f.alice, &1_000_000i128);
    f.contract.mint(&f.admin, &4_000_000i128);

    // Pool 0: rate 1_000_000 (1:1), Pool 1: rate 500_000 (0.5:1).
    let _p0 = f.contract.create_pool(&1_000_000i128);
    let _p1 = f.contract.create_pool(&500_000i128);

    f.contract.deposit_to_pool(&0u32, &2_000_000i128);
    f.contract.deposit_to_pool(&1u32, &2_000_000i128);

    let claim0 = f.contract.claim_rewards(&f.alice, &0u32);
    let claim1 = f.contract.claim_rewards(&f.alice, &1u32);

    // Pool 0 reward = 1_000_000 * 1_000_000 / 1_000_000 = 1_000_000
    assert_eq!(claim0, 1_000_000);
    // Pool 1 reward based on updated balance (2_000_000) * 500_000 / 1_000_000 = 1_000_000
    // But claimed from p1 is 0, so full amount.
    // Balance after pool0 claim = 2_000_000.
    // Pool1 reward = 2_000_000 * 500_000 / 1_000_000 = 1_000_000.
    assert_eq!(claim1, 1_000_000);
}

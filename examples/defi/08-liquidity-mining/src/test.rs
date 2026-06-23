extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

// ─── Test helpers ─────────────────────────────────────────────────────────────

/// Deploy a native Stellar asset token and return (token_address, admin_client).
fn create_token<'a>(env: &'a Env, admin: &Address) -> (Address, TokenClient<'a>) {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let addr = token_id.address();
    let client = TokenClient::new(env, &addr);
    (addr, client)
}

/// Mint `amount` of `token` to `to` using the StellarAssetClient (admin mint).
fn mint(env: &Env, token: &Address, admin: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
    let _ = admin; // admin is embedded in the asset contract
}

struct TestSetup<'a> {
    env: Env,
    admin: Address,
    mining: LiquidityMiningClient<'a>,
    lp_token: Address,
    reward_token: Address,
}

fn setup<'a>() -> TestSetup<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    // Deploy the mining contract
    let contract_id = env.register_contract(None, LiquidityMining);
    let mining = LiquidityMiningClient::new(&env, &contract_id);
    mining.initialize(&admin);

    // Create LP token and reward token
    let (lp_token, _) = create_token(&env, &admin);
    let (reward_token, _) = create_token(&env, &admin);

    // Fund the mining contract with reward tokens so it can pay out
    mint(&env, &reward_token, &admin, &contract_id, 1_000_000_000);

    TestSetup {
        env,
        admin,
        mining,
        lp_token,
        reward_token,
    }
}

// ─── 1. Initialization ────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_admin() {
    let t = setup();
    assert_eq!(t.mining.get_admin(), t.admin);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_initialize_twice_panics() {
    let t = setup();
    let other = Address::generate(&t.env);
    t.mining.initialize(&other);
}

// ─── 2. Pool management ───────────────────────────────────────────────────────

#[test]
fn test_add_pool_stores_config() {
    let t = setup();
    let rate: i128 = 100;
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &rate);

    let pool = t.mining.get_pool(&1u32);
    assert_eq!(pool.lp_token, t.lp_token);
    assert_eq!(pool.reward_token, t.reward_token);
    assert_eq!(pool.reward_rate, rate);
    assert_eq!(pool.total_staked, 0);
    assert!(pool.active);
}

#[test]
#[should_panic(expected = "Pool already exists")]
fn test_add_duplicate_pool_panics() {
    let t = setup();
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &100i128);
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &200i128);
}

#[test]
#[should_panic(expected = "Invalid reward rate")]
fn test_add_pool_zero_rate_panics() {
    let t = setup();
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &0i128);
}

#[test]
fn test_multiple_pools_independent() {
    let t = setup();
    let (lp2, _) = create_token(&t.env, &t.admin);
    let (rw2, _) = create_token(&t.env, &t.admin);
    mint(&t.env, &rw2, &t.admin, &t.mining.address, 1_000_000_000);

    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &100i128);
    t.mining.add_pool(&2u32, &lp2, &rw2, &200i128);

    let pool1 = t.mining.get_pool(&1u32);
    let pool2 = t.mining.get_pool(&2u32);

    assert_eq!(pool1.reward_rate, 100);
    assert_eq!(pool2.reward_rate, 200);
    assert_ne!(pool1.lp_token, pool2.lp_token);
}

// ─── 3. Staking ───────────────────────────────────────────────────────────────

#[test]
fn test_stake_increases_total_staked() {
    let t = setup();
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &100i128);

    let user = Address::generate(&t.env);
    mint(&t.env, &t.lp_token, &t.admin, &user, 1_000);

    t.mining.stake(&1u32, &user, &500i128);

    let pool = t.mining.get_pool(&1u32);
    assert_eq!(pool.total_staked, 500);

    let info = t.mining.get_user_info(&1u32, &user);
    assert_eq!(info.staked, 500);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_stake_zero_panics() {
    let t = setup();
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &100i128);
    let user = Address::generate(&t.env);
    t.mining.stake(&1u32, &user, &0i128);
}

#[test]
#[should_panic(expected = "Pool is not active")]
fn test_stake_into_paused_pool_panics() {
    let t = setup();
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &100i128);
    t.mining.set_pool_active(&1u32, &false);

    let user = Address::generate(&t.env);
    mint(&t.env, &t.lp_token, &t.admin, &user, 1_000);
    t.mining.stake(&1u32, &user, &500i128);
}

// ─── 4. Unstaking ─────────────────────────────────────────────────────────────

#[test]
fn test_unstake_returns_lp_tokens() {
    let t = setup();
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &100i128);

    let user = Address::generate(&t.env);
    mint(&t.env, &t.lp_token, &t.admin, &user, 1_000);

    t.mining.stake(&1u32, &user, &1_000i128);
    t.mining.unstake(&1u32, &user, &600i128);

    let lp_client = TokenClient::new(&t.env, &t.lp_token);
    assert_eq!(lp_client.balance(&user), 600);

    let info = t.mining.get_user_info(&1u32, &user);
    assert_eq!(info.staked, 400);
}

#[test]
#[should_panic(expected = "Insufficient staked balance")]
fn test_unstake_more_than_staked_panics() {
    let t = setup();
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &100i128);

    let user = Address::generate(&t.env);
    mint(&t.env, &t.lp_token, &t.admin, &user, 500);
    t.mining.stake(&1u32, &user, &500i128);
    t.mining.unstake(&1u32, &user, &501i128);
}

// ─── 5. Reward accrual & harvest ─────────────────────────────────────────────

#[test]
fn test_harvest_pays_correct_rewards() {
    let t = setup();
    let rate: i128 = 1_000; // 1000 reward tokens per ledger
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &rate);

    let user = Address::generate(&t.env);
    mint(&t.env, &t.lp_token, &t.admin, &user, 10_000);
    t.mining.stake(&1u32, &user, &10_000i128);

    // Advance 10 ledgers
    t.env.ledger().with_mut(|l| l.sequence_number += 10);

    t.mining.harvest(&1u32, &user);

    let reward_client = TokenClient::new(&t.env, &t.reward_token);
    let balance = reward_client.balance(&user);

    // Expected: 10 ledgers * 1000 rate = 10_000 reward tokens
    assert_eq!(balance, 10_000);
}

#[test]
fn test_pending_rewards_view_matches_harvest() {
    let t = setup();
    let rate: i128 = 500;
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &rate);

    let user = Address::generate(&t.env);
    mint(&t.env, &t.lp_token, &t.admin, &user, 5_000);
    t.mining.stake(&1u32, &user, &5_000i128);

    t.env.ledger().with_mut(|l| l.sequence_number += 20);

    let pending = t.mining.pending_rewards(&1u32, &user);
    // 20 ledgers * 500 rate = 10_000
    assert_eq!(pending, 10_000);

    t.mining.harvest(&1u32, &user);
    let reward_client = TokenClient::new(&t.env, &t.reward_token);
    assert_eq!(reward_client.balance(&user), pending);
}

#[test]
#[should_panic(expected = "Nothing to harvest")]
fn test_harvest_with_no_rewards_panics() {
    let t = setup();
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &100i128);

    let user = Address::generate(&t.env);
    // User never staked — nothing to harvest
    t.mining.harvest(&1u32, &user);
}

// ─── 6. Reward rate adjustment ────────────────────────────────────────────────

#[test]
fn test_set_reward_rate_changes_future_rewards() {
    let t = setup();
    let initial_rate: i128 = 1_000;
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &initial_rate);

    let user = Address::generate(&t.env);
    mint(&t.env, &t.lp_token, &t.admin, &user, 10_000);
    t.mining.stake(&1u32, &user, &10_000i128);

    // Earn 5 ledgers at rate 1000 → 5_000 rewards
    t.env.ledger().with_mut(|l| l.sequence_number += 5);

    // Change rate to 2000
    t.mining.set_reward_rate(&1u32, &2_000i128);

    // Earn 5 more ledgers at rate 2000 → 10_000 rewards
    t.env.ledger().with_mut(|l| l.sequence_number += 5);

    t.mining.harvest(&1u32, &user);

    let reward_client = TokenClient::new(&t.env, &t.reward_token);
    // Total: 5*1000 + 5*2000 = 15_000
    assert_eq!(reward_client.balance(&user), 15_000);
}

#[test]
#[should_panic(expected = "Invalid reward rate")]
fn test_set_reward_rate_zero_panics() {
    let t = setup();
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &100i128);
    t.mining.set_reward_rate(&1u32, &0i128);
}

// ─── 7. Multiple stakers proportional rewards ────────────────────────────────

#[test]
fn test_two_stakers_proportional_rewards() {
    let t = setup();
    let rate: i128 = 2_000;
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &rate);

    let alice = Address::generate(&t.env);
    let bob = Address::generate(&t.env);

    mint(&t.env, &t.lp_token, &t.admin, &alice, 10_000);
    mint(&t.env, &t.lp_token, &t.admin, &bob, 10_000);

    // Both stake equal amounts at the same ledger
    t.mining.stake(&1u32, &alice, &1_000i128);
    t.mining.stake(&1u32, &bob, &1_000i128);

    // Advance 10 ledgers
    t.env.ledger().with_mut(|l| l.sequence_number += 10);

    t.mining.harvest(&1u32, &alice);
    t.mining.harvest(&1u32, &bob);

    let reward_client = TokenClient::new(&t.env, &t.reward_token);
    let alice_bal = reward_client.balance(&alice);
    let bob_bal = reward_client.balance(&bob);

    // Total rewards = 10 * 2000 = 20_000, split 50/50
    assert_eq!(alice_bal, 10_000);
    assert_eq!(bob_bal, 10_000);
}

// ─── 8. Pool pause / resume ───────────────────────────────────────────────────

#[test]
fn test_paused_pool_stops_reward_accrual() {
    let t = setup();
    let rate: i128 = 1_000;
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &rate);

    let user = Address::generate(&t.env);
    mint(&t.env, &t.lp_token, &t.admin, &user, 10_000);
    t.mining.stake(&1u32, &user, &10_000i128);

    // Earn 5 ledgers
    t.env.ledger().with_mut(|l| l.sequence_number += 5);

    // Pause the pool — accumulator is settled at this point
    t.mining.set_pool_active(&1u32, &false);

    // Advance 10 more ledgers while paused
    t.env.ledger().with_mut(|l| l.sequence_number += 10);

    // Resume
    t.mining.set_pool_active(&1u32, &true);

    // Earn 5 more ledgers after resume
    t.env.ledger().with_mut(|l| l.sequence_number += 5);

    // Pending should reflect 5 + 5 = 10 ledgers of rewards (not 20)
    let pending = t.mining.pending_rewards(&1u32, &user);
    assert_eq!(pending, 10_000); // 10 * 1000
}

// ─── 9. Unstake accumulates pending before withdrawal ────────────────────────

#[test]
fn test_unstake_accumulates_pending_rewards() {
    let t = setup();
    let rate: i128 = 1_000;
    t.mining
        .add_pool(&1u32, &t.lp_token, &t.reward_token, &rate);

    let user = Address::generate(&t.env);
    mint(&t.env, &t.lp_token, &t.admin, &user, 10_000);
    t.mining.stake(&1u32, &user, &10_000i128);

    t.env.ledger().with_mut(|l| l.sequence_number += 10);

    // Unstake — rewards should be accumulated in pending_rewards
    t.mining.unstake(&1u32, &user, &10_000i128);

    let info = t.mining.get_user_info(&1u32, &user);
    assert_eq!(info.pending_rewards, 10_000);
    assert_eq!(info.staked, 0);

    // Harvest the accumulated rewards
    t.mining.harvest(&1u32, &user);
    let reward_client = TokenClient::new(&t.env, &t.reward_token);
    assert_eq!(reward_client.balance(&user), 10_000);
}

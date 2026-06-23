extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Spin up a fresh environment, register the contract, and initialise it with
/// the Conservative strategy.  Returns `(env, admin, client)`.
fn setup() -> (Env, Address, VaultContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &StrategyType::Conservative);

    (env, admin, client)
}

// ─────────────────────────────────────────────────────────────────────────────
// Initialisation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_defaults() {
    let (_env, _admin, client) = setup();

    assert_eq!(client.active_strategy(), StrategyType::Conservative);
    assert_eq!(client.total_deposits(), 0);
    assert!(!client.is_paused());
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_initialize_twice_panics() {
    let (env, _admin, client) = setup();
    // setup() already called initialize once; calling again must panic
    let admin2 = Address::generate(&env);
    client.initialize(&admin2, &StrategyType::Balanced);
}

// ─────────────────────────────────────────────────────────────────────────────
// Strategy parameters (strategy interface)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_conservative_strategy_params() {
    let (env, _admin, _client) = setup();
    let params = strategy_params(&env, &StrategyType::Conservative);

    assert_eq!(params.max_allocation_bps, 10_000); // 100 %
    assert_eq!(params.expected_apy_bps, 300); // 3 %
    assert_eq!(params.risk_level, RiskLevel::Low);
}

#[test]
fn test_balanced_strategy_params() {
    let (env, _admin, _client) = setup();
    let params = strategy_params(&env, &StrategyType::Balanced);

    assert_eq!(params.max_allocation_bps, 8_000); // 80 %
    assert_eq!(params.expected_apy_bps, 800); // 8 %
    assert_eq!(params.risk_level, RiskLevel::Medium);
}

#[test]
fn test_aggressive_strategy_params() {
    let (env, _admin, _client) = setup();
    let params = strategy_params(&env, &StrategyType::Aggressive);

    assert_eq!(params.max_allocation_bps, 5_000); // 50 %
    assert_eq!(params.expected_apy_bps, 2_000); // 20 %
    assert_eq!(params.risk_level, RiskLevel::High);
}

#[test]
fn test_strategy_info_returns_active_params() {
    let (_env, admin, client) = setup();

    // Switch to Balanced and verify strategy_info reflects it
    client.switch_strategy(&admin, &StrategyType::Balanced);
    let info = client.strategy_info();
    assert_eq!(info.expected_apy_bps, 800);
    assert_eq!(info.risk_level, RiskLevel::Medium);
}

// ─────────────────────────────────────────────────────────────────────────────
// Deposits
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_deposit_updates_balances() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.deposit(&user, &1_000);

    assert_eq!(client.balance(&user), 1_000);
    assert_eq!(client.total_deposits(), 1_000);
}

#[test]
fn test_multiple_deposits_accumulate() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.deposit(&user, &500);
    client.deposit(&user, &300);

    assert_eq!(client.balance(&user), 800);
    assert_eq!(client.total_deposits(), 800);
}

#[test]
fn test_two_users_deposit_independently() {
    let (env, _admin, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.deposit(&alice, &1_000);
    client.deposit(&bob, &500);

    assert_eq!(client.balance(&alice), 1_000);
    assert_eq!(client.balance(&bob), 500);
    assert_eq!(client.total_deposits(), 1_500);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_deposit_zero_panics() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    client.deposit(&user, &0);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_deposit_negative_panics() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    client.deposit(&user, &-1);
}

// ─────────────────────────────────────────────────────────────────────────────
// Withdrawals
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_withdraw_reduces_balances() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.deposit(&user, &1_000);
    client.withdraw(&user, &400);

    assert_eq!(client.balance(&user), 600);
    assert_eq!(client.total_deposits(), 600);
}

#[test]
fn test_full_withdrawal() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.deposit(&user, &1_000);
    client.withdraw(&user, &1_000);

    assert_eq!(client.balance(&user), 0);
    assert_eq!(client.total_deposits(), 0);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_withdraw_more_than_balance_panics() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.deposit(&user, &500);
    client.withdraw(&user, &501);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_withdraw_zero_panics() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    client.withdraw(&user, &0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Strategy switching
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_switch_to_balanced() {
    let (_env, admin, client) = setup();

    client.switch_strategy(&admin, &StrategyType::Balanced);
    assert_eq!(client.active_strategy(), StrategyType::Balanced);
}

#[test]
fn test_switch_to_aggressive_when_tvl_is_low() {
    let (_env, admin, client) = setup();

    // TVL is 0 — well below MAX_TVL_FOR_AGGRESSIVE
    client.switch_strategy(&admin, &StrategyType::Aggressive);
    assert_eq!(client.active_strategy(), StrategyType::Aggressive);
}

#[test]
fn test_switch_cycle_through_all_strategies() {
    let (_env, admin, client) = setup();

    client.switch_strategy(&admin, &StrategyType::Balanced);
    assert_eq!(client.active_strategy(), StrategyType::Balanced);

    client.switch_strategy(&admin, &StrategyType::Aggressive);
    assert_eq!(client.active_strategy(), StrategyType::Aggressive);

    client.switch_strategy(&admin, &StrategyType::Conservative);
    assert_eq!(client.active_strategy(), StrategyType::Conservative);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_switch_strategy_non_admin_panics() {
    let (env, _admin, client) = setup();
    let attacker = Address::generate(&env);
    client.switch_strategy(&attacker, &StrategyType::Balanced);
}

// ─────────────────────────────────────────────────────────────────────────────
// Risk management — TVL guard for Aggressive strategy
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "TVL too high for aggressive strategy")]
fn test_switch_to_aggressive_blocked_when_tvl_too_high() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    // Deposit more than MAX_TVL_FOR_AGGRESSIVE (1_000_000)
    client.deposit(&user, &1_000_001);

    // Attempt to switch to Aggressive — must be blocked
    client.switch_strategy(&admin, &StrategyType::Aggressive);
}

#[test]
fn test_switch_to_aggressive_allowed_at_exact_tvl_limit() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    // Deposit exactly MAX_TVL_FOR_AGGRESSIVE — should still be allowed
    client.deposit(&user, &1_000_000);
    client.switch_strategy(&admin, &StrategyType::Aggressive);
    assert_eq!(client.active_strategy(), StrategyType::Aggressive);
}

// ─────────────────────────────────────────────────────────────────────────────
// Risk management — allocation cap
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "Exceeds strategy allocation cap")]
fn test_deposit_exceeds_balanced_allocation_cap() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    // Switch to Balanced (80 % cap)
    client.switch_strategy(&admin, &StrategyType::Balanced);

    // First deposit: 100 tokens.  100 / 100 = 100 % > 80 % cap → blocked.
    // The cap check is: amount * BPS_DENOM > new_total * max_allocation_bps
    // 100 * 10_000 = 1_000_000 > 100 * 8_000 = 800_000 → panic
    client.deposit(&user, &100);
}

#[test]
fn test_deposit_within_balanced_allocation_cap() {
    let (env, admin, client) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Switch to Balanced (80 % cap)
    client.switch_strategy(&admin, &StrategyType::Balanced);

    // Deposit 20 tokens first (user2 — acts as "other" TVL)
    // Then deposit 80 tokens (user1).
    // new_total = 100; user1 amount = 80; 80 * 10_000 = 800_000 == 100 * 8_000 = 800_000 → OK
    client.deposit(&user2, &20);
    client.deposit(&user1, &80);

    assert_eq!(client.balance(&user1), 80);
    assert_eq!(client.total_deposits(), 100);
}

// ─────────────────────────────────────────────────────────────────────────────
// Risk management — emergency pause
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "Vault is paused")]
fn test_pause_blocks_deposits() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    client.pause(&admin);
    assert!(client.is_paused());

    // Deposit must be blocked — panics with "Vault is paused"
    client.deposit(&user, &100);
}

#[test]
fn test_pause_does_not_block_withdrawals() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    // Deposit before pausing
    client.deposit(&user, &500);

    client.pause(&admin);
    assert!(client.is_paused());

    // Withdrawal must still succeed
    client.withdraw(&user, &500);
    assert_eq!(client.balance(&user), 0);
}

#[test]
fn test_unpause_re_enables_deposits() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    client.pause(&admin);
    client.unpause(&admin);
    assert!(!client.is_paused());

    client.deposit(&user, &100);
    assert_eq!(client.balance(&user), 100);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_pause_non_admin_panics() {
    let (env, _admin, client) = setup();
    let attacker = Address::generate(&env);
    client.pause(&attacker);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_unpause_non_admin_panics() {
    let (env, admin, client) = setup();
    let attacker = Address::generate(&env);
    client.pause(&admin);
    client.unpause(&attacker);
}

// ─────────────────────────────────────────────────────────────────────────────
// Yield estimation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_estimate_yield_conservative() {
    let (_env, _admin, client) = setup();
    // Conservative: 300 bps APY, 365 days
    // yield = 10_000 * 300 * 365 / (10_000 * 365) = 300
    let y = client.estimate_yield(&10_000, &365);
    assert_eq!(y, 300);
}

#[test]
fn test_estimate_yield_aggressive() {
    let (_env, admin, client) = setup();
    client.switch_strategy(&admin, &StrategyType::Aggressive);
    // Aggressive: 2_000 bps APY, 365 days
    // yield = 10_000 * 2_000 * 365 / (10_000 * 365) = 2_000
    let y = client.estimate_yield(&10_000, &365);
    assert_eq!(y, 2_000);
}

#[test]
fn test_estimate_yield_zero_periods() {
    let (_env, _admin, client) = setup();
    let y = client.estimate_yield(&10_000, &0);
    assert_eq!(y, 0);
}

#[test]
fn test_simulate_yield_helper() {
    // 10_000 tokens, 800 bps APY, 365 days → 800 tokens yield
    assert_eq!(simulate_yield(10_000, 800, 365), 800);
    // 0 tokens → 0 yield
    assert_eq!(simulate_yield(0, 800, 365), 0);
    // 0 periods → 0 yield
    assert_eq!(simulate_yield(10_000, 800, 0), 0);
}

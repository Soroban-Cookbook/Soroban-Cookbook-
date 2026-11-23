//! Unit tests for the reward distribution strategies contract.

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env};

fn at(env: &Env, timestamp: u64) {
    env.ledger().with_mut(|li| li.timestamp = timestamp);
}

fn setup(env: &Env) -> (Address, RewardDistributorClient<'static>) {
    let contract_id = env.register_contract(None, RewardDistributor);
    let client = RewardDistributorClient::new(env, &contract_id);
    (contract_id, client)
}

#[test]
fn test_linear_distribution_releases_over_time() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let (_, client) = setup(&env);

    client
        .try_initialize(&admin, &Strategy::Linear, &1000, &0, &100, &0, &0)
        .unwrap()
        .unwrap();
    client.try_register(&admin, &alice, &1).unwrap().unwrap();

    at(&env, 0);
    assert_eq!(client.claimable(&alice, &0), 0);
    at(&env, 50);
    assert_eq!(client.claimable(&alice, &50), 500);
    at(&env, 100);
    assert_eq!(client.claimable(&alice, &100), 1000);
    // Capped at the pool total once the vesting window has fully elapsed.
    at(&env, 500);
    assert_eq!(client.claimable(&alice, &500), 1000);
}

#[test]
fn test_exponential_decay_matches_formula() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let (_, client) = setup(&env);

    // 10% decay every 10 seconds over a 1000 second window.
    client
        .try_initialize(
            &admin,
            &Strategy::ExponentialDecay,
            &1000,
            &0,
            &1000,
            &1000,
            &10,
        )
        .unwrap()
        .unwrap();
    client.try_register(&admin, &alice, &1).unwrap().unwrap();

    at(&env, 0);
    assert_eq!(client.claimable(&alice, &0), 0);
    // 1 period: remaining = 1000 * 9000/10000 = 900 -> released = 100.
    at(&env, 10);
    assert_eq!(client.claimable(&alice, &10), 100);
    // 2 periods: remaining = 1000 * (9000/10000)^2 = 810 -> released = 190.
    at(&env, 20);
    assert_eq!(client.claimable(&alice, &20), 190);

    // Releases monotonically increase and never exceed the pool total.
    at(&env, 50);
    let at_5_periods = client.claimable(&alice, &50);
    at(&env, 200);
    let at_20_periods = client.claimable(&alice, &200);
    assert!(at_20_periods > at_5_periods);
    assert!(at_20_periods <= 1000);
}

#[test]
fn test_performance_based_splits_by_registered_score() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let (_, client) = setup(&env);

    client
        .try_initialize(&admin, &Strategy::PerformanceBased, &900, &0, &1, &0, &0)
        .unwrap()
        .unwrap();
    client.try_register(&admin, &alice, &100).unwrap().unwrap();
    client.try_register(&admin, &bob, &200).unwrap().unwrap();
    client.try_register(&admin, &carol, &600).unwrap().unwrap();

    // Performance-based rewards are available immediately, independent of time.
    assert_eq!(client.claimable(&alice, &0), 100);
    assert_eq!(client.claimable(&bob, &0), 200);
    assert_eq!(client.claimable(&carol, &0), 600);
}

#[test]
fn test_claim_tracks_already_claimed_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let (_, client) = setup(&env);

    client
        .try_initialize(&admin, &Strategy::Linear, &1000, &0, &100, &0, &0)
        .unwrap()
        .unwrap();
    client.try_register(&admin, &alice, &1).unwrap().unwrap();

    at(&env, 50);
    let claimed = client.try_claim(&alice, &50).unwrap().unwrap();
    assert_eq!(claimed, 500);
    assert_eq!(client.claimable(&alice, &50), 0);

    // Nothing new to claim at the same timestamp.
    assert_eq!(
        client.try_claim(&alice, &50),
        Err(Ok(RewardError::NothingToClaim))
    );

    // More unlocks by the end of the window.
    at(&env, 100);
    let claimed_rest = client.try_claim(&alice, &100).unwrap().unwrap();
    assert_eq!(claimed_rest, 500);
}

#[test]
fn test_register_rejects_non_admin_duplicate_and_invalid_weight() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let not_admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let (_, client) = setup(&env);

    client
        .try_initialize(&admin, &Strategy::PerformanceBased, &100, &0, &1, &0, &0)
        .unwrap()
        .unwrap();

    assert_eq!(
        client.try_register(&not_admin, &alice, &10),
        Err(Ok(RewardError::Unauthorized))
    );
    assert_eq!(
        client.try_register(&admin, &alice, &0),
        Err(Ok(RewardError::InvalidAmount))
    );

    client.try_register(&admin, &alice, &10).unwrap().unwrap();
    assert_eq!(
        client.try_register(&admin, &alice, &10),
        Err(Ok(RewardError::AlreadyRegistered))
    );
}

#[test]
fn test_initialize_rejects_invalid_configuration() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let (_, bad_total) = setup(&env);
    assert_eq!(
        bad_total.try_initialize(&admin, &Strategy::Linear, &0, &0, &100, &0, &0),
        Err(Ok(RewardError::InvalidConfig))
    );

    let (_, bad_duration) = setup(&env);
    assert_eq!(
        bad_duration.try_initialize(&admin, &Strategy::Linear, &1000, &0, &0, &0, &0),
        Err(Ok(RewardError::InvalidConfig))
    );

    let (_, bad_decay) = setup(&env);
    assert_eq!(
        bad_decay.try_initialize(
            &admin,
            &Strategy::ExponentialDecay,
            &1000,
            &0,
            &100,
            &0,
            &10
        ),
        Err(Ok(RewardError::InvalidConfig))
    );

    let (_, bad_period) = setup(&env);
    assert_eq!(
        bad_period.try_initialize(
            &admin,
            &Strategy::ExponentialDecay,
            &1000,
            &0,
            &100,
            &1000,
            &0
        ),
        Err(Ok(RewardError::InvalidConfig))
    );

    let (_, double_init) = setup(&env);
    double_init
        .try_initialize(&admin, &Strategy::Linear, &1000, &0, &100, &0, &0)
        .unwrap()
        .unwrap();
    assert_eq!(
        double_init.try_initialize(&admin, &Strategy::Linear, &1000, &0, &100, &0, &0),
        Err(Ok(RewardError::AlreadyInitialized))
    );
}

/// Registers `alice` and `bob` with equal weight on a 1000-token linear pool
/// that fully unlocks over 100 seconds.
fn setup_two_equal_participants(env: &Env) -> (Address, Address, RewardDistributorClient<'static>) {
    let admin = Address::generate(env);
    let alice = Address::generate(env);
    let bob = Address::generate(env);
    let (_, client) = setup(env);

    client
        .try_initialize(&admin, &Strategy::Linear, &1000, &0, &100, &0, &0)
        .unwrap()
        .unwrap();
    client.try_register(&admin, &alice, &1).unwrap().unwrap();
    client.try_register(&admin, &bob, &1).unwrap().unwrap();
    (alice, bob, client)
}

#[test]
fn test_compound_matches_claim_then_stake() {
    let env = Env::default();
    env.mock_all_auths();
    let (alice, bob, client) = setup_two_equal_participants(&env);

    // Two compounding cycles: alice uses claim + stake, bob uses compound_rewards.
    for now in [50u64, 100u64] {
        at(&env, now);
        let claimed = client.try_claim(&alice, &now).unwrap().unwrap();
        client.try_stake(&alice, &claimed).unwrap().unwrap();
        let compounded = client.try_compound_rewards(&bob, &now).unwrap().unwrap();

        assert_eq!(compounded, claimed);
        assert_eq!(client.position(&bob), client.position(&alice));
        assert_eq!(client.claimable(&bob, &now), client.claimable(&alice, &now));
    }

    assert_eq!(
        client.position(&bob),
        Position {
            claimed: 500,
            staked: 500
        }
    );
    assert_eq!(client.claimable(&bob, &100), 0);
}

#[test]
fn test_compound_costs_one_position_cycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (alice, bob, client) = setup_two_equal_participants(&env);
    at(&env, 50);

    env.cost_estimate().budget().reset_default();
    let claimed = client.claim(&alice, &50);
    let claim_cpu = env.cost_estimate().budget().cpu_instruction_cost();

    env.cost_estimate().budget().reset_default();
    client.stake(&alice, &claimed);
    let stake_cpu = env.cost_estimate().budget().cpu_instruction_cost();

    env.cost_estimate().budget().reset_default();
    client.compound_rewards(&bob, &50);
    let compound_cpu = env.cost_estimate().budget().cpu_instruction_cost();

    // compound_rewards does the same position read and write as a single claim
    // (plus one in-memory addition), so it must cost about the same as claim
    // alone and clearly less than claim + stake.
    assert!(compound_cpu < claim_cpu + stake_cpu);
    assert!(compound_cpu <= claim_cpu + claim_cpu / 20);
}

#[test]
fn test_compound_rejects_nothing_to_claim_and_unregistered() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, bob, client) = setup_two_equal_participants(&env);
    let stranger = Address::generate(&env);

    at(&env, 0);
    assert_eq!(
        client.try_compound_rewards(&bob, &0),
        Err(Ok(RewardError::NothingToClaim))
    );

    at(&env, 50);
    client.try_compound_rewards(&bob, &50).unwrap().unwrap();
    // Already settled up to this timestamp.
    assert_eq!(
        client.try_compound_rewards(&bob, &50),
        Err(Ok(RewardError::NothingToClaim))
    );
    assert_eq!(
        client.try_compound_rewards(&stranger, &50),
        Err(Ok(RewardError::NotRegistered))
    );
}

#[test]
fn test_stake_rejects_invalid_amount_and_unclaimed_rewards() {
    let env = Env::default();
    env.mock_all_auths();
    let (alice, _, client) = setup_two_equal_participants(&env);

    assert_eq!(
        client.try_stake(&alice, &0),
        Err(Ok(RewardError::InvalidAmount))
    );
    // Nothing claimed yet, so nothing can be staked.
    assert_eq!(
        client.try_stake(&alice, &1),
        Err(Ok(RewardError::InsufficientClaimed))
    );

    at(&env, 50);
    let claimed = client.try_claim(&alice, &50).unwrap().unwrap();
    assert_eq!(
        client.try_stake(&alice, &(claimed + 1)),
        Err(Ok(RewardError::InsufficientClaimed))
    );
    client.try_stake(&alice, &claimed).unwrap().unwrap();
    assert_eq!(
        client.position(&alice),
        Position {
            claimed,
            staked: claimed
        }
    );
}

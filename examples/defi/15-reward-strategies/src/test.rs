//! Unit tests for the reward distribution strategies contract.

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

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

    assert_eq!(client.claimable(&alice, &0), 0);
    assert_eq!(client.claimable(&alice, &50), 500);
    assert_eq!(client.claimable(&alice, &100), 1000);
    // Capped at the pool total once the vesting window has fully elapsed.
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

    assert_eq!(client.claimable(&alice, &0), 0);
    // 1 period: remaining = 1000 * 9000/10000 = 900 -> released = 100.
    assert_eq!(client.claimable(&alice, &10), 100);
    // 2 periods: remaining = 1000 * (9000/10000)^2 = 810 -> released = 190.
    assert_eq!(client.claimable(&alice, &20), 190);

    // Releases monotonically increase and never exceed the pool total.
    let at_5_periods = client.claimable(&alice, &50);
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

    let claimed = client.try_claim(&alice, &50).unwrap().unwrap();
    assert_eq!(claimed, 500);
    assert_eq!(client.claimable(&alice, &50), 0);

    // Nothing new to claim at the same timestamp.
    assert_eq!(
        client.try_claim(&alice, &50),
        Err(Ok(RewardError::NothingToClaim))
    );

    // More unlocks by the end of the window.
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

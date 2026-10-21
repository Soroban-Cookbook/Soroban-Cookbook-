#![cfg(test)]

extern crate std;

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    vec, Address, Env, Symbol, Vec,
};

use oracle_consumer_common::{
    testutils::{BrokenFeed, MockFeed, MockFeedClient},
    ConsumerError, PRICE_SCALE,
};

use crate::{AggConfig, AggregatingConsumer, AggregatingConsumerClient};

const ASSET: Symbol = symbol_short!("XLM");
const ONE: i128 = PRICE_SCALE;
const NOW: u64 = 1_000_000;

fn new_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    env
}

fn new_feed(env: &Env, price: i128, timestamp: u64) -> Address {
    let id = env.register(MockFeed, ());
    MockFeedClient::new(env, &id).set_quote(&ASSET, &price, &timestamp);
    id
}

fn agg_config(min_responses: u32) -> AggConfig {
    AggConfig {
        asset: ASSET,
        max_age: 300,
        min_responses,
        min_price: ONE / 100,
        max_price: ONE * 100,
    }
}

fn setup(env: &Env, feeds: Vec<Address>, min_responses: u32) -> AggregatingConsumerClient<'static> {
    let id = env.register(AggregatingConsumer, ());
    let client = AggregatingConsumerClient::new(env, &id);
    client.initialize(&Address::generate(env), &feeds, &agg_config(min_responses));
    client
}

// ── initialization ──────────────────────────────────────────────────────────

#[test]
fn initialize_rejects_a_quorum_larger_than_the_feed_set() {
    let env = new_env();
    let feed = new_feed(&env, ONE, NOW);
    let client = AggregatingConsumerClient::new(&env, &env.register(AggregatingConsumer, ()));

    assert_eq!(
        client.try_initialize(&Address::generate(&env), &vec![&env, feed], &agg_config(2)),
        Err(Ok(ConsumerError::InvalidConfig))
    );
}

#[test]
fn initialize_rejects_a_zero_quorum() {
    let env = new_env();
    let feed = new_feed(&env, ONE, NOW);
    let client = AggregatingConsumerClient::new(&env, &env.register(AggregatingConsumer, ()));

    assert_eq!(
        client.try_initialize(&Address::generate(&env), &vec![&env, feed], &agg_config(0)),
        Err(Ok(ConsumerError::InvalidConfig))
    );
}

// ── aggregation ─────────────────────────────────────────────────────────────

#[test]
fn median_of_an_odd_feed_count() {
    let env = new_env();
    let a = new_feed(&env, ONE, NOW);
    let b = new_feed(&env, ONE * 3, NOW);
    let c = new_feed(&env, ONE * 2, NOW);
    let client = setup(&env, vec![&env, a, b, c], 2);

    assert_eq!(client.median_price(), ONE * 2);
}

#[test]
fn median_of_an_even_feed_count_averages_the_middle_pair() {
    let env = new_env();
    let a = new_feed(&env, ONE, NOW);
    let b = new_feed(&env, ONE * 2, NOW);
    let c = new_feed(&env, ONE * 4, NOW);
    let d = new_feed(&env, ONE * 6, NOW);
    let client = setup(&env, vec![&env, a, b, c, d], 2);

    assert_eq!(client.median_price(), ONE * 3);
}

#[test]
fn stale_and_out_of_bounds_feeds_are_dropped() {
    let env = new_env();
    let fresh = new_feed(&env, ONE, NOW);
    let stale = new_feed(&env, ONE * 50, NOW - 301);
    let silly = new_feed(&env, ONE * 500, NOW);
    let client = setup(&env, vec![&env, fresh, stale, silly], 1);

    assert_eq!(client.usable_prices(), vec![&env, ONE]);
    assert_eq!(client.median_price(), ONE);
}

#[test]
fn a_feed_that_cannot_be_called_is_skipped() {
    let env = new_env();
    let a = new_feed(&env, ONE, NOW);
    let broken = env.register(BrokenFeed, ());
    let b = new_feed(&env, ONE * 3, NOW);
    let client = setup(&env, vec![&env, a, broken, b], 2);

    assert_eq!(client.usable_prices(), vec![&env, ONE, ONE * 3]);
    assert_eq!(client.median_price(), ONE * 2);
}

#[test]
fn a_median_below_quorum_fails_loudly() {
    let env = new_env();
    let fresh = new_feed(&env, ONE, NOW);
    let stale = new_feed(&env, ONE, NOW - 301);
    let client = setup(&env, vec![&env, fresh, stale], 2);

    assert_eq!(
        client.try_median_price(),
        Err(Ok(ConsumerError::QuorumNotMet))
    );
}

// ── feed-set administration ─────────────────────────────────────────────────

#[test]
fn add_feed_widens_the_set() {
    let env = new_env();
    let a = new_feed(&env, ONE, NOW);
    let b = new_feed(&env, ONE * 5, NOW);
    let client = setup(&env, vec![&env, a, b], 2);

    client.add_feed(&new_feed(&env, ONE * 3, NOW));

    assert_eq!(client.feeds().len(), 3);
    assert_eq!(client.median_price(), ONE * 3);
}

#[test]
fn add_feed_rejects_duplicates() {
    let env = new_env();
    let a = new_feed(&env, ONE, NOW);
    let b = new_feed(&env, ONE, NOW);
    let client = setup(&env, vec![&env, a.clone(), b], 2);

    assert_eq!(
        client.try_add_feed(&a),
        Err(Ok(ConsumerError::InvalidConfig))
    );
}

#[test]
fn remove_feed_below_quorum_is_rejected() {
    let env = new_env();
    let a = new_feed(&env, ONE, NOW);
    let b = new_feed(&env, ONE, NOW);
    let client = setup(&env, vec![&env, a.clone(), b], 2);

    assert_eq!(
        client.try_remove_feed(&a),
        Err(Ok(ConsumerError::InvalidConfig))
    );
    assert_eq!(client.feeds().len(), 2);
}

#[test]
fn remove_feed_rejects_an_unknown_address() {
    let env = new_env();
    let a = new_feed(&env, ONE, NOW);
    let b = new_feed(&env, ONE, NOW);
    let client = setup(&env, vec![&env, a, b], 1);

    assert_eq!(
        client.try_remove_feed(&Address::generate(&env)),
        Err(Ok(ConsumerError::FeedNotFound))
    );
}

#[test]
fn remove_feed_succeeds_above_quorum() {
    let env = new_env();
    let a = new_feed(&env, ONE, NOW);
    let b = new_feed(&env, ONE * 3, NOW);
    let client = setup(&env, vec![&env, a.clone(), b], 1);

    client.remove_feed(&a);
    assert_eq!(client.feeds().len(), 1);
    assert_eq!(client.median_price(), ONE * 3);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn add_feed_requires_admin_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let a = env.register(MockFeed, ());
    let b = env.register(MockFeed, ());
    let client = AggregatingConsumerClient::new(&env, &env.register(AggregatingConsumer, ()));

    env.mock_all_auths();
    client.initialize(
        &Address::generate(&env),
        &vec![&env, a, b.clone()],
        &agg_config(1),
    );

    env.set_auths(&[]);
    client.add_feed(&b);
}

#![cfg(test)]

extern crate std;

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env, Symbol,
};

use oracle_consumer_common::{
    testutils::MockFeed, testutils::MockFeedClient, ConsumerError, PRICE_SCALE,
};

use crate::{GuardConfig, GuardedConsumer, GuardedConsumerClient};

const ASSET: Symbol = symbol_short!("XLM");
const ONE: i128 = PRICE_SCALE; // price 1.0
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

fn guard_config(feed: &Address) -> GuardConfig {
    GuardConfig {
        feed: feed.clone(),
        asset: ASSET,
        max_age: 300,
        fallback_max_age: 3_600,
        min_price: ONE / 100,
        max_price: ONE * 100,
    }
}

fn setup(env: &Env, price: i128, timestamp: u64) -> GuardedConsumerClient<'static> {
    let feed = new_feed(env, price, timestamp);
    let id = env.register(GuardedConsumer, ());
    let client = GuardedConsumerClient::new(env, &id);
    client.initialize(&Address::generate(env), &guard_config(&feed));
    client
}

// ── initialization ──────────────────────────────────────────────────────────

#[test]
fn initialize_rejects_zero_max_age() {
    let env = new_env();
    let feed = new_feed(&env, ONE, NOW);
    let client = GuardedConsumerClient::new(&env, &env.register(GuardedConsumer, ()));

    let mut config = guard_config(&feed);
    config.max_age = 0;
    assert_eq!(
        client.try_initialize(&Address::generate(&env), &config),
        Err(Ok(ConsumerError::InvalidConfig))
    );
}

#[test]
fn initialize_rejects_short_fallback_window() {
    let env = new_env();
    let feed = new_feed(&env, ONE, NOW);
    let client = GuardedConsumerClient::new(&env, &env.register(GuardedConsumer, ()));

    let mut config = guard_config(&feed);
    config.fallback_max_age = config.max_age - 1;
    assert_eq!(
        client.try_initialize(&Address::generate(&env), &config),
        Err(Ok(ConsumerError::InvalidConfig))
    );
}

#[test]
fn initialize_rejects_inverted_bounds() {
    let env = new_env();
    let feed = new_feed(&env, ONE, NOW);
    let client = GuardedConsumerClient::new(&env, &env.register(GuardedConsumer, ()));

    let mut config = guard_config(&feed);
    config.max_price = config.min_price - 1;
    assert_eq!(
        client.try_initialize(&Address::generate(&env), &config),
        Err(Ok(ConsumerError::InvalidConfig))
    );
}

#[test]
fn double_initialize_is_rejected() {
    let env = new_env();
    let client = setup(&env, ONE, NOW);
    let feed = client.config().feed;
    assert_eq!(
        client.try_initialize(&Address::generate(&env), &guard_config(&feed)),
        Err(Ok(ConsumerError::AlreadyInitialized))
    );
}

// ── refresh ─────────────────────────────────────────────────────────────────

#[test]
fn refresh_caches_a_validated_quote() {
    let env = new_env();
    let client = setup(&env, ONE * 2, NOW);

    assert_eq!(client.refresh(), ONE * 2);
    assert_eq!(client.price(), ONE * 2);
    assert_eq!(client.cached().map(|q| q.price), Some(ONE * 2));
}

#[test]
fn refresh_rejects_a_stale_quote_and_caches_nothing() {
    let env = new_env();
    let client = setup(&env, ONE, NOW - 301);

    assert_eq!(client.try_refresh(), Err(Ok(ConsumerError::StaleData)));
    assert_eq!(client.cached(), None);
}

#[test]
fn refresh_rejects_a_future_timestamp() {
    let env = new_env();
    let client = setup(&env, ONE, NOW + 1);
    assert_eq!(client.try_refresh(), Err(Ok(ConsumerError::StaleData)));
}

#[test]
fn refresh_rejects_a_price_below_the_lower_bound() {
    let env = new_env();
    let client = setup(&env, ONE / 200, NOW);
    assert_eq!(
        client.try_refresh(),
        Err(Ok(ConsumerError::PriceOutOfBounds))
    );
}

#[test]
fn refresh_rejects_a_price_above_the_upper_bound() {
    let env = new_env();
    let client = setup(&env, ONE * 101, NOW);
    assert_eq!(
        client.try_refresh(),
        Err(Ok(ConsumerError::PriceOutOfBounds))
    );
}

#[test]
fn refresh_rejects_a_non_positive_price() {
    let env = new_env();
    let client = setup(&env, -ONE, NOW);
    assert_eq!(
        client.try_refresh(),
        Err(Ok(ConsumerError::PriceOutOfBounds))
    );
}

// ── read paths ──────────────────────────────────────────────────────────────

#[test]
fn price_without_a_refresh_has_no_cached_value() {
    let env = new_env();
    let client = setup(&env, ONE, NOW);
    assert_eq!(client.try_price(), Err(Ok(ConsumerError::NoCachedValue)));
}

#[test]
fn price_goes_stale_but_last_known_survives() {
    let env = new_env();
    let client = setup(&env, ONE, NOW);
    client.refresh();

    env.ledger().set_timestamp(NOW + 301);
    assert_eq!(client.try_price(), Err(Ok(ConsumerError::StaleData)));
    assert_eq!(client.price_or_last_known(), ONE);
}

#[test]
fn last_known_expires_after_the_fallback_window() {
    let env = new_env();
    let client = setup(&env, ONE, NOW);
    client.refresh();

    env.ledger().set_timestamp(NOW + 3_601);
    assert_eq!(
        client.try_price_or_last_known(),
        Err(Ok(ConsumerError::StaleData))
    );
}

// ── feed rotation ───────────────────────────────────────────────────────────

#[test]
fn set_feed_repoints_and_clears_the_cache() {
    let env = new_env();
    let client = setup(&env, ONE, NOW);
    client.refresh();
    assert!(client.cached().is_some());

    let replacement = new_feed(&env, ONE * 3, NOW);
    client.set_feed(&replacement);

    assert_eq!(client.cached(), None);
    assert_eq!(client.config().feed, replacement);
    assert_eq!(client.refresh(), ONE * 3);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn set_feed_requires_admin_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let feed = env.register(MockFeed, ());
    let client = GuardedConsumerClient::new(&env, &env.register(GuardedConsumer, ()));

    env.mock_all_auths();
    client.initialize(&Address::generate(&env), &guard_config(&feed));

    env.set_auths(&[]);
    client.set_feed(&feed);
}

#[test]
fn initialize_requires_admin_auth() {
    let env = Env::default();
    let feed = env.register(MockFeed, ());
    let client = GuardedConsumerClient::new(&env, &env.register(GuardedConsumer, ()));

    assert!(client
        .try_initialize(&Address::generate(&env), &guard_config(&feed))
        .is_err());
}

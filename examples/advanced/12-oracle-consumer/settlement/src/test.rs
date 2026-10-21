#![cfg(test)]

extern crate std;

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env, Symbol,
};

use oracle_consumer_common::{
    testutils::{MockFeed, MockFeedClient},
    ConsumerError, PRICE_SCALE,
};

use crate::{SettleConfig, SettlementConsumer, SettlementConsumerClient};

const ASSET: Symbol = symbol_short!("XLM");
const ONE: i128 = PRICE_SCALE;
const NOW: u64 = 1_000_000;

fn new_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    env
}

fn settle_config(feed: &Address, max_deviation_bps: i128) -> SettleConfig {
    SettleConfig {
        feed: feed.clone(),
        asset: ASSET,
        max_age: 300,
        max_deviation_bps,
        min_price: ONE / 100,
        max_price: ONE * 100,
    }
}

fn setup(
    env: &Env,
    price: i128,
    max_deviation_bps: i128,
) -> (MockFeedClient<'static>, SettlementConsumerClient<'static>) {
    let feed_id = env.register(MockFeed, ());
    let feed = MockFeedClient::new(env, &feed_id);
    feed.set_quote(&ASSET, &price, &NOW);

    let id = env.register(SettlementConsumer, ());
    let client = SettlementConsumerClient::new(env, &id);
    client.initialize(
        &Address::generate(env),
        &settle_config(&feed_id, max_deviation_bps),
    );
    (feed, client)
}

// ── settlement ──────────────────────────────────────────────────────────────

#[test]
fn credits_value_at_the_current_price() {
    let env = new_env();
    let (_feed, client) = setup(&env, ONE * 2, 500);
    let user = Address::generate(&env);

    assert_eq!(client.settle(&user, &10), 20);
    assert_eq!(client.credit_of(&user), 20);
    assert_eq!(client.last_price(), Some(ONE * 2));
}

#[test]
fn credit_accumulates_across_calls() {
    let env = new_env();
    let (_feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);

    client.settle(&user, &10);
    client.settle(&user, &5);
    assert_eq!(client.credit_of(&user), 15);
}

#[test]
fn rejects_a_non_positive_amount() {
    let env = new_env();
    let (_feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);

    assert_eq!(
        client.try_settle(&user, &0),
        Err(Ok(ConsumerError::InvalidConfig))
    );
}

#[test]
fn rejects_a_stale_quote() {
    let env = new_env();
    let (_feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);

    env.ledger().set_timestamp(NOW + 301);
    assert_eq!(
        client.try_settle(&user, &10),
        Err(Ok(ConsumerError::StaleData))
    );
}

#[test]
fn reports_overflow_instead_of_wrapping() {
    let env = new_env();
    let (_feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);

    assert_eq!(
        client.try_settle(&user, &i128::MAX),
        Err(Ok(ConsumerError::ArithmeticOverflow))
    );
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn settle_requires_the_account_to_authorize() {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let feed_id = env.register(MockFeed, ());
    let client = SettlementConsumerClient::new(&env, &env.register(SettlementConsumer, ()));

    env.mock_all_auths();
    MockFeedClient::new(&env, &feed_id).set_quote(&ASSET, &ONE, &NOW);
    client.initialize(&Address::generate(&env), &settle_config(&feed_id, 500));

    env.set_auths(&[]);
    client.settle(&Address::generate(&env), &10);
}

// ── deviation band ──────────────────────────────────────────────────────────

#[test]
fn allows_a_move_inside_the_deviation_band() {
    let env = new_env();
    let (feed, client) = setup(&env, ONE, 500); // 5%
    let user = Address::generate(&env);
    client.settle(&user, &10);

    // +4% — inside the band.
    feed.set_quote(&ASSET, &(ONE * 104 / 100), &NOW);
    client.settle(&user, &10);

    assert!(!client.is_open());
    assert_eq!(client.last_price(), Some(ONE * 104 / 100));
}

#[test]
fn rejects_a_large_move_without_crediting() {
    let env = new_env();
    let (feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);
    client.settle(&user, &10);

    // +20% — outside the band.
    feed.set_quote(&ASSET, &(ONE * 120 / 100), &NOW);
    assert_eq!(
        client.try_settle(&user, &10),
        Err(Ok(ConsumerError::DeviationTooLarge))
    );

    assert_eq!(client.credit_of(&user), 10);
    assert_eq!(client.last_price(), Some(ONE));
}

// ── circuit breaker ─────────────────────────────────────────────────────────

#[test]
fn keeper_trips_the_breaker_on_a_large_move() {
    let env = new_env();
    let (feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);
    client.settle(&user, &10);

    feed.set_quote(&ASSET, &(ONE * 120 / 100), &NOW);
    assert!(client.trip_if_deviated());
    assert!(client.is_open());
}

#[test]
fn keeper_leaves_the_breaker_closed_inside_the_band() {
    let env = new_env();
    let (feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);
    client.settle(&user, &10);

    feed.set_quote(&ASSET, &(ONE * 104 / 100), &NOW);
    assert!(!client.trip_if_deviated());
    assert!(!client.is_open());
}

#[test]
fn keeper_ignores_a_merely_stale_quote() {
    let env = new_env();
    let (_feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);
    client.settle(&user, &10);

    env.ledger().set_timestamp(NOW + 301);
    assert!(!client.trip_if_deviated());
    assert!(!client.is_open());
}

#[test]
fn keeper_trips_on_an_out_of_bounds_quote() {
    let env = new_env();
    let (feed, client) = setup(&env, ONE, 10_000);
    let user = Address::generate(&env);
    client.settle(&user, &10);

    feed.set_quote(&ASSET, &(ONE * 500), &NOW);
    assert!(client.trip_if_deviated());
    assert!(client.is_open());
}

#[test]
fn keeper_does_nothing_before_the_first_settlement() {
    let env = new_env();
    let (_feed, client) = setup(&env, ONE * 500, 500);

    assert!(!client.trip_if_deviated());
    assert!(!client.is_open());
}

#[test]
fn settlement_stays_blocked_once_the_breaker_is_open() {
    let env = new_env();
    let (feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);
    client.settle(&user, &10);

    feed.set_quote(&ASSET, &(ONE * 120 / 100), &NOW);
    client.trip_if_deviated();

    // Back inside the band — but the breaker does not reset itself.
    feed.set_quote(&ASSET, &ONE, &NOW);
    assert_eq!(
        client.try_settle(&user, &10),
        Err(Ok(ConsumerError::CircuitOpen))
    );
}

#[test]
fn reset_breaker_reanchors_the_reference_price() {
    let env = new_env();
    let (feed, client) = setup(&env, ONE, 500);
    let user = Address::generate(&env);
    client.settle(&user, &10);

    feed.set_quote(&ASSET, &(ONE * 120 / 100), &NOW);
    client.trip_if_deviated();
    assert!(client.is_open());

    client.reset_breaker(&(ONE * 120 / 100));
    assert!(!client.is_open());
    assert_eq!(client.settle(&user, &10), 12);
}

#[test]
fn reset_breaker_rejects_an_out_of_bounds_anchor() {
    let env = new_env();
    let (_feed, client) = setup(&env, ONE, 500);

    assert_eq!(
        client.try_reset_breaker(&(ONE * 500)),
        Err(Ok(ConsumerError::PriceOutOfBounds))
    );
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn reset_breaker_requires_admin_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let feed_id = env.register(MockFeed, ());
    let client = SettlementConsumerClient::new(&env, &env.register(SettlementConsumer, ()));

    env.mock_all_auths();
    client.initialize(&Address::generate(&env), &settle_config(&feed_id, 500));

    env.set_auths(&[]);
    client.reset_breaker(&ONE);
}

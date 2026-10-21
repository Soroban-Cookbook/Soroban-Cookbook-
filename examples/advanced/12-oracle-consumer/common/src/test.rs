#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::Ledger, Env};

use crate::{check_bounds, check_freshness, deviation_bps, ConsumerError, Quote, PRICE_SCALE};

const NOW: u64 = 1_000_000;

fn env_at(now: u64) -> Env {
    let env = Env::default();
    env.ledger().set_timestamp(now);
    env
}

fn quote(price: i128, timestamp: u64) -> Quote {
    Quote { price, timestamp }
}

#[test]
fn freshness_accepts_a_quote_inside_the_window() {
    let env = env_at(NOW);
    assert_eq!(check_freshness(&env, &quote(1, NOW - 300), 300), Ok(()));
}

#[test]
fn freshness_rejects_a_quote_past_the_window() {
    let env = env_at(NOW);
    assert_eq!(
        check_freshness(&env, &quote(1, NOW - 301), 300),
        Err(ConsumerError::StaleData)
    );
}

#[test]
fn freshness_rejects_a_future_timestamp() {
    let env = env_at(NOW);
    assert_eq!(
        check_freshness(&env, &quote(1, NOW + 1), 300),
        Err(ConsumerError::StaleData)
    );
}

#[test]
fn bounds_accept_the_inclusive_endpoints() {
    assert_eq!(check_bounds(10, 10, 20), Ok(()));
    assert_eq!(check_bounds(20, 10, 20), Ok(()));
}

#[test]
fn bounds_reject_outside_values_and_non_positive_prices() {
    assert_eq!(
        check_bounds(9, 10, 20),
        Err(ConsumerError::PriceOutOfBounds)
    );
    assert_eq!(
        check_bounds(21, 10, 20),
        Err(ConsumerError::PriceOutOfBounds)
    );
    assert_eq!(check_bounds(0, 0, 20), Err(ConsumerError::PriceOutOfBounds));
    assert_eq!(
        check_bounds(-5, -10, 20),
        Err(ConsumerError::PriceOutOfBounds)
    );
}

#[test]
fn deviation_is_symmetric_and_in_basis_points() {
    assert_eq!(
        deviation_bps(PRICE_SCALE, PRICE_SCALE * 110 / 100),
        Ok(1_000)
    );
    assert_eq!(
        deviation_bps(PRICE_SCALE, PRICE_SCALE * 90 / 100),
        Ok(1_000)
    );
    assert_eq!(deviation_bps(PRICE_SCALE, PRICE_SCALE), Ok(0));
}

#[test]
fn deviation_rejects_a_non_positive_reference() {
    assert_eq!(
        deviation_bps(0, PRICE_SCALE),
        Err(ConsumerError::PriceOutOfBounds)
    );
}

#[test]
fn deviation_reports_overflow_instead_of_wrapping() {
    assert_eq!(
        deviation_bps(1, i128::MAX),
        Err(ConsumerError::ArithmeticOverflow)
    );
}

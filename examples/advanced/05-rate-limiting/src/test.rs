#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

/// Default limit used by most tests: 3 calls / 1_000 units per 60-second window.
fn default_limit() -> Limit {
    Limit {
        window: 60,
        max_calls: 3,
        max_amount: 1_000,
    }
}

fn setup() -> (Env, Address, Address, RateLimiterContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RateLimiterContract);
    let client = RateLimiterContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin, &default_limit());

    (env, admin, user, client)
}

// ── time-based limits ────────────────────────────────────────────────────────

#[test]
fn test_usage_resets_after_window() {
    let (env, _admin, user, client) = setup();

    client.consume(&user, &400);
    client.consume(&user, &400);
    assert_eq!(client.remaining_calls(&user), 1);
    assert_eq!(client.remaining_amount(&user), 200);

    // A third 400 would exceed the amount cap inside this window.
    assert!(client.try_consume(&user, &400).is_err());

    // Once the window elapses the budget is whole again.
    env.ledger().with_mut(|l| l.timestamp += 61);
    assert_eq!(client.remaining_calls(&user), 3);
    assert_eq!(client.remaining_amount(&user), 1_000);
    client.consume(&user, &400);
}

#[test]
fn test_window_reset_at_tracks_first_consume() {
    let (env, _admin, user, client) = setup();

    env.ledger().with_mut(|l| l.timestamp = 5_000);
    client.consume(&user, &10);

    assert_eq!(client.window_reset_at(&user), 5_060);

    // Still the same window 30 seconds later — the anchor does not slide.
    env.ledger().with_mut(|l| l.timestamp += 30);
    client.consume(&user, &10);
    assert_eq!(client.window_reset_at(&user), 5_060);
    assert_eq!(client.usage_of(&user).calls, 2);
}

// ── amount-based limits ──────────────────────────────────────────────────────

#[test]
fn test_amount_limit_exceeded() {
    let (_env, _admin, user, client) = setup();

    client.consume(&user, &900);
    assert_eq!(
        client.try_consume(&user, &200).err().unwrap().unwrap(),
        RateLimitError::AmountLimitExceeded
    );

    // The rejected call recorded nothing.
    assert_eq!(client.usage_of(&user).amount, 900);
    assert_eq!(client.remaining_amount(&user), 100);
}

#[test]
fn test_call_limit_exceeded_before_amount_runs_out() {
    let (_env, _admin, user, client) = setup();

    for _ in 0..3 {
        client.consume(&user, &1);
    }
    assert_eq!(client.remaining_calls(&user), 0);
    assert_eq!(
        client.try_consume(&user, &1).err().unwrap().unwrap(),
        RateLimitError::CallLimitExceeded
    );
}

#[test]
fn test_non_positive_amount_rejected() {
    let (_env, _admin, user, client) = setup();

    assert_eq!(
        client.try_consume(&user, &0).err().unwrap().unwrap(),
        RateLimitError::InvalidAmount
    );
}

// ── per-user limits ──────────────────────────────────────────────────────────

#[test]
fn test_per_user_override_and_isolation() {
    let (env, _admin, user, client) = setup();
    let vip = Address::generate(&env);

    client.set_user_limit(
        &vip,
        &Limit {
            window: 60,
            max_calls: 10,
            max_amount: 5_000,
        },
    );

    // The override applies to the VIP only.
    assert_eq!(client.remaining_amount(&vip), 5_000);
    assert_eq!(client.remaining_amount(&user), 1_000);

    // Usage is tracked per user, so the VIP's spend leaves the default user alone.
    client.consume(&vip, &4_000);
    assert_eq!(client.remaining_amount(&vip), 1_000);
    assert_eq!(client.remaining_amount(&user), 1_000);

    // Clearing the override drops the VIP back to the default limit.
    client.clear_user_limit(&vip);
    assert_eq!(client.limit_of(&vip), default_limit());
}

#[test]
fn test_admin_reset_clears_usage() {
    let (_env, _admin, user, client) = setup();

    client.consume(&user, &1_000);
    assert_eq!(client.remaining_amount(&user), 0);

    client.reset(&user);
    assert_eq!(client.remaining_amount(&user), 1_000);
    assert_eq!(client.remaining_calls(&user), 3);
}

// ── configuration guards ─────────────────────────────────────────────────────

#[test]
fn test_invalid_limit_rejected() {
    let (_env, _admin, _user, client) = setup();

    let zero_window = Limit {
        window: 0,
        max_calls: 3,
        max_amount: 1_000,
    };
    assert_eq!(
        client
            .try_set_default_limit(&zero_window)
            .err()
            .unwrap()
            .unwrap(),
        RateLimitError::InvalidLimit
    );
}

#[test]
fn test_double_initialize_rejected() {
    let (env, admin, _user, client) = setup();
    let _ = env;

    assert_eq!(
        client
            .try_initialize(&admin, &default_limit())
            .err()
            .unwrap()
            .unwrap(),
        RateLimitError::AlreadyInitialized
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_default_limit_requires_admin_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RateLimiterContract);
    let client = RateLimiterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &default_limit());

    env.set_auths(&[]); // strip auths
    client.set_default_limit(&default_limit());
}

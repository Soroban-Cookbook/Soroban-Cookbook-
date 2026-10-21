#![allow(deprecated)]

use crate::{AmmOracleContract, AmmPoolContract};
use crate::{AmmOracleContractClient, AmmPoolContractClient};
use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _};
use soroban_sdk::{token, Address, Env};
use soroban_validation::test_events::EventList;

fn register_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn set_time(env: &Env, timestamp: u64) {
    env.ledger().with_mut(|l| l.timestamp = timestamp);
}

struct Fixture {
    env: Env,
    alice: Address,
    bob: Address,
    pool: AmmPoolContractClient<'static>,
    oracle: AmmOracleContractClient<'static>,
    token_a: Address,
    token_b: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let token_a = register_token(&env, &owner);
    let token_b = register_token(&env, &owner);

    let pool_id = env.register(AmmPoolContract, ());
    let pool = AmmPoolContractClient::new(&env, &pool_id);
    pool.initialize(&owner, &token_a, &token_b);

    let oracle_id = env.register(AmmOracleContract, ());
    let oracle = AmmOracleContractClient::new(&env, &oracle_id);
    oracle.initialize_oracle(&owner, &pool_id);

    set_time(&env, 100);

    Fixture {
        env,
        alice,
        bob,
        pool,
        oracle,
        token_a,
        token_b,
    }
}

fn mint(f: &Fixture, to: &Address, amount_a: i128, amount_b: i128) {
    token::StellarAssetClient::new(&f.env, &f.token_a).mint(to, &amount_a);
    token::StellarAssetClient::new(&f.env, &f.token_b).mint(to, &amount_b);
}

#[test]
fn test_oracle_current_price_matches_pool() {
    let f = setup();
    mint(&f, &f.alice, 1000, 2000);

    f.pool.deposit(&f.alice, &100, &200);
    f.oracle.update();

    assert_eq!(f.pool.current_price_a_in_b(), f.oracle.current_price());
}

#[test]
fn test_oracle_twap_changes_after_time_passes() {
    let f = setup();
    mint(&f, &f.alice, 1000, 2000);
    mint(&f, &f.bob, 1000, 2000);

    f.pool.deposit(&f.alice, &100, &200);
    f.oracle.update();

    set_time(&f.env, 1000);
    f.pool.swap(&f.bob, &f.token_a, &10, &1);
    f.oracle.update();

    assert!(f.oracle.twap_price() > 0);
}

#[test]
fn test_oracle_emits_update_event() {
    let f = setup();
    mint(&f, &f.alice, 500, 1000);

    f.pool.deposit(&f.alice, &100, &200);
    f.oracle.update();

    assert!(!EventList::new(&f.env, f.env.events().all()).is_empty());
}

#[test]
fn test_oracle_handles_sequential_updates() {
    let f = setup();
    mint(&f, &f.alice, 1000, 1000);

    f.pool.deposit(&f.alice, &100, &100);
    f.oracle.update();

    set_time(&f.env, 500);
    f.oracle.update();

    assert!(f.oracle.twap_price() > 0);
}

#[test]
fn test_oracle_update_requires_pool_price() {
    let f = setup();
    mint(&f, &f.alice, 500, 1000);

    f.pool.deposit(&f.alice, &200, &400);
    f.oracle.update();

    assert_eq!(f.oracle.current_price(), f.pool.current_price_a_in_b());
}

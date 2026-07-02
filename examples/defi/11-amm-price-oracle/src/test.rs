use soroban_validation::test_events::EventList;
#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env, Ledger, Symbol};
use crate::{AmmOracleContractClient, AmmPoolContractClient};

fn register_token(env: &Env, admin: &Address, name: &Symbol, symbol: &Symbol) -> Address {
    let token_id = env.register_contract(None, token::Contract);
    let token_client = token::Client::new(env, &token_id);
    token_client.initialize(admin, name, symbol, &8u32);
    token_id
}

fn setup() -> (
    Env,
    Address,
    Address,
    Address,
    AmmPoolContractClient<'static>,
    AmmOracleContractClient<'static>,
    Address,
    Address,
) {
    let env = Env::default();
    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let token_a_id = register_token(&env, &owner, &Symbol::short("TokenA"), &Symbol::short("TKA"));
    let token_b_id = register_token(&env, &owner, &Symbol::short("TokenB"), &Symbol::short("TKB"));

    let pool_id = env.register_contract(None, AmmPoolContract);
    let pool_client = AmmPoolContractClient::new(&env, &pool_id);
    pool_client.initialize(&owner, &token_a_id, &token_b_id);

    let oracle_id = env.register_contract(None, AmmOracleContract);
    let oracle_client = AmmOracleContractClient::new(&env, &oracle_id);
    oracle_client.initialize(&owner, &pool_id);

    (env, owner, alice, bob, pool_client, oracle_client, token_a_id, token_b_id)
}

#[test]
fn test_oracle_current_price_matches_pool() {
    let (env, _owner, alice, _bob, pool, oracle, token_a_id, token_b_id) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);

    env.mock_all_auths();
    token_a.mint(&alice, &1000i128);
    token_b.mint(&alice, &2000i128);

    pool.deposit(&alice, &100i128, &200i128);
    oracle.update();

    let pool_price = pool.current_price_a_in_b();
    let oracle_price = oracle.current_price();
    assert_eq!(pool_price, oracle_price);
}

#[test]
fn test_oracle_twap_changes_after_time_passes() {
    let (env, _owner, alice, bob, pool, oracle, token_a_id, token_b_id) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);

    env.mock_all_auths();
    token_a.mint(&alice, &1000i128);
    token_b.mint(&alice, &2000i128);
    token_a.mint(&bob, &1000i128);
    token_b.mint(&bob, &2000i128);

    pool.deposit(&alice, &100i128, &200i128);
    oracle.update();

    env.ledger().set(Ledger::timestamp(1000));
    pool.swap(&bob, &token_a_id, &10i128, &1i128);
    oracle.update();

    let twap = oracle.twap_price();
    assert!(twap > 0);
}

#[test]
fn test_oracle_emits_update_event() {
    let (env, _owner, alice, _bob, pool, oracle, token_a_id, token_b_id) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);

    env.mock_all_auths();
    token_a.mint(&alice, &500i128);
    token_b.mint(&alice, &1000i128);

    pool.deposit(&alice, &100i128, &200i128);
    oracle.update();

    assert!(!EventList::new(&env, env.events().all()).is_empty());
}

#[test]
fn test_oracle_handles_sequential_updates() {
    let (env, _owner, alice, _bob, pool, oracle, token_a_id, token_b_id) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);

    env.mock_all_auths();
    token_a.mint(&alice, &1000i128);
    token_b.mint(&alice, &1000i128);

    pool.deposit(&alice, &100i128, &100i128);
    oracle.update();

    env.ledger().set(Ledger::timestamp(500));
    oracle.update();

    let twap = oracle.twap_price();
    assert!(twap > 0);
}

#[test]
fn test_oracle_update_requires_pool_price() {
    let (env, _owner, alice, _bob, pool, oracle, token_a_id, token_b_id) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);

    env.mock_all_auths();
    token_a.mint(&alice, &500i128);
    token_b.mint(&alice, &1000i128);

    pool.deposit(&alice, &200i128, &400i128);
    oracle.update();

    assert_eq!(oracle.current_price(), pool.current_price_a_in_b());
}

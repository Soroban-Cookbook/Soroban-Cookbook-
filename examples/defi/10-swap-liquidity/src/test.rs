use soroban_validation::test_events::EventList;
#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env, Symbol};
use crate::SwapLiquidityContractClient;

fn register_token(env: &Env, admin: &Address, name: &Symbol, symbol: &Symbol) -> Address {
    let token_id = env.register_contract(None, token::Contract);
    let token_client = token::Client::new(env, &token_id);
    token_client.initialize(admin, name, symbol, &8u32);
    token_id
}

fn setup() -> (Env, Address, Address, Address, Address, Address, Address, SwapLiquidityContractClient<'static>) {
    let env = Env::default();
    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let token_a_id = register_token(&env, &owner, &Symbol::short("TokenA"), &Symbol::short("TKA"));
    let token_b_id = register_token(&env, &owner, &Symbol::short("TokenB"), &Symbol::short("TKB"));

    let contract_id = env.register_contract(None, SwapLiquidityContract);
    let lp_token_id = register_token(&env, &contract_id, &Symbol::short("Liquidity"), &Symbol::short("LP"));

    let client = SwapLiquidityContractClient::new(&env, &contract_id);
    client.initialize(&owner, &token_a_id, &token_b_id, &lp_token_id);

    (env, owner, alice, bob, token_a_id, token_b_id, lp_token_id, client)
}

#[test]
fn test_add_liquidity_initializes_pool() {
    let (env, _owner, alice, _bob, token_a_id, token_b_id, _lp_token_id, client) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);

    env.mock_all_auths();
    token_a.mint(&alice, &1000i128);
    token_b.mint(&alice, &1000i128);

    client.add_liquidity(&alice, &200i128, &300i128);

    assert_eq!(client.get_reserves().0, 200i128);
    assert_eq!(client.get_reserves().1, 300i128);
    assert!(client.get_total_shares() > 0);
}

#[test]
fn test_add_liquidity_mints_lp_shares() {
    let (env, _owner, alice, _bob, _token_a_id, _token_b_id, lp_token_id, client) = setup();
    let lp_token = token::Client::new(&env, &lp_token_id);

    env.mock_all_auths();
    assert_eq!(lp_token.balance(&alice), 0i128);
    let token_a = token::Client::new(&env, &_token_a_id);
    let token_b = token::Client::new(&env, &_token_b_id);
    token_a.mint(&alice, &200i128);
    token_b.mint(&alice, &200i128);
    client.add_liquidity(&alice, &200i128, &200i128);
    assert!(lp_token.balance(&alice) > 0);
}

#[test]
fn test_remove_liquidity_returns_tokens() {
    let (env, _owner, alice, _bob, token_a_id, token_b_id, lp_token_id, client) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);
    let lp_token = token::Client::new(&env, &lp_token_id);

    env.mock_all_auths();
    token_a.mint(&alice, &1000i128);
    token_b.mint(&alice, &1000i128);

    client.add_liquidity(&alice, &200i128, &200i128);
    let shares = lp_token.balance(&alice);
    assert!(shares > 0);

    client.remove_liquidity(&alice, &shares);
    assert_eq!(token_a.balance(&alice), 1000i128);
    assert_eq!(token_b.balance(&alice), 1000i128);
}

#[test]
fn test_pool_share_reflects_lp_holdings() {
    let (env, _owner, alice, _bob, token_a_id, token_b_id, _lp_token_id, client) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);

    env.mock_all_auths();
    token_a.mint(&alice, &500i128);
    token_b.mint(&alice, &500i128);

    client.add_liquidity(&alice, &100i128, &100i128);
    let share = client.pool_share(&alice);
    assert!(share > 0);
}

#[test]
#[should_panic(expected = "liquidity amounts must be positive")]
fn test_add_liquidity_rejects_zero_amounts() {
    let (env, _owner, alice, _bob, _token_a_id, _token_b_id, _lp_token_id, client) = setup();
    env.mock_all_auths();
    client.add_liquidity(&alice, &0i128, &100i128);
}

#[test]
#[should_panic(expected = "not enough pool shares")]
fn test_remove_liquidity_rejects_too_many_shares() {
    let (env, _owner, alice, _bob, _token_a_id, _token_b_id, _lp_token_id, client) = setup();
    env.mock_all_auths();
    client.remove_liquidity(&alice, &1i128);
}

#[test]
fn test_liquidity_events_are_emitted() {
    let (env, _owner, alice, _bob, token_a_id, token_b_id, _lp_token_id, client) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);

    env.mock_all_auths();
    token_a.mint(&alice, &200i128);
    token_b.mint(&alice, &200i128);
    client.add_liquidity(&alice, &200i128, &200i128);
    assert!(!EventList::new(&env, env.events().all()).is_empty());
}

#[test]
fn test_liquidity_shares_scale_with_pool_size() {
    let (env, _owner, alice, bob, token_a_id, token_b_id, _lp_token_id, client) = setup();
    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);

    env.mock_all_auths();
    token_a.mint(&alice, &500i128);
    token_b.mint(&alice, &500i128);
    token_a.mint(&bob, &500i128);
    token_b.mint(&bob, &500i128);

    client.add_liquidity(&alice, &100i128, &100i128);
    client.add_liquidity(&bob, &200i128, &200i128);
    let bob_share = client.pool_share(&bob);
    assert!(bob_share > 0);
}

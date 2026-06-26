use soroban_validation::test_events::EventList;
#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env, Symbol};
use crate::SimpleSwapContractClient;

fn register_token(env: &Env, admin: &Address, name: &Symbol, symbol: &Symbol) -> Address {
    let token_id = env.register_contract(None, token::Contract);
    let token_client = token::Client::new(env, &token_id);
    token_client.initialize(admin, name, symbol, &8u32);
    token_id
}

fn setup() -> (Env, Address, Address, Address, SimpleSwapContractClient<'static>) {
    let env = Env::default();
    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let token_a_id = register_token(&env, &owner, &Symbol::short("TokenA"), &Symbol::short("TKA"));
    let token_b_id = register_token(&env, &owner, &Symbol::short("TokenB"), &Symbol::short("TKB"));

    let swap_contract_id = env.register_contract(None, SimpleSwapContract);
    let swap_client = SimpleSwapContractClient::new(&env, &swap_contract_id);
    swap_client.initialize(&owner, &token_a_id, &token_b_id, &2i128, &1i128);

    (env, owner, alice, bob, swap_client)
}

#[test]
fn test_initialize_and_pair_management() {
    let (env, owner, _alice, _bob, swap_client) = setup();
    let pair = swap_client.get_pair();
    assert_eq!(swap_client.get_rate(), (2i128, 1i128));

    let token_c = Address::generate(&env);
    swap_client.update_pair(&owner, &pair.0, &token_c, &1i128, &1i128);
    let new_pair = swap_client.get_pair();
    assert_eq!(new_pair.1, token_c);
}

#[test]
fn test_swap_exact_rate() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let token_a_id = swap_client.get_pair().0;
    let token_b_id = swap_client.get_pair().1;

    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);
    env.mock_all_auths();
    token_a.mint(&alice, &1000i128);
    token_b.mint(&env.current_contract_address(), &5000i128);

    swap_client.swap(&alice, &token_a_id, &100i128, &200i128, &bob);

    assert_eq!(token_a.balance(&alice), 900i128);
    assert_eq!(token_b.balance(&bob), 200i128);
}

#[test]
fn test_swap_rejects_slippage() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let token_a_id = swap_client.get_pair().0;
    let token_b_id = swap_client.get_pair().1;

    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);
    env.mock_all_auths();
    token_a.mint(&alice, &1000i128);
    token_b.mint(&env.current_contract_address(), &5000i128);

    let result = std::panic::catch_unwind(|| {
        swap_client.swap(&alice, &token_a_id, &100i128, &201i128, &bob);
    });
    assert!(result.is_err());
}

#[test]
fn test_quote_function() {
    let (_env, _owner, _alice, _bob, swap_client) = setup();
    assert_eq!(swap_client.quote(&swap_client.get_pair().0, &50i128), 100i128);
    assert_eq!(swap_client.quote(&swap_client.get_pair().1, &100i128), 50i128);
}

#[test]
#[should_panic(expected = "unsupported sell token")]
fn test_quote_rejects_unknown_token() {
    let (env, _owner, _alice, _bob, swap_client) = setup();
    let unknown = Address::generate(&env);
    swap_client.quote(&unknown, &10i128);
}

#[test]
#[should_panic(expected = "rate must be positive")]
fn test_initialize_rejects_zero_rate() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let swap_contract_id = env.register_contract(None, SimpleSwapContract);
    let swap_client = SimpleSwapContractClient::new(&env, &swap_contract_id);
    swap_client.initialize(&owner, &token_a, &token_b, &0i128, &1i128);
}

#[test]
#[should_panic(expected = "slippage exceeded")]
fn test_swap_rejects_low_minimum() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let token_a_id = swap_client.get_pair().0;
    let token_b_id = swap_client.get_pair().1;

    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);
    env.mock_all_auths();
    token_a.mint(&alice, &1000i128);
    token_b.mint(&env.current_contract_address(), &1000i128);

    swap_client.swap(&alice, &token_a_id, &100i128, &201i128, &bob);
}

#[test]
fn test_swap_reverse_direction() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let token_a_id = swap_client.get_pair().0;
    let token_b_id = swap_client.get_pair().1;

    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);
    env.mock_all_auths();
    token_b.mint(&alice, &1000i128);
    token_a.mint(&env.current_contract_address(), &1000i128);

    swap_client.swap(&alice, &token_b_id, &200i128, &99i128, &bob);

    assert_eq!(token_b.balance(&alice), 800i128);
    assert_eq!(token_a.balance(&bob), 100i128);
}

#[test]
fn test_swap_emits_event() {
    let (env, _owner, alice, _bob, swap_client) = setup();
    let token_a_id = swap_client.get_pair().0;
    let token_b_id = swap_client.get_pair().1;

    let token_a = token::Client::new(&env, &token_a_id);
    let token_b = token::Client::new(&env, &token_b_id);
    env.mock_all_auths();
    token_a.mint(&alice, &1000i128);
    token_b.mint(&env.current_contract_address(), &5000i128);

    swap_client.swap(&alice, &token_a_id, &100i128, &200i128, &alice);
    let events = EventList::new(&env, env.events().all());
    assert!(!events.is_empty());
}

#[test]
#[should_panic(expected = "unsupported sell token")]
fn test_swap_rejects_unknown_sell_token() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let unknown = Address::generate(&env);
    env.mock_all_auths();
    swap_client.swap(&alice, &unknown, &100i128, &1i128, &bob);
}

#[test]
#[should_panic(expected = "contract not initialized")]
fn test_contract_requires_initialization() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let swap_contract_id = env.register_contract(None, SimpleSwapContract);
    let swap_client = SimpleSwapContractClient::new(&env, &swap_contract_id);
    swap_client.get_pair();
}

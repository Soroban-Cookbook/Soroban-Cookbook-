#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, IntoVal,
};
use soroban_validation::test_events::EventList;

fn register_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup() -> (
    Env,
    Address,
    Address,
    Address,
    SimpleSwapContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let token_a_id = register_token(&env, &owner);
    let token_b_id = register_token(&env, &owner);

    let swap_contract_id = env.register(SimpleSwapContract, ());
    let swap_client = SimpleSwapContractClient::new(&env, &swap_contract_id);
    swap_client.initialize(&owner, &token_a_id, &token_b_id, &2i128, &1i128);

    (env, owner, alice, bob, swap_client)
}

#[test]
fn test_initialize_and_pair_management() {
    let (env, _owner, _alice, _bob, swap_client) = setup();
    let pair = swap_client.get_pair();
    assert_eq!(swap_client.get_rate(), (2i128, 1i128));

    let token_c = Address::generate(&env);
    swap_client.update_pair(&pair.0, &token_c, &1i128, &1i128);
    let new_pair = swap_client.get_pair();
    assert_eq!(new_pair.1, token_c);
}

#[test]
#[should_panic(expected = "contract already initialized")]
fn test_initialize_rejects_second_call() {
    let (env, owner, _alice, _bob, swap_client) = setup();
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    swap_client.initialize(&owner, &token_a, &token_b, &3i128, &2i128);
}

#[test]
fn test_initialize_requires_owner_auth() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let contract_id = env.register(SimpleSwapContract, ());
    let swap_client = SimpleSwapContractClient::new(&env, &contract_id);

    env.mock_auths(&[MockAuth {
        address: &attacker,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "initialize",
            args: (&owner, &token_a, &token_b, &2i128, &1i128).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    assert!(swap_client
        .try_initialize(&owner, &token_a, &token_b, &2i128, &1i128)
        .is_err());
}

#[test]
fn test_swap_exact_rate() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let (token_a_id, token_b_id) = swap_client.get_pair();
    let token_a = TokenClient::new(&env, &token_a_id);
    let token_b = TokenClient::new(&env, &token_b_id);
    mint(&env, &token_a_id, &alice, 1000);
    mint(&env, &token_b_id, &swap_client.address, 5000);

    swap_client.swap(&alice, &token_a_id, &100i128, &200i128, &bob);

    assert_eq!(token_a.balance(&alice), 900i128);
    assert_eq!(token_b.balance(&bob), 200i128);
}

/// Regression test for the `transfer_from` bug: the trader never approves the
/// swap contract as a spender, yet a full A -> B -> A round trip must succeed.
/// `mock_all_auths` only mocks signatures; token allowances are still enforced,
/// so the old `transfer_from` call fails here with "not enough allowance".
#[test]
fn test_swap_round_trip_without_approval() {
    let (env, _owner, alice, _bob, swap_client) = setup();
    let (token_a_id, token_b_id) = swap_client.get_pair();
    let token_a = TokenClient::new(&env, &token_a_id);
    let token_b = TokenClient::new(&env, &token_b_id);
    mint(&env, &token_a_id, &alice, 1000);
    mint(&env, &token_b_id, &swap_client.address, 5000);

    // No approve() is ever made for the swap contract.
    assert_eq!(token_a.allowance(&alice, &swap_client.address), 0);
    assert_eq!(token_b.allowance(&alice, &swap_client.address), 0);

    // A -> B at 2:1.
    swap_client.swap(&alice, &token_a_id, &100i128, &200i128, &alice);
    assert_eq!(token_a.balance(&alice), 900);
    assert_eq!(token_a.balance(&swap_client.address), 100);
    assert_eq!(token_b.balance(&alice), 200);
    assert_eq!(token_b.balance(&swap_client.address), 4800);

    // B -> A, back to the starting balances.
    swap_client.swap(&alice, &token_b_id, &200i128, &100i128, &alice);
    assert_eq!(token_a.balance(&alice), 1000);
    assert_eq!(token_b.balance(&alice), 0);
    assert_eq!(token_a.balance(&swap_client.address), 0);
    assert_eq!(token_b.balance(&swap_client.address), 5000);

    // Still no allowance was ever needed or created.
    assert_eq!(token_a.allowance(&alice, &swap_client.address), 0);
    assert_eq!(token_b.allowance(&alice, &swap_client.address), 0);
}

/// The trader's single authorization (the swap call plus a token `transfer`
/// from the trader to the swap contract) is enough. This is exactly what a
/// real client signs, with no separate approval step.
#[test]
fn test_swap_needs_only_trader_transfer_auth() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let (token_a_id, token_b_id) = swap_client.get_pair();
    let token_a = TokenClient::new(&env, &token_a_id);
    let token_b = TokenClient::new(&env, &token_b_id);
    mint(&env, &token_a_id, &alice, 1000);
    mint(&env, &token_b_id, &swap_client.address, 5000);

    env.mock_auths(&[MockAuth {
        address: &alice,
        invoke: &MockAuthInvoke {
            contract: &swap_client.address,
            fn_name: "swap",
            args: (&alice, &token_a_id, &100i128, &200i128, &bob).into_val(&env),
            sub_invokes: &[MockAuthInvoke {
                contract: &token_a_id,
                fn_name: "transfer",
                args: (&alice, &swap_client.address, &100i128).into_val(&env),
                sub_invokes: &[],
            }],
        },
    }]);

    swap_client.swap(&alice, &token_a_id, &100i128, &200i128, &bob);

    assert_eq!(token_a.balance(&alice), 900);
    assert_eq!(token_b.balance(&bob), 200);
}

#[test]
fn test_swap_rejects_slippage() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let (token_a_id, token_b_id) = swap_client.get_pair();
    mint(&env, &token_a_id, &alice, 1000);
    mint(&env, &token_b_id, &swap_client.address, 5000);

    assert!(swap_client
        .try_swap(&alice, &token_a_id, &100i128, &201i128, &bob)
        .is_err());
}

#[test]
fn test_quote_function() {
    let (_env, _owner, _alice, _bob, swap_client) = setup();
    assert_eq!(
        swap_client.quote(&swap_client.get_pair().0, &50i128),
        100i128
    );
    assert_eq!(
        swap_client.quote(&swap_client.get_pair().1, &100i128),
        50i128
    );
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
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let swap_contract_id = env.register(SimpleSwapContract, ());
    let swap_client = SimpleSwapContractClient::new(&env, &swap_contract_id);
    swap_client.initialize(&owner, &token_a, &token_b, &0i128, &1i128);
}

#[test]
#[should_panic(expected = "slippage exceeded")]
fn test_swap_rejects_low_minimum() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let (token_a_id, token_b_id) = swap_client.get_pair();
    mint(&env, &token_a_id, &alice, 1000);
    mint(&env, &token_b_id, &swap_client.address, 1000);

    swap_client.swap(&alice, &token_a_id, &100i128, &201i128, &bob);
}

#[test]
fn test_swap_reverse_direction() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let (token_a_id, token_b_id) = swap_client.get_pair();
    let token_a = TokenClient::new(&env, &token_a_id);
    let token_b = TokenClient::new(&env, &token_b_id);
    mint(&env, &token_b_id, &alice, 1000);
    mint(&env, &token_a_id, &swap_client.address, 1000);

    swap_client.swap(&alice, &token_b_id, &200i128, &99i128, &bob);

    assert_eq!(token_b.balance(&alice), 800i128);
    assert_eq!(token_a.balance(&bob), 100i128);
}

#[test]
fn test_swap_emits_event() {
    let (env, _owner, alice, _bob, swap_client) = setup();
    let (token_a_id, token_b_id) = swap_client.get_pair();
    mint(&env, &token_a_id, &alice, 1000);
    mint(&env, &token_b_id, &swap_client.address, 5000);

    swap_client.swap(&alice, &token_a_id, &100i128, &200i128, &alice);
    let events = EventList::new(&env, env.events().all());
    assert!(!events.is_empty());
}

#[test]
#[should_panic(expected = "unsupported sell token")]
fn test_swap_rejects_unknown_sell_token() {
    let (env, _owner, alice, bob, swap_client) = setup();
    let unknown = Address::generate(&env);
    swap_client.swap(&alice, &unknown, &100i128, &1i128, &bob);
}

#[test]
#[should_panic(expected = "token pair not configured")]
fn test_contract_requires_initialization() {
    let env = Env::default();
    let swap_contract_id = env.register(SimpleSwapContract, ());
    let swap_client = SimpleSwapContractClient::new(&env, &swap_contract_id);
    swap_client.get_pair();
}

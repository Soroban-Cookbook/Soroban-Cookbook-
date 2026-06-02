#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Events as _}, Address, Env, Symbol, String, TryFromVal};

struct Fixture {
    env: Env,
    token: Sep41TokenClient<'static>,
    admin: Address,
    alice: Address,
    bob: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_contract(None, Sep41Token);
    let token = Sep41TokenClient::new(&env, &token_id);
    let name = String::from_str(&env, "Soroban USD");
    let symbol = Symbol::new(&env, "SUSD");
    token.initialize(&admin, &name, &symbol, &2u32, &1_000_000i128).unwrap();

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    Fixture {
        env,
        token,
        admin,
        alice,
        bob,
    }
}

#[test]
fn initialize_sets_metadata_and_initial_supply() {
    let f = setup();

    assert_eq!(f.token.name().unwrap(), String::from_str(&f.env, "Soroban USD"));
    assert_eq!(f.token.symbol().unwrap(), Symbol::new(&f.env, "SUSD"));
    assert_eq!(f.token.decimals().unwrap(), 2);
    assert_eq!(f.token.admin().unwrap(), f.admin);
    assert_eq!(f.token.total_supply().unwrap(), 1_000_000);
    assert_eq!(f.token.balance(&f.admin), 1_000_000);
}

#[test]
fn transfer_moves_tokens_and_emits_transfer_event() {
    let f = setup();

    f.token.transfer(&f.admin, &f.alice, &500_000).unwrap();

    assert_eq!(f.token.balance(&f.admin), 500_000);
    assert_eq!(f.token.balance(&f.alice), 500_000);

    let events = f.env.events().all();
    assert_eq!(events.len(), 1);

    let (_id, topics, data) = events.get(0).unwrap();
    assert_eq!(topics.len(), 4);
    let namespace: Symbol = Symbol::try_from_val(&f.env, &topics.get(0).unwrap()).unwrap();
    let action: Symbol = Symbol::try_from_val(&f.env, &topics.get(1).unwrap()).unwrap();
    let from: Address = Address::try_from_val(&f.env, &topics.get(2).unwrap()).unwrap();
    let to: Address = Address::try_from_val(&f.env, &topics.get(3).unwrap()).unwrap();
    let payload: TransferEventData = TransferEventData::try_from_val(&f.env, &data).unwrap();

    assert_eq!(namespace, EVENT_NAMESPACE);
    assert_eq!(action, EVENT_TRANSFER);
    assert_eq!(from, f.admin);
    assert_eq!(to, f.alice);
    assert_eq!(payload.amount, 500_000);
}

#[test]
fn transfer_rejects_insufficient_balance() {
    let f = setup();

    assert_eq!(
        f.token.try_transfer(&f.alice, &f.bob, &1),
        Err(Ok(TokenError::InsufficientBalance))
    );
}

#[test]
fn transfer_rejects_zero_and_negative_amounts() {
    let f = setup();

    assert_eq!(
        f.token.try_transfer(&f.admin, &f.alice, &0),
        Err(Ok(TokenError::InvalidAmount))
    );
    assert_eq!(
        f.token.try_transfer(&f.admin, &f.alice, &-1),
        Err(Ok(TokenError::InvalidAmount))
    );
}

#[test]
fn approve_and_transfer_from_updates_allowance() {
    let f = setup();

    f.token.approve(&f.admin, &f.alice, &300_000).unwrap();
    assert_eq!(f.token.allowance(&f.admin, &f.alice), 300_000);

    f.token.transfer_from(&f.alice, &f.admin, &f.bob, &250_000).unwrap();
    assert_eq!(f.token.balance(&f.admin), 750_000);
    assert_eq!(f.token.balance(&f.bob), 250_000);
    assert_eq!(f.token.allowance(&f.admin, &f.alice), 50_000);
}

#[test]
fn transfer_from_rejects_over_allowance() {
    let f = setup();

    f.token.approve(&f.admin, &f.alice, &100).unwrap();
    assert_eq!(
        f.token.try_transfer_from(&f.alice, &f.admin, &f.bob, &101),
        Err(Ok(TokenError::AllowanceExceeded))
    );
}

#[test]
fn mint_and_burn_update_supply_and_balances() {
    let f = setup();

    f.token.mint(&f.admin, &f.alice, &250_000).unwrap();
    assert_eq!(f.token.balance(&f.alice), 250_000);
    assert_eq!(f.token.total_supply().unwrap(), 1_250_000);

    f.token.burn(&f.alice, &50_000).unwrap();
    assert_eq!(f.token.balance(&f.alice), 200_000);
    assert_eq!(f.token.total_supply().unwrap(), 1_200_000);
}

#[test]
fn mint_rejects_non_admin() {
    let f = setup();

    assert_eq!(
        f.token.try_mint(&f.alice, &f.bob, &1),
        Err(Ok(TokenError::Unauthorized))
    );
}

#[test]
fn double_initialize_is_rejected() {
    let f = setup();

    let name = String::from_str(&f.env, "Soroban USD");
    let symbol = Symbol::new(&f.env, "SUSD");
    assert_eq!(
        f.token.try_initialize(&f.admin, &name, &symbol, &2u32, &1_000_000i128),
        Err(Ok(TokenError::AlreadyInitialized))
    );
}

#[test]
fn uninitialized_contract_returns_not_initialized_for_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let token_id = env.register_contract(None, Sep41Token);
    let token = Sep41TokenClient::new(&env, &token_id);

    assert_eq!(token.try_total_supply(), Err(Ok(TokenError::NotInitialized)));
    assert_eq!(token.try_name(), Err(Ok(TokenError::NotInitialized)));
    assert_eq!(token.try_symbol(), Err(Ok(TokenError::NotInitialized)));
    assert_eq!(token.try_decimals(), Err(Ok(TokenError::NotInitialized)));
    assert_eq!(token.try_admin(), Err(Ok(TokenError::NotInitialized)));
}

#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_validation::test_events::EventList;
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Env, Symbol, String, TryFromVal,
};

struct Fixture {
    env: Env,
    token: AuthorizedTokenClient<'static>,
    admin: Address,
    alice: Address,
    bob: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_contract(None, AuthorizedToken);
    let token = AuthorizedTokenClient::new(&env, &token_id);
    let name = String::from_str(&env, "Compliance Token");
    let symbol = Symbol::new(&env, "CMP");
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

    assert_eq!(f.token.name().unwrap(), String::from_str(&f.env, "Compliance Token"));
    assert_eq!(f.token.symbol().unwrap(), Symbol::new(&f.env, "CMP"));
    assert_eq!(f.token.decimals().unwrap(), 2);
    assert_eq!(f.token.admin().unwrap(), f.admin);
    assert_eq!(f.token.total_supply().unwrap(), 1_000_000);
    assert_eq!(f.token.balance(&f.admin), 1_000_000);
}

#[test]
fn accounts_are_authorized_by_default() {
    let f = setup();
    let carol = Address::generate(&f.env);

    assert!(f.token.authorized(&f.alice).unwrap());
    assert!(f.token.authorized(&carol).unwrap());
}

#[test]
fn admin_can_freeze_and_unfreeze() {
    let f = setup();

    f.token.set_authorized(&f.admin, &f.alice, &false).unwrap();
    assert!(!f.token.authorized(&f.alice).unwrap());

    f.token.set_authorized(&f.admin, &f.alice, &true).unwrap();
    assert!(f.token.authorized(&f.alice).unwrap());
}

#[test]
fn set_authorized_emits_authorize_event() {
    let f = setup();

    f.token.set_authorized(&f.admin, &f.alice, &false).unwrap();

    let events = EventList::new(&f.env, f.env.events().all());
    let (_id, topics, data) = events.last().unwrap();

    assert_eq!(topics.len(), 4);
    let namespace: Symbol = Symbol::try_from_val(&f.env, &topics.get(0).unwrap()).unwrap();
    let action: Symbol = Symbol::try_from_val(&f.env, &topics.get(1).unwrap()).unwrap();
    let admin: Address = Address::try_from_val(&f.env, &topics.get(2).unwrap()).unwrap();
    let target: Address = Address::try_from_val(&f.env, &topics.get(3).unwrap()).unwrap();
    let payload: AuthorizeEventData = AuthorizeEventData::try_from_val(&f.env, &data).unwrap();

    assert_eq!(namespace, EVENT_NAMESPACE);
    assert_eq!(action, EVENT_AUTHORIZE);
    assert_eq!(admin, f.admin);
    assert_eq!(target, f.alice);
    assert!(!payload.authorized);
}

#[test]
fn set_authorized_rejects_non_admin() {
    let f = setup();

    assert_eq!(
        f.token.try_set_authorized(&f.alice, &f.bob, &false),
        Err(Ok(TokenError::Unauthorized))
    );
}

#[test]
fn transfer_rejects_frozen_sender() {
    let f = setup();

    f.token.set_authorized(&f.admin, &f.admin, &false).unwrap();

    assert_eq!(
        f.token.try_transfer(&f.admin, &f.alice, &100),
        Err(Ok(TokenError::NotAuthorized))
    );
    // Balances are untouched when the transfer is rejected.
    assert_eq!(f.token.balance(&f.admin), 1_000_000);
    assert_eq!(f.token.balance(&f.alice), 0);
}

#[test]
fn transfer_works_again_after_unfreeze() {
    let f = setup();

    f.token.set_authorized(&f.admin, &f.admin, &false).unwrap();
    assert_eq!(
        f.token.try_transfer(&f.admin, &f.alice, &100),
        Err(Ok(TokenError::NotAuthorized))
    );

    f.token.set_authorized(&f.admin, &f.admin, &true).unwrap();
    f.token.transfer(&f.admin, &f.alice, &100).unwrap();
    assert_eq!(f.token.balance(&f.alice), 100);
}

#[test]
fn approve_rejects_frozen_owner() {
    let f = setup();

    f.token.set_authorized(&f.admin, &f.admin, &false).unwrap();

    assert_eq!(
        f.token.try_approve(&f.admin, &f.alice, &100),
        Err(Ok(TokenError::NotAuthorized))
    );
}

#[test]
fn transfer_from_rejects_frozen_spender() {
    let f = setup();

    f.token.transfer(&f.admin, &f.alice, &500_000).unwrap();
    f.token.approve(&f.alice, &f.bob, &300_000).unwrap();
    f.token.set_authorized(&f.admin, &f.bob, &false).unwrap();

    assert_eq!(
        f.token.try_transfer_from(&f.bob, &f.alice, &f.admin, &100),
        Err(Ok(TokenError::NotAuthorized))
    );
}

#[test]
fn transfer_from_rejects_frozen_owner() {
    let f = setup();

    f.token.transfer(&f.admin, &f.alice, &500_000).unwrap();
    f.token.approve(&f.alice, &f.bob, &300_000).unwrap();
    // Freeze the owner, not the spender.
    f.token.set_authorized(&f.admin, &f.alice, &false).unwrap();

    assert_eq!(
        f.token.try_transfer_from(&f.bob, &f.alice, &f.admin, &100),
        Err(Ok(TokenError::NotAuthorized))
    );
}

#[test]
fn transfer_from_works_when_both_parties_are_authorized() {
    let f = setup();

    f.token.transfer(&f.admin, &f.alice, &500_000).unwrap();
    f.token.approve(&f.alice, &f.bob, &300_000).unwrap();

    f.token.transfer_from(&f.bob, &f.alice, &f.admin, &200_000).unwrap();
    assert_eq!(f.token.balance(&f.alice), 300_000);
    assert_eq!(f.token.balance(&f.bob), 0);
    assert_eq!(f.token.allowance(&f.alice, &f.bob), 100_000);
}

#[test]
fn burn_rejects_frozen_owner() {
    let f = setup();

    f.token.set_authorized(&f.admin, &f.admin, &false).unwrap();

    assert_eq!(
        f.token.try_burn(&f.admin, &100),
        Err(Ok(TokenError::NotAuthorized))
    );
}

#[test]
fn freeze_does_not_change_balances_or_supply() {
    let f = setup();

    f.token.transfer(&f.admin, &f.alice, &500_000).unwrap();
    f.token.set_authorized(&f.admin, &f.alice, &false).unwrap();

    assert_eq!(f.token.balance(&f.alice), 500_000);
    assert_eq!(f.token.total_supply().unwrap(), 1_000_000);
}

#[test]
fn freeze_does_not_block_authorized_query() {
    let f = setup();

    f.token.set_authorized(&f.admin, &f.alice, &false).unwrap();
    assert!(f.token.authorized(&f.alice).is_ok());
}

#[test]
fn double_initialize_is_rejected() {
    let f = setup();

    let name = String::from_str(&f.env, "Compliance Token");
    let symbol = Symbol::new(&f.env, "CMP");
    assert_eq!(
        f.token.try_initialize(&f.admin, &name, &symbol, &2u32, &1_000_000i128),
        Err(Ok(TokenError::AlreadyInitialized))
    );
}

#[test]
fn uninitialized_contract_rejects_set_authorized() {
    let env = Env::default();
    env.mock_all_auths();

    let token_id = env.register_contract(None, AuthorizedToken);
    let token = AuthorizedTokenClient::new(&env, &token_id);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    assert_eq!(
        token.try_set_authorized(&admin, &target, &false),
        Err(Ok(TokenError::NotInitialized))
    );
}

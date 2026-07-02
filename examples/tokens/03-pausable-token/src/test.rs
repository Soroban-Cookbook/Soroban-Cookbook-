#![cfg(test)]

use super::*;
use soroban_validation::test_events::EventList;
use soroban_sdk::{testutils::{Address as _, Events as _}, Address, Env, Symbol, String, TryFromVal};

struct Fixture {
    env: Env,
    token: PausableTokenClient<'static>,
    admin: Address,
    alice: Address,
    bob: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_contract(None, PausableToken);
    let token = PausableTokenClient::new(&env, &token_id);
    let name = String::from_str(&env, "Pausable USD");
    let symbol = Symbol::new(&env, "PUSD");
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
fn initialize_sets_metadata_and_unpaused_state() {
    let f = setup();

    assert_eq!(f.token.name().unwrap(), String::from_str(&f.env, "Pausable USD"));
    assert_eq!(f.token.symbol().unwrap(), Symbol::new(&f.env, "PUSD"));
    assert_eq!(f.token.decimals().unwrap(), 2);
    assert_eq!(f.token.admin().unwrap(), f.admin);
    assert_eq!(f.token.total_supply().unwrap(), 1_000_000);
    assert_eq!(f.token.balance(&f.admin), 1_000_000);
    assert!(!f.token.is_paused());
}

#[test]
fn transfer_works_when_unpaused() {
    let f = setup();

    f.token.transfer(&f.admin, &f.alice, &500_000).unwrap();

    assert_eq!(f.token.balance(&f.admin), 500_000);
    assert_eq!(f.token.balance(&f.alice), 500_000);

    let events = EventList::new(&f.env, f.env.events().all());
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
fn transfer_fails_when_paused() {
    let f = setup();

    f.token.pause().unwrap();
    assert!(f.token.is_paused());

    let result = f.token.try_transfer(&f.admin, &f.alice, &100);
    assert_eq!(result, Err(Ok(TokenError::ContractPaused)));
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
fn approve_and_transfer_from_work_when_unpaused() {
    let f = setup();

    f.token.approve(&f.admin, &f.alice, &300_000).unwrap();
    assert_eq!(f.token.allowance(&f.admin, &f.alice), 300_000);

    f.token.transfer_from(&f.alice, &f.admin, &f.bob, &250_000).unwrap();
    assert_eq!(f.token.balance(&f.admin), 750_000);
    assert_eq!(f.token.balance(&f.bob), 250_000);
    assert_eq!(f.token.allowance(&f.admin, &f.alice), 50_000);
}

#[test]
fn approve_emits_approval_event() {
    let f = setup();

    f.token.approve(&f.admin, &f.alice, &123_456).unwrap();

    let events = EventList::new(&f.env, f.env.events().all());
    assert_eq!(events.len(), 1);

    let (_id, topics, data) = events.get(0).unwrap();
    let namespace: Symbol = Symbol::try_from_val(&f.env, &topics.get(0).unwrap()).unwrap();
    let action: Symbol = Symbol::try_from_val(&f.env, &topics.get(1).unwrap()).unwrap();
    let owner: Address = Address::try_from_val(&f.env, &topics.get(2).unwrap()).unwrap();
    let spender: Address = Address::try_from_val(&f.env, &topics.get(3).unwrap()).unwrap();
    let payload: ApprovalEventData = ApprovalEventData::try_from_val(&f.env, &data).unwrap();

    assert_eq!(namespace, EVENT_NAMESPACE);
    assert_eq!(action, EVENT_APPROVE);
    assert_eq!(owner, f.admin);
    assert_eq!(spender, f.alice);
    assert_eq!(payload.amount, 123_456);
}

#[test]
fn approve_and_transfer_from_fail_when_paused() {
    let f = setup();

    f.token.pause().unwrap();

    let approve_result = f.token.try_approve(&f.admin, &f.alice, &300_000);
    assert_eq!(approve_result, Err(Ok(TokenError::ContractPaused)));

    let transfer_from_result = f.token.try_transfer_from(&f.alice, &f.admin, &f.bob, &100);
    assert_eq!(transfer_from_result, Err(Ok(TokenError::ContractPaused)));
}

#[test]
fn mint_and_burn_work_when_unpaused() {
    let f = setup();

    f.token.mint(&f.admin, &f.alice, &250_000).unwrap();
    assert_eq!(f.token.balance(&f.alice), 250_000);
    assert_eq!(f.token.total_supply().unwrap(), 1_250_000);

    f.token.burn(&f.alice, &50_000).unwrap();
    assert_eq!(f.token.balance(&f.alice), 200_000);
    assert_eq!(f.token.total_supply().unwrap(), 1_200_000);
}

#[test]
fn mint_and_burn_fail_when_paused() {
    let f = setup();

    f.token.pause().unwrap();

    let mint_result = f.token.try_mint(&f.admin, &f.alice, &100);
    assert_eq!(mint_result, Err(Ok(TokenError::ContractPaused)));

    f.token.unpause().unwrap();
    f.token.mint(&f.admin, &f.alice, &100).unwrap();
    f.token.pause().unwrap();

    let burn_result = f.token.try_burn(&f.alice, &50);
    assert_eq!(burn_result, Err(Ok(TokenError::ContractPaused)));
}

#[test]
fn admin_can_pause_and_unpause() {
    let f = setup();

    assert!(!f.token.is_paused());

    f.token.pause().unwrap();
    assert!(f.token.is_paused());

    f.token.unpause().unwrap();
    assert!(!f.token.is_paused());
}

#[test]
fn pause_emits_event() {
    let f = setup();

    f.token.pause().unwrap();

    let events = EventList::new(&f.env, f.env.events().all());
    let pause_event = events.iter().find(|(_, topics, _)| {
        let action: Symbol = Symbol::try_from_val(&f.env, &topics.get(1).unwrap()).unwrap();
        action == EVENT_PAUSE
    });

    assert!(pause_event.is_some());
}

#[test]
fn unpause_emits_event() {
    let f = setup();

    f.token.pause().unwrap();
    f.env.events().all(); // Clear events

    f.token.unpause().unwrap();

    let events = EventList::new(&f.env, f.env.events().all());
    let unpause_event = events.iter().find(|(_, topics, _)| {
        let action: Symbol = Symbol::try_from_val(&f.env, &topics.get(1).unwrap()).unwrap();
        action == EVENT_UNPAUSE
    });

    assert!(unpause_event.is_some());
}

#[test]
fn operations_again_after_unpause() {
    let f = setup();

    f.token.transfer(&f.admin, &f.alice, &100_000).unwrap();
    f.token.pause().unwrap();

    let result = f.token.try_transfer(&f.alice, &f.bob, &50_000);
    assert_eq!(result, Err(Ok(TokenError::ContractPaused)));

    f.token.unpause().unwrap();
    f.token.transfer(&f.alice, &f.bob, &50_000).unwrap();

    assert_eq!(f.token.balance(&f.alice), 50_000);
    assert_eq!(f.token.balance(&f.bob), 50_000);
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
fn transfer_from_rejects_zero_amount() {
    let f = setup();

    f.token.approve(&f.admin, &f.alice, &100).unwrap();
    assert_eq!(
        f.token.try_transfer_from(&f.alice, &f.admin, &f.bob, &0),
        Err(Ok(TokenError::InvalidAmount))
    );
}

#[test]
fn transfer_from_rejects_when_owner_balance_is_too_low() {
    let f = setup();

    f.token.transfer(&f.admin, &f.bob, &999_950).unwrap();
    f.token.approve(&f.admin, &f.alice, &100).unwrap();

    assert_eq!(
        f.token.try_transfer_from(&f.alice, &f.admin, &f.bob, &100),
        Err(Ok(TokenError::InsufficientBalance))
    );
}

#[test]
fn burn_rejects_insufficient_balance() {
    let f = setup();

    assert_eq!(
        f.token.try_burn(&f.alice, &1),
        Err(Ok(TokenError::InsufficientBalance))
    );
}

#[test]
fn burn_rejects_zero_amount() {
    let f = setup();

    assert_eq!(f.token.try_burn(&f.admin, &0), Err(Ok(TokenError::InvalidAmount)));
}

#[test]
fn approve_rejects_negative_amount() {
    let f = setup();

    assert_eq!(
        f.token.try_approve(&f.admin, &f.alice, &-1),
        Err(Ok(TokenError::InvalidAmount))
    );
}

#[test]
fn balance_and_allowance_default_to_zero() {
    let f = setup();
    let carol = Address::generate(&f.env);

    assert_eq!(f.token.balance(&carol), 0);
    assert_eq!(f.token.allowance(&f.admin, &carol), 0);
}

#[test]
fn double_initialize_is_rejected() {
    let f = setup();

    let name = String::from_str(&f.env, "Pausable USD");
    let symbol = Symbol::new(&f.env, "PUSD");
    assert_eq!(
        f.token.try_initialize(&f.admin, &name, &symbol, &2u32, &1_000_000i128),
        Err(Ok(TokenError::AlreadyInitialized))
    );
}

#[test]
fn uninitialized_contract_returns_not_initialized_for_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let token_id = env.register_contract(None, PausableToken);
    let token = PausableTokenClient::new(&env, &token_id);

    assert_eq!(token.try_total_supply(), Err(Ok(TokenError::NotInitialized)));
    assert_eq!(token.try_name(), Err(Ok(TokenError::NotInitialized)));
    assert_eq!(token.try_symbol(), Err(Ok(TokenError::NotInitialized)));
    assert_eq!(token.try_decimals(), Err(Ok(TokenError::NotInitialized)));
    assert_eq!(token.try_admin(), Err(Ok(TokenError::NotInitialized)));
}

#[test]
fn read_only_works_while_paused() {
    let f = setup();

    f.token.transfer(&f.admin, &f.alice, &100_000).unwrap();
    f.token.pause().unwrap();

    assert_eq!(f.token.balance(&f.alice), 100_000);
    assert_eq!(f.token.balance(&f.bob), 0);
    assert_eq!(f.token.total_supply().unwrap(), 1_000_000);
    assert_eq!(f.token.decimals().unwrap(), 2);
    assert!(f.token.is_paused());
}

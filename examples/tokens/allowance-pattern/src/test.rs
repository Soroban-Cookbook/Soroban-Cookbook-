#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, Symbol, TryFromVal,
};

const INITIAL_SUPPLY: i128 = 1_000_000;
const FAR_FUTURE: u32 = 100_000;

struct Fixture {
    env: Env,
    contract: AllowancePatternClient<'static>,
    admin: Address,
    alice: Address,
    bob: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AllowancePattern);
    let contract = AllowancePatternClient::new(&env, &contract_id);
    contract.initialize(&admin, &INITIAL_SUPPLY).unwrap();

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    Fixture {
        env,
        contract,
        admin,
        alice,
        bob,
    }
}

#[test]
fn initialize_credits_admin_balance() {
    let f = setup();

    assert_eq!(f.contract.admin().unwrap(), f.admin);
    assert_eq!(f.contract.balance(&f.admin), INITIAL_SUPPLY);
}

#[test]
fn double_initialize_is_rejected() {
    let f = setup();

    assert_eq!(
        f.contract.try_initialize(&f.admin, &INITIAL_SUPPLY),
        Err(Ok(AllowanceError::AlreadyInitialized))
    );
}

#[test]
fn initialize_rejects_negative_supply() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AllowancePattern);
    let contract = AllowancePatternClient::new(&env, &contract_id);

    assert_eq!(
        contract.try_initialize(&admin, &-1),
        Err(Ok(AllowanceError::InvalidAmount))
    );
}

#[test]
fn approve_sets_allowance_and_emits_event() {
    let f = setup();

    f.contract
        .approve(&f.admin, &f.alice, &300_000, &FAR_FUTURE);

    assert_eq!(f.contract.allowance(&f.admin, &f.alice), 300_000);
    assert_eq!(
        f.contract.allowance_details(&f.admin, &f.alice),
        AllowanceValue {
            amount: 300_000,
            expiration_ledger: FAR_FUTURE,
        }
    );

    let events = f.env.events().all();
    let (_id, topics, data) = events.last().unwrap();
    let namespace: Symbol = Symbol::try_from_val(&f.env, &topics.get(0).unwrap()).unwrap();
    let action: Symbol = Symbol::try_from_val(&f.env, &topics.get(1).unwrap()).unwrap();
    let owner: Address = Address::try_from_val(&f.env, &topics.get(2).unwrap()).unwrap();
    let spender: Address = Address::try_from_val(&f.env, &topics.get(3).unwrap()).unwrap();
    let payload: ApproveEventData = ApproveEventData::try_from_val(&f.env, &data).unwrap();

    assert_eq!(namespace, EVENT_NAMESPACE);
    assert_eq!(action, EVENT_APPROVE);
    assert_eq!(owner, f.admin);
    assert_eq!(spender, f.alice);
    assert_eq!(payload.amount, 300_000);
    assert_eq!(payload.expiration_ledger, FAR_FUTURE);
}

#[test]
fn transfer_from_moves_tokens_and_decrements_allowance() {
    let f = setup();

    f.contract
        .approve(&f.admin, &f.alice, &300_000, &FAR_FUTURE);
    f.contract
        .transfer_from(&f.alice, &f.admin, &f.bob, &250_000);

    assert_eq!(f.contract.balance(&f.admin), INITIAL_SUPPLY - 250_000);
    assert_eq!(f.contract.balance(&f.bob), 250_000);
    assert_eq!(f.contract.allowance(&f.admin, &f.alice), 50_000);

    // The remaining allowance keeps its original expiration.
    assert_eq!(
        f.contract
            .allowance_details(&f.admin, &f.alice)
            .expiration_ledger,
        FAR_FUTURE
    );

    let events = f.env.events().all();
    let (_id, topics, data) = events.last().unwrap();
    let action: Symbol = Symbol::try_from_val(&f.env, &topics.get(1).unwrap()).unwrap();
    let from: Address = Address::try_from_val(&f.env, &topics.get(2).unwrap()).unwrap();
    let to: Address = Address::try_from_val(&f.env, &topics.get(3).unwrap()).unwrap();
    let payload: TransferEventData = TransferEventData::try_from_val(&f.env, &data).unwrap();

    assert_eq!(action, EVENT_TRANSFER);
    assert_eq!(from, f.admin);
    assert_eq!(to, f.bob);
    assert_eq!(payload.amount, 250_000);
}

#[test]
fn transfer_from_rejects_over_allowance() {
    let f = setup();

    f.contract.approve(&f.admin, &f.alice, &100, &FAR_FUTURE);

    assert_eq!(
        f.contract
            .try_transfer_from(&f.alice, &f.admin, &f.bob, &101),
        Err(Ok(AllowanceError::InsufficientAllowance))
    );
}

#[test]
fn transfer_from_rejects_insufficient_balance() {
    let f = setup();

    // Allowance exceeds the owner's actual balance.
    f.contract
        .approve(&f.admin, &f.alice, &(INITIAL_SUPPLY * 2), &FAR_FUTURE);

    assert_eq!(
        f.contract
            .try_transfer_from(&f.alice, &f.admin, &f.bob, &(INITIAL_SUPPLY + 1)),
        Err(Ok(AllowanceError::InsufficientBalance))
    );
}

#[test]
fn transfer_from_rejects_non_positive_amount() {
    let f = setup();

    f.contract.approve(&f.admin, &f.alice, &100, &FAR_FUTURE);

    assert_eq!(
        f.contract.try_transfer_from(&f.alice, &f.admin, &f.bob, &0),
        Err(Ok(AllowanceError::InvalidAmount))
    );
    assert_eq!(
        f.contract
            .try_transfer_from(&f.alice, &f.admin, &f.bob, &-5),
        Err(Ok(AllowanceError::InvalidAmount))
    );
}

#[test]
fn approve_rejects_negative_amount() {
    let f = setup();

    assert_eq!(
        f.contract.try_approve(&f.admin, &f.alice, &-1, &FAR_FUTURE),
        Err(Ok(AllowanceError::InvalidAmount))
    );
}

#[test]
fn approve_zero_revokes_allowance() {
    let f = setup();

    f.contract
        .approve(&f.admin, &f.alice, &300_000, &FAR_FUTURE);
    assert_eq!(f.contract.allowance(&f.admin, &f.alice), 300_000);

    f.contract.approve(&f.admin, &f.alice, &0, &FAR_FUTURE);
    assert_eq!(f.contract.allowance(&f.admin, &f.alice), 0);
}

#[test]
fn revoke_clears_allowance() {
    let f = setup();

    f.contract
        .approve(&f.admin, &f.alice, &300_000, &FAR_FUTURE);
    f.contract.revoke(&f.admin, &f.alice);

    assert_eq!(f.contract.allowance(&f.admin, &f.alice), 0);
    assert_eq!(
        f.contract.try_transfer_from(&f.alice, &f.admin, &f.bob, &1),
        Err(Ok(AllowanceError::InsufficientAllowance))
    );
}

#[test]
fn expired_allowance_is_zero_and_blocks_transfer_from() {
    let f = setup();

    f.contract.approve(&f.admin, &f.alice, &300_000, &100);

    // Advance the ledger past the allowance expiration.
    f.env.ledger().with_mut(|li| li.sequence_number = 101);

    assert_eq!(f.contract.allowance(&f.admin, &f.alice), 0);
    // The raw entry still holds the stale amount; only the spendable view is 0.
    assert_eq!(
        f.contract.allowance_details(&f.admin, &f.alice).amount,
        300_000
    );
    assert_eq!(
        f.contract.try_transfer_from(&f.alice, &f.admin, &f.bob, &1),
        Err(Ok(AllowanceError::InsufficientAllowance))
    );
}

#[test]
fn approve_rejects_past_expiration_for_live_allowance() {
    let f = setup();

    f.env.ledger().with_mut(|li| li.sequence_number = 100);

    assert_eq!(
        f.contract.try_approve(&f.admin, &f.alice, &300_000, &50),
        Err(Ok(AllowanceError::InvalidExpiration))
    );
}

#[test]
fn allowance_valid_through_expiration_ledger() {
    let f = setup();

    f.contract.approve(&f.admin, &f.alice, &300_000, &100);

    // Exactly at the expiration ledger the allowance is still spendable.
    f.env.ledger().with_mut(|li| li.sequence_number = 100);
    assert_eq!(f.contract.allowance(&f.admin, &f.alice), 300_000);
    f.contract
        .transfer_from(&f.alice, &f.admin, &f.bob, &100_000);
    assert_eq!(f.contract.balance(&f.bob), 100_000);
}

#[test]
fn uninitialized_calls_return_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AllowancePattern);
    let contract = AllowancePatternClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);

    assert_eq!(
        contract.try_admin(),
        Err(Ok(AllowanceError::NotInitialized))
    );
    assert_eq!(
        contract.try_approve(&owner, &spender, &1, &FAR_FUTURE),
        Err(Ok(AllowanceError::NotInitialized))
    );
    assert_eq!(
        contract.try_transfer_from(&spender, &owner, &spender, &1),
        Err(Ok(AllowanceError::NotInitialized))
    );
}

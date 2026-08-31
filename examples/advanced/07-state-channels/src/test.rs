#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a fresh environment with the contract registered and initialised.
fn setup() -> (Env, Address, StateChannelContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StateChannelContract);
    let client = StateChannelContractClient::new(&env, &contract_id);
    client.initialize().unwrap();
    (env, contract_id, client)
}

/// Open a channel with default parameters (100 + 100 deposit, custom period).
fn open_channel(
    env: &Env,
    client: &StateChannelContractClient,
    party_a: &Address,
    party_b: &Address,
    deposit_a: i128,
    deposit_b: i128,
    dispute_period: Option<u32>,
) -> u64 {
    client
        .open(party_a, party_b, &deposit_a, &deposit_b, &dispute_period)
        .unwrap()
}

/// Advance the ledger by `delta` sequences (keeping timestamp in sync).
fn advance_ledger(env: &Env, delta: u32) {
    env.ledger().with_mut(|l| {
        l.sequence_number += delta;
        l.timestamp += (delta as u64) * 5;
    });
}

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StateChannelContract);
    let client = StateChannelContractClient::new(&env, &contract_id);
    // Should succeed on a fresh contract.
    assert!(client.initialize().is_ok());
}

#[test]
fn test_initialize_twice_fails() {
    let (_env, _contract_id, client) = setup();
    // Second call must fail with AlreadyInitialized.
    let err = client.try_initialize().unwrap_err().unwrap();
    assert_eq!(err, ChannelError::AlreadyInitialized);
}

// ---------------------------------------------------------------------------
// Channel opening
// ---------------------------------------------------------------------------

#[test]
fn test_open_channel_success() {
    let (env, _id, client) = setup();
    let party_a = Address::generate(&env);
    let party_b = Address::generate(&env);

    let channel_id = open_channel(&env, &client, &party_a, &party_b, 100, 200, None);
    assert_eq!(channel_id, 1);

    let ch = client.get_channel(&channel_id).unwrap();
    assert_eq!(ch.party_a, party_a);
    assert_eq!(ch.party_b, party_b);
    assert_eq!(ch.deposit_a, 100);
    assert_eq!(ch.deposit_b, 200);
    assert_eq!(ch.status, ChannelStatus::Open);
    assert_eq!(ch.sequence, 0);
}

#[test]
fn test_open_channel_increments_id() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let id1 = open_channel(&env, &client, &a, &b, 50, 50, None);
    let id2 = open_channel(&env, &client, &a, &b, 50, 50, None);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn test_open_channel_zero_deposit_fails() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let err = client.try_open(&a, &b, &0, &100, &None).unwrap_err().unwrap();
    assert_eq!(err, ChannelError::InvalidDeposit);
}

#[test]
fn test_open_channel_custom_dispute_period() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let id = open_channel(&env, &client, &a, &b, 100, 100, Some(300));
    let ch = client.get_channel(&id).unwrap();
    assert_eq!(ch.dispute_period, 300);
}

// ---------------------------------------------------------------------------
// Challenge (unilateral state submission)
// ---------------------------------------------------------------------------

#[test]
fn test_challenge_opens_dispute() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, None);

    // party_a submits a state where they have 60 and party_b has 140.
    client
        .challenge(&channel_id, &a, &1, &60, &140)
        .unwrap();

    let ch = client.get_channel(&channel_id).unwrap();
    assert_eq!(ch.status, ChannelStatus::Disputed);
    assert_eq!(ch.sequence, 1);
    assert_eq!(ch.balance_a, 60);
    assert_eq!(ch.balance_b, 140);
}

#[test]
fn test_challenge_higher_sequence_replaces_state() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, None);

    client.challenge(&channel_id, &a, &1, &60, &140).unwrap();
    // party_b counter-challenges with a newer state.
    client.challenge(&channel_id, &b, &5, &80, &120).unwrap();

    let ch = client.get_channel(&channel_id).unwrap();
    assert_eq!(ch.sequence, 5);
    assert_eq!(ch.balance_a, 80);
    assert_eq!(ch.balance_b, 120);
}

#[test]
fn test_challenge_sequence_too_low_fails() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, None);

    client.challenge(&channel_id, &a, &5, &50, &150).unwrap();
    // Submitting sequence 3 after sequence 5 is already on-chain.
    let err = client
        .try_challenge(&channel_id, &b, &3, &70, &130)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ChannelError::SequenceTooLow);
}

#[test]
fn test_challenge_balance_mismatch_fails() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, None);

    // 50 + 60 = 110 ≠ 200 (the total deposit).
    let err = client
        .try_challenge(&channel_id, &a, &1, &50, &60)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ChannelError::BalanceMismatch);
}

#[test]
fn test_challenge_by_non_party_fails() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, None);
    let outsider = Address::generate(&env);

    let err = client
        .try_challenge(&channel_id, &outsider, &1, &100, &100)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ChannelError::Unauthorized);
}

// ---------------------------------------------------------------------------
// Cooperative close
// ---------------------------------------------------------------------------

#[test]
fn test_cooperative_close_success() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, None);

    // Cooperatively settle: a gets 70, b gets 130.
    client.close(&channel_id, &1, &70, &130).unwrap();

    let ch = client.get_channel(&channel_id).unwrap();
    assert_eq!(ch.status, ChannelStatus::Closed);
    assert_eq!(ch.balance_a, 70);
    assert_eq!(ch.balance_b, 130);
}

#[test]
fn test_cooperative_close_balance_mismatch_fails() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, None);

    let err = client
        .try_close(&channel_id, &1, &50, &60)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ChannelError::BalanceMismatch);
}

#[test]
fn test_cooperative_close_already_closed_fails() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, None);

    client.close(&channel_id, &1, &100, &100).unwrap();
    let err = client
        .try_close(&channel_id, &2, &100, &100)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ChannelError::AlreadyClosed);
}

// ---------------------------------------------------------------------------
// Finalize (after dispute period)
// ---------------------------------------------------------------------------

#[test]
fn test_finalize_after_dispute_period() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    // Short dispute period for easier testing.
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, Some(10));

    client.challenge(&channel_id, &a, &2, &40, &160).unwrap();

    // Advance ledger past the dispute window.
    advance_ledger(&env, 11);

    client.finalize(&channel_id).unwrap();

    let ch = client.get_channel(&channel_id).unwrap();
    assert_eq!(ch.status, ChannelStatus::Closed);
    assert_eq!(ch.balance_a, 40);
    assert_eq!(ch.balance_b, 160);
}

#[test]
fn test_finalize_before_dispute_period_fails() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, Some(100));

    client.challenge(&channel_id, &a, &1, &50, &150).unwrap();

    // Only advance a few ledgers – still within the dispute window.
    advance_ledger(&env, 5);

    let err = client.try_finalize(&channel_id).unwrap_err().unwrap();
    assert_eq!(err, ChannelError::DisputePeriodActive);
}

#[test]
fn test_finalize_open_channel_fails() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, None);

    let err = client.try_finalize(&channel_id).unwrap_err().unwrap();
    assert_eq!(err, ChannelError::InvalidChannelState);
}

#[test]
fn test_finalize_closed_channel_fails() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, Some(5));

    client.challenge(&channel_id, &a, &1, &100, &100).unwrap();
    advance_ledger(&env, 6);
    client.finalize(&channel_id).unwrap();

    let err = client.try_finalize(&channel_id).unwrap_err().unwrap();
    assert_eq!(err, ChannelError::AlreadyClosed);
}

// ---------------------------------------------------------------------------
// Read helpers
// ---------------------------------------------------------------------------

#[test]
fn test_get_sequence_and_balances() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 300, 700, None);

    // Before any challenge the sequence is 0 and balances match deposits.
    assert_eq!(client.get_sequence(&channel_id).unwrap(), 0);
    assert_eq!(
        client.get_balances(&channel_id).unwrap(),
        (300_i128, 700_i128)
    );

    client.challenge(&channel_id, &a, &3, &400, &600).unwrap();
    assert_eq!(client.get_sequence(&channel_id).unwrap(), 3);
    assert_eq!(
        client.get_balances(&channel_id).unwrap(),
        (400_i128, 600_i128)
    );
}

#[test]
fn test_get_status_transitions() {
    let (env, _id, client) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let channel_id = open_channel(&env, &client, &a, &b, 100, 100, Some(5));

    assert_eq!(client.get_status(&channel_id).unwrap(), ChannelStatus::Open);

    client.challenge(&channel_id, &a, &1, &100, &100).unwrap();
    assert_eq!(
        client.get_status(&channel_id).unwrap(),
        ChannelStatus::Disputed
    );

    advance_ledger(&env, 6);
    client.finalize(&channel_id).unwrap();
    assert_eq!(
        client.get_status(&channel_id).unwrap(),
        ChannelStatus::Closed
    );
}

// ---------------------------------------------------------------------------
// Error paths: not initialised, channel not found
// ---------------------------------------------------------------------------

#[test]
fn test_operations_without_init_fail() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StateChannelContract);
    let client = StateChannelContractClient::new(&env, &contract_id);

    // None of the public methods should succeed before initialize().
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let err = client.try_open(&a, &b, &100, &100, &None).unwrap_err().unwrap();
    assert_eq!(err, ChannelError::NotInitialized);
}

#[test]
fn test_get_channel_not_found() {
    let (_env, _id, client) = setup();
    let err = client.try_get_channel(&999).unwrap_err().unwrap();
    assert_eq!(err, ChannelError::ChannelNotFound);
}

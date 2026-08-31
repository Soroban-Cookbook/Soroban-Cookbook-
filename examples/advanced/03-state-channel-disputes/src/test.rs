#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup() -> (
    Env,
    Address,
    Address,
    StateChannelDisputesContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StateChannelDisputesContract);
    let client = StateChannelDisputesContractClient::new(&env, &contract_id);
    let party_a = Address::generate(&env);
    let party_b = Address::generate(&env);
    (env, party_a, party_b, client)
}

// ── Open Channel Tests ───────────────────────────────────────────────────────

#[test]
fn test_open_channel_success() {
    let (_env, party_a, party_b, client) = setup();
    let res = client.try_open_channel(&party_a, &party_b, &100, &50, &3600);
    assert!(res.is_ok());

    let ch = client.get_channel();
    assert_eq!(ch.participant_a, party_a);
    assert_eq!(ch.participant_b, party_b);
    assert_eq!(ch.balance_a, 100);
    assert_eq!(ch.balance_b, 50);
    assert_eq!(ch.total_deposit, 150);
    assert_eq!(ch.dispute_window, 3600);
    assert_eq!(ch.status, ChannelStatus::Open);
}

#[test]
fn test_open_channel_same_participant_fails() {
    let (_env, party_a, _party_b, client) = setup();
    let res = client.try_open_channel(&party_a, &party_a, &100, &50, &3600);
    assert_eq!(res, Err(Ok(Error::InvalidParticipant)));
}

#[test]
fn test_open_channel_negative_balance_fails() {
    let (_env, party_a, party_b, client) = setup();
    let res = client.try_open_channel(&party_a, &party_b, &-10, &50, &3600);
    assert_eq!(res, Err(Ok(Error::InvalidBalance)));
}

#[test]
fn test_open_channel_zero_total_deposit_fails() {
    let (_env, party_a, party_b, client) = setup();
    let res = client.try_open_channel(&party_a, &party_b, &0, &0, &3600);
    assert_eq!(res, Err(Ok(Error::InvalidBalance)));
}

#[test]
fn test_open_channel_zero_dispute_window_fails() {
    let (_env, party_a, party_b, client) = setup();
    let res = client.try_open_channel(&party_a, &party_b, &100, &50, &0);
    assert_eq!(res, Err(Ok(Error::InvalidDisputeWindow)));
}

#[test]
fn test_open_channel_already_initialized_fails() {
    let (_env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);
    let res = client.try_open_channel(&party_a, &party_b, &100, &50, &3600);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

// ── Challenge (Open Dispute) Tests ─────────────────────────────────────────

#[test]
fn test_open_dispute_success() {
    let (env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);

    let now = env.ledger().timestamp();
    let res = client.try_open_dispute(&party_a, &1, &80, &70);
    assert!(res.is_ok());

    let ch = client.get_channel();
    assert_eq!(ch.status, ChannelStatus::Disputed);

    let dispute = client.get_dispute();
    assert_eq!(dispute.challenger, party_a);
    assert_eq!(dispute.sequence, 1);
    assert_eq!(dispute.proposed_balance_a, 80);
    assert_eq!(dispute.proposed_balance_b, 70);
    assert_eq!(dispute.deadline, now + 3600);
}

#[test]
fn test_open_dispute_unauthorized_participant_fails() {
    let (env, party_a, party_b, client) = setup();
    let stranger = Address::generate(&env);
    client.open_channel(&party_a, &party_b, &100, &50, &3600);

    let res = client.try_open_dispute(&stranger, &1, &80, &70);
    assert_eq!(res, Err(Ok(Error::InvalidParticipant)));
}

#[test]
fn test_open_dispute_invalid_balance_sum_fails() {
    let (_env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600); // total 150

    // Sum is 160 != 150
    let res = client.try_open_dispute(&party_a, &1, &90, &70);
    assert_eq!(res, Err(Ok(Error::InvalidBalanceSum)));
}

#[test]
fn test_open_dispute_when_already_disputed_fails() {
    let (_env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);
    client.open_dispute(&party_a, &1, &80, &70);

    let res = client.try_open_dispute(&party_b, &2, &60, &90);
    assert_eq!(res, Err(Ok(Error::ChannelNotOpen)));
}

// ── Response Mechanism Tests ───────────────────────────────────────────────

#[test]
fn test_respond_dispute_success() {
    let (_env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);
    client.open_dispute(&party_a, &1, &80, &70);

    // Party B responds with sequence 2
    let res = client.try_respond_dispute(&party_b, &2, &40, &110);
    assert!(res.is_ok());

    let dispute = client.get_dispute();
    assert_eq!(dispute.sequence, 2);
    assert_eq!(dispute.proposed_balance_a, 40);
    assert_eq!(dispute.proposed_balance_b, 110);
}

#[test]
fn test_respond_dispute_stale_sequence_fails() {
    let (_env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);
    client.open_dispute(&party_a, &2, &80, &70);

    // Responding with lower or equal sequence number
    let res = client.try_respond_dispute(&party_b, &2, &40, &110);
    assert_eq!(res, Err(Ok(Error::StaleSequence)));
}

#[test]
fn test_respond_dispute_after_deadline_fails() {
    let (env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);
    client.open_dispute(&party_a, &1, &80, &70);

    // Advance time past deadline (3600s)
    env.ledger().with_mut(|l| l.timestamp += 3601);

    let res = client.try_respond_dispute(&party_b, &2, &40, &110);
    assert_eq!(res, Err(Ok(Error::DisputeExpired)));
}

// ── Fraud Proof Tests ──────────────────────────────────────────────────────

#[test]
fn test_submit_fraud_proof_slashes_challenger() {
    let (_env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600); // total 150
    client.open_dispute(&party_a, &1, &80, &70);

    // Party B proves sequence 1 submitted by party A is fraudulent
    let res = client.try_submit_fraud_proof(&party_b, &1);
    assert!(res.is_ok());

    let ch = client.get_channel();
    assert_eq!(ch.status, ChannelStatus::Closed);
    assert_eq!(ch.balance_a, 0);
    assert_eq!(ch.balance_b, 150); // party B receives full total deposit
}

#[test]
fn test_submit_fraud_proof_by_proposer_fails() {
    let (_env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);
    client.open_dispute(&party_a, &1, &80, &70);

    // Party A (the proposer of sequence 1) tries to submit fraud proof on their own sequence
    let res = client.try_submit_fraud_proof(&party_a, &1);
    assert_eq!(res, Err(Ok(Error::InvalidFraudProof)));
}

#[test]
fn test_submit_fraud_proof_invalid_sequence_target_fails() {
    let (_env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);
    client.open_dispute(&party_a, &1, &80, &70);

    let res = client.try_submit_fraud_proof(&party_b, &999);
    assert_eq!(res, Err(Ok(Error::InvalidFraudProof)));
}

// ── Timeout & Finalization Tests ───────────────────────────────────────────

#[test]
fn test_finalize_dispute_before_deadline_fails() {
    let (_env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);
    client.open_dispute(&party_a, &1, &80, &70);

    let res = client.try_finalize_dispute();
    assert_eq!(res, Err(Ok(Error::DisputeNotExpired)));
}

#[test]
fn test_finalize_dispute_after_deadline_succeeds() {
    let (env, party_a, party_b, client) = setup();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);
    client.open_dispute(&party_a, &1, &80, &70);

    // Party B updates state to sequence 2
    client.respond_dispute(&party_b, &2, &40, &110);

    // Advance time past deadline
    env.ledger().with_mut(|l| l.timestamp += 3601);

    let res = client.try_finalize_dispute();
    assert!(res.is_ok());

    let ch = client.get_channel();
    assert_eq!(ch.status, ChannelStatus::Closed);
    assert_eq!(ch.balance_a, 40);
    assert_eq!(ch.balance_b, 110);

    // Dispute data removed
    let dispute_res = client.try_get_dispute();
    assert_eq!(dispute_res, Err(Ok(Error::ChannelNotInDispute)));
}

// ── Auth Guard Tests ───────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_open_dispute_unauthorized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StateChannelDisputesContract);
    let client = StateChannelDisputesContractClient::new(&env, &contract_id);
    let party_a = Address::generate(&env);
    let party_b = Address::generate(&env);

    env.mock_all_auths();
    client.open_channel(&party_a, &party_b, &100, &50, &3600);

    // Clear auths and try to dispute
    env.set_auths(&[]);
    client.open_dispute(&party_a, &1, &80, &70);
}

// ── Full Lifecycle Test ─────────────────────────────────────────────────────

#[test]
fn test_full_dispute_lifecycle() {
    let (env, party_a, party_b, client) = setup();

    // 1. Open Channel
    client.open_channel(&party_a, &party_b, &500, &500, &1000);
    assert_eq!(client.get_channel().status, ChannelStatus::Open);

    // 2. Party A challenges with stale state (sequence 5)
    client.open_dispute(&party_a, &5, &700, &300);
    assert_eq!(client.get_channel().status, ChannelStatus::Disputed);

    // 3. Party B responds with latest state (sequence 9)
    client.respond_dispute(&party_b, &9, &200, &800);
    assert_eq!(client.get_dispute().sequence, 9);

    // 4. Time passes past dispute window
    env.ledger().with_mut(|l| l.timestamp += 1001);

    // 5. Dispute finalized
    client.finalize_dispute();
    let final_ch = client.get_channel();
    assert_eq!(final_ch.status, ChannelStatus::Closed);
    assert_eq!(final_ch.balance_a, 200);
    assert_eq!(final_ch.balance_b, 800);
}

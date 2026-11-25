//! Unit tests for the real-world case studies contract.

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

fn setup(env: &Env) -> (Address, CaseStudiesClient<'static>) {
    let contract_id = env.register_contract(None, CaseStudies);
    let client = CaseStudiesClient::new(env, &contract_id);
    (contract_id, client)
}

// ---------------------------------------------------------------------
// Case study 1: reward claim
// ---------------------------------------------------------------------

#[test]
fn test_claim_reward_zeroes_balance_and_prevents_double_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let (_, client) = setup(&env);

    client.try_initialize(&admin).unwrap().unwrap();
    client
        .try_fund_reward(&admin, &alice, &500)
        .unwrap()
        .unwrap();
    assert_eq!(client.reward_balance(&alice), 500);

    let claimed = client.try_claim_reward(&alice).unwrap().unwrap();
    assert_eq!(claimed, 500);
    // Balance is zeroed immediately, so a second claim (e.g. a naive
    // reentrant call) has nothing left to take.
    assert_eq!(client.reward_balance(&alice), 0);
    assert_eq!(
        client.try_claim_reward(&alice),
        Err(Ok(Error::NothingToClaim))
    );
}

#[test]
fn test_fund_reward_requires_admin_and_positive_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let not_admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let (_, client) = setup(&env);

    client.try_initialize(&admin).unwrap().unwrap();

    assert_eq!(
        client.try_fund_reward(&not_admin, &alice, &100),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        client.try_fund_reward(&admin, &alice, &0),
        Err(Ok(Error::InvalidAmount))
    );
}

// ---------------------------------------------------------------------
// Case study 2: fee calculation
// ---------------------------------------------------------------------

#[test]
fn test_calculate_fee_matches_expected_value() {
    // 2.5% of 1_000_000 units.
    assert_eq!(CaseStudies::calculate_fee(1_000_000, 250).unwrap(), 25_000);
    assert_eq!(CaseStudies::calculate_fee(0, 250).unwrap(), 0);
}

#[test]
fn test_calculate_fee_rejects_overflow_and_invalid_input() {
    assert_eq!(
        CaseStudies::calculate_fee(-1, 250),
        Err(Error::InvalidAmount)
    );
    assert_eq!(
        CaseStudies::calculate_fee(100, 10_001),
        Err(Error::InvalidAmount)
    );
    // Large enough that `amount * fee_bps` overflows i128 before dividing.
    assert_eq!(
        CaseStudies::calculate_fee(i128::MAX, 10_000),
        Err(Error::Overflow)
    );
}

// ---------------------------------------------------------------------
// Case study 3: commit-reveal bidding
// ---------------------------------------------------------------------

fn commitment(env: &Env, amount: i128, salt: &BytesN<32>) -> BytesN<32> {
    env.crypto()
        .sha256(&(amount, salt.clone()).to_xdr(env))
        .to_bytes()
}

#[test]
fn test_commit_then_reveal_updates_highest_bid() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let (_, client) = setup(&env);
    client.try_initialize(&admin).unwrap().unwrap();

    let alice_salt = BytesN::from_array(&env, &[1u8; 32]);
    let bob_salt = BytesN::from_array(&env, &[2u8; 32]);
    let alice_commitment = commitment(&env, 100, &alice_salt);
    let bob_commitment = commitment(&env, 150, &bob_salt);

    client.commit_bid(&alice, &alice_commitment);
    client.commit_bid(&bob, &bob_commitment);

    client.reveal_bid(&alice, &100, &alice_salt);
    assert_eq!(client.highest_bid(), 100);
    assert_eq!(client.highest_bidder(), Some(alice.clone()));

    client.reveal_bid(&bob, &150, &bob_salt);
    assert_eq!(client.highest_bid(), 150);
    assert_eq!(client.highest_bidder(), Some(bob));
}

#[test]
fn test_reveal_rejects_mismatched_amount_or_salt() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let (_, client) = setup(&env);
    client.try_initialize(&admin).unwrap().unwrap();

    let salt = BytesN::from_array(&env, &[7u8; 32]);
    let wrong_salt = BytesN::from_array(&env, &[9u8; 32]);
    let bid_commitment = commitment(&env, 200, &salt);
    client.commit_bid(&alice, &bid_commitment);

    // Wrong revealed amount.
    assert_eq!(
        client.try_reveal_bid(&alice, &201, &salt),
        Err(Ok(Error::CommitmentMismatch))
    );
    // Wrong salt.
    assert_eq!(
        client.try_reveal_bid(&alice, &200, &wrong_salt),
        Err(Ok(Error::CommitmentMismatch))
    );
    // Correct reveal still works afterward.
    client.reveal_bid(&alice, &200, &salt);
    assert_eq!(client.highest_bid(), 200);
}

#[test]
fn test_reveal_without_commitment_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let (_, client) = setup(&env);
    client.try_initialize(&admin).unwrap().unwrap();

    let salt = BytesN::from_array(&env, &[3u8; 32]);
    assert_eq!(
        client.try_reveal_bid(&alice, &100, &salt),
        Err(Ok(Error::NoCommitment))
    );
}

#[test]
fn test_double_commit_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let (_, client) = setup(&env);
    client.try_initialize(&admin).unwrap().unwrap();

    let salt = BytesN::from_array(&env, &[4u8; 32]);
    let bid_commitment = commitment(&env, 100, &salt);
    client.commit_bid(&alice, &bid_commitment);

    assert_eq!(
        client.try_commit_bid(&alice, &bid_commitment),
        Err(Ok(Error::AlreadyCommitted))
    );
}

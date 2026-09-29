#![allow(deprecated)]
//! Tests for the Delegation Marketplace contract.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_client(env: &Env) -> DelegationMarketplaceClient<'_> {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, DelegationMarketplace);
    DelegationMarketplaceClient::new(env, &contract_id)
}

// ---------------------------------------------------------------------------
// Offer listing
// ---------------------------------------------------------------------------

#[test]
fn list_offer_stores_offer() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);

    client.list_offer(&delegator, &100u64, &10i128);

    let offer = client.get_offer(&delegator).unwrap();
    assert_eq!(offer.delegator, delegator);
    assert_eq!(offer.voting_power, 100);
    assert_eq!(offer.price_per_unit, 10);
}

#[test]
fn list_offer_fails_when_offer_already_exists() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);

    client.list_offer(&delegator, &100u64, &10i128);

    let err = client
        .try_list_offer(&delegator, &50u64, &5i128)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::OfferAlreadyExists));
}

#[test]
fn list_offer_fails_with_zero_voting_power() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);

    let err = client
        .try_list_offer(&delegator, &0u64, &10i128)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::InvalidAmount));
}

#[test]
fn list_offer_fails_with_zero_price() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);

    let err = client
        .try_list_offer(&delegator, &100u64, &0i128)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::InvalidAmount));
}

// ---------------------------------------------------------------------------
// Offer cancellation
// ---------------------------------------------------------------------------

#[test]
fn cancel_offer_removes_offer() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);

    client.list_offer(&delegator, &100u64, &10i128);
    client.cancel_offer(&delegator);

    assert!(client.get_offer(&delegator).is_none());
}

#[test]
fn cancel_offer_fails_when_no_offer() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);

    let err = client
        .try_cancel_offer(&delegator)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::OfferNotFound));
}

// ---------------------------------------------------------------------------
// Renting voting power
// ---------------------------------------------------------------------------

#[test]
fn rent_voting_power_transfers_fee_and_records_delegation() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    client.list_offer(&delegator, &100u64, &5i128);
    client.fund_account(&renter, &500i128);

    client.rent_voting_power(&renter, &delegator, &10u64, &3600u64);

    // Fee = 10 * 5 = 50
    assert_eq!(client.get_balance(&renter), 450);
    assert_eq!(client.get_balance(&delegator), 50);

    let delegation = client.get_delegation(&renter, &delegator).unwrap();
    assert_eq!(delegation.units, 10);
    assert_eq!(delegation.renter, renter);
    assert_eq!(delegation.delegator, delegator);
}

#[test]
fn rent_reduces_offer_voting_power() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    client.list_offer(&delegator, &100u64, &1i128);
    client.fund_account(&renter, &1000i128);

    client.rent_voting_power(&renter, &delegator, &30u64, &3600u64);

    let offer = client.get_offer(&delegator).unwrap();
    assert_eq!(offer.voting_power, 70);
}

#[test]
fn rent_removes_offer_when_all_power_rented() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    client.list_offer(&delegator, &50u64, &1i128);
    client.fund_account(&renter, &1000i128);

    client.rent_voting_power(&renter, &delegator, &50u64, &3600u64);

    assert!(client.get_offer(&delegator).is_none());
}

#[test]
fn rent_fails_when_offer_not_found() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    client.fund_account(&renter, &1000i128);

    let err = client
        .try_rent_voting_power(&renter, &delegator, &10u64, &3600u64)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::OfferNotFound));
}

#[test]
fn rent_fails_when_insufficient_voting_power() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    client.list_offer(&delegator, &5u64, &1i128);
    client.fund_account(&renter, &1000i128);

    let err = client
        .try_rent_voting_power(&renter, &delegator, &10u64, &3600u64)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::InsufficientVotingPower));
}

#[test]
fn rent_fails_when_insufficient_balance() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    client.list_offer(&delegator, &100u64, &100i128);
    client.fund_account(&renter, &50i128);

    let err = client
        .try_rent_voting_power(&renter, &delegator, &1u64, &3600u64)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::InsufficientBalance));
}

#[test]
fn rent_fails_when_delegation_already_exists() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    // List with enough supply for two rentals
    client.list_offer(&delegator, &100u64, &1i128);
    client.fund_account(&renter, &1000i128);

    // First rental succeeds
    client.rent_voting_power(&renter, &delegator, &10u64, &3600u64);

    // Attempting a second rental from the same renter to the same delegator
    // should fail because the delegation already exists.
    let err = client
        .try_rent_voting_power(&renter, &delegator, &5u64, &3600u64)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::DelegationAlreadyExists));
}

// ---------------------------------------------------------------------------
// Expiry
// ---------------------------------------------------------------------------

#[test]
fn expire_delegation_removes_record_and_returns_power() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    client.list_offer(&delegator, &100u64, &1i128);
    client.fund_account(&renter, &1000i128);

    client.rent_voting_power(&renter, &delegator, &20u64, &3600u64);

    // Advance time past expiry
    env.ledger().with_mut(|li| li.timestamp += 4000);

    client.expire_delegation(&renter, &delegator);

    assert!(client.get_delegation(&renter, &delegator).is_none());

    // 80 units remain in offer + 20 returned = 100
    let offer = client.get_offer(&delegator).unwrap();
    assert_eq!(offer.voting_power, 100);
}

#[test]
fn expire_delegation_fails_before_expiry() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    client.list_offer(&delegator, &100u64, &1i128);
    client.fund_account(&renter, &1000i128);

    client.rent_voting_power(&renter, &delegator, &10u64, &3600u64);

    let err = client
        .try_expire_delegation(&renter, &delegator)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::DelegationNotExpired));
}

#[test]
fn expire_delegation_fails_when_not_found() {
    let env = Env::default();
    let client = make_client(&env);
    let delegator = Address::generate(&env);
    let renter = Address::generate(&env);

    let err = client
        .try_expire_delegation(&renter, &delegator)
        .expect_err("should fail");
    assert_eq!(err, Ok(MarketplaceError::DelegationNotFound));
}

// ---------------------------------------------------------------------------
// Balance helpers
// ---------------------------------------------------------------------------

#[test]
fn fund_account_and_get_balance() {
    let env = Env::default();
    let client = make_client(&env);
    let account = Address::generate(&env);

    assert_eq!(client.get_balance(&account), 0);

    client.fund_account(&account, &1000i128);
    assert_eq!(client.get_balance(&account), 1000);

    client.fund_account(&account, &500i128);
    assert_eq!(client.get_balance(&account), 1500);
}

#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn create_token(env: &Env) -> Address {
    env.register_stellar_asset_contract_v2(Address::generate(env))
        .address()
}

fn setup() -> (Env, Address, Address, StakingPoolContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let staker = Address::generate(&env);
    let token_id = create_token(&env);

    let contract_id = env.register_contract(None, StakingPoolContract);
    let client = StakingPoolContractClient::new(&env, &contract_id);
    client.initialize(&token_id);

    (env, staker, contract_id, client, token_id)
}

#[test]
fn test_get_lockup_options() {
    let (_, _, _, client, _) = setup();
    let options = client.get_lockup_options();

    assert_eq!(options.len(), 3);
    assert_eq!(options.get(0).unwrap().duration, LOCKUP_30_DAYS);
    assert_eq!(options.get(1).unwrap().boost_bps, BOOST_90_DAYS_BPS);
    assert_eq!(options.get(2).unwrap().boost_bps, BOOST_180_DAYS_BPS);
}

#[test]
fn test_stake_and_get_stake_info() {
    let (env, staker, _, client, token_id) = setup();
    let token = token::StellarAssetClient::new(&env, &token_id);
    token.mint(&staker, &1_000i128);
    client.stake(&staker, &1_000i128, &LOCKUP_90_DAYS);

    let stake = client.get_stake(&staker).expect("stake exists");
    assert_eq!(stake.amount, 1_000);
    assert_eq!(stake.duration, LOCKUP_90_DAYS);
    assert_eq!(stake.boost_bps, BOOST_90_DAYS_BPS);
    assert_eq!(stake.mature_ts, env.ledger().timestamp() + LOCKUP_90_DAYS);
}

#[test]
fn test_withdraw_after_maturity_applies_boost() {
    let (env, staker, contract_id, client, token_id) = setup();
    let token = token::StellarAssetClient::new(&env, &token_id);
    token.mint(&staker, &1_000i128);
    token.mint(&contract_id, &250i128);
    client.stake(&staker, &1_000i128, &LOCKUP_180_DAYS);
    env.ledger()
        .with_mut(|l| l.timestamp += LOCKUP_180_DAYS + 1);

    let payout = client.withdraw(&staker);
    assert_eq!(payout, 1_250);
    assert!(client.get_stake(&staker).is_none());
}

#[test]
fn test_early_withdrawal_penalty() {
    let (env, staker, _, client, token_id) = setup();
    let token = token::StellarAssetClient::new(&env, &token_id);
    token.mint(&staker, &1_000i128);
    client.stake(&staker, &1_000i128, &LOCKUP_180_DAYS);
    env.ledger().with_mut(|l| l.timestamp += LOCKUP_30_DAYS);

    let payout = client.withdraw(&staker);
    assert_eq!(payout, 800);
    assert!(client.get_stake(&staker).is_none());
}

#[test]
#[should_panic(expected = "Active stake exists")]
fn test_stake_duplicate_fails() {
    let (env, staker, _, client, token_id) = setup();
    let token = token::StellarAssetClient::new(&env, &token_id);
    token.mint(&staker, &2_000i128);
    client.stake(&staker, &1_000i128, &LOCKUP_30_DAYS);
    client.stake(&staker, &500i128, &LOCKUP_30_DAYS);
}

#[test]
#[should_panic(expected = "Invalid lockup duration")]
fn test_stake_invalid_duration() {
    let (env, staker, _, client, token_id) = setup();
    let token = token::StellarAssetClient::new(&env, &token_id);
    token.mint(&staker, &1_000i128);
    client.stake(&staker, &1_000i128, &42);
}

#[test]
#[should_panic(expected = "No active stake")]
fn test_withdraw_without_stake() {
    let (_, staker, _, client, _) = setup();
    client.withdraw(&staker);
}

extern crate std;

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

fn setup() -> (Env, Address, StakingPoolContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StakingPoolContract);
    let client = StakingPoolContractClient::new(&env, &contract_id);
    let staker = Address::generate(&env);
    (env, staker, client)
}

#[test]
fn test_get_lockup_options() {
    let (_, _, client) = setup();
    let options = client.get_lockup_options();

    assert_eq!(options.len(), 3);
    assert_eq!(options[0].duration, LOCKUP_30_DAYS);
    assert_eq!(options[1].boost_bps, BOOST_90_DAYS_BPS);
    assert_eq!(options[2].boost_bps, BOOST_180_DAYS_BPS);
}

#[test]
fn test_stake_and_get_stake_info() {
    let (env, staker, client) = setup();
    client.stake(&staker, &1_000, &LOCKUP_90_DAYS);

    let stake = client.get_stake(&staker).expect("stake exists");
    assert_eq!(stake.amount, 1_000);
    assert_eq!(stake.duration, LOCKUP_90_DAYS);
    assert_eq!(stake.boost_bps, BOOST_90_DAYS_BPS);
    assert_eq!(stake.mature_ts, env.ledger().timestamp() + LOCKUP_90_DAYS);
}

#[test]
fn test_withdraw_after_maturity_applies_boost() {
    let (env, staker, client) = setup();
    client.stake(&staker, &1_000, &LOCKUP_180_DAYS);
    env.ledger().with_mut(|l| l.timestamp += LOCKUP_180_DAYS + 1);

    let payout = client.withdraw(&staker);
    assert_eq!(payout, 1_250);
    assert!(client.get_stake(&staker).is_none());
}

#[test]
fn test_early_withdrawal_penalty() {
    let (env, staker, client) = setup();
    client.stake(&staker, &1_000, &LOCKUP_180_DAYS);
    env.ledger().with_mut(|l| l.timestamp += LOCKUP_30_DAYS);

    let payout = client.withdraw(&staker);
    assert_eq!(payout, 800);
    assert!(client.get_stake(&staker).is_none());
}

#[test]
#[should_panic(expected = "Active stake exists")]
fn test_stake_duplicate_fails() {
    let (_, staker, client) = setup();
    client.stake(&staker, &1_000, &LOCKUP_30_DAYS);
    client.stake(&staker, &500, &LOCKUP_30_DAYS);
}

#[test]
#[should_panic(expected = "Invalid lockup duration")]
fn test_stake_invalid_duration() {
    let (_, staker, client) = setup();
    client.stake(&staker, &1_000, &42);
}

#[test]
#[should_panic(expected = "No active stake")]
fn test_withdraw_without_stake() {
    let (_, staker, client) = setup();
    client.withdraw(&staker);
}

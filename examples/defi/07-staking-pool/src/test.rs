#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{token, Address, Env, Symbol};
use crate::{StakingPoolContract, StakingPoolContractClient};

fn register_token(env: &Env, admin: &Address, _name: &Symbol, _symbol: &Symbol) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn mint(env: &Env, token_id: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token_id).mint(to, &amount);
}

fn setup() -> (Env, Address, Address, Address, StakingPoolContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let staking_token = register_token(&env, &owner, &Symbol::short("Stake"), &Symbol::short("STK"));
    let reward_token = register_token(&env, &owner, &Symbol::short("Reward"), &Symbol::short("RWD"));

    let contract_id = env.register_contract(None, StakingPoolContract);
    let client = StakingPoolContractClient::new(&env, &contract_id);
    client.initialize(&owner, &staking_token, &reward_token, &10i128);

    (env, owner, alice, bob, client, staking_token, reward_token)
}

#[test]
fn test_stake_increases_balance() {
    let (env, _owner, alice, _bob, client, staking_token, _reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);

    env.mock_all_auths();
    mint(&env, &staking_token, &alice, 1000);

    client.stake(&alice, &100i128);
    assert_eq!(client.balance_of(&alice), 100i128);
}

#[test]
fn test_unstake_returns_tokens() {
    let (env, _owner, alice, _bob, client, staking_token, _reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);

    env.mock_all_auths();
    mint(&env, &staking_token, &alice, 1000);

    client.stake(&alice, &100i128);
    client.unstake(&alice, &50i128);
    assert_eq!(client.balance_of(&alice), 50i128);
}

#[test]
fn test_earned_rewards_accumulate_over_time() {
    let (env, owner, alice, _bob, client, staking_token, reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);
    let reward = token::Client::new(&env, &reward_token);

    env.mock_all_auths();
    mint(&env, &staking_token, &alice, 1000);
    mint(&env, &reward_token, &client.address, 1000);

    client.stake(&alice, &100i128);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    assert!(client.earned(&alice) > 0);
}

#[test]
fn test_claim_rewards_sends_reward_tokens() {
    let (env, _owner, alice, _bob, client, staking_token, reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);
    let reward = token::Client::new(&env, &reward_token);

    env.mock_all_auths();
    mint(&env, &staking_token, &alice, 1000);
    mint(&env, &reward_token, &client.address, 10_000);

    client.stake(&alice, &100i128);
    env.ledger().with_mut(|li| li.timestamp = 500);
    client.claim_rewards(&alice);

    assert!(reward.balance(&alice) > 0);
}

#[test]
fn test_reward_per_token_increases_with_time() {
    let (env, _owner, alice, _bob, client, staking_token, _reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);

    env.mock_all_auths();
    mint(&env, &staking_token, &alice, 1000);

    client.stake(&alice, &100i128);
    let before = client.reward_per_token();
    env.ledger().with_mut(|li| li.timestamp = 500);
    let after = client.reward_per_token();
    assert!(after >= before);
}

#[test]
fn test_multiple_stakers_share_rewards() {
    let (env, _owner, alice, bob, client, staking_token, reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);
    let reward = token::Client::new(&env, &reward_token);

    env.mock_all_auths();
    mint(&env, &staking_token, &alice, 1000);
    mint(&env, &staking_token, &bob, 1000);
    mint(&env, &reward_token, &client.address, 2000);

    client.stake(&alice, &100i128);
    env.ledger().with_mut(|li| li.timestamp = 100);
    client.stake(&bob, &100i128);
    env.ledger().with_mut(|li| li.timestamp = 500);

    let alice_earned = client.earned(&alice);
    let bob_earned = client.earned(&bob);
    assert!(alice_earned >= bob_earned);
}

#[test]
fn test_total_supply_updates_when_staking() {
    let (env, _owner, alice, bob, client, staking_token, _reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);

    env.mock_all_auths();
    mint(&env, &staking_token, &alice, 1000);
    mint(&env, &staking_token, &bob, 1000);

    client.stake(&alice, &100i128);
    client.stake(&bob, &200i128);
    assert_eq!(client.balance_of(&alice), 100i128);
    assert_eq!(client.balance_of(&bob), 200i128);
}

#[test]
#[should_panic(expected = "missing required authorization")]
fn test_stake_requires_auth() {
    let (env, _owner, alice, _bob, client, staking_token, _reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);

    staking.mint(&alice, &1000i128);
    client.stake(&alice, &100i128);
}

#[test]
#[should_panic(expected = "missing required authorization")]
fn test_unstake_requires_auth() {
    let (env, _owner, alice, _bob, client, staking_token, _reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);

    staking.mint(&alice, &1000i128);
    client.stake(&alice, &100i128);
    client.unstake(&alice, &25i128);
}

#[test]
#[should_panic(expected = "missing required authorization")]
fn test_claim_rewards_requires_auth() {
    let (env, _owner, alice, _bob, client, staking_token, reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);
    let reward = token::Client::new(&env, &reward_token);

    staking.mint(&alice, &1000i128);
    reward.mint(&env.current_contract_address(), &1000i128);
    client.stake(&alice, &100i128);
    env.ledger().set(Ledger::timestamp(500));
    client.claim_rewards(&alice);
}

#[test]
#[should_panic(expected = "missing required authorization")]
fn test_initialize_requires_owner_auth() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let staking_token = register_token(&env, &owner, &Symbol::short("Stake"), &Symbol::short("STK"));
    let reward_token = register_token(&env, &owner, &Symbol::short("Reward"), &Symbol::short("RWD"));

    let contract_id = env.register_contract(None, StakingPoolContract);
    let client = StakingPoolContractClient::new(&env, &contract_id);

    client.initialize(&attacker, &staking_token, &reward_token, &10i128);
}

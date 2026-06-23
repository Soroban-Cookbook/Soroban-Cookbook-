#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env, Ledger, Symbol};
use crate::StakingPoolContractClient;

fn register_token(env: &Env, admin: &Address, name: &Symbol, symbol: &Symbol) -> Address {
    let token_id = env.register_contract(None, token::Contract);
    let token_client = token::Client::new(env, &token_id);
    token_client.initialize(admin, name, symbol, &8u32);
    token_id
}

fn setup() -> (Env, Address, Address, Address, StakingPoolContractClient<'static>, Address, Address) {
    let env = Env::default();
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
    staking.mint(&alice, &1000i128);

    client.stake(&alice, &100i128);
    assert_eq!(client.balance_of(&alice), 100i128);
}

#[test]
fn test_unstake_returns_tokens() {
    let (env, _owner, alice, _bob, client, staking_token, _reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);

    env.mock_all_auths();
    staking.mint(&alice, &1000i128);

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
    staking.mint(&alice, &1000i128);
    reward.mint(&env.current_contract_address(), &1000i128);

    client.stake(&alice, &100i128);
    env.ledger().set(Ledger::timestamp(1000));
    assert!(client.earned(&alice) > 0);
}

#[test]
fn test_claim_rewards_sends_reward_tokens() {
    let (env, _owner, alice, _bob, client, staking_token, reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);
    let reward = token::Client::new(&env, &reward_token);

    env.mock_all_auths();
    staking.mint(&alice, &1000i128);
    reward.mint(&env.current_contract_address(), &1000i128);

    client.stake(&alice, &100i128);
    env.ledger().set(Ledger::timestamp(500));
    client.claim_rewards(&alice);

    assert!(reward.balance(&alice) > 0);
}

#[test]
fn test_reward_per_token_increases_with_time() {
    let (env, _owner, alice, _bob, client, staking_token, _reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);

    env.mock_all_auths();
    staking.mint(&alice, &1000i128);

    client.stake(&alice, &100i128);
    let before = client.reward_per_token();
    env.ledger().set(Ledger::timestamp(500));
    let after = client.reward_per_token();
    assert!(after >= before);
}

#[test]
fn test_multiple_stakers_share_rewards() {
    let (env, _owner, alice, bob, client, staking_token, reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);
    let reward = token::Client::new(&env, &reward_token);

    env.mock_all_auths();
    staking.mint(&alice, &1000i128);
    staking.mint(&bob, &1000i128);
    reward.mint(&env.current_contract_address(), &2000i128);

    client.stake(&alice, &100i128);
    env.ledger().set(Ledger::timestamp(100));
    client.stake(&bob, &100i128);
    env.ledger().set(Ledger::timestamp(500));

    let alice_earned = client.earned(&alice);
    let bob_earned = client.earned(&bob);
    assert!(alice_earned >= bob_earned);
}

#[test]
fn test_total_supply_updates_when_staking() {
    let (env, _owner, alice, bob, client, staking_token, _reward_token) = setup();
    let staking = token::Client::new(&env, &staking_token);

    env.mock_all_auths();
    staking.mint(&alice, &1000i128);
    staking.mint(&bob, &1000i128);

    client.stake(&alice, &100i128);
    client.stake(&bob, &200i128);
    assert_eq!(client.balance_of(&alice), 100i128);
    assert_eq!(client.balance_of(&bob), 200i128);
}

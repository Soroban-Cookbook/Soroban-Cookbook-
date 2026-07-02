#![cfg(test)]
use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{token, Address, Env};

fn setup_test() -> (Env, FarmingPoolContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, FarmingPoolContract);
    let client = FarmingPoolContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn create_token(env: &Env, admin: &Address) -> (Address, token::Client<'static>, token::StellarAssetClient<'static>) {
    let id = env.register_stellar_asset_contract(admin.clone());
    let token = token::Client::new(env, &id);
    let admin_client = token::StellarAssetClient::new(env, &id);
    (id, token, admin_client)
}

#[test]
fn test_initialize() {
    let (_env, client, admin) = setup_test();
    // Verification is implicit in setup_test as it calls initialize
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_initialize_twice() {
    let (_env, client, admin) = setup_test();
    client.initialize(&admin);
}

#[test]
fn test_add_and_remove_pool() {
    let (env, client, admin) = setup_test();
    let (staking_token, _, _) = create_token(&env, &admin);
    let (reward_token, _, _) = create_token(&env, &admin);

    let pool_id = client.add_pool(&admin, &staking_token, &reward_token, &100, &1);
    assert_eq!(pool_id, 0);

    client.remove_pool(&admin, &pool_id);
    // Should panic if we try to access it now
}

#[test]
fn test_reward_logic() {
    let (env, client, admin) = setup_test();
    let (staking_token_id, staking_token, staking_admin) = create_token(&env, &admin);
    let (reward_token_id, reward_token, reward_admin) = create_token(&env, &admin);

    let pool_id = client.add_pool(&admin, &staking_token_id, &reward_token_id, &100, &1);

    let user = Address::generate(&env);
    staking_admin.mint(&user, &1000);
    reward_admin.mint(&client.address, &10000);

    // Initial deposit
    client.deposit(&user, &pool_id, &500);
    assert_eq!(staking_token.balance(&user), 500);
    assert_eq!(staking_token.balance(&client.address), 500);

    // Advance ledger
    env.ledger().with_mut(|li| li.sequence += 10);

    // Withdraw and check rewards
    // 10 ledgers * 100 reward_rate = 1000 rewards
    client.withdraw(&user, &pool_id, &500);
    
    assert_eq!(staking_token.balance(&user), 1000);
    assert_eq!(reward_token.balance(&user), 1000);
}

#[test]
fn test_admin_adjustments() {
    let (env, client, admin) = setup_test();
    let (staking_token, _, _) = create_token(&env, &admin);
    let (reward_token, _, _) = create_token(&env, &admin);

    let pool_id = client.add_pool(&admin, &staking_token, &reward_token, &100, &1);

    client.set_reward_rate(&admin, &pool_id, &200);
    client.set_pool_weight(&admin, &pool_id, &2);
}

#[test]
fn test_emergency_withdraw_admin() {
    let (env, client, admin) = setup_test();
    let (token_id, token, token_admin) = create_token(&env, &admin);

    token_admin.mint(&client.address, &1000);
    assert_eq!(token.balance(&client.address), 1000);

    let recipient = Address::generate(&env);
    client.emergency_withdraw_admin(&admin, &token_id, &recipient, &1000);

    assert_eq!(token.balance(&client.address), 0);
    assert_eq!(token.balance(&recipient), 1000);
}

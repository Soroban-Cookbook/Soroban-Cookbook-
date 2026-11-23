#![allow(deprecated)]
use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

fn create_token(env: &Env) -> Address {
    env.register_stellar_asset_contract_v2(Address::generate(env))
        .address()
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    client.initialize(&token, &5, &10, &80);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    client.initialize(&token, &5, &10, &80);
    client.initialize(&token, &5, &10, &80);
}

#[test]
fn test_deposit() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &1000);
    let position = client.get_user_position(&user);
    assert_eq!(position.deposit, 1000);
    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&user), 0);
    assert_eq!(token_client.balance(&contract_id), 1000);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_deposit_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &0);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_deposit_negative() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &-100);
}

#[test]
fn test_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &1000);
    client.withdraw(&user, &500);
    let position = client.get_user_position(&user);
    assert_eq!(position.deposit, 500);
    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&user), 500);
    assert_eq!(token_client.balance(&contract_id), 500);
}

#[test]
#[should_panic(expected = "insufficient deposit")]
fn test_withdraw_too_much() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &1000);
    client.withdraw(&user, &2000);
}

#[test]
fn test_borrow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &500);
    let position = client.get_user_position(&user);
    assert_eq!(position.borrow, 500);
    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&user), 500);
    assert_eq!(token_client.balance(&contract_id), 500);
}

#[test]
#[should_panic(expected = "exceeds borrow limit")]
fn test_borrow_too_much() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &950);
}

#[test]
fn test_repay() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &500);
    client.repay(&user, &300);
    let position = client.get_user_position(&user);
    assert_eq!(position.borrow, 200);
    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&user), 200);
    assert_eq!(token_client.balance(&contract_id), 800);
}

#[test]
fn test_repay_full() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &500);
    client.repay(&user, &600);
    let position = client.get_user_position(&user);
    assert_eq!(position.borrow, 0);
    // Only the outstanding 500 is pulled back, not the full 600 requested.
    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&user), 0);
    assert_eq!(token_client.balance(&contract_id), 1000);
}

#[test]
fn test_utilization_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &500);
    let utilization = client.get_utilization();
    assert_eq!(utilization, 50);
}

#[test]
fn test_interest_rate_model_below_kink() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    client.initialize(&token, &5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &400);
    let rate = client.get_borrow_rate();
    assert_eq!(rate, 10);
}

#[test]
fn test_interest_rate_model_above_kink() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LendingPool, ());
    let client = LendingPoolClient::new(&env, &contract_id);
    let token = create_token(&env);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&user, &1000);
    // Kink at 75%: max per-user borrow is 80%, so 80% pool utilization is above kink.
    client.initialize(&token, &5, &10, &75);
    client.deposit(&user, &1000);
    client.borrow(&user, &800);
    let rate = client.get_borrow_rate();
    // excess_utilization = 5; rate = 5 + 10 + (5 * 200 / 25) = 55
    assert_eq!(rate, 55);
}

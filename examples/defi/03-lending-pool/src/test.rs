use super::*;
use soroban_sdk::{Address, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    client.initialize(&5, &10, &80);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    client.initialize(&5, &10, &80);
    client.initialize(&5, &10, &80);
}

#[test]
fn test_deposit() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    let position = client.get_user_position(&user);
    assert_eq!(position.deposit, 1000);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_deposit_zero() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &0);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_deposit_negative() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &-100);
}

#[test]
fn test_withdraw() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    client.withdraw(&user, &500);
    let position = client.get_user_position(&user);
    assert_eq!(position.deposit, 500);
}

#[test]
#[should_panic(expected = "insufficient deposit")]
fn test_withdraw_too_much() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    client.withdraw(&user, &2000);
}

#[test]
fn test_borrow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &500);
    let position = client.get_user_position(&user);
    assert_eq!(position.borrow, 500);
}

#[test]
#[should_panic(expected = "exceeds borrow limit")]
fn test_borrow_too_much() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &900);
}

#[test]
fn test_repay() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &500);
    client.repay(&user, &300);
    let position = client.get_user_position(&user);
    assert_eq!(position.borrow, 200);
}

#[test]
fn test_repay_full() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &500);
    client.repay(&user, &600);
    let position = client.get_user_position(&user);
    assert_eq!(position.borrow, 0);
}

#[test]
fn test_utilization_tracking() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &500);
    let utilization = client.get_utilization();
    assert_eq!(utilization, 50);
}

#[test]
fn test_interest_rate_model_below_kink() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &400);
    let rate = client.get_borrow_rate();
    assert_eq!(rate, 10);
}

#[test]
fn test_interest_rate_model_above_kink() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingPool);
    let client = LendingPoolClient::new(&env, &contract_id);
    let user = Address::random(&env);
    client.initialize(&5, &10, &80);
    client.deposit(&user, &1000);
    client.borrow(&user, &850);
    let rate = client.get_borrow_rate();
    assert_eq!(rate, 65);
}

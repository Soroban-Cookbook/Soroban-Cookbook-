use super::*;
use soroban_sdk::{Address, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);

    client.initialize(&80, &85, &10, &50);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);

    client.initialize(&80, &85, &10, &50);
    client.initialize(&80, &85, &10, &50);
}

#[test]
fn test_deposit_collateral() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &1000);

    let position = client.get_position(&user);
    assert_eq!(position.collateral, 1000);
    assert_eq!(position.debt, 0);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_deposit_zero() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &0);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_deposit_negative() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &-100);
}

#[test]
fn test_withdraw_collateral() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &1000);
    client.withdraw_collateral(&user, &500);

    let position = client.get_position(&user);
    assert_eq!(position.collateral, 500);
}

#[test]
#[should_panic(expected = "insufficient collateral")]
fn test_withdraw_too_much() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &1000);
    client.withdraw_collateral(&user, &2000);
}

#[test]
fn test_borrow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &1000);
    client.borrow(&user, &500);

    let position = client.get_position(&user);
    assert_eq!(position.debt, 500);
}

#[test]
#[should_panic(expected = "exceeds maximum borrow amount")]
fn test_borrow_too_much() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &1000);
    client.borrow(&user, &900);
}

#[test]
fn test_repay() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &1000);
    client.borrow(&user, &500);
    client.repay(&user, &300);

    let position = client.get_position(&user);
    assert_eq!(position.debt, 200);
}

#[test]
fn test_repay_full() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &1000);
    client.borrow(&user, &500);
    client.repay(&user, &600);

    let position = client.get_position(&user);
    assert_eq!(position.debt, 0);
}

#[test]
fn test_partial_liquidation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let borrower = Address::random(&env);
    let liquidator = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&borrower, &1000);
    client.borrow(&borrower, &800);

    client.liquidate(&liquidator, &borrower, &400);

    let borrower_position = client.get_position(&borrower);
    assert_eq!(borrower_position.debt, 400);
    assert_eq!(borrower_position.collateral, 1000 - 440);

    let liquidator_position = client.get_position(&liquidator);
    assert_eq!(liquidator_position.collateral, 440);
}

#[test]
fn test_liquidation_incentive() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let borrower = Address::random(&env);
    let liquidator = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&borrower, &1000);
    client.borrow(&borrower, &800);

    client.liquidate(&liquidator, &borrower, &100);

    let liquidator_position = client.get_position(&liquidator);
    assert_eq!(liquidator_position.collateral, 110);
}

#[test]
#[should_panic(expected = "position is healthy")]
fn test_liquidate_healthy() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let borrower = Address::random(&env);
    let liquidator = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&borrower, &2000);
    client.borrow(&borrower, &800);

    client.liquidate(&liquidator, &borrower, &400);
}

#[test]
fn test_health_factor_max() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    let health_factor = client.get_health_factor(&user);
    assert_eq!(health_factor, i128::MAX);
}

#[test]
fn test_health_factor_calculation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &1000);
    client.borrow(&user, &500);

    let health_factor = client.get_health_factor(&user);
    assert_eq!(health_factor, (1000 * 85) / (500 * 100));
}

#[test]
fn test_multiple_users() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user1 = Address::random(&env);
    let user2 = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user1, &1000);
    client.deposit_collateral(&user2, &2000);

    let position1 = client.get_position(&user1);
    let position2 = client.get_position(&user2);

    assert_eq!(position1.collateral, 1000);
    assert_eq!(position2.collateral, 2000);
}

#[test]
fn test_withdraw_after_repay() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&user, &1000);
    client.borrow(&user, &500);
    client.repay(&user, &500);
    client.withdraw_collateral(&user, &1000);

    let position = client.get_position(&user);
    assert_eq!(position.collateral, 0);
    assert_eq!(position.debt, 0);
}

#[test]
fn test_emergency_pause() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let admin = Address::random(&env);
    let user = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.set_emergency_pause(&admin, &true);

    let result = std::panic::catch_unwind(|| {
        client.deposit_collateral(&user, &1000);
    });
    assert!(result.is_err());
}

#[test]
fn test_emergency_liquidate() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let admin = Address::random(&env);
    let borrower = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&borrower, &1000);
    client.borrow(&borrower, &800);
    client.set_emergency_pause(&admin, &true);
    client.emergency_liquidate(&admin, &borrower);

    let position = client.get_position(&borrower);
    assert_eq!(position.collateral, 0);
    assert_eq!(position.debt, 0);
}

#[test]
#[should_panic(expected = "not in emergency mode")]
fn test_emergency_liquidate_not_paused() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    let admin = Address::random(&env);
    let borrower = Address::random(&env);

    client.initialize(&80, &85, &10, &50);
    client.deposit_collateral(&borrower, &1000);
    client.borrow(&borrower, &800);
    client.emergency_liquidate(&admin, &borrower);
}

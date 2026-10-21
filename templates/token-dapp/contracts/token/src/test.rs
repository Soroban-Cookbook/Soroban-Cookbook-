#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env, String};

#[test]
fn test_initialize_and_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TokenContract, ());
    let client = TokenContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let name = String::from_str(&env, "Community Token");
    let symbol = symbol_short!("COMM");
    let decimals = 7u32;
    let initial_supply = 1_000_000_0000000i128;

    client.initialize(&admin, &name, &symbol, &decimals, &initial_supply);

    assert_eq!(client.name(), name);
    assert_eq!(client.symbol(), symbol);
    assert_eq!(client.decimals(), 7);
    assert_eq!(client.total_supply(), initial_supply);
    assert_eq!(client.balance(&admin), initial_supply);
}

#[test]
fn test_transfer_and_burn() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TokenContract, ());
    let client = TokenContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let name = String::from_str(&env, "Test Token");
    let symbol = symbol_short!("TEST");

    client.initialize(&admin, &name, &symbol, &7, &1000);

    // Transfer from admin to user1
    client.transfer(&admin, &user1, &300);
    assert_eq!(client.balance(&admin), 700);
    assert_eq!(client.balance(&user1), 300);

    // Burn from user1
    client.burn(&user1, &100);
    assert_eq!(client.balance(&user1), 200);
    assert_eq!(client.total_supply(), 900);
}

#[test]
fn test_allowance_and_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TokenContract, ());
    let client = TokenContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let name = String::from_str(&env, "Test Token");
    let symbol = symbol_short!("TEST");

    client.initialize(&admin, &name, &symbol, &7, &1000);

    // Admin approves spender for 400 tokens until ledger 1000
    client.approve(&admin, &spender, &400, &1000);
    assert_eq!(client.allowance(&admin, &spender), 400);

    // Spender transfers 250 on behalf of admin
    client.transfer_from(&spender, &admin, &recipient, &250);
    assert_eq!(client.balance(&admin), 750);
    assert_eq!(client.balance(&recipient), 250);
    assert_eq!(client.allowance(&admin, &spender), 150);
}

#[test]
fn test_mint_admin_only() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TokenContract, ());
    let client = TokenContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let name = String::from_str(&env, "Test Token");
    let symbol = symbol_short!("TEST");

    client.initialize(&admin, &name, &symbol, &7, &1000);

    client.mint(&admin, &user1, &500);
    assert_eq!(client.balance(&user1), 500);
    assert_eq!(client.total_supply(), 1500);
}

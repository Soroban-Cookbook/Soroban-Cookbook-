#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_get_invoker_success() {
    let env = Env::default();
    let user_address = Address::generate(&env);
    let contract_id = env.register_contract(None, AuthContextContract);
    let client = AuthContextContractClient::new(&env, &contract_id);

    env.mock_all_auths();
    let returned_invoker = client.get_invoker(&user_address);
    assert_eq!(returned_invoker, user_address);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_get_invoker_unauthorized() {
    let env = Env::default();
    let user_address = Address::generate(&env);
    let contract_id = env.register_contract(None, AuthContextContract);
    let client = AuthContextContractClient::new(&env, &contract_id);

    // No `env.mock_all_auths()`, so this should panic
    client.get_invoker(&user_address);
}

#[test]
fn test_get_current_address() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContextContract);
    let client = AuthContextContractClient::new(&env, &contract_id);
    let current = client.get_current_address();
    assert_eq!(current, contract_id);
}

#[test]
fn test_admin_only_op_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContextContract);
    let client = AuthContextContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    let result = client.admin_only_op(&admin, &admin);
    assert!(result);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_admin_only_op_unauthorized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContextContract);
    let client = AuthContextContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    // No auth
    client.admin_only_op(&admin, &admin);
}

#[test]
fn test_admin_only_op_wrong_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthContextContract);
    let client = AuthContextContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let hacker = Address::generate(&env);

    env.mock_all_auths();
    let result = client.admin_only_op(&hacker, &admin);
    assert!(!result);
}

#[test]
fn test_proxy_call_success() {
    let env = Env::default();
    let user_address = Address::generate(&env);
    let contract_id = env.register_contract(None, AuthContextContract);
    let proxy_id = env.register_contract(None, ProxyContract);
    let proxy_client = ProxyContractClient::new(&env, &proxy_id);

    env.mock_all_auths();
    let returned_invoker = proxy_client.proxy_call(&contract_id, &user_address);
    // Because the proxy is passing `user_address` directly and that authorizes it,
    // the target returns the proxy address as the immediate caller if it fetched it natively.
    // Wait, the lib says `client.get_invoker(&user)` is called, so it passes user.
    // The target contract `get_invoker` expects `invoker` matches what was passed, which is `user_address`.
    assert_eq!(returned_invoker, user_address);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_proxy_call_unauthorized() {
    let env = Env::default();
    let user_address = Address::generate(&env);
    let contract_id = env.register_contract(None, AuthContextContract);
    let proxy_id = env.register_contract(None, ProxyContract);
    let proxy_client = ProxyContractClient::new(&env, &proxy_id);

    // No mock_all_auths
    proxy_client.proxy_call(&contract_id, &user_address);
}

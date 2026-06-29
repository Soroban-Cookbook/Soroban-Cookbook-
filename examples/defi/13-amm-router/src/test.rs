use super::*;
use soroban_sdk::{Address, Env, Vec};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AMMRouter);
    let client = AMMRouterClient::new(&env, &contract_id);
    client.initialize();
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AMMRouter);
    let client = AMMRouterClient::new(&env, &contract_id);
    client.initialize();
    client.initialize();
}

#[test]
fn test_add_pool() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AMMRouter);
    let client = AMMRouterClient::new(&env, &contract_id);
    let token_a = Address::random(&env);
    let token_b = Address::random(&env);

    client.initialize();
    client.add_pool(&Pool {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        reserve_a: 1000,
        reserve_b: 1000,
    });

    let pool = client.get_pool(&token_a, &token_b);
    assert!(pool.is_some());
}

#[test]
fn test_single_hop_swap() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AMMRouter);
    let client = AMMRouterClient::new(&env, &contract_id);
    let user = Address::random(&env);
    let to = Address::random(&env);
    let token_a = Address::random(&env);
    let token_b = Address::random(&env);

    client.initialize();
    client.add_pool(&Pool {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        reserve_a: 1000,
        reserve_b: 1000,
    });

    let mut path = Vec::new(&env);
    path.push_back(token_a.clone());
    path.push_back(token_b.clone());

    let amount_out = client.swap_exact_tokens_for_tokens(
        &user,
        &100,
        &90,
        &path,
        &to,
        &(env.ledger().timestamp() + 100),
    );

    assert!(amount_out >= 90);
}

#[test]
fn test_multi_hop_swap() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AMMRouter);
    let client = AMMRouterClient::new(&env, &contract_id);
    let user = Address::random(&env);
    let to = Address::random(&env);
    let token_a = Address::random(&env);
    let token_b = Address::random(&env);
    let token_c = Address::random(&env);

    client.initialize();
    client.add_pool(&Pool {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        reserve_a: 1000,
        reserve_b: 1000,
    });
    client.add_pool(&Pool {
        token_a: token_b.clone(),
        token_b: token_c.clone(),
        reserve_a: 1000,
        reserve_b: 1000,
    });

    let mut path = Vec::new(&env);
    path.push_back(token_a.clone());
    path.push_back(token_b.clone());
    path.push_back(token_c.clone());

    let amount_out = client.swap_exact_tokens_for_tokens(
        &user,
        &100,
        &80,
        &path,
        &to,
        &(env.ledger().timestamp() + 100),
    );

    assert!(amount_out >= 80);
}

#[test]
#[should_panic(expected = "deadline exceeded")]
fn test_deadline_exceeded() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AMMRouter);
    let client = AMMRouterClient::new(&env, &contract_id);
    let user = Address::random(&env);
    let to = Address::random(&env);
    let token_a = Address::random(&env);
    let token_b = Address::random(&env);

    client.initialize();
    client.add_pool(&Pool {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        reserve_a: 1000,
        reserve_b: 1000,
    });

    let mut path = Vec::new(&env);
    path.push_back(token_a.clone());
    path.push_back(token_b.clone());

    client.swap_exact_tokens_for_tokens(
        &user,
        &100,
        &90,
        &path,
        &to,
        &(env.ledger().timestamp() - 1),
    );
}

#[test]
#[should_panic(expected = "insufficient output amount")]
fn test_slippage_control() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AMMRouter);
    let client = AMMRouterClient::new(&env, &contract_id);
    let user = Address::random(&env);
    let to = Address::random(&env);
    let token_a = Address::random(&env);
    let token_b = Address::random(&env);

    client.initialize();
    client.add_pool(&Pool {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        reserve_a: 1000,
        reserve_b: 1000,
    });

    let mut path = Vec::new(&env);
    path.push_back(token_a.clone());
    path.push_back(token_b.clone());

    client.swap_exact_tokens_for_tokens(
        &user,
        &100,
        &200,
        &path,
        &to,
        &(env.ledger().timestamp() + 100),
    );
}

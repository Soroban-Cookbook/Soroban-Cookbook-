#![allow(deprecated)]
use super::*;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, Vec};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AMMRouter, ());
    let client = AMMRouterClient::new(&env, &contract_id);
    client.initialize(&Address::generate(&env));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AMMRouter, ());
    let client = AMMRouterClient::new(&env, &contract_id);
    client.initialize(&Address::generate(&env));
    client.initialize(&Address::generate(&env));
}

#[test]
fn test_add_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AMMRouter, ());
    let client = AMMRouterClient::new(&env, &contract_id);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    client.initialize(&Address::generate(&env));
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
    env.mock_all_auths();
    let contract_id = env.register(AMMRouter, ());
    let client = AMMRouterClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let to = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    client.initialize(&Address::generate(&env));
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
    env.mock_all_auths();
    let contract_id = env.register(AMMRouter, ());
    let client = AMMRouterClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let to = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let token_c = Address::generate(&env);

    client.initialize(&Address::generate(&env));
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
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 100);
    let contract_id = env.register(AMMRouter, ());
    let client = AMMRouterClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let to = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    client.initialize(&Address::generate(&env));
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
    env.mock_all_auths();
    let contract_id = env.register(AMMRouter, ());
    let client = AMMRouterClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let to = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    client.initialize(&Address::generate(&env));
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

#[test]
fn test_pool_count_beyond_640() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AMMRouter, ());
    let client = AMMRouterClient::new(&env, &contract_id);
    client.initialize(&Address::generate(&env));

    // 650 pools: a monolithic Map would exceed Soroban's 64 KiB per-entry
    // limit around ~640 pools; individual persistent entries scale fine.
    let base = Address::generate(&env);
    let mut tokens: Vec<Address> = Vec::new(&env);
    for _ in 0..650 {
        tokens.push_back(Address::generate(&env));
    }
    for token in tokens.iter() {
        client.add_pool(&Pool {
            token_a: base.clone(),
            token_b: token,
            reserve_a: 1000,
            reserve_b: 1000,
        });
    }

    assert!(client.get_pool(&base, &tokens.get(0).unwrap()).is_some());
    assert!(client.get_pool(&base, &tokens.get(324).unwrap()).is_some());
    assert!(client.get_pool(&base, &tokens.get(649).unwrap()).is_some());
    assert!(client.get_pool(&base, &Address::generate(&env)).is_none());
}

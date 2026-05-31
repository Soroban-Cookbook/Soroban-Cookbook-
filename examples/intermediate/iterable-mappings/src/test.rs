use super::*;
use soroban_sdk::{Env, Symbol};

fn key(env: &Env, value: &str) -> Symbol {
    Symbol::new(env, value)
}

#[test]
fn test_set_and_get_roundtrip() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let alpha = key(&env, "alpha");
    client.set(&alpha, &9);

    assert_eq!(client.get(&alpha), Some(9));
    assert_eq!(client.len(), 1);
}

#[test]
fn test_keys_and_values_paginate_in_order() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let alpha = key(&env, "alpha");
    let beta = key(&env, "beta");
    let gamma = key(&env, "gamma");

    client.set(&alpha, &10);
    client.set(&beta, &20);
    client.set(&gamma, &30);

    let first_page = client.keys(&1, &2);
    assert_eq!(first_page.len(), 2);
    assert_eq!(first_page.get(0).unwrap(), alpha);
    assert_eq!(first_page.get(1).unwrap(), beta);

    let second_page = client.keys(&2, &2);
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page.get(0).unwrap(), gamma);

    let first_values = client.values(&1, &2);
    assert_eq!(first_values.get(0).unwrap(), 10u32);
    assert_eq!(first_values.get(1).unwrap(), 20u32);
}

#[test]
fn test_remove_keeps_index_consistent() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let alpha = key(&env, "alpha");
    let beta = key(&env, "beta");

    client.set(&alpha, &1);
    client.set(&beta, &2);
    client.remove(&alpha);

    assert_eq!(client.len(), 1);
    assert_eq!(client.get(&alpha), None);
    assert_eq!(client.keys(&1, &10).get(0).unwrap(), beta);
}

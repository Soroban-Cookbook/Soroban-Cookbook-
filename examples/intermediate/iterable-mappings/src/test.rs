#![allow(deprecated)]
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

// -----------------------------------------------------------------------------
// FILTERING TESTS
// -----------------------------------------------------------------------------

#[test]
fn test_filter_by_min_value_empty_map() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let filtered = client.filter_by_min_value(&10);
    assert!(filtered.is_empty());
}

#[test]
fn test_filter_by_min_value_no_matches() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    client.set(&key(&env, "a"), &5);
    client.set(&key(&env, "b"), &8);

    let filtered = client.filter_by_min_value(&10);
    assert!(filtered.is_empty());
}

#[test]
fn test_filter_by_min_value_all_matching() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let a = key(&env, "a");
    let b = key(&env, "b");
    client.set(&a, &10);
    client.set(&b, &20);

    let filtered = client.filter_by_min_value(&10);
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered.get(a), Some(10));
    assert_eq!(filtered.get(b), Some(20));
}

#[test]
fn test_filter_by_min_value_some_matching() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let a = key(&env, "a");
    let b = key(&env, "b");
    let c = key(&env, "c");
    client.set(&a, &5);
    client.set(&b, &15);
    client.set(&c, &25);

    let filtered = client.filter_by_min_value(&10);
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered.get(a), None);
    assert_eq!(filtered.get(b), Some(15));
    assert_eq!(filtered.get(c), Some(25));
}

#[test]
fn test_filter_by_page() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let a = key(&env, "a");
    let b = key(&env, "b");
    let c = key(&env, "c");
    client.set(&a, &10);
    client.set(&b, &5);
    client.set(&c, &20);

    // Page 1 with size 2 checks 'a' (10) and 'b' (5). Matching >= 10 is 'a' (10).
    let page1_filtered = client.filter_by_page(&10, &1, &2);
    assert_eq!(page1_filtered.len(), 1);
    assert_eq!(page1_filtered.get(a), Some(10));
    assert_eq!(page1_filtered.get(b), None);

    // Page 2 with size 2 checks 'c' (20). Matching >= 10 is 'c' (20).
    let page2_filtered = client.filter_by_page(&10, &2, &2);
    assert_eq!(page2_filtered.len(), 1);
    assert_eq!(page2_filtered.get(c), Some(20));
}

// -----------------------------------------------------------------------------
// MAPPING TESTS
// -----------------------------------------------------------------------------

#[test]
fn test_map_values_scale_empty_map() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let mapped = client.map_values_scale(&2);
    assert!(mapped.is_empty());
}

#[test]
fn test_map_values_scale_multiple() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let a = key(&env, "a");
    let b = key(&env, "b");
    client.set(&a, &10);
    client.set(&b, &25);

    let mapped = client.map_values_scale(&3);
    assert_eq!(mapped.len(), 2);
    assert_eq!(mapped.get(a), Some(30));
    assert_eq!(mapped.get(b), Some(75));
}

#[test]
fn test_map_values_scale_page() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let a = key(&env, "a");
    let b = key(&env, "b");
    let c = key(&env, "c");
    client.set(&a, &2);
    client.set(&b, &4);
    client.set(&c, &6);

    let page1_mapped = client.map_values_scale_page(&10, &1, &2);
    assert_eq!(page1_mapped.len(), 2);
    assert_eq!(page1_mapped.get(a), Some(20));
    assert_eq!(page1_mapped.get(b), Some(40));
    assert_eq!(page1_mapped.get(c), None);
}

#[test]
fn test_map_values_saturating_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    let a = key(&env, "a");
    client.set(&a, &u32::MAX);

    let mapped = client.map_values_scale(&2);
    assert_eq!(mapped.get(a), Some(u32::MAX));
}

// -----------------------------------------------------------------------------
// REDUCTION TESTS
// -----------------------------------------------------------------------------

#[test]
fn test_reduce_sum_empty_map() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    assert_eq!(client.reduce_sum(), 0u64);
}

#[test]
fn test_reduce_sum_single_entry() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    client.set(&key(&env, "a"), &42);
    assert_eq!(client.reduce_sum(), 42u64);
}

#[test]
fn test_reduce_sum_multiple_entries() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    client.set(&key(&env, "a"), &10);
    client.set(&key(&env, "b"), &20);
    client.set(&key(&env, "c"), &30);

    assert_eq!(client.reduce_sum(), 60u64);
}

#[test]
fn test_reduce_sum_page() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    client.set(&key(&env, "a"), &10);
    client.set(&key(&env, "b"), &20);
    client.set(&key(&env, "c"), &30);

    // Page 1 size 2 sums 'a' (10) + 'b' (20) = 30
    assert_eq!(client.reduce_sum_page(&1, &2), 30u64);
    // Page 2 size 2 sums 'c' (30) = 30
    assert_eq!(client.reduce_sum_page(&2, &2), 30u64);
}

#[test]
fn test_reduce_sum_large_values_u64() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IterableMappings);
    let client = IterableMappingsClient::new(&env, &contract_id);

    client.set(&key(&env, "a"), &u32::MAX);
    client.set(&key(&env, "b"), &u32::MAX);

    let expected = (u32::MAX as u64) + (u32::MAX as u64);
    assert_eq!(client.reduce_sum(), expected);
}

// -----------------------------------------------------------------------------
// GENERIC FUNCTIONAL HELPER TESTS
// -----------------------------------------------------------------------------

#[test]
fn test_functional_helpers_direct() {
    let env = Env::default();
    let mut entries = Map::new(&env);
    let mut keys = Vec::new(&env);

    let k1 = key(&env, "k1");
    let k2 = key(&env, "k2");
    let k3 = key(&env, "k3");

    entries.set(k1.clone(), 100);
    keys.push_back(k1.clone());

    entries.set(k2.clone(), 200);
    keys.push_back(k2.clone());

    entries.set(k3.clone(), 300);
    keys.push_back(k3.clone());

    // Test filter_by_predicate
    let filtered = filter_by_predicate(&entries, &keys, |_, val| val > 150);
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered.get(k1.clone()), None);
    assert_eq!(filtered.get(k2.clone()), Some(200));

    // Test transform_values
    let transformed = transform_values(&entries, &keys, |_, val| val / 10);
    assert_eq!(transformed.get(k1.clone()), Some(10));
    assert_eq!(transformed.get(k2.clone()), Some(20));

    // Test reduce_values
    let total = reduce_values(&entries, &keys, 0u64, |acc, _, val| acc + (val as u64));
    assert_eq!(total, 600u64);
}

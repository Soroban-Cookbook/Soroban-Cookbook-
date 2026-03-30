//! Tests for collection type operations.

use super::*;
use soroban_sdk::{symbol_short, vec, Env, Map, Symbol, Vec};

// ---------------------------------------------------------------------------
// Vec — storage operations
// ---------------------------------------------------------------------------

#[test]
fn test_vec_push_and_list() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    client.vec_push(&10);
    client.vec_push(&20);
    client.vec_push(&30);

    let list = client.vec_list();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0), Some(10));
    assert_eq!(list.get(1), Some(20));
    assert_eq!(list.get(2), Some(30));
}

#[test]
fn test_vec_pop() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    client.vec_push(&1);
    client.vec_push(&2);
    client.vec_push(&3);

    assert_eq!(client.vec_pop(), Some(3));
    assert_eq!(client.vec_pop(), Some(2));
    assert_eq!(client.vec_pop(), Some(1));
    assert_eq!(client.vec_pop(), None); // empty
}

// ---------------------------------------------------------------------------
// Vec — iteration patterns
// ---------------------------------------------------------------------------

#[test]
fn test_vec_sum() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let items = vec![&env, 1_i128, 2, 3, 4, 5];
    assert_eq!(client.vec_sum(&items), 15);
}

#[test]
fn test_vec_sum_empty() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let empty: Vec<i128> = Vec::new(&env);
    assert_eq!(client.vec_sum(&empty), 0);
}

#[test]
fn test_vec_filter_positive() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let items = vec![&env, -3_i128, 0, 5, -1, 10, 7];
    let result = client.vec_filter_positive(&items);

    assert_eq!(result.len(), 3);
    assert_eq!(result.get(0), Some(5));
    assert_eq!(result.get(1), Some(10));
    assert_eq!(result.get(2), Some(7));
}

#[test]
fn test_vec_max() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let items = vec![&env, 3_i128, 1, 4, 1, 5, 9, 2, 6];
    assert_eq!(client.vec_max(&items), Some(9));
}

#[test]
fn test_vec_max_empty() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let empty: Vec<i128> = Vec::new(&env);
    assert_eq!(client.vec_max(&empty), None);
}

#[test]
fn test_vec_contains() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let items = vec![&env, 10_i128, 20, 30];
    assert!(client.vec_contains(&items, &20));
    assert!(!client.vec_contains(&items, &99));
}

// ---------------------------------------------------------------------------
// Map — storage operations
// ---------------------------------------------------------------------------

#[test]
fn test_map_set_and_get() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    client.map_set(&symbol_short!("alice"), &100);
    client.map_set(&symbol_short!("bob"), &200);

    assert_eq!(client.map_get(&symbol_short!("alice")), Some(100));
    assert_eq!(client.map_get(&symbol_short!("bob")), Some(200));
    assert_eq!(client.map_get(&symbol_short!("carol")), None);
}

#[test]
fn test_map_remove() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    client.map_set(&symbol_short!("key"), &42);
    assert_eq!(client.map_get(&symbol_short!("key")), Some(42));

    client.map_remove(&symbol_short!("key"));
    assert_eq!(client.map_get(&symbol_short!("key")), None);
}

#[test]
fn test_map_overwrite() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    client.map_set(&symbol_short!("score"), &10);
    client.map_set(&symbol_short!("score"), &99); // overwrite
    assert_eq!(client.map_get(&symbol_short!("score")), Some(99));
}

// ---------------------------------------------------------------------------
// Map — iteration patterns
// ---------------------------------------------------------------------------

#[test]
fn test_map_sum_values() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let mut data: Map<Symbol, i128> = Map::new(&env);
    data.set(symbol_short!("a"), 100);
    data.set(symbol_short!("b"), 200);
    data.set(symbol_short!("c"), 300);

    assert_eq!(client.map_sum_values(&data), 600);
}

#[test]
fn test_map_keys_and_values() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let mut data: Map<Symbol, i128> = Map::new(&env);
    // Map returns keys in sorted order.
    data.set(symbol_short!("alpha"), 1);
    data.set(symbol_short!("beta"), 2);
    data.set(symbol_short!("gamma"), 3);

    let keys = client.map_keys(&data);
    assert_eq!(keys.len(), 3);

    let values = client.map_values(&data);
    assert_eq!(values.len(), 3);

    // Sum of values via map_values Vec.
    let sum: i128 = values.iter().sum();
    assert_eq!(sum, 6);
}

#[test]
fn test_map_max_key() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let mut data: Map<Symbol, i128> = Map::new(&env);
    data.set(symbol_short!("low"), 5);
    data.set(symbol_short!("high"), 1000);
    data.set(symbol_short!("mid"), 50);

    assert_eq!(client.map_max_key(&data), Some(symbol_short!("high")));
}

#[test]
fn test_map_max_key_empty() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let empty: Map<Symbol, i128> = Map::new(&env);
    assert_eq!(client.map_max_key(&empty), None);
}

// ---------------------------------------------------------------------------
// Cross-collection iteration patterns
// ---------------------------------------------------------------------------

#[test]
fn test_zip_to_map() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let keys = vec![
        &env,
        symbol_short!("x"),
        symbol_short!("y"),
        symbol_short!("z"),
    ];
    let values = vec![&env, 10_i128, 20, 30];

    let result = client.zip_to_map(&keys, &values);
    assert_eq!(result.len(), 3);
    assert_eq!(result.get(symbol_short!("x")), Some(10));
    assert_eq!(result.get(symbol_short!("y")), Some(20));
    assert_eq!(result.get(symbol_short!("z")), Some(30));
}

#[test]
fn test_zip_to_map_unequal_lengths() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let keys = vec![&env, symbol_short!("a"), symbol_short!("b")];
    let values = vec![&env, 1_i128, 2, 3]; // one extra value, ignored

    let result = client.zip_to_map(&keys, &values);
    assert_eq!(result.len(), 2); // only min(2, 3) = 2 pairs
}

#[test]
fn test_map_increment_values() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let mut data: Map<Symbol, i128> = Map::new(&env);
    data.set(symbol_short!("p"), 10);
    data.set(symbol_short!("q"), 20);
    data.set(symbol_short!("r"), 30);

    let result = client.map_increment_values(&data, &5);
    assert_eq!(result.get(symbol_short!("p")), Some(15));
    assert_eq!(result.get(symbol_short!("q")), Some(25));
    assert_eq!(result.get(symbol_short!("r")), Some(35));
    // Original is unchanged.
    assert_eq!(data.get(symbol_short!("p")), Some(10));
}

#[test]
fn test_map_increment_values_negative_delta() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CollectionTypesContract);
    let client = CollectionTypesContractClient::new(&env, &contract_id);

    let mut data: Map<Symbol, i128> = Map::new(&env);
    data.set(symbol_short!("bal"), 100);

    let result = client.map_increment_values(&data, &-10);
    assert_eq!(result.get(symbol_short!("bal")), Some(90));
}

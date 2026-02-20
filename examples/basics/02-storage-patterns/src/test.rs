//! Unit tests for Storage Patterns contract

#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, Env};

#[test]
fn test_persistent_storage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("balance");
    let value = 1000u64;

    // Initially, key should not exist
    assert!(!client.has_persistent(&key));

    // Set value
    client.set_persistent(&key, &value);

    // Key should now exist
    assert!(client.has_persistent(&key));

    // Retrieved value should match
    assert_eq!(client.get_persistent(&key), value);

    // Remove value
    client.remove_persistent(&key);

    // Key should no longer exist
    assert!(!client.has_persistent(&key));
}

#[test]
fn test_temporary_storage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("temp");
    let value = 42u64;

    // Initially, key should not exist
    assert!(!client.has_temporary(&key));

    // Set value
    client.set_temporary(&key, &value);

    // Key should now exist
    assert!(client.has_temporary(&key));

    // Retrieved value should match
    assert_eq!(client.get_temporary(&key), value);
}

#[test]
fn test_instance_storage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("config");
    let value = 999u64;

    // Initially, key should not exist
    assert!(!client.has_instance(&key));

    // Set value
    client.set_instance(&key, &value);

    // Key should now exist
    assert!(client.has_instance(&key));

    // Retrieved value should match
    assert_eq!(client.get_instance(&key), value);

    // Remove value
    client.remove_instance(&key);

    // Key should no longer exist
    assert!(!client.has_instance(&key));
}

#[test]
fn test_storage_isolation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("data");

    // Set different values in each storage type
    client.set_persistent(&key, &100);
    client.set_temporary(&key, &200);
    client.set_instance(&key, &300);

    // Each storage type should maintain its own value
    assert_eq!(client.get_persistent(&key), 100);
    assert_eq!(client.get_temporary(&key), 200);
    assert_eq!(client.get_instance(&key), 300);
}

#[test]
fn test_multiple_keys() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    // Store multiple key-value pairs
    let keys = [
        symbol_short!("key1"),
        symbol_short!("key2"),
        symbol_short!("key3"),
    ];

    for (i, key) in keys.iter().enumerate() {
        client.set_persistent(key, &((i as u64) * 100));
    }

    // Verify all values are correctly stored
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(client.get_persistent(key), (i as u64) * 100);
    }
}

#[test]
fn test_update_existing_value() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("counter");

    // Set initial value
    client.set_persistent(&key, &10);
    assert_eq!(client.get_persistent(&key), 10);

    // Update value
    client.set_persistent(&key, &20);
    assert_eq!(client.get_persistent(&key), 20);

    // Update again
    client.set_persistent(&key, &30);
    assert_eq!(client.get_persistent(&key), 30);
}

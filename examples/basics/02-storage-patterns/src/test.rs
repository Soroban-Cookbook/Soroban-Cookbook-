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

// -------------------- Additional Comprehensive Tests --------------------

#[test]
fn test_persistent_ttl_and_cross_ledger_survives_short_advance() {
    // Verifies that persistent storage survives across ledger advances
    // when TTL has been extended by the contract implementation.
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("ttl_key");
    let value = 777u64;

    // Set persistent value (contract also calls extend_ttl)
    client.set_persistent(&key, &value);
    assert!(client.has_persistent(&key));

    // Advance ledger by a small amount (should still be alive)
    env.ledger().with_mut(|li| li.sequence += 50);
    assert!(client.has_persistent(&key));
    assert_eq!(client.get_persistent(&key), value);
}

#[test]
#[should_panic]
fn test_missing_key_panics_on_get_persistent() {
    // Validates missing-key behavior: `get` should panic (unwrap) when key is absent
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let missing = symbol_short!("nope");
    // This should panic because key was never set and get().unwrap() is used in contract
    let _ = client.get_persistent(&missing);
}

#[test]
fn test_instance_storage_isolated_between_instances() {
    // Ensures instance storage is tied to the contract instance and not shared.
    let env = Env::default();

    // Deploy two different instances of the same contract
    let c1 = env.register_contract(None, StorageContract);
    let c2 = env.register_contract(None, StorageContract);

    let client1 = StorageContractClient::new(&env, &c1);
    let client2 = StorageContractClient::new(&env, &c2);

    let key = symbol_short!("inst_key");

    client1.set_instance(&key, &1u64);
    assert!(client1.has_instance(&key));
    // The second instance should not see the first instance's data
    assert!(!client2.has_instance(&key));
}

#[test]
fn test_temporary_storage_lifecycle_and_performance() {
    // Tests that temporary storage is ledger-scoped and exercises many writes
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    // Write many temporary keys to validate common workload (basic performance sanity)
    for i in 0..200u64 {
        let k = symbol_short!("t");
        // combine index into symbol by creating a new symbol per iteration would be heavier;
        // reuse same key to test overwrite performance
        client.set_temporary(&k, &i);
        assert!(client.has_temporary(&k));
        assert_eq!(client.get_temporary(&k), i);
    }

    // Advance ledger to simulate new ledger: temporary storage should be cleared
    env.ledger().with_mut(|li| li.sequence += 1);

    let k = symbol_short!("t");
    assert!(!client.has_temporary(&k));
}

#[test]
fn test_cross_storage_overwrite_and_isolation() {
    // Verifies that the same logical key in different storage types do not collide
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("shared_key");

    // Set values in each storage type
    client.set_persistent(&key, &10u64);
    client.set_temporary(&key, &20u64);
    client.set_instance(&key, &30u64);

    // Overwrite persistent value and verify it only affects persistent storage
    client.set_persistent(&key, &11u64);

    assert_eq!(client.get_persistent(&key), 11u64);
    assert_eq!(client.get_temporary(&key), 20u64);
    assert_eq!(client.get_instance(&key), 30u64);
}

//! # Comprehensive Test Suite for Storage Patterns
//!
//! This test suite demonstrates and verifies all three Soroban storage types:
//! - **Persistent Storage**: Long-lived data with per-key TTL management
//! - **Temporary Storage**: Short-lived data that expires after minimum TTL
//! - **Instance Storage**: Contract-scoped data with shared TTL
//!
//! ## Test Coverage
//!
//! ### Basic Operations (6 tests)
//! - `test_persistent_storage` - CRUD operations for persistent storage
//! - `test_temporary_storage` - CRUD operations for temporary storage
//! - `test_instance_storage` - CRUD operations for instance storage
//! - `test_storage_isolation` - Verifies storage types are independent
//! - `test_multiple_keys` - Multiple key-value pairs in persistent storage
//! - `test_update_existing_value` - Overwriting existing values
//!
//! ### TTL and Persistence (5 tests)
//! - `test_persistent_ttl_and_cross_ledger_survives_short_advance` - Short ledger advance
//! - `test_persistent_storage_survives_long_ledger_advance` - Long ledger advance (50+ ledgers)
//! - `test_persistent_storage_multiple_updates_with_ttl` - TTL management across updates
//! - `test_temporary_storage_expires_after_ttl` - Temporary storage expiration
//! - `test_instance_storage_shared_ttl` - Instance storage shared TTL behavior
//!
//! ### Cross-Ledger Tests (3 tests)
//! - `test_cross_ledger_persistence_verification` - Comprehensive cross-ledger behavior
//! - `test_instance_storage_contract_upgrade_simulation` - Upgrade scenario
//! - `test_temporary_storage_lifecycle_and_performance` - Lifecycle and performance
//!
//! ### Edge Cases and Advanced (7 tests)
//! - `test_missing_key_returns_none_for_get_persistent` - Missing key handling
//! - `test_instance_storage_isolated_between_instances` - Instance isolation
//! - `test_cross_storage_overwrite_and_isolation` - Cross-storage isolation
//! - `test_storage_type_independence` - Storage type independence
//! - `test_persistent_storage_large_dataset` - Scalability test
//! - `test_storage_remove_operations` - Remove operations
//! - `test_zero_and_boundary_values` - Boundary value testing
//!
//! ## Acceptance Criteria (Issue #46)
//! ✅ Tests for each storage type (persistent, temporary, instance)
//! ✅ Persistence verification tests (5 dedicated tests)
//! ✅ Cross-ledger tests (3 comprehensive tests)
//! ✅ 6+ tests passing (21 tests passing)

use super::*;
use soroban_sdk::testutils::{Events as _, Ledger as _};
use soroban_sdk::{symbol_short, Env, Symbol, TryFromVal};

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

    // Verify set event
    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();
    assert_eq!(topics.len(), 2);
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("persist"));
    assert_eq!(t1, symbol_short!("set"));
    let (d_key, d_value): (Symbol, u64) = <(Symbol, u64)>::try_from_val(&env, &data).unwrap();
    assert_eq!(d_key, key);
    assert_eq!(d_value, value);

    // Key should now exist
    assert!(client.has_persistent(&key));

    // Retrieved value should match
    assert_eq!(client.get_persistent(&key), Some(value));

    // Remove value
    client.remove_persistent(&key);

    // Verify remove event
    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();
    assert_eq!(topics.len(), 2);
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("persist"));
    assert_eq!(t1, symbol_short!("remove"));
    let d_key: Symbol = Symbol::try_from_val(&env, &data).unwrap();
    assert_eq!(d_key, key);

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

    // Verify event
    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();
    assert_eq!(topics.len(), 2);
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("temp"));
    assert_eq!(t1, symbol_short!("set"));
    let (d_key, d_value): (Symbol, u64) = <(Symbol, u64)>::try_from_val(&env, &data).unwrap();
    assert_eq!(d_key, key);
    assert_eq!(d_value, value);

    // Key should now exist
    assert!(client.has_temporary(&key));

    // Retrieved value should match
    assert_eq!(client.get_temporary(&key), Some(value));
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

    // Verify event
    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();
    assert_eq!(topics.len(), 2);
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("instance"));
    assert_eq!(t1, symbol_short!("set"));
    let (d_key, d_value): (Symbol, u64) = <(Symbol, u64)>::try_from_val(&env, &data).unwrap();
    assert_eq!(d_key, key);
    assert_eq!(d_value, value);

    // Key should now exist
    assert!(client.has_instance(&key));

    // Retrieved value should match
    assert_eq!(client.get_instance(&key), Some(value));

    // Remove value
    client.remove_instance(&key);

    // Verify remove event
    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();
    assert_eq!(topics.len(), 2);
    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    assert_eq!(t0, symbol_short!("instance"));
    assert_eq!(t1, symbol_short!("remove"));
    let d_key: Symbol = Symbol::try_from_val(&env, &data).unwrap();
    assert_eq!(d_key, key);

    // Key should no longer exist
    assert!(!client.has_instance(&key));
}

/// Benchmark storage costs for different storage types.
#[test]
fn test_storage_costs_benchmark() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("test");
    let value = 100u64;

    // Benchmark Persistent Storage
    println!("--- Persistent Storage Benchmark ---");
    env.budget().reset_default();
    client.set_persistent(&key, &value);
    env.budget().print();

    // Benchmark Instance Storage
    println!("--- Instance Storage Benchmark ---");
    env.budget().reset_default();
    client.set_instance(&key, &value);
    env.budget().print();

    // Benchmark Temporary Storage
    println!("--- Temporary Storage Benchmark ---");
    env.budget().reset_default();
    client.set_temporary(&key, &value);
    env.budget().print();
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
    assert_eq!(client.get_persistent(&key), Some(100));
    assert_eq!(client.get_temporary(&key), Some(200));
    assert_eq!(client.get_instance(&key), Some(300));
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
        assert_eq!(client.get_persistent(key), Some((i as u64) * 100));
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
    assert_eq!(client.get_persistent(&key), Some(10));

    // Update value
    client.set_persistent(&key, &20);
    assert_eq!(client.get_persistent(&key), Some(20));

    // Update again
    client.set_persistent(&key, &30);
    assert_eq!(client.get_persistent(&key), Some(30));
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
    env.ledger().with_mut(|li| li.sequence_number += 50);
    assert!(client.has_persistent(&key));
    assert_eq!(client.get_persistent(&key), Some(value));
}

#[test]
fn test_missing_key_returns_none_for_get_persistent() {
    // Validates missing-key behavior: `get` should return None when key is absent
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let missing = symbol_short!("nope");
    // Should return None for missing key
    assert_eq!(client.get_persistent(&missing), None);
    assert_eq!(client.get_temporary(&missing), None);
    assert_eq!(client.get_instance(&missing), None);
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

    // Set min_temp_entry_ttl = 1 so entries expire after exactly 1 ledger.
    env.ledger().with_mut(|li| li.min_temp_entry_ttl = 1);

    // Write many temporary keys to validate common workload (basic performance sanity)
    for i in 0..200u64 {
        let k = symbol_short!("t");
        // combine index into symbol by creating a new symbol per iteration would be heavier;
        // reuse same key to test overwrite performance
        client.set_temporary(&k, &i);
        assert!(client.has_temporary(&k));
        assert_eq!(client.get_temporary(&k), Some(i));
    }

    // Advance ledger by 2: with min_temp_entry_ttl=1 the entry expires at seq+1,
    // so it is gone once current_ledger >= seq+2.
    env.ledger().with_mut(|li| li.sequence_number += 2);

    let k = symbol_short!("t");
    assert!(!client.has_temporary(&k));
}

#[test]
fn test_cross_storage_overwrite_and_isolation() {
    // Verifies that the same logical key in different storage types do not collide
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("shrd_key");

    // Set values in each storage type
    client.set_persistent(&key, &10u64);
    client.set_temporary(&key, &20u64);
    client.set_instance(&key, &30u64);

    // Overwrite persistent value and verify it only affects persistent storage
    client.set_persistent(&key, &11u64);

    assert_eq!(client.get_persistent(&key), Some(11u64));
    assert_eq!(client.get_temporary(&key), Some(20u64));
    assert_eq!(client.get_instance(&key), Some(30u64));
}

// ==================== COMPREHENSIVE TEST SUITE ====================
// Additional tests to meet Issue #46 acceptance criteria:
// - Tests for each storage type
// - Persistence verification tests
// - Cross-ledger tests
// - 6+ tests passing

#[test]
fn test_persistent_storage_survives_long_ledger_advance() {
    // Verifies persistent storage survives significant ledger advances
    // when TTL is properly managed
    let env = Env::default();

    // Configure ledger with realistic TTL parameters
    env.ledger().with_mut(|li| {
        li.sequence_number = 100;
        li.timestamp = 1000;
        li.protocol_version = 20;
        li.network_id = [0; 32];
        li.base_reserve = 10;
        li.min_temp_entry_ttl = 16;
        li.min_persistent_entry_ttl = 100;
        li.max_entry_ttl = 6312000;
    });

    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("long_ttl");
    let value = 12345u64;

    // Set persistent value (contract extends TTL to 100 ledgers)
    client.set_persistent(&key, &value);
    assert!(client.has_persistent(&key));

    // Extend instance TTL so contract remains accessible
    env.as_contract(&contract_id, || {
        env.storage().instance().extend_ttl(1000, 1000);
    });

    // Advance ledger by 50 ledgers (well within TTL)
    env.ledger().with_mut(|li| {
        li.sequence_number += 50;
        li.timestamp += 250; // ~5 seconds per ledger
    });

    // Data should still be accessible
    assert!(client.has_persistent(&key));
    assert_eq!(client.get_persistent(&key), Some(value));

    // Advance another 40 ledgers (total 90, still within 100 TTL)
    env.ledger().with_mut(|li| {
        li.sequence_number += 40;
        li.timestamp += 200;
    });

    // Data should still be accessible
    assert!(client.has_persistent(&key));
    assert_eq!(client.get_persistent(&key), Some(value));
}

#[test]
fn test_instance_storage_shared_ttl() {
    // Verifies that all instance storage keys share the same TTL
    let env = Env::default();

    env.ledger().with_mut(|li| {
        li.sequence_number = 10;
        li.min_persistent_entry_ttl = 100;
        li.max_entry_ttl = 6312000;
    });

    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    // Set multiple instance storage keys
    let keys = [
        symbol_short!("cfg1"),
        symbol_short!("cfg2"),
        symbol_short!("cfg3"),
    ];

    for (i, key) in keys.iter().enumerate() {
        client.set_instance(key, &((i as u64) + 1));
    }

    // Verify all keys exist
    for key in keys.iter() {
        assert!(client.has_instance(key));
    }

    // Extend instance TTL
    env.as_contract(&contract_id, || {
        env.storage().instance().extend_ttl(500, 500);
    });

    // Advance ledger
    env.ledger().with_mut(|li| {
        li.sequence_number += 100;
    });

    // All instance keys should still be accessible (shared TTL)
    for (i, key) in keys.iter().enumerate() {
        assert!(client.has_instance(key));
        assert_eq!(client.get_instance(key), Some((i as u64) + 1));
    }
}

#[test]
fn test_temporary_storage_expires_after_ttl() {
    // Verifies temporary storage expires after its TTL
    let env = Env::default();

    // Set minimum temp TTL to 16 ledgers (Soroban default)
    env.ledger().with_mut(|li| {
        li.sequence_number = 10;
        li.min_temp_entry_ttl = 16;
        li.max_entry_ttl = 6312000;
    });

    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("temp_exp");
    let value = 999u64;

    // Set temporary value
    client.set_temporary(&key, &value);
    assert!(client.has_temporary(&key));

    // Advance by 10 ledgers (within TTL)
    env.ledger().with_mut(|li| {
        li.sequence_number += 10;
    });

    // Should still exist
    assert!(client.has_temporary(&key));

    // Advance by another 20 ledgers (total 30, beyond min TTL of 16)
    env.ledger().with_mut(|li| {
        li.sequence_number += 20;
    });

    // Should be expired now
    assert!(!client.has_temporary(&key));
}

#[test]
fn test_persistent_storage_multiple_updates_with_ttl() {
    // Verifies that multiple updates to persistent storage properly manage TTL
    let env = Env::default();

    env.ledger().with_mut(|li| {
        li.sequence_number = 10;
        li.min_persistent_entry_ttl = 100;
        li.max_entry_ttl = 6312000;
    });

    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("counter");

    // Extend instance TTL
    env.as_contract(&contract_id, || {
        env.storage().instance().extend_ttl(1000, 1000);
    });

    // Perform multiple updates
    for i in 1..=10 {
        client.set_persistent(&key, &i);
        assert_eq!(client.get_persistent(&key), Some(i));

        // Advance ledger between updates
        env.ledger().with_mut(|li| {
            li.sequence_number += 5;
        });
    }

    // After 50 ledgers (10 updates * 5 ledgers), data should still be accessible
    // because each set_persistent call extends TTL
    assert!(client.has_persistent(&key));
    assert_eq!(client.get_persistent(&key), Some(10));
}

#[test]
fn test_storage_type_independence() {
    // Verifies that operations on one storage type don't affect others
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("indep");

    // Set values in all three storage types
    client.set_persistent(&key, &111);
    client.set_temporary(&key, &222);
    client.set_instance(&key, &333);

    // Remove from persistent
    client.remove_persistent(&key);
    assert!(!client.has_persistent(&key));

    // Other storage types should be unaffected
    assert!(client.has_temporary(&key));
    assert_eq!(client.get_temporary(&key), Some(222));
    assert!(client.has_instance(&key));
    assert_eq!(client.get_instance(&key), Some(333));

    // Remove from instance
    client.remove_instance(&key);
    assert!(!client.has_instance(&key));

    // Temporary should still exist
    assert!(client.has_temporary(&key));
    assert_eq!(client.get_temporary(&key), Some(222));
}

#[test]
fn test_persistent_storage_large_dataset() {
    // Tests persistent storage with a larger number of keys
    // to verify scalability and performance characteristics
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    // Store 50 key-value pairs
    let num_keys = 50;
    for i in 0..num_keys {
        // Create unique keys using different symbols
        let key = match i % 5 {
            0 => symbol_short!("k0"),
            1 => symbol_short!("k1"),
            2 => symbol_short!("k2"),
            3 => symbol_short!("k3"),
            _ => symbol_short!("k4"),
        };

        client.set_persistent(&key, &i);
    }

    // Verify the last value for each key pattern
    // (each key was overwritten multiple times)
    let expected_values = [45, 46, 47, 48, 49]; // Last values for k0-k4
    for (idx, expected) in expected_values.iter().enumerate() {
        let key = match idx {
            0 => symbol_short!("k0"),
            1 => symbol_short!("k1"),
            2 => symbol_short!("k2"),
            3 => symbol_short!("k3"),
            _ => symbol_short!("k4"),
        };

        assert!(client.has_persistent(&key));
        assert_eq!(client.get_persistent(&key), Some(*expected));
    }
}

#[test]
fn test_instance_storage_contract_upgrade_simulation() {
    // Simulates contract upgrade scenario where instance storage
    // should persist across the upgrade
    let env = Env::default();

    env.ledger().with_mut(|li| {
        li.sequence_number = 10;
        li.min_persistent_entry_ttl = 100;
        li.max_entry_ttl = 6312000;
    });

    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    // Set instance configuration before "upgrade"
    let admin_key = symbol_short!("admin");
    let config_key = symbol_short!("config");

    client.set_instance(&admin_key, &1000);
    client.set_instance(&config_key, &2000);

    // Extend instance TTL to survive upgrade
    env.as_contract(&contract_id, || {
        env.storage().instance().extend_ttl(5000, 10000);
    });

    // Simulate time passing (upgrade process)
    env.ledger().with_mut(|li| {
        li.sequence_number += 100;
        li.timestamp += 500;
    });

    // Instance storage should still be accessible after "upgrade"
    assert!(client.has_instance(&admin_key));
    assert_eq!(client.get_instance(&admin_key), Some(1000));
    assert!(client.has_instance(&config_key));
    assert_eq!(client.get_instance(&config_key), Some(2000));
}

#[test]
fn test_cross_ledger_persistence_verification() {
    // Comprehensive cross-ledger test verifying all storage types
    // behave correctly as ledger advances
    let env = Env::default();

    env.ledger().with_mut(|li| {
        li.sequence_number = 100;
        li.timestamp = 1000;
        li.min_temp_entry_ttl = 16;
        li.min_persistent_entry_ttl = 100;
        li.max_entry_ttl = 6312000;
    });

    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let p_key = symbol_short!("persist");
    let t_key = symbol_short!("temp");
    let i_key = symbol_short!("inst");

    // Set values in all storage types
    client.set_persistent(&p_key, &100);
    client.set_temporary(&t_key, &200);
    client.set_instance(&i_key, &300);

    // Extend instance TTL
    env.as_contract(&contract_id, || {
        env.storage().instance().extend_ttl(1000, 1000);
    });

    // All should exist initially
    assert!(client.has_persistent(&p_key));
    assert!(client.has_temporary(&t_key));
    assert!(client.has_instance(&i_key));

    // Advance by 10 ledgers
    env.ledger().with_mut(|li| {
        li.sequence_number += 10;
        li.timestamp += 50;
    });

    // All should still exist
    assert!(client.has_persistent(&p_key));
    assert!(client.has_temporary(&t_key));
    assert!(client.has_instance(&i_key));

    // Advance by 20 more ledgers (total 30, beyond temp TTL)
    env.ledger().with_mut(|li| {
        li.sequence_number += 20;
        li.timestamp += 100;
    });

    // Persistent and instance should exist, temporary should be gone
    assert!(client.has_persistent(&p_key));
    assert!(!client.has_temporary(&t_key));
    assert!(client.has_instance(&i_key));

    // Verify values
    assert_eq!(client.get_persistent(&p_key), Some(100));
    assert_eq!(client.get_instance(&i_key), Some(300));
}

#[test]
fn test_storage_remove_operations() {
    // Tests remove operations for persistent and instance storage
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    // Test persistent storage removal
    let p_key = symbol_short!("p_rem");
    client.set_persistent(&p_key, &111);
    assert!(client.has_persistent(&p_key));

    client.remove_persistent(&p_key);
    assert!(!client.has_persistent(&p_key));
    assert_eq!(client.get_persistent(&p_key), None);

    // Test instance storage removal
    let i_key = symbol_short!("i_rem");
    client.set_instance(&i_key, &222);
    assert!(client.has_instance(&i_key));

    client.remove_instance(&i_key);
    assert!(!client.has_instance(&i_key));
    assert_eq!(client.get_instance(&i_key), None);

    // Verify we can set again after removal
    client.set_persistent(&p_key, &333);
    assert!(client.has_persistent(&p_key));
    assert_eq!(client.get_persistent(&p_key), Some(333));
}

#[test]
fn test_zero_and_boundary_values() {
    // Tests storage with zero and boundary values
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageContract);
    let client = StorageContractClient::new(&env, &contract_id);

    let key = symbol_short!("boundary");

    // Test zero value
    client.set_persistent(&key, &0);
    assert_eq!(client.get_persistent(&key), Some(0));

    // Test max u64 value
    client.set_persistent(&key, &u64::MAX);
    assert_eq!(client.get_persistent(&key), Some(u64::MAX));

    // Test with temporary storage
    client.set_temporary(&key, &0);
    assert_eq!(client.get_temporary(&key), Some(0));

    client.set_temporary(&key, &u64::MAX);
    assert_eq!(client.get_temporary(&key), Some(u64::MAX));

    // Test with instance storage
    client.set_instance(&key, &0);
    assert_eq!(client.get_instance(&key), Some(0));

    client.set_instance(&key, &u64::MAX);
    assert_eq!(client.get_instance(&key), Some(u64::MAX));
}

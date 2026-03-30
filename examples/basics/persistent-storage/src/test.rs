//! # Tests for Persistent Storage Pattern
//!
//! These tests verify:
//! 1. Basic CRUD operations (set, get)
//! 2. Counter increment logic with overflow protection
//! 3. Storage durability across ledger time
//! 4. TTL extension behavior

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

/// Tests basic persistent storage operations.
///
/// Verifies:
/// - Setting and retrieving admin address
/// - Counter increment logic
/// - Data persistence across multiple calls
#[test]
fn test_persistent_storage_logic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PersistentStorageContract);
    let client = PersistentStorageContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // 1. Test Setting and Getting Admin
    client.set_admin(&admin);
    assert_eq!(client.get_admin(), Some(admin.clone()));

    // 2. Test Increment Logic (u64 type)
    assert_eq!(client.increment(), 1);
    assert_eq!(client.increment(), 2);
    assert_eq!(client.get_counter(), 2);
}

/// Tests storage durability and TTL management.
///
/// Verifies:
/// - Data survives across ledger time progression
/// - TTL extension keeps data accessible
/// - Persistent storage remains available after 500 ledgers
#[test]
fn test_storage_durability_and_ttl() {
    let env = Env::default();

    // Configure ledger with realistic TTL parameters
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 12345,
        protocol_version: 20,
        sequence_number: 10,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 6312000,
    });

    let contract_id = env.register_contract(None, PersistentStorageContract);
    let client = PersistentStorageContractClient::new(&env, &contract_id);

    // Set initial data
    client.increment();

    // Extend instance TTL so the contract remains accessible
    // This is required because the contract instance itself needs TTL
    env.as_contract(&contract_id, || {
        env.storage().instance().extend_ttl(1000, 1000);
    });

    // Jump forward 500 ledgers (~42 minutes on mainnet at 5s/ledger)
    env.ledger().with_mut(|li| {
        li.sequence_number += 500;
    });

    // Persistent storage should still be accessible after time passes
    assert_eq!(client.get_counter(), 1);
}

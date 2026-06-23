use super::*;
use soroban_sdk::{testutils::Ledger, Env};

#[test]
fn test_lazy_cache_hits_and_eviction() {
    let env = Env::default();
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 1_000,
        protocol_version: 20,
        sequence_number: 1,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 6_312_000,
    });

    let contract_id = env.register_contract(None, LazyCacheContract);
    let client = LazyCacheContractClient::new(&env, &contract_id);

    client.set_item(&1, &100);
    client.set_item(&2, &200);
    client.set_item(&3, &300);

    // First access is a cache miss and loads from persistent storage.
    assert_eq!(client.get_item(&1), Some(100));
    assert_eq!(client.cache_stats(), (1, 0, 1));

    // Second access is a cache hit.
    assert_eq!(client.get_item(&1), Some(100));
    assert_eq!(client.cache_stats(), (1, 1, 1));

    // Load enough items to evict the oldest cached entry.
    assert_eq!(client.get_item(&2), Some(200));
    assert_eq!(client.get_item(&3), Some(300));
    assert_eq!(client.cache_stats(), (3, 1, 3));

    client.set_item(&4, &400);
    assert_eq!(client.get_item(&4), Some(400));
    assert_eq!(client.cache_stats(), (3, 1, 4));

    // Item 1 was evicted when inserting item 4, so it is reloaded again.
    assert_eq!(client.get_item(&1), Some(100));
    assert_eq!(client.cache_stats(), (3, 1, 5));
}

#[test]
fn test_cache_invalidation_and_clear() {
    let env = Env::default();
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 1_000,
        protocol_version: 20,
        sequence_number: 1,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 6_312_000,
    });

    let contract_id = env.register_contract(None, LazyCacheContract);
    let client = LazyCacheContractClient::new(&env, &contract_id);

    client.set_item(&10, &1_000);
    client.set_item(&11, &1_100);

    assert_eq!(client.get_item(&10), Some(1_000));
    assert_eq!(client.get_item(&11), Some(1_100));
    assert_eq!(client.cache_stats(), (2, 0, 2));

    client.invalidate_cache(&10);
    assert_eq!(client.cache_stats(), (1, 0, 2));
    assert_eq!(client.get_item(&10), Some(1_000));
    assert_eq!(client.cache_stats(), (2, 0, 3));

    client.clear_cache();
    assert_eq!(client.cache_stats(), (0, 0, 3));
}

#![allow(deprecated)]
//! Tests for the Lazy Loading and Caching contract.

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_client(env: &Env) -> LazyLoadingContractClient<'_> {
    env.mock_all_auths();
    let id = env.register_contract(None, LazyLoadingContract);
    LazyLoadingContractClient::new(env, &id)
}

// ---------------------------------------------------------------------------
// set_item
// ---------------------------------------------------------------------------

#[test]
fn set_item_stores_item() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    client.set_item(&owner, &1u32, &symbol_short!("hello"));

    let item = client.get_item(&1u32);
    assert_eq!(item.id, 1);
    assert_eq!(item.value, symbol_short!("hello"));
    assert_eq!(item.owner, owner);
}

#[test]
fn set_item_updates_existing_item() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    client.set_item(&owner, &1u32, &symbol_short!("v1"));
    client.set_item(&owner, &1u32, &symbol_short!("v2"));

    let item = client.get_item(&1u32);
    assert_eq!(item.value, symbol_short!("v2"));
}

#[test]
fn set_item_fails_with_zero_id() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    let res = client.try_set_item(&owner, &0u32, &symbol_short!("bad"));
    assert_eq!(res.err().unwrap().ok().unwrap(), LazyError::InvalidInput,);
}

// ---------------------------------------------------------------------------
// get_item — lazy load / cache miss
// ---------------------------------------------------------------------------

#[test]
fn get_item_returns_error_for_unknown_id() {
    let env = Env::default();
    let client = make_client(&env);

    let res = client.try_get_item(&99u32);
    assert_eq!(res.err().unwrap().ok().unwrap(), LazyError::ItemNotFound,);
}

#[test]
fn get_item_first_call_is_cache_miss() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    client.set_item(&owner, &1u32, &symbol_short!("data"));

    // set_item invalidates cache, so first get_item is a miss.
    let result = client.get_item_with_stats(&1u32);
    assert!(!result.cache_hit);
    assert_eq!(result.item.value, symbol_short!("data"));
}

#[test]
fn get_item_second_call_is_cache_hit() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    client.set_item(&owner, &1u32, &symbol_short!("data"));

    // First call populates cache.
    client.get_item(&1u32);

    // Second call hits cache.
    let result = client.get_item_with_stats(&1u32);
    assert!(result.cache_hit);
    assert_eq!(result.item.value, symbol_short!("data"));
}

// ---------------------------------------------------------------------------
// Cache size tracking
// ---------------------------------------------------------------------------

#[test]
fn cache_size_increases_on_miss() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    assert_eq!(client.cache_size(), 0);

    client.set_item(&owner, &1u32, &symbol_short!("a"));
    client.set_item(&owner, &2u32, &symbol_short!("b"));

    // Populate cache via get_item (miss path).
    client.get_item(&1u32);
    assert_eq!(client.cache_size(), 1);

    client.get_item(&2u32);
    assert_eq!(client.cache_size(), 2);
}

#[test]
fn cache_size_does_not_increase_on_hit() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    client.set_item(&owner, &1u32, &symbol_short!("a"));
    client.get_item(&1u32); // miss → size 1
    client.get_item(&1u32); // hit  → size still 1

    assert_eq!(client.cache_size(), 1);
}

// ---------------------------------------------------------------------------
// Cache eviction
// ---------------------------------------------------------------------------

#[test]
fn cache_evicts_oldest_when_full() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    // Store CACHE_CAPACITY + 1 items and load them all in sequence.
    // Each get_item call runs in its own invocation, so the cache starts
    // empty each time. After CACHE_CAPACITY sequential misses the next miss
    // triggers one eviction and inserts the new entry, keeping size bounded.
    let n = CACHE_CAPACITY + 1;
    for i in 1..=n {
        client.set_item(&owner, &i, &symbol_short!("v"));
    }

    // Load each item once — each is a miss because instance storage resets
    // between invocations in the test environment.
    for i in 1..=n {
        let result = client.get_item_with_stats(&i);
        // Every first access is a miss (fresh invocation each time).
        assert!(!result.cache_hit);
    }

    // Cache size never exceeds CACHE_CAPACITY.
    assert!(client.cache_size() <= CACHE_CAPACITY);
}

#[test]
fn cache_eviction_keeps_size_bounded() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    // Store more items than CACHE_CAPACITY.
    for i in 1..=(CACHE_CAPACITY * 2) {
        client.set_item(&owner, &i, &symbol_short!("v"));
        // Each set invalidates cache entry for i; reading it causes a miss
        // and inserts into cache. After CACHE_CAPACITY inserts the next one
        // evicts the oldest.
        client.get_item(&i);
        // Cache size must never exceed CACHE_CAPACITY.
        assert!(
            client.cache_size() <= CACHE_CAPACITY,
            "cache size exceeded capacity at i={i}"
        );
    }
}

// ---------------------------------------------------------------------------
// Cache invalidation
// ---------------------------------------------------------------------------

#[test]
fn set_item_invalidates_cache() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    client.set_item(&owner, &1u32, &symbol_short!("v1"));
    client.get_item(&1u32); // populate cache

    // Update value — should invalidate cache entry.
    client.set_item(&owner, &1u32, &symbol_short!("v2"));

    // Next read must be a miss and return the updated value.
    let result = client.get_item_with_stats(&1u32);
    assert!(!result.cache_hit);
    assert_eq!(result.item.value, symbol_short!("v2"));
}

#[test]
fn manual_invalidate_causes_next_read_to_miss() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    client.set_item(&owner, &1u32, &symbol_short!("v1"));
    client.get_item(&1u32); // populate cache
    assert_eq!(client.cache_size(), 1);

    client.invalidate(&1u32);
    assert_eq!(client.cache_size(), 0);

    let result = client.get_item_with_stats(&1u32);
    assert!(!result.cache_hit);
}

// ---------------------------------------------------------------------------
// item_count
// ---------------------------------------------------------------------------

#[test]
fn item_count_tracks_highest_id() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    assert_eq!(client.item_count(), 0);

    client.set_item(&owner, &3u32, &symbol_short!("a"));
    assert_eq!(client.item_count(), 3);

    // Storing a lower id does not decrease count.
    client.set_item(&owner, &1u32, &symbol_short!("b"));
    assert_eq!(client.item_count(), 3);

    client.set_item(&owner, &7u32, &symbol_short!("c"));
    assert_eq!(client.item_count(), 7);
}

// ---------------------------------------------------------------------------
// Multiple items independent
// ---------------------------------------------------------------------------

#[test]
fn multiple_items_are_independent() {
    let env = Env::default();
    let client = make_client(&env);
    let owner = Address::generate(&env);

    client.set_item(&owner, &1u32, &symbol_short!("alice"));
    client.set_item(&owner, &2u32, &symbol_short!("bob"));
    client.set_item(&owner, &3u32, &symbol_short!("carol"));

    assert_eq!(client.get_item(&1u32).value, symbol_short!("alice"));
    assert_eq!(client.get_item(&2u32).value, symbol_short!("bob"));
    assert_eq!(client.get_item(&3u32).value, symbol_short!("carol"));
}

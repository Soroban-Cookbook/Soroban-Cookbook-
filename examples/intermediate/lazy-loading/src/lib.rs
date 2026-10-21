//! # Lazy Loading and Caching
//!
//! Demonstrates how to reduce gas costs by loading only the subset of storage
//! data that is actually needed for a given invocation, and by maintaining a
//! bounded in-contract cache so that repeated reads within the same invocation
//! hit instance storage (cheap) rather than persistent storage (expensive).
//!
//! ## Key Ideas
//!
//! ### Lazy Loading
//! Instead of reading every item from persistent storage on every call, the
//! contract loads items **on demand**: `get_item(id)` reads from persistent
//! storage only the first time that `id` is requested.
//!
//! ### Bounded Cache
//! A cache stored in **instance storage** holds the most recently accessed
//! items, up to `CACHE_CAPACITY`. When the cache is full, the oldest entry
//! (lowest key) is evicted to make room — a simple FIFO/LRU approximation
//! that keeps the cache size predictable.
//!
//! ### Cache Invalidation
//! `set_item` writes through: it updates both persistent storage and removes
//! the stale cache entry so the next read re-fetches the fresh value.
//!
//! ### Performance Measurement
//! `get_item_with_stats` returns whether the value came from the cache (cache
//! hit) or was loaded from persistent storage (cache miss), enabling callers
//! to measure cache effectiveness.
//!
//! ## Storage Layout
//!
//! | Key | Type | Storage | Purpose |
//! |-----|------|---------|---------|
//! | `Item(u32)` | `Item` | Persistent | canonical item data |
//! | `ItemCount` | `u32` | Instance | total items ever stored |
//! | `Cache(u32)` | `Item` | Instance | cached copy of item |
//! | `CacheKeys` | `Vec<u32>` | Instance | ordered list of cached ids |

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of items kept in the instance-storage cache.
/// Keeping this small ensures instance storage TTL and size stay bounded.
pub const CACHE_CAPACITY: u32 = 10;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum LazyError {
    /// No item exists with the given id.
    ItemNotFound = 1,
    /// `value` must be non-empty (len > 0) and id must be > 0.
    InvalidInput = 2,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A simple item stored in the registry.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    /// Unique identifier.
    pub id: u32,
    /// Arbitrary payload stored with the item.
    pub value: Symbol,
    /// Address of the account that last wrote this item.
    pub owner: Address,
}

/// Returned by `get_item_with_stats` to expose cache hit/miss information.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadResult {
    pub item: Item,
    /// `true` if the item was served from the cache; `false` if loaded from
    /// persistent storage (cache miss).
    pub cache_hit: bool,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Canonical persistent record for an item id.
    Item(u32),
    /// Total number of items ever stored (monotonically increasing).
    ItemCount,
    /// Instance-storage cache entry for an item id.
    Cache(u32),
    /// Ordered list of ids currently held in the cache (for eviction).
    CacheKeys,
}

// ---------------------------------------------------------------------------
// Event topics
// ---------------------------------------------------------------------------

const NS: Symbol = symbol_short!("lazy");
const EVT_SET: Symbol = symbol_short!("set");
const EVT_HIT: Symbol = symbol_short!("cache_hit");
const EVT_MISS: Symbol = symbol_short!("cache_ms");
const EVT_EVICT: Symbol = symbol_short!("evict");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct LazyLoadingContract;

#[contractimpl]
impl LazyLoadingContract {
    // -----------------------------------------------------------------------
    // Write path
    // -----------------------------------------------------------------------

    /// Store or update an item.
    ///
    /// Writes to persistent storage and invalidates any stale cache entry so
    /// the next `get_item` call fetches the fresh value.
    pub fn set_item(env: Env, owner: Address, id: u32, value: Symbol) -> Result<(), LazyError> {
        owner.require_auth();

        if id == 0 {
            return Err(LazyError::InvalidInput);
        }

        let item = Item {
            id,
            value: value.clone(),
            owner: owner.clone(),
        };

        // Write to canonical persistent storage.
        env.storage().persistent().set(&DataKey::Item(id), &item);

        // Update item count if this is a new id.
        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ItemCount)
            .unwrap_or(0);
        if count < id {
            env.storage().instance().set(&DataKey::ItemCount, &id);
        }

        // Cache invalidation: remove the stale cache entry so the next read
        // re-loads the updated value from persistent storage.
        Self::invalidate_cache(&env, id);

        env.events().publish((NS, EVT_SET, owner), (id, value));

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Read path — lazy load with cache
    // -----------------------------------------------------------------------

    /// Load an item by id.
    ///
    /// 1. Check the instance-storage cache (cheap).
    /// 2. On cache miss, load from persistent storage and populate the cache.
    /// 3. If the cache is full, evict the oldest entry first.
    pub fn get_item(env: Env, id: u32) -> Result<Item, LazyError> {
        Ok(Self::load(env, id)?.item)
    }

    /// Like `get_item` but also returns whether the value came from the cache.
    ///
    /// Use this to measure cache effectiveness in tests and benchmarks.
    pub fn get_item_with_stats(env: Env, id: u32) -> Result<LoadResult, LazyError> {
        Self::load(env, id)
    }

    // -----------------------------------------------------------------------
    // Cache management
    // -----------------------------------------------------------------------

    /// Return the number of items currently in the cache.
    pub fn cache_size(env: Env) -> u32 {
        Self::cache_keys(&env).len()
    }

    /// Manually evict a specific item from the cache.
    ///
    /// Useful when a caller knows a cache entry is stale without going through
    /// `set_item` (e.g. after a cross-contract update).
    pub fn invalidate(env: Env, id: u32) {
        Self::invalidate_cache(&env, id);
    }

    /// Return the total number of items ever stored.
    pub fn item_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ItemCount)
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Core lazy-load logic shared by `get_item` and `get_item_with_stats`.
    fn load(env: Env, id: u32) -> Result<LoadResult, LazyError> {
        // 1. Cache hit — instance storage is cheaper than persistent.
        if let Some(item) = env
            .storage()
            .instance()
            .get::<DataKey, Item>(&DataKey::Cache(id))
        {
            env.events().publish((NS, EVT_HIT), id);
            return Ok(LoadResult {
                item,
                cache_hit: true,
            });
        }

        // 2. Cache miss — load from persistent storage.
        let item: Item = env
            .storage()
            .persistent()
            .get(&DataKey::Item(id))
            .ok_or(LazyError::ItemNotFound)?;

        env.events().publish((NS, EVT_MISS), id);

        // 3. Populate cache, evicting oldest entry if at capacity.
        Self::insert_cache(&env, id, &item);

        Ok(LoadResult {
            item,
            cache_hit: false,
        })
    }

    /// Read the ordered list of cached ids from instance storage.
    fn cache_keys(env: &Env) -> Vec<u32> {
        env.storage()
            .instance()
            .get(&DataKey::CacheKeys)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Insert `item` into the cache under `id`.
    ///
    /// If the cache is already at `CACHE_CAPACITY`, the first (oldest) key is
    /// evicted before inserting the new entry.
    fn insert_cache(env: &Env, id: u32, item: &Item) {
        let mut keys = Self::cache_keys(env);

        // Evict oldest entry if cache is full.
        if keys.len() >= CACHE_CAPACITY {
            let oldest = keys.get(0).unwrap();
            keys.remove(0);
            env.storage().instance().remove(&DataKey::Cache(oldest));
            env.events().publish((NS, EVT_EVICT), oldest);
        }

        // Add new entry.
        keys.push_back(id);
        env.storage().instance().set(&DataKey::Cache(id), item);
        env.storage().instance().set(&DataKey::CacheKeys, &keys);
    }

    /// Remove `id` from the cache (both the entry and the keys list).
    fn invalidate_cache(env: &Env, id: u32) {
        let mut keys = Self::cache_keys(env);

        // Find the position of `id` in the keys list.
        let mut found_idx: Option<u32> = None;
        for (i, k) in keys.iter().enumerate() {
            if k == id {
                found_idx = Some(i as u32);
                break;
            }
        }

        if let Some(idx) = found_idx {
            keys.remove(idx);
            env.storage().instance().remove(&DataKey::Cache(id));
            env.storage().instance().set(&DataKey::CacheKeys, &keys);
        }
    }
}

#[cfg(test)]
mod test;

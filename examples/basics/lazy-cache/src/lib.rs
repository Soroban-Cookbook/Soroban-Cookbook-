#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Vec};

const MAX_CACHE_SIZE: u32 = 3;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Item(u32),
    Cache(u32),
    CacheMetadata,
}

#[contracttype]
#[derive(Clone)]
pub struct CacheMetadata {
    ids: Vec<u32>,
    hits: u64,
    misses: u64,
}

#[contract]
pub struct LazyCacheContract;

#[contractimpl]
impl LazyCacheContract {
    /// Store a large item in persistent storage.
    /// The item is not cached until it is requested.
    pub fn set_item(env: Env, id: u32, value: u64) {
        env.storage()
            .persistent()
            .set(&StorageKey::Item(id), &value);
        env.events().publish(
            (symbol_short!("cache"), symbol_short!("store")),
            (id, value),
        );
    }

    /// Lazy-loads the requested item and caches it in temporary storage.
    /// Subsequent reads return the cached value until it is invalidated or evicted.
    pub fn get_item(env: Env, id: u32) -> Option<u64> {
        let mut metadata = load_metadata(&env);
        prune_expired_cache(&env, &mut metadata);

        let cache_key = StorageKey::Cache(id);
        if env.storage().temporary().has(&cache_key) {
            metadata.hits += 1;
            save_metadata(&env, &metadata);
            env.events()
                .publish((symbol_short!("cache"), symbol_short!("hit")), id);
            return env.storage().temporary().get(&cache_key);
        }

        metadata.misses += 1;
        if let Some(value) = env.storage().persistent().get(&StorageKey::Item(id)) {
            add_to_cache(&env, &mut metadata, id, value);
            save_metadata(&env, &metadata);
            env.events()
                .publish((symbol_short!("cache"), symbol_short!("miss")), id);
            return Some(value);
        }

        save_metadata(&env, &metadata);
        None
    }

    /// Remove the cached entry for a specific item.
    pub fn invalidate_cache(env: Env, id: u32) {
        let mut metadata = load_metadata(&env);
        prune_expired_cache(&env, &mut metadata);
        remove_cached_entry(&env, &mut metadata, id);
        save_metadata(&env, &metadata);
        env.events()
            .publish((symbol_short!("cache"), symbol_short!("invalidate")), id);
    }

    /// Clear the entire temporary cache.
    pub fn clear_cache(env: Env) {
        let mut metadata = load_metadata(&env);
        for cached_id in metadata.ids.iter() {
            env.storage()
                .temporary()
                .remove(&StorageKey::Cache(*cached_id));
        }
        metadata.ids = Vec::new(&env);
        save_metadata(&env, &metadata);
        env.events().publish(
            (symbol_short!("cache"), symbol_short!("clear")),
            metadata.ids.len(),
        );
    }

    /// Returns the current cache size and the recorded hit/miss counts.
    pub fn cache_stats(env: Env) -> (u32, u64, u64) {
        let mut metadata = load_metadata(&env);
        prune_expired_cache(&env, &mut metadata);
        let current_size = metadata.ids.len();
        save_metadata(&env, &metadata);
        (current_size, metadata.hits, metadata.misses)
    }
}

fn load_metadata(env: &Env) -> CacheMetadata {
    env.storage()
        .instance()
        .get(&StorageKey::CacheMetadata)
        .unwrap_or(CacheMetadata {
            ids: Vec::new(env),
            hits: 0,
            misses: 0,
        })
}

fn save_metadata(env: &Env, metadata: &CacheMetadata) {
    env.storage()
        .instance()
        .set(&StorageKey::CacheMetadata, metadata);
}

fn prune_expired_cache(env: &Env, metadata: &mut CacheMetadata) {
    let mut active_ids = Vec::new(env);
    for id in metadata.ids.iter() {
        if env.storage().temporary().has(&StorageKey::Cache(*id)) {
            active_ids.push_back(*id);
        }
    }
    metadata.ids = active_ids;
}

fn add_to_cache(env: &Env, metadata: &mut CacheMetadata, id: u32, value: u64) {
    if !metadata.ids.iter().any(|cached_id| *cached_id == id) {
        if metadata.ids.len() >= MAX_CACHE_SIZE {
            if let Some(evicted_id) = metadata.ids.get(0) {
                env.storage()
                    .temporary()
                    .remove(&StorageKey::Cache(*evicted_id));
                metadata.ids.remove(0);
                env.events().publish(
                    (symbol_short!("cache"), symbol_short!("evict")),
                    *evicted_id,
                );
            }
        }
        metadata.ids.push_back(id);
    }

    env.storage()
        .temporary()
        .set(&StorageKey::Cache(id), &value);
    env.storage()
        .temporary()
        .extend_ttl(&StorageKey::Cache(id), 2, 100);
}

fn remove_cached_entry(env: &Env, metadata: &mut CacheMetadata, id: u32) {
    env.storage().temporary().remove(&StorageKey::Cache(id));
    let mut remaining = Vec::new(env);
    for cached_id in metadata.ids.iter() {
        if *cached_id != id {
            remaining.push_back(*cached_id);
        }
    }
    metadata.ids = remaining;
}

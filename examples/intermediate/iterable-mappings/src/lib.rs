//! # Iterable Mapping
//!
//! This example shows how to build a small key-value map that remains
//! enumerable in Soroban, where native iteration over a `Map` is limited.
//! The contract keeps a `Map<Symbol, u32>` for direct lookups and a separate
//! `Vec<Symbol>` index so callers can page through keys and values safely.
//!
//! The extra key index makes enumeration possible but adds storage overhead,
//! so the pattern is best for moderate-sized collections where iteration is
//! more important than minimizing write cost.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env, Map, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Entries,
    Keys,
}

#[contract]
pub struct IterableMappings;

#[contractimpl]
impl IterableMappings {
    /// Insert or update an entry in the map.
    ///
    /// If the key is new, it is appended to the side index so pagination can
    /// enumerate the collection later.
    pub fn set(env: Env, key: Symbol, value: u32) {
        let mut entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));

        let is_new = !entries.contains_key(key.clone());
        entries.set(key.clone(), value);

        if is_new {
            let mut keys: Vec<Symbol> = env
                .storage()
                .instance()
                .get(&DataKey::Keys)
                .unwrap_or_else(|| Vec::new(&env));
            keys.push_back(key);
            env.storage().instance().set(&DataKey::Keys, &keys);
        }

        env.storage().instance().set(&DataKey::Entries, &entries);
    }

    /// Remove an entry and keep the indexed key list in sync.
    pub fn remove(env: Env, key: Symbol) {
        let mut entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));

        if entries.contains_key(key.clone()) {
            entries.remove(key.clone());
            let keys: Vec<Symbol> = env
                .storage()
                .instance()
                .get(&DataKey::Keys)
                .unwrap_or_else(|| Vec::new(&env));

            let mut filtered = Vec::new(&env);
            for existing in keys.iter() {
                if existing != key {
                    filtered.push_back(existing);
                }
            }

            env.storage().instance().set(&DataKey::Keys, &filtered);
            env.storage().instance().set(&DataKey::Entries, &entries);
        }
    }

    /// Return the current value for `key`, if present.
    pub fn get(env: Env, key: Symbol) -> Option<u32> {
        let entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));
        entries.get(key)
    }

    /// Return the number of indexed entries.
    pub fn len(env: Env) -> u32 {
        let keys: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::Keys)
            .unwrap_or_else(|| Vec::new(&env));
        keys.len()
    }

    /// Return a page of keys for iteration.
    pub fn keys(env: Env, page: u32, page_size: u32) -> Vec<Symbol> {
        if page_size == 0 {
            return Vec::new(&env);
        }

        let keys: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::Keys)
            .unwrap_or_else(|| Vec::new(&env));

        let start = (page.saturating_sub(1) * page_size) as u32;
        let end = start.saturating_add(page_size).min(keys.len());

        let mut page_keys = Vec::new(&env);
        for index in start..end {
            page_keys.push_back(keys.get(index).unwrap());
        }
        page_keys
    }

    /// Return the values that correspond to the provided page of keys.
    pub fn values(env: Env, page: u32, page_size: u32) -> Vec<u32> {
        let entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));
        let keys = Self::keys(env.clone(), page, page_size);

        let mut page_values = Vec::new(&env);
        for key in keys.iter() {
            page_values.push_back(entries.get(key.clone()).unwrap());
        }
        page_values
    }
}

#[cfg(test)]
mod test;

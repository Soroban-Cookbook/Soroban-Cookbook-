#![allow(deprecated)]
//! # Iterable Mapping Utilities
//!
//! This example shows how to build and manipulate an enumerable key-value map in Soroban.
//! Native iteration over a Soroban `Map` is limited, so the contract maintains a
//! `Map<Symbol, u32>` for lookups and a separate `Vec<Symbol>` index for key iteration.
//!
//! In addition to storage operations, this module provides helper functions and contract
//! methods for filtering, mapping, and reducing iterable maps safely and predictably:
//! - **Filtering**: `filter_by_min_value` / `filter_by_page` (creates new maps containing matching entries)
//! - **Mapping**: `map_values_scale` / `map_values_scale_page` (transforms entries safely)
//! - **Reducing**: `reduce_sum` / `reduce_sum_page` (accumulates values deterministically into a scalar)
//! - Functional helpers: `filter_by_predicate`, `transform_values`, `reduce_values`

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{contract, contractimpl, contracttype, Env, Map, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Entries,
    Keys,
}

/// Generic functional helper to filter map entries by a predicate function.
/// Returns a new `Map` containing only the entries for which `predicate(&key, value)` returns true.
pub fn filter_by_predicate<F>(
    entries: &Map<Symbol, u32>,
    keys: &Vec<Symbol>,
    predicate: F,
) -> Map<Symbol, u32>
where
    F: Fn(&Symbol, u32) -> bool,
{
    let env = entries.env();
    let mut result = Map::new(env);
    for key in keys.iter() {
        if let Some(val) = entries.get(key.clone()) {
            if predicate(&key, val) {
                result.set(key, val);
            }
        }
    }
    result
}

/// Generic functional helper to transform map values safely.
/// Returns a new `Map` with transformed values using the supplied `transform` function.
pub fn transform_values<F>(
    entries: &Map<Symbol, u32>,
    keys: &Vec<Symbol>,
    transform: F,
) -> Map<Symbol, u32>
where
    F: Fn(&Symbol, u32) -> u32,
{
    let env = entries.env();
    let mut result = Map::new(env);
    for key in keys.iter() {
        if let Some(val) = entries.get(key.clone()) {
            let new_val = transform(&key, val);
            result.set(key, new_val);
        }
    }
    result
}

/// Generic functional helper to accumulate values from an iterable map into a deterministic result.
pub fn reduce_values<F, A>(entries: &Map<Symbol, u32>, keys: &Vec<Symbol>, init: A, f: F) -> A
where
    F: Fn(A, &Symbol, u32) -> A,
{
    let mut acc = init;
    for key in keys.iter() {
        if let Some(val) = entries.get(key.clone()) {
            acc = f(acc, &key, val);
        }
    }
    acc
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

    /// Return whether the indexed map has no entries.
    pub fn is_empty(env: Env) -> bool {
        Self::len(env) == 0
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

        let start = page.saturating_sub(1).saturating_mul(page_size);
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

    /// Filter the map entries by a minimum value threshold over all entries.
    /// Returns a new `Map<Symbol, u32>` containing only entries where value >= `min_value`.
    pub fn filter_by_min_value(env: Env, min_value: u32) -> Map<Symbol, u32> {
        let entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));
        let keys: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::Keys)
            .unwrap_or_else(|| Vec::new(&env));

        filter_by_predicate(&entries, &keys, |_, val| val >= min_value)
    }

    /// Filter the map entries by a minimum value threshold over a single page of keys.
    /// This pattern bounds iteration costs to `page_size` items for predictable gas consumption.
    pub fn filter_by_page(env: Env, min_value: u32, page: u32, page_size: u32) -> Map<Symbol, u32> {
        let entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));
        let page_keys = Self::keys(env.clone(), page, page_size);

        filter_by_predicate(&entries, &page_keys, |_, val| val >= min_value)
    }

    /// Transform values by multiplying each value in the map by `factor` across all entries.
    /// Uses saturating arithmetic to avoid overflow panics.
    pub fn map_values_scale(env: Env, factor: u32) -> Map<Symbol, u32> {
        let entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));
        let keys: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::Keys)
            .unwrap_or_else(|| Vec::new(&env));

        transform_values(&entries, &keys, |_, val| val.saturating_mul(factor))
    }

    /// Transform values by multiplying each value in a single page by `factor`.
    /// Bounded by `page_size` to ensure bounded execution resources.
    pub fn map_values_scale_page(
        env: Env,
        factor: u32,
        page: u32,
        page_size: u32,
    ) -> Map<Symbol, u32> {
        let entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));
        let page_keys = Self::keys(env.clone(), page, page_size);

        transform_values(&entries, &page_keys, |_, val| val.saturating_mul(factor))
    }

    /// Calculate the sum of all values in the map as a `u64`.
    /// Accumulates deterministically without modifying contract storage.
    pub fn reduce_sum(env: Env) -> u64 {
        let entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));
        let keys: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::Keys)
            .unwrap_or_else(|| Vec::new(&env));

        reduce_values(&entries, &keys, 0u64, |acc, _, val| {
            acc.saturating_add(val as u64)
        })
    }

    /// Calculate the sum of values within a specific page of keys as a `u64`.
    pub fn reduce_sum_page(env: Env, page: u32, page_size: u32) -> u64 {
        let entries: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&DataKey::Entries)
            .unwrap_or_else(|| Map::new(&env));
        let page_keys = Self::keys(env.clone(), page, page_size);

        reduce_values(&entries, &page_keys, 0u64, |acc, _, val| {
            acc.saturating_add(val as u64)
        })
    }
}

#[cfg(test)]
mod test;

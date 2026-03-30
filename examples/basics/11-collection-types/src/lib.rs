//! # Collection Types in Soroban
//!
//! This example demonstrates `Vec<T>` and `Map<K, V>` — Soroban's two built-in
//! collection types — and the iteration patterns available for each.
//!
//! ## What's Covered
//!
//! - **Vec operations** — push, pop, random access, length, contains
//! - **Map operations** — insert, lookup, remove, keys/values extraction
//! - **Iteration** — `for` loops and fold-based aggregations over both types
//! - **Performance considerations** — when to prefer Vec vs Map
//!
//! ## Performance Notes
//!
//! | Operation     | Vec        | Map        |
//! |---------------|------------|------------|
//! | Indexed access| O(1)       | O(log n)   |
//! | Membership    | O(n) scan  | O(log n)   |
//! | Ordered iter  | Natural    | Sorted by key |
//! | Storage cost  | Lower      | Higher     |
//!
//! **Rule of thumb**
//! - Use `Vec` for ordered sequences and batch operations.
//! - Use `Map` when you need O(log n) key lookups or a sorted key-value store.

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Map, Symbol, Vec};

/// Contract demonstrating collection type operations.
#[contract]
pub struct CollectionTypesContract;

#[contractimpl]
impl CollectionTypesContract {
    // -----------------------------------------------------------------------
    // Vec<i128> Operations
    // -----------------------------------------------------------------------

    /// Append `item` to the persistent `Vec<i128>` stored under key `"vec"`.
    pub fn vec_push(env: Env, item: i128) {
        let mut v: Vec<i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("vec"))
            .unwrap_or_else(|| Vec::new(&env));
        v.push_back(item);
        env.storage().instance().set(&symbol_short!("vec"), &v);
    }

    /// Remove and return the last element of the stored `Vec`, or `None` if empty.
    pub fn vec_pop(env: Env) -> Option<i128> {
        let mut v: Vec<i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("vec"))
            .unwrap_or_else(|| Vec::new(&env));
        let item = v.pop_back();
        env.storage().instance().set(&symbol_short!("vec"), &v);
        item
    }

    /// Return the stored `Vec<i128>` (empty if never written).
    pub fn vec_list(env: Env) -> Vec<i128> {
        env.storage()
            .instance()
            .get(&symbol_short!("vec"))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Sum every element in `items`.
    ///
    /// Demonstrates a simple `for item in items.iter()` loop over a `Vec`.
    pub fn vec_sum(_env: Env, items: Vec<i128>) -> i128 {
        let mut total: i128 = 0;
        for item in items.iter() {
            total += item;
        }
        total
    }

    /// Return a new `Vec` containing only the elements of `items` that are > 0.
    ///
    /// Demonstrates building a result `Vec` while iterating another.
    pub fn vec_filter_positive(env: Env, items: Vec<i128>) -> Vec<i128> {
        let mut result = Vec::new(&env);
        for item in items.iter() {
            if item > 0 {
                result.push_back(item);
            }
        }
        result
    }

    /// Return the largest element of `items`, or `None` for an empty `Vec`.
    ///
    /// Demonstrates a fold-style accumulation using `Iterator::fold`.
    pub fn vec_max(_env: Env, items: Vec<i128>) -> Option<i128> {
        if items.is_empty() {
            return None;
        }
        let first = items.get(0).unwrap();
        let max = items
            .iter()
            .fold(first, |acc, x| if x > acc { x } else { acc });
        Some(max)
    }

    /// Return `true` when `target` is present in `items`.
    ///
    /// Demonstrates `Iterator::any` — short-circuits on first match.
    /// Note: O(n) for Vec; prefer `Map::contains_key` for frequent lookups.
    pub fn vec_contains(_env: Env, items: Vec<i128>, target: i128) -> bool {
        items.iter().any(|x| x == target)
    }

    // -----------------------------------------------------------------------
    // Map<Symbol, i128> Operations
    // -----------------------------------------------------------------------

    /// Insert or overwrite `key → value` in the stored `Map`.
    ///
    /// Map keeps entries sorted by key, so retrieval is O(log n).
    pub fn map_set(env: Env, key: Symbol, value: i128) {
        let mut m: Map<Symbol, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("map"))
            .unwrap_or_else(|| Map::new(&env));
        m.set(key, value);
        env.storage().instance().set(&symbol_short!("map"), &m);
    }

    /// Look up `key` in the stored `Map`. Returns `None` when absent.
    pub fn map_get(env: Env, key: Symbol) -> Option<i128> {
        let m: Map<Symbol, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("map"))
            .unwrap_or_else(|| Map::new(&env));
        m.get(key)
    }

    /// Remove `key` from the stored `Map` (no-op if the key is absent).
    pub fn map_remove(env: Env, key: Symbol) {
        let mut m: Map<Symbol, i128> = env
            .storage()
            .instance()
            .get(&symbol_short!("map"))
            .unwrap_or_else(|| Map::new(&env));
        m.remove(key);
        env.storage().instance().set(&symbol_short!("map"), &m);
    }

    /// Return the stored `Map` (empty if never written).
    pub fn map_get_all(env: Env) -> Map<Symbol, i128> {
        env.storage()
            .instance()
            .get(&symbol_short!("map"))
            .unwrap_or_else(|| Map::new(&env))
    }

    /// Sum all values in `data`.
    ///
    /// Demonstrates `for (_key, value) in data.iter()` over a `Map`.
    pub fn map_sum_values(_env: Env, data: Map<Symbol, i128>) -> i128 {
        let mut total: i128 = 0;
        for (_key, value) in data.iter() {
            total += value;
        }
        total
    }

    /// Return all keys in `data` as a sorted `Vec<Symbol>`.
    ///
    /// `Map::keys()` returns keys in their natural sorted order.
    pub fn map_keys(_env: Env, data: Map<Symbol, i128>) -> Vec<Symbol> {
        data.keys()
    }

    /// Return all values in `data` as a `Vec<i128>` (in key-sorted order).
    pub fn map_values(_env: Env, data: Map<Symbol, i128>) -> Vec<i128> {
        data.values()
    }

    /// Return the key whose associated value is the highest.
    ///
    /// Returns `None` for an empty map. Demonstrates destructuring
    /// `(key, value)` pairs while iterating a `Map`.
    pub fn map_max_key(_env: Env, data: Map<Symbol, i128>) -> Option<Symbol> {
        let mut best_key: Option<Symbol> = None;
        let mut best_val: i128 = 0;
        let mut first = true;

        for (key, value) in data.iter() {
            if first || value > best_val {
                best_val = value;
                best_key = Some(key);
                first = false;
            }
        }
        best_key
    }

    // -----------------------------------------------------------------------
    // Iteration Patterns — combining Vec and Map
    // -----------------------------------------------------------------------

    /// Build a `Map` from parallel `keys` and `values` Vecs.
    ///
    /// Extra elements in the longer Vec are ignored (zip semantics).
    /// Demonstrates iterating two Vecs together by index.
    pub fn zip_to_map(env: Env, keys: Vec<Symbol>, values: Vec<i128>) -> Map<Symbol, i128> {
        let mut m = Map::new(&env);
        let len = keys.len().min(values.len());
        for i in 0..len {
            m.set(keys.get(i).unwrap(), values.get(i).unwrap());
        }
        m
    }

    /// Return a new `Map` where every value has been incremented by `delta`.
    ///
    /// Demonstrates reading a `Map` via iteration and writing a transformed
    /// copy — the idiomatic Soroban pattern because `Map` has no in-place
    /// value mutation.
    pub fn map_increment_values(
        env: Env,
        data: Map<Symbol, i128>,
        delta: i128,
    ) -> Map<Symbol, i128> {
        let mut result = Map::new(&env);
        for (key, value) in data.iter() {
            result.set(key, value + delta);
        }
        result
    }
}

#[cfg(test)]
mod test;

//! # Storage Pagination Contract
//!
//! This contract demonstrates cursor-based pagination for efficiently
//! retrieving large on-chain collections without exceeding instruction limits.

#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Symbol, Vec};

const ITEMS_KEY: u32 = 0;

#[contract]
pub struct PaginationContract;

/// Result of a pagination query
#[derive(Clone)]
pub struct Page {
    /// Items in this page
    pub items: Vec<Symbol>,
    /// Cursor for the next page (empty if no more pages)
    pub next_cursor: Option<u32>,
}

#[contractimpl]
impl PaginationContract {
    /// Add an item to the collection.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `item` - the Symbol item to add
    pub fn add_item(env: Env, item: Symbol) {
        let mut items: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&ITEMS_KEY)
            .unwrap_or(Vec::new(&env));

        items.push_back(item);
        env.storage().persistent().set(&ITEMS_KEY, &items);
    }

    /// Get a page of items starting from a cursor.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `cursor` - Starting index (0 for first page, or from previous next_cursor)
    /// * `page_size` - Maximum items to return (suggested: 10-50)
    ///
    /// # Returns
    /// A Page containing items and the cursor for the next page
    /// (next_cursor is None if this is the last page)
    pub fn list(env: Env, cursor: u32, page_size: u32) -> (Vec<Symbol>, Option<u32>) {
        let items: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&ITEMS_KEY)
            .unwrap_or(Vec::new(&env));

        if page_size == 0 {
            panic!("Page size must be greater than 0");
        }

        let start = cursor as usize;
        let page_size_usize = page_size as usize;
        let end = (start + page_size_usize).min(items.len());

        let mut page_items = Vec::new(&env);
        for i in start..end {
            if let Some(item) = items.get(i as u32) {
                page_items.push_back(item);
            }
        }

        let next_cursor = if end < items.len() {
            Some(end as u32)
        } else {
            None
        };

        (page_items, next_cursor)
    }

    /// Get the total number of items in the collection.
    ///
    /// # Returns
    /// The total item count
    pub fn count(env: Env) -> u32 {
        let items: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&ITEMS_KEY)
            .unwrap_or(Vec::new(&env));

        items.len() as u32
    }

    /// Get an item by index.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `index` - The index of the item
    ///
    /// # Returns
    /// The item at the given index
    ///
    /// # Panics
    /// Panics if index is out of bounds
    pub fn get_item(env: Env, index: u32) -> Symbol {
        let items: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&ITEMS_KEY)
            .unwrap_or(Vec::new(&env));

        items
            .get(index)
            .unwrap_or_else(|| panic!("Index out of bounds"))
    }
}

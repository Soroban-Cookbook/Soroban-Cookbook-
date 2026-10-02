#![allow(deprecated)]
//! # Storage Pagination
//!
//! Cursor-based pagination over a large on-chain collection without loading
//! every item in a single call.
//!
//! ## Storage layout
//!
//! Items are stored **individually** under indexed persistent keys, with a
//! separate counter for the next append index. `list` therefore reads only the
//! keys for the requested page instead of deserializing a single giant `Vec`.
//!
//! ## Cursor format (treat as opaque)
//!
//! Clients should treat cursors as opaque `Bytes` values. The on-wire layout is
//! documented here so implementers can validate and debug:
//!
//! ```text
//! [0..4]  magic   = b"SPG1"   (Storage PaGination v1)
//! [4..8]  index   = u32 big-endian absolute storage index
//! ```
//!
//! Total length is exactly 8 bytes. Any other length, wrong magic, or truncated
//! payload is rejected with `PaginationError::InvalidCursor`.
//!
//! ## Consistency model
//!
//! This example is **append-only**. A cursor encodes an absolute index, so it
//! remains valid after new items are appended (those appear on later pages).
//! There is **no snapshot isolation**: if a client pages while writers add
//! items, later pages may include newly appended entries. Deletes / reordering
//! are out of scope for this example.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Bytes, Env, Symbol, Vec};

/// Hard cap on items returned by a single `list` call.
pub const MAX_PAGE_SIZE: u32 = 50;

/// Cursor magic bytes: "SPG1".
const CURSOR_MAGIC: [u8; 4] = [b'S', b'P', b'G', b'1'];
const CURSOR_LEN: u32 = 8;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Next absolute index to assign (also the item count for append-only).
    NextIndex,
    /// Persistent value stored at absolute index `n`.
    Item(u32),
}

/// One page of results plus an opaque resume cursor.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Page {
    pub items: Vec<Symbol>,
    /// Opaque cursor for the next page. `None` means end of collection.
    pub next_cursor: Option<Bytes>,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PaginationError {
    /// `page_size` was zero or greater than `MAX_PAGE_SIZE`.
    InvalidPageSize = 1,
    /// Cursor bytes failed length / magic / format checks.
    InvalidCursor = 2,
}

#[contract]
pub struct StoragePagination;

#[contractimpl]
impl StoragePagination {
    /// Append `item` and return its absolute index.
    pub fn add_item(env: Env, item: Symbol) -> u32 {
        let index = read_next_index(&env);
        env.storage().persistent().set(&DataKey::Item(index), &item);
        let next = index.saturating_add(1);
        env.storage().instance().set(&DataKey::NextIndex, &next);
        index
    }

    /// Return a page of items starting at `cursor`.
    ///
    /// - `cursor = None` starts at index 0.
    /// - `page_size` must be in `1..=MAX_PAGE_SIZE`.
    /// - A well-formed cursor at or past the end yields an empty page and
    ///   `next_cursor = None` (not an error).
    pub fn list(env: Env, page_size: u32, cursor: Option<Bytes>) -> Result<Page, PaginationError> {
        if page_size == 0 || page_size > MAX_PAGE_SIZE {
            return Err(PaginationError::InvalidPageSize);
        }

        let start = match cursor {
            Some(bytes) => decode_cursor(&bytes)?,
            None => 0,
        };

        let total = read_next_index(&env);
        if start >= total {
            return Ok(Page {
                items: Vec::new(&env),
                next_cursor: None,
            });
        }

        let end = start.saturating_add(page_size).min(total);
        let mut items = Vec::new(&env);
        let mut index = start;
        while index < end {
            if let Some(item) = env
                .storage()
                .persistent()
                .get::<_, Symbol>(&DataKey::Item(index))
            {
                items.push_back(item);
            }
            index = index.saturating_add(1);
        }

        let next_cursor = if end < total {
            Some(encode_cursor(&env, end))
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }

    /// Number of stored items (append-only count).
    pub fn count(env: Env) -> u32 {
        read_next_index(&env)
    }

    /// Read the item at absolute `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn get_item(env: Env, index: u32) -> Symbol {
        env.storage()
            .persistent()
            .get(&DataKey::Item(index))
            .unwrap_or_else(|| panic!("Index out of bounds"))
    }

    /// Build an opaque cursor for a known absolute index (client-side demos).
    pub fn cursor_from_index(env: Env, index: u32) -> Bytes {
        encode_cursor(&env, index)
    }
}

fn read_next_index(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::NextIndex)
        .unwrap_or(0)
}

fn encode_cursor(env: &Env, index: u32) -> Bytes {
    let mut raw = [0u8; CURSOR_LEN as usize];
    raw[0..4].copy_from_slice(&CURSOR_MAGIC);
    raw[4..8].copy_from_slice(&index.to_be_bytes());
    Bytes::from_array(env, &raw)
}

fn decode_cursor(bytes: &Bytes) -> Result<u32, PaginationError> {
    if bytes.len() != CURSOR_LEN {
        return Err(PaginationError::InvalidCursor);
    }

    for (i, expected) in CURSOR_MAGIC.iter().enumerate() {
        let got = bytes.get(i as u32).ok_or(PaginationError::InvalidCursor)?;
        if got != *expected {
            return Err(PaginationError::InvalidCursor);
        }
    }

    Ok(read_u32_be(bytes, 4))
}

fn read_u32_be(bytes: &Bytes, offset: u32) -> u32 {
    let mut result = 0u32;
    for i in 0..4 {
        let byte = bytes.get(offset + i).unwrap_or(0);
        result = (result << 8) | (byte as u32);
    }
    result
}

#[cfg(test)]
mod test;

//! # Event History Example
//!
//! Demonstrates on-chain history storage, pagination, and time filtering for audit trails.

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    MaxEntries,
    StartIndex,
    NextIndex,
    Count,
    Entry(u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEntry {
    pub actor: Address,
    pub action: Symbol,
    pub timestamp: u64,
    pub details: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryStats {
    pub start_index: u32,
    pub next_index: u32,
    pub count: u32,
    pub max_entries: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryCursor {
    pub index: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryPage {
    pub entries: Vec<AuditEntry>,
    pub next_cursor: Option<HistoryCursor>,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum HistoryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidCapacity = 4,
    InvalidPagination = 5,
    InvalidCursor = 6,
}

const EVENT_AUDIT: Symbol = symbol_short!("audit");

#[contract]
pub struct EventHistory;

#[contractimpl]
impl EventHistory {
    pub fn initialize(env: Env, admin: Address, max_entries: u32) -> Result<(), HistoryError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(HistoryError::AlreadyInitialized);
        }
        if max_entries == 0 {
            return Err(HistoryError::InvalidCapacity);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::MaxEntries, &max_entries);
        env.storage().instance().set(&DataKey::StartIndex, &0u32);
        env.storage().instance().set(&DataKey::NextIndex, &0u32);
        env.storage().instance().set(&DataKey::Count, &0u32);
        Ok(())
    }

    pub fn append_event(
        env: Env,
        actor: Address,
        action: Symbol,
        details: Symbol,
    ) -> Result<u32, HistoryError> {
        actor.require_auth();
        let admin = read_admin(&env)?;
        let _ = admin;

        let max_entries = read_max_entries(&env);
        let mut start_index = read_start_index(&env);
        let next_index = read_next_index(&env);
        let mut count = read_count(&env);

        if count == max_entries {
            env.storage()
                .persistent()
                .remove(&DataKey::Entry(start_index));
            start_index = start_index.checked_add(1).unwrap_or(0);
            env.storage().instance().set(&DataKey::StartIndex, &start_index);
        } else {
            count = count.checked_add(1).unwrap_or(count);
            env.storage().instance().set(&DataKey::Count, &count);
        }

        let timestamp = env.ledger().timestamp();
        let entry = AuditEntry {
            actor: actor.clone(),
            action: action.clone(),
            timestamp,
            details: details.clone(),
        };

        env.storage().persistent().set(&DataKey::Entry(next_index), &entry);
        env.storage().instance().set(&DataKey::NextIndex, &(next_index + 1));
        env.events().publish((EVENT_AUDIT, action, actor.clone()), details);

        Ok(next_index)
    }

    pub fn get_events(env: Env, start: u32, limit: u32) -> Vec<AuditEntry> {
        if limit == 0 {
            return Vec::new(&env);
        }

        let start_index = read_start_index(&env);
        let count = read_count(&env);
        if start >= count {
            return Vec::new(&env);
        }

        let mut result = Vec::new(&env);
        let mut index = start_index + start;
        let end = (index + limit).min(read_next_index(&env));

        while index < end {
            if let Some(entry) = env.storage().persistent().get(&DataKey::Entry(index)) {
                result.push_back(entry);
            }
            index += 1;
        }

        result
    }

    /// Return a page of events using an opaque cursor.
    ///
    /// The cursor is a stable absolute storage index. It may be reused across
    /// requests as long as the history window has not advanced past it.
    /// If the contract has trimmed older entries due to capacity limits, an
    /// expired cursor is rejected with `InvalidCursor`.
    pub fn get_events_page(
        env: Env,
        cursor: Option<HistoryCursor>,
        limit: u32,
    ) -> Result<HistoryPage, HistoryError> {
        if limit == 0 {
            return Ok(HistoryPage {
                entries: Vec::new(&env),
                next_cursor: None,
            });
        }

        let start_index = read_start_index(&env);
        let next_index = read_next_index(&env);
        let start = cursor.map_or(start_index, |c| c.index);

        if start < start_index || start > next_index {
            return Err(HistoryError::InvalidCursor);
        }

        if start >= next_index {
            return Ok(HistoryPage {
                entries: Vec::new(&env),
                next_cursor: None,
            });
        }

        let end = (start + limit).min(next_index);
        let mut result = Vec::new(&env);
        let mut index = start;

        while index < end {
            if let Some(entry) = env.storage().persistent().get(&DataKey::Entry(index)) {
                result.push_back(entry);
            }
            index += 1;
        }

        let next_cursor = if end >= next_index {
            None
        } else {
            Some(HistoryCursor { index: end })
        };

        Ok(HistoryPage {
            entries: result,
            next_cursor,
        })
    }

    pub fn query_by_time(
        env: Env,
        earliest: u64,
        latest: u64,
        limit: u32,
    ) -> Vec<AuditEntry> {
        if limit == 0 {
            return Vec::new(&env);
        }

        let mut result = Vec::new(&env);
        let mut index = read_start_index(&env);
        let next_index = read_next_index(&env);

        while index < next_index && result.len() < limit {
            let entry: Option<AuditEntry> = env.storage().persistent().get(&DataKey::Entry(index));
            if let Some(entry) = entry {
                if entry.timestamp >= earliest && entry.timestamp <= latest {
                    result.push_back(entry);
                }
            }
            index += 1;
        }

        result
    }

    pub fn history_stats(env: Env) -> HistoryStats {
        HistoryStats {
            start_index: read_start_index(&env),
            next_index: read_next_index(&env),
            count: read_count(&env),
            max_entries: read_max_entries(&env),
        }
    }
}

fn read_admin(env: &Env) -> Result<Address, HistoryError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(HistoryError::NotInitialized)
}

fn read_max_entries(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::MaxEntries)
        .unwrap_or(0)
}

fn read_start_index(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::StartIndex)
        .unwrap_or(0)
}

fn read_next_index(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::NextIndex)
        .unwrap_or(0)
}

fn read_count(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Count).unwrap_or(0)
}

#[cfg(test)]
mod test;

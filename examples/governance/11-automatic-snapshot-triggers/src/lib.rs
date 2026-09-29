#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotRecord {
    pub ledger: u32,
    pub timestamp: u64,
    pub value: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Frequency,
    LastSnapshot,
    TotalSnapshots,
    Enabled,
    PruneThreshold,
    Snapshots(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SnapshotError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidFrequency = 4,
    NoSnapshotDue = 5,
    NothingToPrune = 6,
    IndexOutOfBounds = 7,
    EmptyHistory = 8,
}

const EVENT_NS: Symbol = symbol_short!("snapshot");

#[contract]
pub struct SnapshotTrigger;

#[contractimpl]
impl SnapshotTrigger {
    pub fn initialize(env: Env, admin: Address, frequency: u32) -> Result<(), SnapshotError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(SnapshotError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Frequency, &frequency);
        env.storage().instance().set(&DataKey::LastSnapshot, &0u32);
        env.storage().instance().set(&DataKey::TotalSnapshots, &0u32);
        env.storage().instance().set(&DataKey::Enabled, &true);
        env.storage().instance().set(&DataKey::PruneThreshold, &0u32);
        env.events().publish(
            (EVENT_NS, symbol_short!("init")),
            (admin, frequency),
        );
        Ok(())
    }

    pub fn set_frequency(env: Env, admin: Address, frequency: u32) -> Result<(), SnapshotError> {
        Self::require_admin(&env, &admin)?;
        let old: u32 = env.storage().instance().get(&DataKey::Frequency).unwrap_or(0);
        env.storage().instance().set(&DataKey::Frequency, &frequency);
        env.events().publish(
            (EVENT_NS, symbol_short!("freq")),
            (old, frequency),
        );
        Ok(())
    }

    pub fn set_enabled(env: Env, admin: Address, enabled: bool) -> Result<(), SnapshotError> {
        Self::require_admin(&env, &admin)?;
        let old: bool = env.storage().instance().get(&DataKey::Enabled).unwrap_or(true);
        env.storage().instance().set(&DataKey::Enabled, &enabled);
        env.events().publish(
            (EVENT_NS, symbol_short!("enable")),
            (old, enabled),
        );
        Ok(())
    }

    pub fn set_prune_threshold(
        env: Env,
        admin: Address,
        older_than: u32,
    ) -> Result<(), SnapshotError> {
        Self::require_admin(&env, &admin)?;
        let old: u32 = env.storage().instance().get(&DataKey::PruneThreshold).unwrap_or(0);
        env.storage().instance().set(&DataKey::PruneThreshold, &older_than);
        env.events().publish(
            (EVENT_NS, symbol_short!("prune_cfg")),
            (old, older_than),
        );
        Ok(())
    }

    /// Event-based snapshot: always records a snapshot when called.
    pub fn record_value(env: Env, owner: Address, value: i128) {
        let now = env.ledger().sequence();
        let ts = env.ledger().timestamp();
        let record = SnapshotRecord {
            ledger: now,
            timestamp: ts,
            value,
        };
        let mut snapshots: Vec<SnapshotRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Snapshots(owner.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        snapshots.push_back(record);
        env.storage()
            .persistent()
            .set(&DataKey::Snapshots(owner.clone()), &snapshots);

        let mut total: u32 = env.storage().instance().get(&DataKey::TotalSnapshots).unwrap_or(0);
        total += 1;
        env.storage().instance().set(&DataKey::TotalSnapshots, &total);

        env.events().publish(
            (EVENT_NS, symbol_short!("record")),
            (owner, value, now),
        );
    }

    /// Time-based snapshot: only records if enough ledgers have elapsed.
    pub fn auto_snapshot(env: Env, owner: Address) -> Result<(), SnapshotError> {
        let enabled: bool = env.storage().instance().get(&DataKey::Enabled).unwrap_or(true);
        if !enabled {
            return Err(SnapshotError::NoSnapshotDue);
        }
        let frequency: u32 = env.storage().instance().get(&DataKey::Frequency).unwrap_or(1);
        if frequency == 0 {
            return Err(SnapshotError::InvalidFrequency);
        }
        let last: u32 = env.storage().instance().get(&DataKey::LastSnapshot).unwrap_or(0);
        let now = env.ledger().sequence();
        if now < last + frequency {
            return Err(SnapshotError::NoSnapshotDue);
        }
        env.storage().instance().set(&DataKey::LastSnapshot, &now);

        let value = Self::latest_value(&env, &owner);
        Self::record_value(env, owner, value);
        Ok(())
    }

    fn latest_value(env: &Env, owner: &Address) -> i128 {
        let snapshots: Vec<SnapshotRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Snapshots(owner.clone()))
            .unwrap_or_else(|| Vec::new(env));
        if snapshots.is_empty() {
            return 0;
        }
        snapshots.get_unchecked(snapshots.len() - 1).value
    }

    /// Prune snapshots for an owner that are older than the given ledger.
    pub fn prune(
        env: Env,
        admin: Address,
        owner: Address,
        older_than: u32,
    ) -> Result<u32, SnapshotError> {
        Self::require_admin(&env, &admin)?;
        let snapshots: Vec<SnapshotRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Snapshots(owner.clone()))
            .ok_or(SnapshotError::NothingToPrune)?;
        let original_len = snapshots.len();
        let mut kept: Vec<SnapshotRecord> = Vec::new(&env);
        for i in 0..original_len {
            let s = snapshots.get_unchecked(i);
            if s.ledger >= older_than {
                kept.push_back(s);
            }
        }
        let pruned = original_len - kept.len();
        if pruned == 0 {
            return Err(SnapshotError::NothingToPrune);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Snapshots(owner.clone()), &kept);

        env.events().publish(
            (EVENT_NS, symbol_short!("prune")),
            (admin, owner, pruned),
        );
        Ok(pruned)
    }

    pub fn get_snapshot(env: Env, owner: Address, index: u32) -> Result<SnapshotRecord, SnapshotError> {
        let snapshots: Vec<SnapshotRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Snapshots(owner))
            .ok_or(SnapshotError::EmptyHistory)?;
        if index >= snapshots.len() {
            return Err(SnapshotError::IndexOutOfBounds);
        }
        Ok(snapshots.get_unchecked(index))
    }

    pub fn get_latest(env: Env, owner: Address) -> Result<SnapshotRecord, SnapshotError> {
        let snapshots: Vec<SnapshotRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Snapshots(owner))
            .ok_or(SnapshotError::EmptyHistory)?;
        if snapshots.is_empty() {
            return Err(SnapshotError::EmptyHistory);
        }
        Ok(snapshots.get_unchecked(snapshots.len() - 1))
    }

    pub fn snapshot_count(env: Env, owner: Address) -> u32 {
        env.storage()
            .persistent()
            .get::<_, Vec<SnapshotRecord>>(&DataKey::Snapshots(owner))
            .map(|s| s.len())
            .unwrap_or(0)
    }

    pub fn get_all_snapshots(env: Env, owner: Address) -> Vec<SnapshotRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::Snapshots(owner))
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_config(env: Env) -> (u32, bool, u32, u32) {
        let frequency: u32 = env.storage().instance().get(&DataKey::Frequency).unwrap_or(0);
        let enabled: bool = env.storage().instance().get(&DataKey::Enabled).unwrap_or(true);
        let last: u32 = env.storage().instance().get(&DataKey::LastSnapshot).unwrap_or(0);
        let total: u32 = env.storage().instance().get(&DataKey::TotalSnapshots).unwrap_or(0);
        (frequency, enabled, last, total)
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), SnapshotError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(SnapshotError::NotInitialized)?;
        if caller != &admin {
            return Err(SnapshotError::Unauthorized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;

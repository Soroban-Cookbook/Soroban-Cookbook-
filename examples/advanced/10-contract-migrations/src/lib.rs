//! # Contract Migrations (Advanced)
//!
//! End-to-end storage migration for live contracts: prepare → batched migrate →
//! finalize, with version gates so v2 entry points refuse to run on unmigrated
//! data.
//!
//! Complements [`07-upgrade-patterns`](../07-upgrade-patterns/) (single-key
//! schema change + WASM upgrade) and the intermediate `storage-migration`
//! example (batched rewrite). This crate combines both: admin-gated prepare,
//! gas-bounded batches, dual-read during migration, and a final version bump.
//!
//! ## Schema change modelled here
//!
//! | Version | User record |
//! |---------|-------------|
//! | v1 | `LegacyAccount { balance: i128 }` under `DataKey::Legacy(Address)` |
//! | v2 | `AccountV2 { balance: i128, last_active: u64, tier: u32 }` under `DataKey::Account(Address)` |
//!
//! During migration, reads prefer v2 and fall back to v1 so the contract stays
//! usable while batches run.

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    Symbol, Vec,
};

pub const VERSION_V1: u32 = 1;
pub const VERSION_V2: u32 = 2;
pub const CURRENT_VERSION: u32 = VERSION_V2;

const NS: Symbol = symbol_short!("migrate");
const EV_INIT: Symbol = symbol_short!("init");
const EV_PREP: Symbol = symbol_short!("prepare");
const EV_BATCH: Symbol = symbol_short!("batch");
const EV_DONE: Symbol = symbol_short!("done");
const EV_UPGRADE: Symbol = symbol_short!("upgrade");

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Version,
    MigrationState,
    UserList,
    /// v1 layout — removed entry-by-entry during migration.
    Legacy(Address),
    /// v2 layout.
    Account(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyAccount {
    pub balance: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountV2 {
    pub balance: i128,
    pub last_active: u64,
    pub tier: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrationState {
    None,
    /// `(target_version, next_index)`
    InProgress(u32, u32),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MigrationError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidVersion = 4,
    MigrationNotPrepared = 5,
    MigrationAlreadyPrepared = 6,
    NoMoreEntries = 7,
    InvalidBatchSize = 8,
    InvalidAmount = 9,
    /// v2 entry point called before migration finished.
    MigrationRequired = 10,
    AlreadyMigrated = 11,
}

#[contract]
pub struct ContractMigrations;

#[contractimpl]
impl ContractMigrations {
    pub fn initialize(env: Env, admin: Address) -> Result<(), MigrationError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(MigrationError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Version, &VERSION_V1);
        env.storage()
            .instance()
            .set(&DataKey::UserList, &Vec::<Address>::new(&env));
        env.storage()
            .instance()
            .set(&DataKey::MigrationState, &MigrationState::None);

        env.events().publish((NS, EV_INIT, admin), VERSION_V1);
        Ok(())
    }

    /// Seed a v1 account (admin only). Used to build state before migrating.
    pub fn add_user(env: Env, user: Address, balance: i128) -> Result<(), MigrationError> {
        require_admin(&env)?;
        if balance < 0 {
            return Err(MigrationError::InvalidAmount);
        }
        if env.storage().instance().get(&DataKey::Version).unwrap_or(0) != VERSION_V1 {
            return Err(MigrationError::InvalidVersion);
        }
        if !matches!(read_migration_state(&env), MigrationState::None) {
            return Err(MigrationError::MigrationAlreadyPrepared);
        }

        let mut users: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::UserList)
            .unwrap_or_else(|| Vec::new(&env));
        users.push_back(user.clone());
        env.storage().instance().set(&DataKey::UserList, &users);
        env.storage()
            .persistent()
            .set(&DataKey::Legacy(user), &LegacyAccount { balance });
        Ok(())
    }

    /// Begin a migration toward `target_version` (must be `CURRENT_VERSION`).
    pub fn prepare_migration(env: Env, target_version: u32) -> Result<(), MigrationError> {
        require_admin(&env)?;
        let current = read_version(&env);
        if current >= target_version || target_version != CURRENT_VERSION {
            return Err(MigrationError::InvalidVersion);
        }
        if !matches!(read_migration_state(&env), MigrationState::None) {
            return Err(MigrationError::MigrationAlreadyPrepared);
        }

        env.storage().instance().set(
            &DataKey::MigrationState,
            &MigrationState::InProgress(target_version, 0),
        );
        env.events()
            .publish((NS, EV_PREP), (current, target_version));
        Ok(())
    }

    /// Migrate up to `batch_size` users from v1 → v2. Returns how many were
    /// processed. When the list is exhausted, bumps `Version` and clears state.
    pub fn migrate_batch(env: Env, batch_size: u32) -> Result<u32, MigrationError> {
        require_admin(&env)?;
        if batch_size == 0 || batch_size > 100 {
            return Err(MigrationError::InvalidBatchSize);
        }

        let (target_version, mut next_index) = match read_migration_state(&env) {
            MigrationState::InProgress(t, i) => (t, i),
            MigrationState::None => return Err(MigrationError::MigrationNotPrepared),
        };

        let users: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::UserList)
            .unwrap_or_else(|| Vec::new(&env));
        let total = users.len();
        if next_index >= total {
            return Err(MigrationError::NoMoreEntries);
        }

        let now = env.ledger().timestamp();
        let mut processed = 0u32;
        while processed < batch_size && next_index < total {
            let user = users.get(next_index).unwrap();
            let legacy: LegacyAccount = env
                .storage()
                .persistent()
                .get(&DataKey::Legacy(user.clone()))
                .unwrap_or(LegacyAccount { balance: 0 });

            let tier = if legacy.balance >= 10_000 {
                2
            } else if legacy.balance >= 1_000 {
                1
            } else {
                0
            };

            env.storage().persistent().set(
                &DataKey::Account(user.clone()),
                &AccountV2 {
                    balance: legacy.balance,
                    last_active: now,
                    tier,
                },
            );
            env.storage().persistent().remove(&DataKey::Legacy(user));

            processed += 1;
            next_index += 1;
        }

        if next_index >= total {
            env.storage()
                .instance()
                .set(&DataKey::Version, &target_version);
            env.storage()
                .instance()
                .set(&DataKey::MigrationState, &MigrationState::None);
            env.events()
                .publish((NS, EV_DONE), (processed, target_version));
        } else {
            env.storage().instance().set(
                &DataKey::MigrationState,
                &MigrationState::InProgress(target_version, next_index),
            );
            env.events()
                .publish((NS, EV_BATCH), (processed, next_index));
        }

        Ok(processed)
    }

    pub fn cancel_migration(env: Env) -> Result<(), MigrationError> {
        require_admin(&env)?;
        if matches!(read_migration_state(&env), MigrationState::None) {
            return Err(MigrationError::MigrationNotPrepared);
        }
        env.storage()
            .instance()
            .set(&DataKey::MigrationState, &MigrationState::None);
        Ok(())
    }

    /// v2-only credit. Refuses to run until migration has finished.
    pub fn credit(env: Env, user: Address, amount: i128) -> Result<AccountV2, MigrationError> {
        require_admin(&env)?;
        require_migrated(&env)?;
        if amount <= 0 {
            return Err(MigrationError::InvalidAmount);
        }

        let mut account = read_account_v2(&env, &user)?.ok_or(MigrationError::NotInitialized)?;
        account.balance = account
            .balance
            .checked_add(amount)
            .ok_or(MigrationError::InvalidAmount)?;
        account.last_active = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Account(user), &account);
        Ok(account)
    }

    /// Dual-read helper: prefer v2, fall back to v1 while migration is running.
    pub fn balance_of(env: Env, user: Address) -> i128 {
        if let Some(v2) = env
            .storage()
            .persistent()
            .get::<_, AccountV2>(&DataKey::Account(user.clone()))
        {
            return v2.balance;
        }
        env.storage()
            .persistent()
            .get::<_, LegacyAccount>(&DataKey::Legacy(user))
            .map(|a| a.balance)
            .unwrap_or(0)
    }

    pub fn get_account(env: Env, user: Address) -> Option<AccountV2> {
        env.storage().persistent().get(&DataKey::Account(user))
    }

    pub fn get_version(env: Env) -> u32 {
        read_version(&env)
    }

    pub fn migration_state(env: Env) -> MigrationState {
        read_migration_state(&env)
    }

    pub fn user_count(env: Env) -> u32 {
        let users: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::UserList)
            .unwrap_or_else(|| Vec::new(&env));
        users.len()
    }

    /// Admin-gated WASM swap. Call `prepare_migration` + `migrate_batch` after
    /// deploying a schema-changing binary. Host may reject in unit tests.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), MigrationError> {
        require_admin(&env)?;
        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
        env.events().publish((NS, EV_UPGRADE), new_wasm_hash);
        Ok(())
    }
}

fn require_admin(env: &Env) -> Result<(), MigrationError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(MigrationError::NotInitialized)?;
    admin.require_auth();
    Ok(())
}

fn require_migrated(env: &Env) -> Result<(), MigrationError> {
    if read_version(env) < CURRENT_VERSION {
        return Err(MigrationError::MigrationRequired);
    }
    if !matches!(read_migration_state(env), MigrationState::None) {
        return Err(MigrationError::MigrationRequired);
    }
    Ok(())
}

fn read_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::Version)
        .unwrap_or(VERSION_V1)
}

fn read_migration_state(env: &Env) -> MigrationState {
    env.storage()
        .instance()
        .get(&DataKey::MigrationState)
        .unwrap_or(MigrationState::None)
}

fn read_account_v2(env: &Env, user: &Address) -> Result<Option<AccountV2>, MigrationError> {
    Ok(env
        .storage()
        .persistent()
        .get(&DataKey::Account(user.clone())))
}

#[cfg(test)]
mod test;

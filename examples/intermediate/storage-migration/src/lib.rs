//! # Storage Migration Example
//!
//! Demonstrates a versioned migration pattern for upgrading contract storage safely.

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Version,
    UserList,
    LegacyBalance(Address),
    Profile(Address),
    MigrationState,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrationState {
    None,
    Prepared(u32, u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    pub balance: i128,
    pub member_since: u64,
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
}

const DEFAULT_VERSION: u32 = 1;

#[contract]
pub struct StorageMigration;

#[contractimpl]
impl StorageMigration {
    pub fn initialize(env: Env, admin: Address) -> Result<(), MigrationError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(MigrationError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Version, &DEFAULT_VERSION);
        env.storage()
            .instance()
            .set(&DataKey::UserList, &Vec::<Address>::new(&env));
        env.storage()
            .instance()
            .set(&DataKey::MigrationState, &MigrationState::None);

        Ok(())
    }

    pub fn add_user(env: Env, user: Address, balance: i128) -> Result<(), MigrationError> {
        admin_require_auth(&env)?;
        if balance < 0 {
            return Err(MigrationError::InvalidBatchSize);
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
            .set(&DataKey::LegacyBalance(user), &balance);

        Ok(())
    }

    pub fn prepare_migration(env: Env, target_version: u32) -> Result<(), MigrationError> {
        admin_require_auth(&env)?;
        let current_version = read_version(&env);
        if current_version >= target_version {
            return Err(MigrationError::InvalidVersion);
        }
        if !matches!(read_migration_state(&env), MigrationState::None) {
            return Err(MigrationError::MigrationAlreadyPrepared);
        }
        env.storage().instance().set(
            &DataKey::MigrationState,
            &MigrationState::Prepared(target_version, 0),
        );
        Ok(())
    }

    pub fn migrate_batch(env: Env, batch_size: u32) -> Result<u32, MigrationError> {
        admin_require_auth(&env)?;
        if batch_size == 0 {
            return Err(MigrationError::InvalidBatchSize);
        }

        let state = read_migration_state(&env);
        let (target_version, mut next_index) = match state {
            MigrationState::Prepared(target_version, next_index) => (target_version, next_index),
            _ => return Err(MigrationError::MigrationNotPrepared),
        };

        let users: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::UserList)
            .unwrap_or_else(|| Vec::new(&env));

        let total_entries = users.len();
        if next_index >= total_entries {
            return Err(MigrationError::NoMoreEntries);
        }

        let mut processed = 0u32;
        while processed < batch_size && next_index < total_entries {
            let user = users.get(next_index).unwrap().clone();
            let legacy_balance: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::LegacyBalance(user.clone()))
                .unwrap_or(0);
            let profile = Profile {
                balance: legacy_balance,
                member_since: env.ledger().timestamp(),
            };
            env.storage()
                .persistent()
                .set(&DataKey::Profile(user.clone()), &profile);
            env.storage().persistent().remove(&DataKey::LegacyBalance(user.clone()));

            processed += 1;
            next_index += 1;
        }

        if next_index >= total_entries {
            env.storage().instance().set(&DataKey::Version, &target_version);
            env.storage()
                .instance()
                .set(&DataKey::MigrationState, &MigrationState::None);
        } else {
            env.storage().instance().set(
                &DataKey::MigrationState,
                &MigrationState::Prepared(target_version, next_index),
            );
        }

        Ok(processed)
    }

    pub fn cancel_migration(env: Env) -> Result<(), MigrationError> {
        admin_require_auth(&env)?;
        env.storage()
            .instance()
            .set(&DataKey::MigrationState, &MigrationState::None);
        Ok(())
    }

    pub fn get_version(env: Env) -> u32 {
        read_version(&env)
    }

    pub fn migration_state(env: Env) -> MigrationState {
        read_migration_state(&env)
    }

    pub fn legacy_balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::LegacyBalance(user))
            .unwrap_or(0)
    }

    pub fn profile(env: Env, user: Address) -> Option<Profile> {
        env.storage().persistent().get(&DataKey::Profile(user))
    }
}

fn admin_require_auth(env: &Env) -> Result<(), MigrationError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(MigrationError::NotInitialized)?;
    admin.require_auth();
    Ok(())
}

fn read_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::Version)
        .unwrap_or(DEFAULT_VERSION)
}

fn read_migration_state(env: &Env) -> MigrationState {
    env.storage()
        .instance()
        .get(&DataKey::MigrationState)
        .unwrap_or(MigrationState::None)
}

#[cfg(test)]
mod test;

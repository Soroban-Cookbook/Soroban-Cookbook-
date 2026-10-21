#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Symbol, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StorageError {
    NotAuthorized = 1,
    NotFound = 2,
    AlreadyExists = 3,
}

/// Packed storage: multiple fields stored as a single struct
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackedUserData {
    pub balance: i128,
    pub nonce: u64,
    pub flags: u32,
    pub delegate: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactConfig {
    pub admin: Address,
    pub paused: bool,
    pub fee_bps: u32,
    pub min_deposit: i128,
    pub max_deposit: i128,
}

#[contracttype]
pub enum DataKey {
    Config,
    User(Address),
    BatchCounter,
    BatchResult(Symbol),
}

#[contract]
pub struct StorageOptimization;

#[contractimpl]
impl StorageOptimization {
    /// Initialize with packed config (single storage entry instead of 5)
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Config) {
            panic!("Already initialized");
        }
        let config = CompactConfig {
            admin: admin.clone(),
            paused: false,
            fee_bps: 25,
            min_deposit: 100,
            max_deposit: 1_000_000,
        };
        env.storage().instance().set(&DataKey::Config, &config);
    }

    /// Lazy loading: read config only when needed
    pub fn get_config(env: Env) -> CompactConfig {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .expect("Not initialized")
    }

    /// Deposit with packed user storage
    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();

        let config: CompactConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("Not initialized");
        if config.paused {
            panic!("Contract is paused");
        }
        if amount < config.min_deposit || amount > config.max_deposit {
            panic!("Amount out of range");
        }

        let mut data: PackedUserData = env
            .storage()
            .persistent()
            .get(&DataKey::User(user.clone()))
            .unwrap_or(PackedUserData {
                balance: 0,
                nonce: 0,
                flags: 0,
                delegate: user.clone(),
            });

        data.balance += amount;
        data.nonce += 1;

        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &data);
    }

    /// Withdraw and return new balance
    pub fn withdraw(env: Env, user: Address, amount: i128) -> i128 {
        user.require_auth();

        let config: CompactConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("Not initialized");
        if config.paused {
            panic!("Contract is paused");
        }

        let mut data: PackedUserData = env
            .storage()
            .persistent()
            .get(&DataKey::User(user.clone()))
            .expect("User not found");

        if data.balance < amount {
            panic!("Insufficient balance");
        }

        data.balance -= amount;
        data.nonce += 1;

        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &data);

        data.balance
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get::<_, PackedUserData>(&DataKey::User(user))
            .map_or(0, |d| d.balance)
    }

    pub fn get_user_data(env: Env, user: Address) -> PackedUserData {
        env.storage()
            .persistent()
            .get(&DataKey::User(user))
            .expect("User not found")
    }

    /// Batch get balances (lazy loading pattern)
    pub fn batch_get_balances(env: Env, users: Vec<Address>) -> Vec<i128> {
        let mut balances: Vec<i128> = Vec::new(&env);
        for user in users.iter() {
            balances.push_back(Self::get_balance(env.clone(), user));
        }
        balances
    }

    /// Batch deposit to multiple users
    pub fn batch_deposit(env: Env, deposits: Vec<(Address, i128)>) -> u32 {
        let config: CompactConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("Not initialized");
        if config.paused {
            panic!("Contract is paused");
        }

        let count: u32 = deposits.len();
        for (user, amount) in deposits.iter() {
            let mut data: PackedUserData = env
                .storage()
                .persistent()
                .get(&DataKey::User(user.clone()))
                .unwrap_or(PackedUserData {
                    balance: 0,
                    nonce: 0,
                    flags: 0,
                    delegate: user.clone(),
                });

            data.balance += amount;
            data.nonce += 1;

            env.storage()
                .persistent()
                .set(&DataKey::User(user.clone()), &data);
        }

        env.storage().instance().set(&DataKey::BatchCounter, &count);
        count
    }

    pub fn get_batch_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::BatchCounter)
            .unwrap_or(0)
    }

    /// Admin update packed fee config
    pub fn update_fee(env: Env, fee_bps: u32) {
        let mut config: CompactConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("Not initialized");
        config.admin.require_auth();

        if fee_bps > 10000 {
            panic!("Fee too high");
        }
        config.fee_bps = fee_bps;
        env.storage().instance().set(&DataKey::Config, &config);
    }

    pub fn set_paused(env: Env, paused: bool) {
        let mut config: CompactConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("Not initialized");
        config.admin.require_auth();
        config.paused = paused;
        env.storage().instance().set(&DataKey::Config, &config);
    }
}

#[cfg(test)]
mod test;

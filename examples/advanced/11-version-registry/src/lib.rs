#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

extern crate alloc;

use alloc::format;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VersionError {
    NotAuthorized = 1,
    NotFound = 2,
    AlreadyAtVersion = 3,
    EmptyHistory = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionEntry {
    pub version: Symbol,
    pub contract_address: Address,
    pub hash: BytesN<32>,
    pub timestamp: u64,
    pub metadata: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Versions,
    CurrentVersion,
    History(Address),
}

#[contract]
pub struct VersionRegistry;

#[contractimpl]
impl VersionRegistry {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Versions, &Vec::<VersionEntry>::new(&env));
        env.storage()
            .instance()
            .set(&DataKey::CurrentVersion, &0_u32);

        env.events()
            .publish((symbol_short!("version"), symbol_short!("init"), admin), ());
    }

    pub fn register(
        env: Env,
        contract_address: Address,
        hash: BytesN<32>,
        metadata: Symbol,
    ) -> Result<VersionEntry, VersionError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let mut versions: Vec<VersionEntry> = env
            .storage()
            .instance()
            .get(&DataKey::Versions)
            .unwrap_or(Vec::new(&env));

        let current: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentVersion)
            .unwrap_or(0);

        let version_num = current + 1;
        let version_str = Symbol::new(&env, &format!("v{}", version_num));
        let timestamp = env.ledger().timestamp();

        let entry = VersionEntry {
            version: version_str.clone(),
            contract_address: contract_address.clone(),
            hash: hash.clone(),
            timestamp,
            metadata: metadata.clone(),
        };

        versions.push_back(entry.clone());
        env.storage().instance().set(&DataKey::Versions, &versions);
        env.storage()
            .instance()
            .set(&DataKey::CurrentVersion, &version_num);

        let mut history: Vec<VersionEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::History(contract_address.clone()))
            .unwrap_or(Vec::new(&env));
        history.push_back(entry.clone());
        env.storage()
            .persistent()
            .set(&DataKey::History(contract_address.clone()), &history);

        env.events().publish(
            (
                symbol_short!("version"),
                symbol_short!("register"),
                contract_address,
            ),
            entry.clone(),
        );

        Ok(entry)
    }

    pub fn get_all_versions(env: Env) -> Vec<VersionEntry> {
        env.storage()
            .instance()
            .get(&DataKey::Versions)
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_latest_version(env: Env) -> Result<VersionEntry, VersionError> {
        let versions: Vec<VersionEntry> = env
            .storage()
            .instance()
            .get(&DataKey::Versions)
            .unwrap_or(Vec::new(&env));
        versions.last().ok_or(VersionError::NotFound)
    }

    pub fn get_version_by_number(env: Env, number: u32) -> Result<VersionEntry, VersionError> {
        let versions: Vec<VersionEntry> = env
            .storage()
            .instance()
            .get(&DataKey::Versions)
            .unwrap_or(Vec::new(&env));
        let idx: u32 = number.checked_sub(1).ok_or(VersionError::NotFound)?;
        versions.get(idx).ok_or(VersionError::NotFound)
    }

    pub fn rollback(env: Env) -> Result<VersionEntry, VersionError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let mut versions: Vec<VersionEntry> = env
            .storage()
            .instance()
            .get(&DataKey::Versions)
            .unwrap_or(Vec::new(&env));

        let removed = versions.pop_back().ok_or(VersionError::EmptyHistory)?;
        env.storage().instance().set(&DataKey::Versions, &versions);

        let current: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentVersion)
            .unwrap_or(0);
        if current > 0 {
            env.storage()
                .instance()
                .set(&DataKey::CurrentVersion, &(current - 1));
        }

        env.events().publish(
            (symbol_short!("version"), symbol_short!("rollback")),
            removed.clone(),
        );

        Ok(removed)
    }

    pub fn get_contract_history(env: Env, contract_address: Address) -> Vec<VersionEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::History(contract_address))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_current_version_number(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CurrentVersion)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;

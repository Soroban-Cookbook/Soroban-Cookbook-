#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    AlreadyRegistered = 1,
    NotFound = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractMetadata {
    pub name: Symbol,
    pub category: Symbol,
    pub version: Symbol,
    pub address: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryKey {
    Entry(Symbol),
    Category(Symbol),
    Categories,
    Count,
}

#[contract]
pub struct ContractRegistry;

#[contractimpl]
impl ContractRegistry {
    /// Register a contract under `name` with `category`, `version` and `address`.
    /// Fails if `name` is already registered.
    pub fn register(
        env: Env,
        name: Symbol,
        category: Symbol,
        version: Symbol,
        address: Address,
    ) -> Result<(), RegistryError> {
        let entry_key = RegistryKey::Entry(name.clone());
        if env.storage().persistent().has(&entry_key) {
            return Err(RegistryError::AlreadyRegistered);
        }

        let metadata = ContractMetadata {
            name: name.clone(),
            category: category.clone(),
            version: version.clone(),
            address: address.clone(),
        };

        env.storage().persistent().set(&entry_key, &metadata);

        // Increment the running count for paging off-chain tooling.
        let mut count: u32 = env
            .storage()
            .persistent()
            .get(&RegistryKey::Count)
            .unwrap_or(0);
        count = count.saturating_add(1);
        env.storage().persistent().set(&RegistryKey::Count, &count);

        // Add name to category index
        let cat_key = RegistryKey::Category(category.clone());
        let mut names: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&cat_key)
            .unwrap_or(Vec::new(&env));
        names.push_back(name.clone());
        env.storage().persistent().set(&cat_key, &names);

        // Track known categories
        let mut cats: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&RegistryKey::Categories)
            .unwrap_or(Vec::new(&env));
        if !cats.iter().any(|s| s == category) {
            cats.push_back(category.clone());
            env.storage()
                .persistent()
                .set(&RegistryKey::Categories, &cats);
        }

        env.events()
            .publish((symbol_short!("reg"), name), metadata.clone());
        Ok(())
    }

    /// Get metadata by registered `name`.
    pub fn get_by_name(env: Env, name: Symbol) -> Result<ContractMetadata, RegistryError> {
        env.storage()
            .persistent()
            .get(&RegistryKey::Entry(name.clone()))
            .ok_or(RegistryError::NotFound)
    }

    /// List registered names for a `category`.
    pub fn list_by_category(env: Env, category: Symbol) -> Vec<Symbol> {
        env.storage()
            .persistent()
            .get(&RegistryKey::Category(category))
            .unwrap_or(Vec::new(&env))
    }

    /// List known categories.
    pub fn list_categories(env: Env) -> Vec<Symbol> {
        env.storage()
            .persistent()
            .get(&RegistryKey::Categories)
            .unwrap_or(Vec::new(&env))
    }

    /// Total number of registered entries — the page bound for off-chain
    /// tooling that iterates the index.
    pub fn count(env: Env) -> u32 {
        env.storage()
            .persistent()
            .get(&RegistryKey::Count)
            .unwrap_or(0)
    }

    /// Remove an entry and clean up its category index. Returns an error when
    /// the name was never registered.
    pub fn deregister(env: Env, name: Symbol) -> Result<(), RegistryError> {
        let entry_key = RegistryKey::Entry(name.clone());
        let metadata = env
            .storage()
            .persistent()
            .get::<_, ContractMetadata>(&entry_key)
            .ok_or(RegistryError::NotFound)?;

        env.storage().persistent().remove(&entry_key);

        // Drop the name from its category index.
        let cat_key = RegistryKey::Category(metadata.category);
        if let Some(names) = env.storage().persistent().get::<_, Vec<Symbol>>(&cat_key) {
            let mut remaining: Vec<Symbol> = Vec::new(&env);
            for candidate in names.iter() {
                if candidate != name {
                    remaining.push_back(candidate);
                }
            }
            if remaining.is_empty() {
                env.storage().persistent().remove(&cat_key);
            } else {
                env.storage().persistent().set(&cat_key, &remaining);
            }
        }

        let count: u32 = env
            .storage()
            .persistent()
            .get(&RegistryKey::Count)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&RegistryKey::Count, &count.saturating_sub(1));

        env.events().publish((symbol_short!("drop"), name), ());
        Ok(())
    }
}

#[cfg(test)]
mod test;

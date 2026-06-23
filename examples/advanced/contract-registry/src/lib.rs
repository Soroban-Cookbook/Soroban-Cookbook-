#![no_std]

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
        if !cats.iter().any(|s| *s == category) {
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
}

#[cfg(test)]
mod test;

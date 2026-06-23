//! # Cross-Contract Optimization
//!
//! This example compares an unoptimized sequential caller contract with an
//! optimized batched caller contract. The target contract stores and updates
//! numeric values keyed by `Symbol`.

#![no_std]

#[cfg(test)]
extern crate std;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Entry(Symbol),
}

#[contracttype]
#[derive(Clone)]
pub struct PackedUpdate {
    pub key: Symbol,
    pub delta: i128,
}

#[contract]
pub struct TargetContract;

#[contractimpl]
impl TargetContract {
    /// Increment the stored value for a symbol by a delta.
    pub fn update_entry(env: Env, key: Symbol, delta: i128) -> i128 {
        let current: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Entry(key.clone()))
            .unwrap_or(0);
        let updated = current + delta;
        env.storage()
            .persistent()
            .set(&DataKey::Entry(key), &updated);
        updated
    }

    /// Update a batch of entries in a single cross-contract invocation.
    pub fn batch_update_entries(env: Env, updates: Vec<PackedUpdate>) -> i128 {
        let mut last_value = 0i128;
        for update in updates.iter() {
            last_value = Self::update_entry(env.clone(), update.key.clone(), update.delta);
        }
        last_value
    }

    /// Read the stored value for a symbol.
    pub fn get_entry(env: Env, key: Symbol) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Entry(key))
            .unwrap_or(0)
    }
}

#[contract]
pub struct UnoptimizedCaller;

#[contractimpl]
impl UnoptimizedCaller {
    /// Invoke the target contract sequentially for each packed update.
    pub fn invoke_updates_sequential(
        env: Env,
        target: Address,
        updates: Vec<PackedUpdate>,
    ) -> i128 {
        let client = TargetContractClient::new(&env, &target);
        let mut last_value = 0i128;
        for update in updates.iter() {
            last_value = client.update_entry(&update.key, &update.delta);
        }
        last_value
    }
}

#[contract]
pub struct OptimizedCaller;

#[contractimpl]
impl OptimizedCaller {
    /// Invoke the target contract once with a batched update argument.
    pub fn invoke_updates_batched(env: Env, target: Address, updates: Vec<PackedUpdate>) -> i128 {
        let client = TargetContractClient::new(&env, &target);
        client.batch_update_entries(&updates)
    }
}

#[cfg(test)]
mod test;

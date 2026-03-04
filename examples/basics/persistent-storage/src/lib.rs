#![no_std]
// Removed unused Symbol and symbol_short to clear the warnings
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Counter,
}

#[contract]
pub struct PersistentStorageContract;

#[contractimpl]
impl PersistentStorageContract {
    pub fn set_admin(env: Env, address: Address) {
        let key = DataKey::Admin;
        env.storage().persistent().set(&key, &address);
        env.storage().persistent().extend_ttl(&key, 2000, 10000);
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Admin)
    }

    pub fn increment(env: Env) -> u64 {
        let key = DataKey::Counter;
        let mut count: u64 = env.storage().persistent().get(&key).unwrap_or(0);

        count = count.checked_add(1).expect("counter overflow");
        env.storage().persistent().set(&key, &count);
        env.storage().persistent().extend_ttl(&key, 2000, 10000);

        count
    }

    // --- ADD THIS METHOD ---
    /// Retrieves the current counter value.
    /// This allows the test client to verify the increment logic.
    pub fn get_counter(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Counter)
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[cfg(test)]
mod test;

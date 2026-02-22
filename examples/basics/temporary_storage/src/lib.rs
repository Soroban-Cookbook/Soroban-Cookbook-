#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env};

#[contracttype]
#[derive(Clone)]
pub enum TempKey {
    ReentrancyGuard, // Flag to prevent recursive calls
    InternalResult,  // Store intermediate calculation
}

#[contract]
pub struct TemporaryStorageContract;

#[contractimpl]
impl TemporaryStorageContract {
    /// Demonstrates a Reentrancy Guard using temporary storage.
    /// This is a classic use case: we only need to know if a function
    /// is currently executing within the SAME transaction.
    pub fn guarded_function(env: Env) {
        let key = TempKey::ReentrancyGuard;

        // 1. Check if the flag exists
        if env.storage().temporary().has(&key) {
            panic!("Reentrancy forbidden");
        }

        // 2. Set the flag (cheapest storage write possible)
        env.storage().temporary().set(&key, &true);

        // ... logic of the contract ...

        // 3. Remove it (optional, but good practice)
        env.storage().temporary().remove(&key);
    }

    /// Demonstrates storing an intermediate result.
    /// Useful for cross-function state within a single invocation.
    pub fn start_calculation(env: Env, value: u64) {
        let key = TempKey::InternalResult;

        // Storing a temporary value for a later step in the same ledger
        env.storage().temporary().set(&key, &value);

        // Even temporary storage has a TTL, but it's very short by default.
        // We extend it slightly to ensure it lasts the current ledger cycle.
        env.storage().temporary().extend_ttl(&key, 16, 32);
    }

    pub fn get_temp_value(env: Env) -> u64 {
        env.storage()
            .temporary()
            .get(&TempKey::InternalResult)
            .unwrap_or(0)
    }
}

mod test;

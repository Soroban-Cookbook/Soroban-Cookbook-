//! # Ajo Factory Example
//!
//! This example demonstrates the factory pattern in Soroban:
//! 1. **Ajo Contract**: The template contract to be deployed.
//! 2. **AjoFactory Contract**: A factory that spawns new Ajo instances.
//!
//! This pattern is the Soroban equivalent of Ethereum's EIP-1167 clones,
//! using `env.deployer()` to deploy multiple instances of the same Wasm hash.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Ajo Contract (The Template)
// ---------------------------------------------------------------------------

#[contract]
pub struct Ajo;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AjoDataKey {
    Amount,
    MaxMembers,
    Creator,
}

#[contractimpl]
impl Ajo {
    /// Initialize a new Ajo instance.
    pub fn initialize(env: Env, amount: i128, max_members: u32, creator: Address) {
        // Prevent re-initialization
        if env.storage().instance().has(&AjoDataKey::Creator) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&AjoDataKey::Amount, &amount);
        env.storage()
            .instance()
            .set(&AjoDataKey::MaxMembers, &max_members);
        env.storage().instance().set(&AjoDataKey::Creator, &creator);
    }

    pub fn get_creator(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&AjoDataKey::Creator)
            .expect("Not initialized")
    }

    pub fn get_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&AjoDataKey::Amount)
            .expect("Not initialized")
    }
}

// ---------------------------------------------------------------------------
// AjoFactory Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AjoFactory;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactoryDataKey {
    WasmHash,
    DeployedAjos,
}

#[contractimpl]
impl AjoFactory {
    /// Set the Wasm hash of the Ajo contract to be deployed.
    pub fn initialize(env: Env, wasm_hash: BytesN<32>) {
        if env.storage().instance().has(&FactoryDataKey::WasmHash) {
            panic!("Factory already initialized");
        }
        env.storage()
            .instance()
            .set(&FactoryDataKey::WasmHash, &wasm_hash);
        
        // Initialize an empty list of deployed Ajos
        let ajos: Vec<Address> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&FactoryDataKey::DeployedAjos, &ajos);
    }

    /// Create a new Ajo instance.
    pub fn create_ajo(env: Env, amount: i128, max_members: u32, creator: Address) -> Address {
        creator.require_auth();

        // Get the Wasm hash
        let wasm_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::WasmHash)
            .expect("Factory not initialized");

        // Generate a salt for unique deployment
        // Using the current number of deployed Ajos as a salt component
        let mut ajos: Vec<Address> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::DeployedAjos)
            .unwrap_or(Vec::new(&env));

        // Deployment salt: combining creator address and a sequence number
        let salt = env.crypto().sha256(&(&creator, ajos.len()).into_val(&env));

        // Deploy the contract
        let deployed_address = env
            .deployer()
            .with_current_contract(salt)
            .deploy(wasm_hash);

        // Initialize the new Ajo instance
        // We use the Ajo contract client to call its initialize method
        let ajo_client = AjoClient::new(&env, &deployed_address);
        ajo_client.initialize(&amount, &max_members, &creator);

        // Track the deployed Ajo
        ajos.push_back(deployed_address.clone());
        env.storage()
            .instance()
            .set(&FactoryDataKey::DeployedAjos, &ajos);

        // Emit an event
        env.events().publish(
            (symbol_short!("Created"), deployed_address.clone()),
            creator,
        );

        deployed_address
    }

    /// Get all deployed Ajos.
    pub fn get_deployed_ajos(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&FactoryDataKey::DeployedAjos)
            .unwrap_or(Vec::new(&env))
    }
}

mod test;

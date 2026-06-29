//! # Ajo Factory Example
//!
//! Factory contract that deploys new [`ajo::Ajo`] instances via `env.deployer()`.

#![no_std]
use ajo::AjoClient;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
enum DeploySalt {
    Ajo(Address, u32),
}

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
    pub fn initialize(env: Env, wasm_hash: BytesN<32>) -> Result<(), FactoryError> {
        if env.storage().instance().has(&FactoryDataKey::WasmHash) {
            return Err(FactoryError::AlreadyInitialized);
        }
        env.storage()
            .instance()
            .set(&FactoryDataKey::WasmHash, &wasm_hash);

        let ajos: Vec<Address> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&FactoryDataKey::DeployedAjos, &ajos);

        Ok(())
    }

    /// Create a new Ajo instance.
    #[allow(deprecated)]
    pub fn create_ajo(
        env: Env,
        amount: i128,
        max_members: u32,
        creator: Address,
    ) -> Result<Address, FactoryError> {
        creator.require_auth();

        let wasm_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::WasmHash)
            .ok_or(FactoryError::NotInitialized)?;

        let mut ajos: Vec<Address> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::DeployedAjos)
            .unwrap_or(Vec::new(&env));

        let nonce = ajos.len();
        let salt_preimage = DeploySalt::Ajo(creator.clone(), nonce);
        let salt = env.crypto().sha256(&salt_preimage.to_xdr(&env));

        let deployed_address = env.deployer().with_current_contract(salt).deploy(wasm_hash);

        let ajo_client = AjoClient::new(&env, &deployed_address);
        ajo_client.initialize(&amount, &max_members, &creator);

        ajos.push_back(deployed_address.clone());
        env.storage()
            .instance()
            .set(&FactoryDataKey::DeployedAjos, &ajos);

        env.events().publish(
            (symbol_short!("Created"), deployed_address.clone()),
            creator,
        );

        Ok(deployed_address)
    }

    /// Get all deployed Ajos.
    pub fn get_deployed_ajos(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&FactoryDataKey::DeployedAjos)
            .unwrap_or(Vec::new(&env))
    }
}

// Re-export the template contract for WASM upload in tests and integration tests.
pub use ajo::{Ajo, AjoError};

#[cfg(test)]
mod test;

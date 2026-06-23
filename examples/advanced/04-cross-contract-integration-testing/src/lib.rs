#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, Symbol};

// -------------------------------------------------
// Registry Contract
// -------------------------------------------------

#[contract]
pub struct Registry;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryKey {
    Entry(Symbol),
}

#[contractimpl]
impl Registry {
    pub fn register(env: Env, name: Symbol, addr: Address) {
        // allow anyone to register in this example
        env.storage()
            .persistent()
            .set(&RegistryKey::Entry(name.clone()), &addr);
        // track list omitted for brevity
    }

    pub fn lookup(env: Env, name: Symbol) -> Option<Address> {
        env.storage().persistent().get(&RegistryKey::Entry(name))
    }
}

// -------------------------------------------------
// Factory Contract
// -------------------------------------------------

#[contract]
pub struct Factory;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactoryKey {
    WasmHash,
    RegistryAddr,
}

#[contractimpl]
impl Factory {
    pub fn initialize(env: Env, wasm_hash: BytesN<32>, registry: Address) {
        if env.storage().instance().has(&FactoryKey::WasmHash) {
            return;
        }
        env.storage()
            .instance()
            .set(&FactoryKey::WasmHash, &wasm_hash);
        env.storage()
            .instance()
            .set(&FactoryKey::RegistryAddr, &registry);
    }

    pub fn create_instance(env: Env, salt: i128, name: Symbol, creator: Address) -> Address {
        creator.require_auth();

        let wasm_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&FactoryKey::WasmHash)
            .expect("factory not initialized");

        // derive a salt from provided number for deterministic address in tests
        let mut salt_input: Bytes = Bytes::new(&env);
        salt_input.extend_from_slice(&salt.to_be_bytes());
        let salt_hash: BytesN<32> = env.crypto().sha256(&salt_input).into();
        let deployed_address = env
            .deployer()
            .with_current_contract(salt_hash)
            .deploy(wasm_hash);

        // register deployed address in registry
        let registry: Address = env
            .storage()
            .instance()
            .get(&FactoryKey::RegistryAddr)
            .expect("registry missing");
        let registry_client = RegistryClient::new(&env, &registry);
        registry_client.register(&name, &deployed_address);

        deployed_address
    }
}

// -------------------------------------------------
// Target / Proxy-backed Contract (simple example)
// - stores a value in instance storage
// - has an `upgrade_marker` in persistent storage (for demonstration)
// -------------------------------------------------

#[contract]
pub struct Target;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetKey {
    Value,
    UpgradeMarker,
}

#[contractimpl]
impl Target {
    pub fn set_value(env: Env, v: i128) {
        env.storage().instance().set(&TargetKey::Value, &v);
    }

    pub fn get_value(env: Env) -> Option<i128> {
        env.storage().instance().get(&TargetKey::Value)
    }

    // Simple upgrade marker to simulate writing a new implementation hash
    pub fn set_upgrade_marker(env: Env, marker: i128) {
        env.storage()
            .persistent()
            .set(&TargetKey::UpgradeMarker, &marker);
    }

    pub fn get_upgrade_marker(env: Env) -> Option<i128> {
        env.storage().persistent().get(&TargetKey::UpgradeMarker)
    }
}

// Export generated clients
// generated clients are provided by the `#[contract]` macro during build

#[cfg(test)]
mod test;

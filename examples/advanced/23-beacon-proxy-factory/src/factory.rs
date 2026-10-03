#![allow(deprecated)]
//! # Beacon Proxy Factory Contract
//!
//! The factory is the top-level orchestrator. It:
//!
//! 1. Receives WASM hashes for the beacon and proxy contracts on initialisation.
//! 2. Deploys a single shared `BeaconContract` and registers the first implementation.
//! 3. Deploys individual `ProxyContract` instances on demand, each bound to the
//!    shared beacon.
//! 4. Tracks every deployed proxy in `instance` storage so callers can enumerate them.
//! 5. Provides a `batch_deploy` function to deploy many proxies in one transaction.
//! 6. Provides `upgrade_beacon` to atomically upgrade all proxies with a single call.
//!
//! ## Gas optimisation notes
//!
//! - Proxy list and factory metadata are stored in `instance` storage. Instance
//!   reads are cheaper than individual persistent reads because all instance entries
//!   share a single ledger entry that is loaded once per transaction.
//! - `batch_deploy` amortises the per-transaction overhead across N deployments.
//! - The beacon address is cached in the factory so there is no need to query an
//!   external registry on every `deploy_proxy` call.
//!
//! ## Storage layout (`instance`)
//!
//! | Key | Type | Description |
//! |---|---|---|
//! | `DataKey::Admin` | `Address` | Factory administrator |
//! | `DataKey::Beacon` | `Address` | Address of the deployed shared beacon |
//! | `DataKey::ProxyWasmHash` | `BytesN<32>` | WASM hash used to deploy new proxies |
//! | `DataKey::Proxies` | `Vec<Address>` | Ordered list of all deployed proxy addresses |

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env, IntoVal,
    Symbol, Vec,
};
use soroban_sdk::xdr::ToXdr;

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

/// Factory-level storage keys (all in `instance` storage for cheap bulk reads).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Factory administrator address.
    Admin,
    /// Address of the shared Beacon contract.
    Beacon,
    /// WASM hash used to deploy new proxy instances.
    ProxyWasmHash,
    /// Ordered list of deployed proxy addresses.
    Proxies,
    /// Number of deployed proxies, cached for cheap count and salt lookups.
    ProxyCount,
    /// Salt seed for deterministic proxy deployment.
    ProxySaltSeed(u32),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// The Beacon Proxy Factory.
///
/// Deploys and tracks multiple proxy contracts that all share one beacon,
/// enabling atomic O(1) upgrades across the entire fleet.
#[contract]
pub struct BeaconProxyFactory;

#[contractimpl]
impl BeaconProxyFactory {
    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// Initialise the factory.
    ///
    /// Deploys the shared Beacon contract using `beacon_wasm_hash`, registers
    /// `initial_implementation` as version 1, and stores the proxy WASM hash
    /// for future deployments.
    ///
    /// Can only be called once.
    ///
    /// # Arguments
    /// * `admin` - address authorised to deploy proxies and upgrade the beacon
    /// * `beacon_wasm_hash` - WASM hash of the `BeaconContract` to deploy
    /// * `proxy_wasm_hash` - WASM hash of the `ProxyContract` to deploy for each proxy
    /// * `initial_implementation` - address of the first implementation contract
    /// * `impl_label` - human-readable label for version 1 (max 9 chars)
    pub fn init(
        env: Env,
        admin: Address,
        beacon_wasm_hash: BytesN<32>,
        proxy_wasm_hash: BytesN<32>,
        initial_implementation: Address,
        impl_label: Symbol,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        admin.require_auth();

        // Deploy the shared beacon using a deterministic salt derived from the
        // factory's own address. This makes the beacon address reproducible.
        let beacon_salt_input = DataKey::ProxySaltSeed(0u32);
        let beacon_salt = env.crypto().sha256(&beacon_salt_input.to_xdr(&env));

        #[allow(deprecated)]
        let beacon_addr = env
            .deployer()
            .with_current_contract(beacon_salt)
            .deploy(beacon_wasm_hash);

        // Initialise the beacon: admin is the factory itself so only the factory
        // can trigger upgrades, maintaining a single point of control.
        let factory_addr = env.current_contract_address();
        env.invoke_contract::<()>(
            &beacon_addr,
            &Symbol::new(&env, "init"),
            vec![
                &env,
                factory_addr.into_val(&env),
                initial_implementation.into_val(&env),
                impl_label.into_val(&env),
            ],
        );

        // Persist factory state.
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Beacon, &beacon_addr);
        env.storage()
            .instance()
            .set(&DataKey::ProxyWasmHash, &proxy_wasm_hash);
        env.storage()
            .instance()
            .set(&DataKey::Proxies, &Vec::<Address>::new(&env));
        env.storage().instance().set(&DataKey::ProxyCount, &0u32);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("factory"), symbol_short!("init")),
            beacon_addr,
        );
    }

    // -----------------------------------------------------------------------
    // Proxy deployment
    // -----------------------------------------------------------------------

    /// Deploy a single new proxy contract bound to the shared beacon.
    ///
    /// The proxy is initialised with the factory admin as its admin so the
    /// factory retains the ability to re-point proxies if needed.
    ///
    /// # Arguments
    /// * `deployer` - address paying for deployment (must authorise)
    ///
    /// # Returns
    /// Address of the newly deployed proxy contract.
    pub fn deploy_proxy(env: Env, deployer: Address) -> Address {
        deployer.require_auth();
        Self::do_deploy_proxy(&env)
    }

    /// Deploy `count` proxy contracts in a single transaction.
    ///
    /// Compared with calling `deploy_proxy` N times this saves one transaction
    /// round-trip per additional proxy, reducing total gas by the per-transaction
    /// base cost × (N - 1).
    ///
    /// # Arguments
    /// * `deployer` - address paying for all deployments (must authorise)
    /// * `count` - number of proxies to deploy (must be ≥ 1 and ≤ 10)
    ///
    /// # Returns
    /// `Vec<Address>` of the newly deployed proxy contracts in deployment order.
    pub fn batch_deploy(env: Env, deployer: Address, count: u32) -> Vec<Address> {
        if count == 0 {
            panic!("Count must be at least 1");
        }
        if count > 10 {
            panic!("Batch size too large: max 10");
        }

        deployer.require_auth();

        let mut deployed = Vec::new(&env);
        for _ in 0..count {
            let addr = Self::do_deploy_proxy(&env);
            deployed.push_back(addr);
        }

        deployed
    }

    // -----------------------------------------------------------------------
    // Upgrade (batch upgrade via beacon)
    // -----------------------------------------------------------------------

    /// Upgrade the shared beacon to a new implementation.
    ///
    /// Because all proxies resolve their implementation through this single
    /// beacon, this call atomically upgrades **every** proxy in O(1) — regardless
    /// of how many proxies have been deployed.
    ///
    /// Only the factory admin can call this.
    ///
    /// # Arguments
    /// * `new_implementation` - address of the new implementation contract
    /// * `label` - human-readable tag for this version (max 9 chars)
    pub fn upgrade_beacon(env: Env, new_implementation: Address, label: Symbol) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Not initialized"));

        admin.require_auth();

        let beacon: Address = env
            .storage()
            .instance()
            .get(&DataKey::Beacon)
            .unwrap_or_else(|| panic!("Not initialized"));

        // The beacon's admin is the factory contract itself, so we call upgrade
        // as the factory (current_contract_address). The auth chain is:
        //   admin → factory.upgrade_beacon → beacon.upgrade (factory is beacon admin)
        env.invoke_contract::<()>(
            &beacon,
            &Symbol::new(&env, "upgrade"),
            vec![
                &env,
                new_implementation.into_val(&env),
                label.into_val(&env),
            ],
        );

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("factory"), symbol_short!("upgraded")),
            new_implementation,
        );
    }

    // -----------------------------------------------------------------------
    // Admin management
    // -----------------------------------------------------------------------

    /// Transfer factory admin rights to a new address.
    ///
    /// Note: this does NOT transfer the beacon's admin (which remains the factory
    /// contract address). Only the factory admin key is updated.
    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Not initialized"));

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("factory"), symbol_short!("adm_xfr")),
            new_admin,
        );
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the address of the shared Beacon contract.
    pub fn get_beacon(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Beacon)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    /// Return the list of all deployed proxy addresses.
    pub fn get_proxies(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Proxies)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Return the total number of deployed proxies.
    pub fn get_proxy_count(env: Env) -> u32 {
        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProxyCount)
            .unwrap_or_else(|| panic!("Not initialized"));
        count
    }

    /// Return the factory admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Core proxy deployment logic.
    ///
    /// Uses the current proxy count as the nonce component of the salt so that
    /// every proxy gets a unique, deterministic address.
    fn do_deploy_proxy(env: &Env) -> Address {
        let proxy_wasm_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::ProxyWasmHash)
            .unwrap_or_else(|| panic!("Not initialized"));

        let beacon: Address = env
            .storage()
            .instance()
            .get(&DataKey::Beacon)
            .unwrap_or_else(|| panic!("Not initialized"));

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Not initialized"));

        // Load and increment the proxy counter atomically.
        let mut proxies: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Proxies)
            .unwrap_or_else(|| Vec::new(env));

        let nonce: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProxyCount)
            .unwrap_or_else(|| panic!("Not initialized"));

        // Build a deterministic salt from the current proxy count.
        // Using nonce + 1 to distinguish proxy salts from the beacon salt (nonce 0).
        let salt_seed = DataKey::ProxySaltSeed(
            nonce.checked_add(1).unwrap_or_else(|| panic!("Nonce overflow")),
        );
        let salt = env.crypto().sha256(&salt_seed.to_xdr(env));

        #[allow(deprecated)]
        let proxy_addr = env
            .deployer()
            .with_current_contract(salt)
            .deploy(proxy_wasm_hash);

        // Initialise the proxy: admin = factory admin, beacon = shared beacon.
        env.invoke_contract::<()>(
            &proxy_addr,
            &Symbol::new(env, "init"),
            vec![
                env,
                admin.into_val(env),
                beacon.into_val(env),
            ],
        );

        proxies.push_back(proxy_addr.clone());
        env.storage().instance().set(&DataKey::Proxies, &proxies);
        env.storage()
            .instance()
            .set(&DataKey::ProxyCount, &(nonce + 1));

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("factory"), symbol_short!("deployed")),
            proxy_addr.clone(),
        );

        proxy_addr
    }
}

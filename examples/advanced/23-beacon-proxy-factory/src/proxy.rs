#![allow(deprecated)]
//! # Proxy Contract
//!
//! The Proxy is a thin delegation layer. It stores the address of a Beacon
//! and, on every call, queries the Beacon for the live implementation before
//! forwarding the invocation via `env.invoke_contract`.
//!
//! In the factory pattern, proxies are deployed by the factory and bound to the
//! factory's shared beacon at deployment time. The factory admin is the proxy
//! admin by default, keeping control centralised.
//!
//! ## Storage keys (all `persistent`)
//!
//! | Key | Type | Description |
//! |---|---|---|
//! | `BEACON_KEY` | `Address` | The Beacon this proxy delegates through |
//! | `ADMIN_KEY` | `Address` | Can re-point this proxy to a different Beacon |

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, IntoVal, Symbol};

const BEACON_KEY: Symbol = symbol_short!("beacon");
const ADMIN_KEY: Symbol = symbol_short!("admin");

/// The Proxy contract.
///
/// Delegates all business-logic calls to the implementation address returned
/// by the Beacon it is registered with.
#[contract]
pub struct ProxyContract;

#[contractimpl]
impl ProxyContract {
    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// Initialise the proxy by binding it to a Beacon.
    ///
    /// Can only be called once.
    ///
    /// # Arguments
    /// * `admin` - address that can call `set_beacon`
    /// * `beacon` - address of the Beacon contract this proxy will consult
    pub fn init(env: Env, admin: Address, beacon: Address) {
        if env.storage().persistent().has(&BEACON_KEY) {
            panic!("Already initialized");
        }
        env.storage().persistent().set(&ADMIN_KEY, &admin);
        env.storage().persistent().set(&BEACON_KEY, &beacon);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("proxy"), symbol_short!("init")),
            beacon,
        );
    }

    // -----------------------------------------------------------------------
    // Beacon management
    // -----------------------------------------------------------------------

    /// Re-point this proxy to a different Beacon.
    ///
    /// Only the proxy admin can call this. Useful for canary / A-B deployments.
    pub fn set_beacon(env: Env, new_beacon: Address) {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN_KEY)
            .unwrap_or_else(|| panic!("Not initialized"));

        admin.require_auth();

        env.storage().persistent().set(&BEACON_KEY, &new_beacon);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("proxy"), symbol_short!("set_bcn")),
            new_beacon,
        );
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the address of the Beacon this proxy is bound to.
    pub fn get_beacon(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&BEACON_KEY)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    /// Resolve and return the current implementation address from the Beacon.
    pub fn get_implementation(env: Env) -> Address {
        let beacon = Self::resolve_beacon(&env);
        env.invoke_contract(&beacon, &Symbol::new(&env, "get_implementation"), vec![&env])
    }

    // -----------------------------------------------------------------------
    // Delegated calls
    // -----------------------------------------------------------------------

    /// Delegate: add two integers.
    pub fn add(env: Env, a: i128, b: i128) -> i128 {
        let impl_addr = Self::resolve_impl(&env);
        env.invoke_contract(
            &impl_addr,
            &symbol_short!("add"),
            vec![&env, a.into_val(&env), b.into_val(&env)],
        )
    }

    /// Delegate: subtract two integers.
    pub fn sub(env: Env, a: i128, b: i128) -> i128 {
        let impl_addr = Self::resolve_impl(&env);
        env.invoke_contract(
            &impl_addr,
            &symbol_short!("sub"),
            vec![&env, a.into_val(&env), b.into_val(&env)],
        )
    }

    /// Delegate: multiply two integers (v2+).
    pub fn mul(env: Env, a: i128, b: i128) -> i128 {
        let impl_addr = Self::resolve_impl(&env);
        env.invoke_contract(
            &impl_addr,
            &symbol_short!("mul"),
            vec![&env, a.into_val(&env), b.into_val(&env)],
        )
    }

    /// Delegate: increment the counter stored inside the implementation.
    pub fn increment(env: Env) -> u32 {
        let impl_addr = Self::resolve_impl(&env);
        env.invoke_contract(&impl_addr, &symbol_short!("increment"), vec![&env])
    }

    /// Delegate: get the counter stored inside the implementation.
    pub fn get_counter(env: Env) -> u32 {
        let impl_addr = Self::resolve_impl(&env);
        env.invoke_contract(&impl_addr, &symbol_short!("get_count"), vec![&env])
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn resolve_beacon(env: &Env) -> Address {
        env.storage()
            .persistent()
            .get(&BEACON_KEY)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    fn resolve_impl(env: &Env) -> Address {
        let beacon = Self::resolve_beacon(env);
        env.invoke_contract(&beacon, &Symbol::new(env, "get_implementation"), vec![env])
    }
}

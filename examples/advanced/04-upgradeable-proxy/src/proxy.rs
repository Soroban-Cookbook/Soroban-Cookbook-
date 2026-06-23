//! # Proxy Contract
//!
//! This is the proxy contract that forwards calls to an implementation contract.
//! The proxy maintains the upgrade logic and delegates actual operations to
//! the implementation contract.

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, InvokeContract};

const ADMIN_KEY: Symbol = symbol_short!("admin");
const IMPLEMENTATION_KEY: Symbol = symbol_short!("impl");

#[contract]
pub struct ProxyContract;

#[contractimpl]
impl ProxyContract {
    /// Initialize the proxy with an admin and initial implementation.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `admin` - the address that can authorize upgrades
    /// * `implementation` - the initial implementation contract address
    pub fn init(env: Env, admin: Address, implementation: Address) {
        if env.storage().persistent().has(&ADMIN_KEY) {
            panic!("Already initialized");
        }

        env.storage().persistent().set(&ADMIN_KEY, &admin);
        env.storage().persistent().set(&IMPLEMENTATION_KEY, &implementation);
    }

    /// Upgrade to a new implementation contract.
    /// Only the admin can call this.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `admin` - (implicit) the caller must be the admin
    /// * `new_implementation` - the address of the new implementation contract
    pub fn upgrade(env: Env, new_implementation: Address) {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN_KEY)
            .unwrap_or_else(|| panic!("Admin not set"));

        admin.require_auth();

        env.storage()
            .persistent()
            .set(&IMPLEMENTATION_KEY, &new_implementation);

        env.events()
            .publish((symbol_short!("upgraded"),), new_implementation);
    }

    /// Get the current implementation address.
    ///
    /// # Returns
    /// The address of the current implementation contract
    pub fn get_implementation(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&IMPLEMENTATION_KEY)
            .unwrap_or_else(|| panic!("Implementation not set"))
    }

    /// Forward a call to the implementation contract's add function.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `a` - first number to add
    /// * `b` - second number to add
    ///
    /// # Returns
    /// The sum of a and b
    pub fn add(env: Env, a: i128, b: i128) -> i128 {
        let implementation: Address = Self::get_implementation(&env);
        
        env.invoke_contract(
            &implementation,
            &symbol_short!("add"),
            soroban_sdk::vec![&env, a, b],
        )
    }

    /// Forward a call to the implementation contract's subtract function.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `a` - the minuend
    /// * `b` - the subtrahend
    ///
    /// # Returns
    /// The difference (a - b)
    pub fn subtract(env: Env, a: i128, b: i128) -> i128 {
        let implementation: Address = Self::get_implementation(&env);
        
        env.invoke_contract(
            &implementation,
            &symbol_short!("sub"),
            soroban_sdk::vec![&env, a, b],
        )
    }
}

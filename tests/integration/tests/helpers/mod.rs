//! # Test Utilities and Helpers
//!
//! Reusable test infrastructure for Soroban Cookbook integration tests.
//! Provides environment setup, contract registration, and assertion helpers.

use soroban_sdk::{
    testutils::Address as _, testutils::Ledger as _, Address, Env, IntoVal, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Environment Setup
// ---------------------------------------------------------------------------

/// Create a fresh test environment with all auth mocked.
pub fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

/// Create a test environment with a specific ledger timestamp.
pub fn setup_env_with_timestamp(timestamp: u64) -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(timestamp);
    env
}

// ---------------------------------------------------------------------------
// Address Generation
// ---------------------------------------------------------------------------

/// Generate N unique test addresses.
pub fn generate_addresses(env: &Env, count: usize) -> std::vec::Vec<Address> {
    (0..count).map(|_| Address::generate(env)).collect()
}

// ---------------------------------------------------------------------------
// Invocation Helpers
// ---------------------------------------------------------------------------

/// Invoke a contract function with no arguments and return the result.
#[allow(dead_code)]
pub fn invoke_no_args<T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(
    env: &Env,
    contract_id: &Address,
    fn_name: &str,
) -> T {
    env.invoke_contract(contract_id, &Symbol::new(env, fn_name), Vec::new(env))
}

/// Invoke a contract function with a single argument.
#[allow(dead_code)]
pub fn invoke_one_arg<T, A>(env: &Env, contract_id: &Address, fn_name: &str, arg: A) -> T
where
    T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>,
    A: IntoVal<Env, soroban_sdk::Val>,
{
    env.invoke_contract(
        contract_id,
        &Symbol::new(env, fn_name),
        Vec::from_array(env, [arg.into_val(env)]),
    )
}

// ---------------------------------------------------------------------------
// Assertion Helpers
// ---------------------------------------------------------------------------

/// Assert that a contract's storage contains a particular balance for a user.
pub fn assert_balance(env: &Env, auth_contract: &Address, user: &Address, expected: i128) {
    let actual: i128 = env.invoke_contract(
        auth_contract,
        &Symbol::new(env, "get_balance"),
        Vec::from_array(env, [user.clone().into_val(env)]),
    );
    assert_eq!(
        actual, expected,
        "Balance mismatch for user: expected {expected}, got {actual}"
    );
}

/// Assert that a counter contract has a specific count.
#[allow(dead_code)]
pub fn assert_event_count(env: &Env, events_contract: &Address, expected: u32) {
    let actual: u32 = env.invoke_contract(
        events_contract,
        &Symbol::new(env, "get_number"),
        Vec::new(env),
    );
    assert_eq!(
        actual, expected,
        "Event count mismatch: expected {expected}, got {actual}"
    );
}

// ---------------------------------------------------------------------------
// Setup/Teardown Patterns
// ---------------------------------------------------------------------------

/// Standard test fixture: auth contract initialized with admin + funded users.
#[allow(dead_code)]
pub struct AuthFixture {
    pub contract_id: Address,
    pub admin: Address,
    pub users: std::vec::Vec<Address>,
}

impl AuthFixture {
    /// Create an auth fixture with N funded users, each starting with `balance`.
    pub fn setup(env: &Env, user_count: usize, balance: i128) -> Self {
        #![allow(deprecated)]
        let contract_id = env.register_contract(None, authentication::AuthContract);
        let admin = Address::generate(env);
        let users = generate_addresses(env, user_count);

        // Initialize
        env.invoke_contract::<()>(
            &contract_id,
            &Symbol::new(env, "initialize"),
            Vec::from_array(env, [admin.clone().into_val(env)]),
        );

        // Fund each user
        for user in &users {
            env.invoke_contract::<()>(
                &contract_id,
                &Symbol::new(env, "set_balance"),
                Vec::from_array(
                    env,
                    [
                        admin.clone().into_val(env),
                        user.clone().into_val(env),
                        balance.into_val(env),
                    ],
                ),
            );
        }

        Self {
            contract_id,
            admin,
            users,
        }
    }
}

/// Standard test fixture: events counter contract.
pub struct EventsFixture {
    pub contract_id: Address,
}

impl EventsFixture {
    pub fn setup(env: &Env) -> Self {
        #![allow(deprecated)]
        let contract_id = env.register_contract(None, events_structured::EventsContract);
        Self { contract_id }
    }

    pub fn increment(&self, env: &Env) {
        env.invoke_contract::<()>(
            &self.contract_id,
            &soroban_sdk::symbol_short!("increment"),
            Vec::new(env),
        );
    }

    pub fn get_count(&self, env: &Env) -> u32 {
        env.invoke_contract(
            &self.contract_id,
            &Symbol::new(env, "get_number"),
            Vec::new(env),
        )
    }
}

/// Standard test fixture: storage contract.
pub struct StorageFixture {
    pub contract_id: Address,
}

impl StorageFixture {
    pub fn setup(env: &Env) -> Self {
        #![allow(deprecated)]
        let contract_id = env.register_contract(None, storage_patterns::StorageContract);
        Self { contract_id }
    }

    pub fn set_persistent(&self, env: &Env, key: Symbol, value: u64) {
        env.invoke_contract::<()>(
            &self.contract_id,
            &Symbol::new(env, "set_persistent"),
            Vec::from_array(env, [key.into_val(env), value.into_val(env)]),
        );
    }

    pub fn get_persistent(&self, env: &Env, key: Symbol) -> Option<u64> {
        env.invoke_contract(
            &self.contract_id,
            &Symbol::new(env, "get_persistent"),
            Vec::from_array(env, [key.into_val(env)]),
        )
    }
}

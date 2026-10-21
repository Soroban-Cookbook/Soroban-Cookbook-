//! Common test helper functions for contract deployment and invocation.

use soroban_sdk::{Address, Env, IntoVal, Val};

/// Test environment container holding environment and contract state.
#[derive(Clone)]
pub struct TestEnv {
    pub env: Env,
    pub contract_id: Address,
}

impl TestEnv {
    /// Create a new TestEnv with the given environment and contract ID.
    pub fn new(env: Env, contract_id: Address) -> Self {
        TestEnv { env, contract_id }
    }
}

/// Initialize a test environment with default Soroban environment.
///
/// # Example
/// ```ignore
/// let test_env = init_test_env();
/// ```
pub fn init_test_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

/// Deploy a contract to the given environment.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `contract` - The contract implementation
///
/// # Returns
/// A tuple of (env, contract_address)
///
/// # Example
/// ```ignore
/// let (env, contract_id) = deploy_contract(&env, HelloContract);
/// ```
pub fn deploy_contract<T: soroban_sdk::contractimpl::ContractImpl>(
    env: &Env,
    contract: T,
) -> Address {
    env.register_contract(None, contract)
}

/// Invoke a contract function with arguments.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `contract` - The contract address
/// * `function` - The function name as a symbol
/// * `args` - Arguments vector
///
/// # Returns
/// Result of type T or an error
///
/// # Example
/// ```ignore
/// let result: Vec<Symbol> = invoke_contract(
///     &env,
///     &contract_id,
///     &symbol_short!("hello"),
///     &Vec::from_array(&env, [symbol_short!("Alice").into_val(&env)])
/// )?;
/// ```
pub fn invoke_contract<T: soroban_sdk::TryFromVal<Env, Val>>(
    env: &Env,
    contract: &Address,
    function: &soroban_sdk::Symbol,
    args: &soroban_sdk::Vec<Val>,
) -> T {
    env.invoke_contract(contract, function, args.clone())
}

/// Jump ledger sequence forward by the specified number of ledgers.
///
/// Useful for testing time-dependent functionality.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `count` - Number of ledgers to jump
///
/// # Example
/// ```ignore
/// jump_ledger_sequence(&env, 100);
/// ```
pub fn jump_ledger_sequence(env: &Env, count: u32) {
    let current = env.ledger().sequence();
    env.ledger().set_sequence(current + count);
}

/// Setup ledger with initial data.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `key` - Storage key
/// * `value` - Value to store
///
/// # Example
/// ```ignore
/// setup_ledger_with_data(&env, "test_key", &100u64.into_val(&env));
/// ```
pub fn setup_ledger_with_data(env: &Env, _key: &str, _value: &Val) {
    // This is a placeholder for ledger setup utilities
    // Implementation depends on specific Soroban SDK version and use case
}

#![cfg(test)]

use crate::helpers::{perf::measure_execution, setup_env};
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env, IntoVal, Symbol};

mod helpers;

#[contract]
struct RegistryLookup;

#[contractimpl]
impl RegistryLookup {
    pub fn lookup(env: Env, registry: Address, name: Symbol) -> Option<Address> {
        cross_contract_integration_testing::RegistryClient::new(&env, &registry).lookup(&name)
    }
}

#[test]
fn test_basic_contract_performance() {
    let env = setup_env();

    // Register the Hello World contract for performance testing
    let contract_id = env.register_contract(None, hello_world::HelloContract);

    let to_val = Symbol::new(&env, "Dev");

    let (result, metrics) = measure_execution(&env, || {
        env.invoke_contract::<soroban_sdk::Vec<Symbol>>(
            &contract_id,
            &Symbol::new(&env, "hello"),
            soroban_sdk::Vec::from_array(&env, [to_val.into_val(&env)]),
        )
    });

    metrics.print("Hello World - hello()");

    assert_eq!(result.len(), 2);

    assert!(metrics.execution_time_ns > 0);
    assert!(metrics.cpu_instructions > 0);
}

#[test]
fn test_cross_contract_performance() {
    let env = setup_env();

    let registry_id = env.register(cross_contract_integration_testing::Registry, ());
    let target_id = env.register(cross_contract_integration_testing::Target, ());
    let name = soroban_sdk::symbol_short!("target");
    cross_contract_integration_testing::RegistryClient::new(&env, &registry_id)
        .register(&name, &target_id);
    let caller_id = env.register(RegistryLookup, ());

    let (result, metrics) = measure_execution(&env, || {
        env.invoke_contract::<Option<Address>>(
            &caller_id,
            &Symbol::new(&env, "lookup"),
            soroban_sdk::Vec::from_array(&env, [registry_id.into_val(&env), name.into_val(&env)]),
        )
    });

    metrics.print("Cross Contract - registry lookup");

    assert_eq!(result, Some(target_id));
    assert!(metrics.cpu_instructions > 0);
}

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol};
use crate::helpers::{perf::measure_execution, setup_env};

mod helpers;

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
    
    let contract_a = env.register_contract(None, cross_contract_integration_testing::contract_a::ContractA);
    let contract_b = env.register_contract(None, cross_contract_integration_testing::contract_b::ContractB);
    
    let (result, metrics) = measure_execution(&env, || {
        env.invoke_contract::<u32>(
            &contract_a,
            &Symbol::new(&env, "add_with"),
            soroban_sdk::Vec::from_array(&env, [
                contract_b.into_val(&env),
                5u32.into_val(&env),
                7u32.into_val(&env)
            ]),
        )
    });

    metrics.print("Cross Contract - add_with()");
    
    assert_eq!(result, 12);
    assert!(metrics.cpu_instructions > 0);
}


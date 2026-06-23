use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Env};
use std::println;

fn setup_contracts(env: &Env) -> (Address, Address, Address) {
    let target = env.register_contract(None, TargetContract);
    let unoptimized = env.register_contract(None, UnoptimizedCaller);
    let optimized = env.register_contract(None, OptimizedCaller);
    (target, unoptimized, optimized)
}

fn build_updates(env: &Env) -> Vec<PackedUpdate> {
    vec![
        env,
        PackedUpdate {
            key: symbol_short!("alpha"),
            delta: 10,
        },
        PackedUpdate {
            key: symbol_short!("beta"),
            delta: 20,
        },
        PackedUpdate {
            key: symbol_short!("alpha"),
            delta: 5,
        },
        PackedUpdate {
            key: symbol_short!("gamma"),
            delta: 15,
        },
    ]
}

#[test]
fn test_sequential_and_batched_cross_contract_updates_match() {
    let env = Env::default();
    let (target, unoptimized, optimized) = setup_contracts(&env);
    let target_client = TargetContractClient::new(&env, &target);
    let unoptimized_client = UnoptimizedCallerClient::new(&env, &unoptimized);
    let optimized_client = OptimizedCallerClient::new(&env, &optimized);

    let updates = build_updates(&env);

    env.budget().reset_default();
    println!("--- Sequential cross-contract invocation ---");
    let sequential_last = unoptimized_client.invoke_updates_sequential(&target, &updates);
    env.budget().print();

    assert_eq!(sequential_last, 15);
    assert_eq!(target_client.get_entry(&symbol_short!("alpha")), 15);
    assert_eq!(target_client.get_entry(&symbol_short!("beta")), 20);
    assert_eq!(target_client.get_entry(&symbol_short!("gamma")), 15);

    // Register a fresh target contract for the optimized path to compare final state independently.
    let optimized_target = env.register_contract(None, TargetContract);
    let optimized_target_client = TargetContractClient::new(&env, &optimized_target);

    env.budget().reset_default();
    println!("--- Batched cross-contract invocation ---");
    let batched_last = optimized_client.invoke_updates_batched(&optimized_target, &updates);
    env.budget().print();

    assert_eq!(batched_last, sequential_last);
    assert_eq!(
        optimized_target_client.get_entry(&symbol_short!("alpha")),
        15
    );
    assert_eq!(
        optimized_target_client.get_entry(&symbol_short!("beta")),
        20
    );
    assert_eq!(
        optimized_target_client.get_entry(&symbol_short!("gamma")),
        15
    );
}

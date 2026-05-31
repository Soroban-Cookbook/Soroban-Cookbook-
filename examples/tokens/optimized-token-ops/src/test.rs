use super::*;
use soroban_sdk::{testutils::Address as _, vec, vec::Vec, Env};

fn setup(env: &Env) -> OptimizedTokenClient<'_> {
    let contract_id = env.register_contract(None, OptimizedToken);
    OptimizedTokenClient::new(env, &contract_id)
}

fn sample_payments(env: &Env) -> Vec<Payment> {
    vec![
        &env,
        Payment {
            recipient: Address::generate(env),
            amount: 100,
        },
        Payment {
            recipient: Address::generate(env),
            amount: 150,
        },
        Payment {
            recipient: Address::generate(env),
            amount: 250,
        },
    ]
}

#[test]
fn test_initialize_sets_owner_balance() {
    let env = Env::default();
    let client = setup(&env);
    let owner = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&owner, &1000);

    assert_eq!(client.balance(&owner), 1000);
    assert_eq!(client.total_supply(), 1000);
}

#[test]
fn test_batch_transfer_naive_completes() {
    let env = Env::default();
    let client = setup(&env);
    let owner = Address::generate(&env);
    let payments = sample_payments(&env);

    env.mock_all_auths();
    client.initialize(&owner, &1000);
    client.batch_transfer_naive(&owner, &payments).unwrap();

    assert_eq!(client.balance(&owner), 450);
    assert_eq!(client.balance(&payments[0].recipient), 100);
    assert_eq!(client.balance(&payments[1].recipient), 150);
    assert_eq!(client.balance(&payments[2].recipient), 250);
}

#[test]
fn test_batch_transfer_optimized_completes() {
    let env = Env::default();
    let client = setup(&env);
    let owner = Address::generate(&env);
    let payments = sample_payments(&env);

    env.mock_all_auths();
    client.initialize(&owner, &1000);
    client.batch_transfer_optimized(&owner, &payments).unwrap();

    assert_eq!(client.balance(&owner), 450);
    assert_eq!(client.balance(&payments[0].recipient), 100);
    assert_eq!(client.balance(&payments[1].recipient), 150);
    assert_eq!(client.balance(&payments[2].recipient), 250);
}

#[test]
fn test_batch_transfer_benchmarks() {
    let env = Env::default();
    let client = setup(&env);
    let owner = Address::generate(&env);
    let payments = sample_payments(&env);

    env.mock_all_auths();
    client.initialize(&owner, &1000);

    println!("--- Naive Batch Transfer Benchmark ---");
    env.budget().reset_default();
    client.batch_transfer_naive(&owner, &payments).unwrap();
    env.budget().print();

    let env = Env::default();
    let client = setup(&env);
    let owner = Address::generate(&env);
    let payments = sample_payments(&env);

    env.mock_all_auths();
    client.initialize(&owner, &1000);

    println!("--- Optimized Batch Transfer Benchmark ---");
    env.budget().reset_default();
    client.batch_transfer_optimized(&owner, &payments).unwrap();
    env.budget().print();
}

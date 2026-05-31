extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{Address, Env, Symbol};

#[test]
fn test_whitelist_and_registration() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Initialize as whitelist-only with no fee
    client.init(&owner, &true, &0i128);

    // Owner adds Alice to whitelist
    client.add_whitelist(&alice);

    // Alice can register (payment 0)
    client.register(&alice, &0i128);
    assert!(client.is_registered(&alice));
}

// Bob is not whitelisted -> registration should panic
#[test]
#[should_panic(expected = "not whitelisted")]
fn test_register_not_whitelisted() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let bob = Address::generate(&env);

    client.init(&owner, &true, &0i128);
    client.register(&bob, &0i128);
}

#[test]
fn test_fee_enforcement() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let alice = Address::generate(&env);

    // Initialize without whitelist-only and fee = 10
    client.init(&owner, &false, &10i128);
}

#[test]
#[should_panic(expected = "insufficient fee")]
fn test_register_insufficient_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let alice = Address::generate(&env);

    // Initialize without whitelist-only and fee = 10
    client.init(&owner, &false, &10i128);

    // Insufficient payment should panic
    client.register(&alice, &5i128);
}

#[test]
fn test_fee_enforcement_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let alice = Address::generate(&env);

    // Initialize without whitelist-only and fee = 10
    client.init(&owner, &false, &10i128);

    // Sufficient payment should succeed
    client.register(&alice, &10i128);
    assert!(client.is_registered(&alice));
}

#[test]
fn test_dispute_and_owner_removal() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let reporter = Address::generate(&env);

    client.init(&owner, &false, &0i128);
    client.register(&alice, &0i128);
    assert!(client.is_registered(&alice));

    // Reporter files removal request
    client.request_removal(&reporter, &alice, &Symbol::new(&env, "spam"));

    // Owner approves removal
    client.resolve_removal(&alice, &true);

    assert!(!client.is_registered(&alice));
}

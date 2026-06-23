use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_ajo_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Ajo);
    let client = AjoClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    client.initialize(&1000, &10, &creator);

    assert_eq!(client.get_creator(), creator);
    assert_eq!(client.get_amount(), 1000);
}

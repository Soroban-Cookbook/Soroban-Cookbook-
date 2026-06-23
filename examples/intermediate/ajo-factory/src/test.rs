use super::*;
use ajo::{Ajo, AjoClient, AjoError};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_ajo_template_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let ajo_id = env.register_contract(None, Ajo);
    let ajo_client = AjoClient::new(&env, &ajo_id);

    let creator = Address::generate(&env);
    ajo_client.initialize(&1000, &10, &creator);

    assert_eq!(ajo_client.get_creator(), creator);
    assert_eq!(ajo_client.get_amount(), 1000);
}

#[test]
fn test_factory_initialize_and_tracking() {
    let env = Env::default();
    let factory_id = env.register_contract(None, AjoFactory);
    let factory_client = AjoFactoryClient::new(&env, &factory_id);

    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    factory_client.initialize(&wasm_hash);

    let deployed_ajos = factory_client.get_deployed_ajos();
    assert_eq!(deployed_ajos.len(), 0);
}

#[test]
fn test_factory_cannot_be_reinitialized() {
    let env = Env::default();
    let factory_id = env.register_contract(None, AjoFactory);
    let factory_client = AjoFactoryClient::new(&env, &factory_id);

    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    factory_client.initialize(&wasm_hash);
    let result = factory_client.try_initialize(&wasm_hash);
    assert_eq!(result, Err(Ok(FactoryError::AlreadyInitialized)));
}

#[test]
fn test_ajo_cannot_be_reinitialized() {
    let env = Env::default();
    env.mock_all_auths();

    let ajo_id = env.register_contract(None, Ajo);
    let ajo_client = AjoClient::new(&env, &ajo_id);

    let creator = Address::generate(&env);
    ajo_client.initialize(&100, &10, &creator);

    let result = ajo_client.try_initialize(&100, &10, &creator);
    assert_eq!(result, Err(Ok(AjoError::AlreadyInitialized)));
}

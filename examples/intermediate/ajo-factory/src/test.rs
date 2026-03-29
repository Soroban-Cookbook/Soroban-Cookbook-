use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_ajo_factory_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Setup - Upload Ajo contract WASM
    let wasm_hash = env.deployer().upload_contract_wasm(Ajo::WASM);

    // 2. Initialize the Factory
    let factory_id = env.register_contract(None, AjoFactory);
    let factory_client = AjoFactoryClient::new(&env, &factory_id);
    factory_client.initialize(&wasm_hash);

    // 3. Create a new Ajo instance
    let creator = Address::generate(&env);
    let amount = 1000i128;
    let max_members = 10u32;

    let ajo_address = factory_client.create_ajo(&amount, &max_members, &creator);

    // 4. Verify the new Ajo instance
    let ajo_client = AjoClient::new(&env, &ajo_address);
    assert_eq!(ajo_client.get_creator(), creator);
    assert_eq!(ajo_client.get_amount(), amount);

    // 5. Verify Factory tracking
    let deployed_ajos = factory_client.get_deployed_ajos();
    assert_eq!(deployed_ajos.len(), 1);
    assert_eq!(deployed_ajos.get(0).unwrap(), ajo_address);

    // 6. Create another Ajo instance with the same creator (different salt)
    let amount2 = 2000i128;
    let ajo_address2 = factory_client.create_ajo(&amount2, &max_members, &creator);

    assert_ne!(ajo_address, ajo_address2);

    let deployed_ajos2 = factory_client.get_deployed_ajos();
    assert_eq!(deployed_ajos2.len(), 2);
    assert_eq!(deployed_ajos2.get(1).unwrap(), ajo_address2);
}

#[test]
#[should_panic(expected = "Factory already initialized")]
fn test_factory_cannot_be_reinitialized() {
    let env = Env::default();
    let wasm_hash = env.deployer().upload_contract_wasm(Ajo::WASM);
    let factory_id = env.register_contract(None, AjoFactory);
    let factory_client = AjoFactoryClient::new(&env, &factory_id);

    factory_client.initialize(&wasm_hash);
    factory_client.initialize(&wasm_hash);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_ajo_cannot_be_reinitialized() {
    let env = Env::default();
    env.mock_all_auths();

    let wasm_hash = env.deployer().upload_contract_wasm(Ajo::WASM);

    let factory_id = env.register_contract(None, AjoFactory);
    let factory_client = AjoFactoryClient::new(&env, &factory_id);
    factory_client.initialize(&wasm_hash);

    let creator = Address::generate(&env);
    let ajo_address = factory_client.create_ajo(&100, &10, &creator);

    let ajo_client = AjoClient::new(&env, &ajo_address);
    // Attempting to call initialize again should panic
    ajo_client.initialize(&100, &10, &creator);
}

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, testutils::Ledger as _, Address, Env};

#[test]
fn test_cross_contract_integration_and_upgrade_simulation() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy registry and factory
    let registry_id = env.register_contract(None, Registry);
    let registry_client = RegistryClient::new(&env, &registry_id);

    // Upload a placeholder WASM for factory initialization (example pattern)
    let template_wasm: [u8; 0] = [0u8; 0];
    let wasm_hash = env
        .deployer()
        .upload_contract_wasm(template_wasm.as_slice());

    let factory_id = env.register_contract(None, Factory);
    let factory_client = FactoryClient::new(&env, &factory_id);
    factory_client.initialize(&wasm_hash, &registry_id);

    // Use factory to create an "instance" and confirm registry discovers it
    let name = symbol_short!("example1");
    let creator = Address::generate(&env);
    env.mock_all_auths();
    let inst_addr = factory_client.create_instance(&1i128, &name, &creator);

    // Registry should resolve the name to the deployed address
    let resolved = registry_client.lookup(&name);
    assert_eq!(resolved, Some(inst_addr));

    // Now simulate a live upgrade flow on an actual target contract instance:
    // - Register a concrete `Target` contract
    let target_id = env.register_contract(None, Target);
    let target_client = TargetClient::new(&env, &target_id);

    // Set some instance-local state
    target_client.set_value(&12345i128);
    assert_eq!(target_client.get_value(), Some(12345i128));

    // Extend instance TTL so it would survive an upgrade process
    env.as_contract(&target_id, || {
        env.storage().instance().extend_ttl(5000, 10000);
    });

    // Simulate time passing during upgrade
    env.ledger().with_mut(|li| {
        li.sequence_number += 1000;
        li.timestamp += 5000;
    });

    // After "upgrade", instance state should still be present
    assert_eq!(target_client.get_value(), Some(12345i128));

    // Mark an upgrade via persistent storage marker and verify
    target_client.set_upgrade_marker(&7i128);
    assert_eq!(target_client.get_upgrade_marker(), Some(7i128));
}

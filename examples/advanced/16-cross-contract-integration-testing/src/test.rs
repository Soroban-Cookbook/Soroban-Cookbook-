#![allow(deprecated)]
use super::*;
use soroban_sdk::{
    symbol_short, testutils::Address as _, testutils::Ledger as _, Address, BytesN, Env,
};

#[test]
fn test_cross_contract_integration_and_upgrade_simulation() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(Registry, ());
    let registry_client = RegistryClient::new(&env, &registry_id);

    let factory_id = env.register(Factory, ());
    let factory_client = FactoryClient::new(&env, &factory_id);
    let wasm_hash = BytesN::from_array(&env, &[0x42; 32]);
    factory_client.initialize(&wasm_hash, &registry_id);

    let name = symbol_short!("example1");
    let _creator = Address::generate(&env);

    // Register a native Target instance (factory deploy needs pre-built WASM in release CI).
    let target_id = env.register(Target, ());
    registry_client.register(&name, &target_id);
    let resolved = registry_client.lookup(&name);
    assert_eq!(resolved, Some(target_id.clone()));

    let target_client = TargetClient::new(&env, &target_id);
    target_client.set_value(&12345i128);
    assert_eq!(target_client.get_value(), Some(12345i128));

    env.as_contract(&target_id, || {
        env.storage().instance().extend_ttl(5000, 10000);
    });

    env.ledger().with_mut(|li| {
        li.sequence_number += 1000;
        li.timestamp += 5000;
    });

    assert_eq!(target_client.get_value(), Some(12345i128));
    target_client.set_upgrade_marker(&7i128);
    assert_eq!(target_client.get_upgrade_marker(), Some(7i128));
}

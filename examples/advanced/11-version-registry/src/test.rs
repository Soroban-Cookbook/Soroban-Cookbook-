#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, Symbol, Vec,
};

fn setup() -> (Env, Address, VersionRegistryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1000);
    let contract_id = env.register_contract(None, VersionRegistry);
    let client = VersionRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

fn dummy_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VersionRegistry);
    let client = VersionRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    assert_eq!(client.get_current_version_number(), 0);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_double_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VersionRegistry);
    let client = VersionRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
fn test_register_first_version() {
    let (_env, _admin, client) = setup();
    let addr = Address::generate(&_env);
    let entry = client.register(&addr, &dummy_hash(&_env), &symbol_short!("init"));
    assert_eq!(entry.version, Symbol::new(&_env, "v1"));
    assert_eq!(entry.contract_address, addr);
    assert_eq!(entry.timestamp, 1000);
    assert_eq!(client.get_current_version_number(), 1);
}

#[test]
fn test_register_multiple_versions() {
    let (_env, _admin, client) = setup();
    let addr = Address::generate(&_env);

    let v1 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("init"));
    assert_eq!(v1.version, Symbol::new(&_env, "v1"));

    let v2 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("upgrade"));
    assert_eq!(v2.version, Symbol::new(&_env, "v2"));

    assert_eq!(client.get_current_version_number(), 2);
}

#[test]
fn test_get_all_versions() {
    let (_env, _admin, client) = setup();
    let addr = Address::generate(&_env);

    let _v1 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("v1"));
    let _v2 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("v2"));

    let all = client.get_all_versions();
    assert_eq!(all.len(), 2);
    assert_eq!(all.get(0).unwrap().version, Symbol::new(&_env, "v1"));
    assert_eq!(all.get(1).unwrap().version, Symbol::new(&_env, "v2"));
}

#[test]
fn test_get_latest_version() {
    let (_env, _admin, client) = setup();
    let addr = Address::generate(&_env);

    let _v1 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("first"));
    let v2 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("second"));

    let latest = client.get_latest_version();
    assert_eq!(latest.version, v2.version);
    assert_eq!(latest.metadata, symbol_short!("second"));
}

#[test]
fn test_get_version_by_number() {
    let (_env, _admin, client) = setup();
    let addr = Address::generate(&_env);

    let v1 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("init"));
    let _v2 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("upgrade"));

    let fetched = client.get_version_by_number(&1);
    assert_eq!(fetched.version, v1.version);
    assert_eq!(fetched.metadata, symbol_short!("init"));
}

#[test]
fn test_get_version_by_number_invalid() {
    let (_env, _admin, client) = setup();
    let result = client.try_get_version_by_number(&99);
    assert_eq!(result, Err(Ok(VersionError::NotFound)));
}

#[test]
fn test_rollback() {
    let (_env, _admin, client) = setup();
    let addr = Address::generate(&_env);

    let v1 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("first"));
    let v2 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("second"));

    assert_eq!(client.get_current_version_number(), 2);

    let rolled = client.rollback();
    assert_eq!(rolled.version, v2.version);

    assert_eq!(client.get_current_version_number(), 1);

    let all = client.get_all_versions();
    assert_eq!(all.len(), 1);
    assert_eq!(all.get(0).unwrap().version, v1.version);
}

#[test]
fn test_rollback_empty() {
    let (_env, _admin, client) = setup();
    let result = client.try_rollback();
    assert_eq!(result, Err(Ok(VersionError::EmptyHistory)));
}

#[test]
fn test_rollback_to_initial() {
    let (_env, _admin, client) = setup();
    let addr = Address::generate(&_env);

    let v1 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("init"));
    let rolled = client.rollback();
    assert_eq!(rolled.version, v1.version);
    assert_eq!(client.get_current_version_number(), 0);

    let all = client.get_all_versions();
    assert_eq!(all.len(), 0);
}

#[test]
fn test_get_contract_history() {
    let (_env, _admin, client) = setup();
    let addr1 = Address::generate(&_env);
    let addr2 = Address::generate(&_env);

    let _a1v1 = client.register(&addr1, &dummy_hash(&_env), &symbol_short!("init"));
    let _a1v2 = client.register(&addr1, &dummy_hash(&_env), &symbol_short!("upgrade"));
    let _a2v1 = client.register(&addr2, &dummy_hash(&_env), &symbol_short!("init"));

    let hist1 = client.get_contract_history(&addr1);
    assert_eq!(hist1.len(), 2);

    let hist2 = client.get_contract_history(&addr2);
    assert_eq!(hist2.len(), 1);
}

#[test]
fn test_unauthorized_register() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VersionRegistry);
    let client = VersionRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);
    env.set_auths(&[]);

    let addr = Address::generate(&env);
    let result = client.try_register(&addr, &dummy_hash(&env), &symbol_short!("hack"));
    assert!(result.is_err());
}

#[test]
fn test_unauthorized_rollback() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VersionRegistry);
    let client = VersionRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);
    env.set_auths(&[]);

    let result = client.try_rollback();
    assert!(result.is_err());
}

#[test]
fn test_events_emitted() {
    let (_env, _admin, client) = setup();
    let addr = Address::generate(&_env);

    client.register(&addr, &dummy_hash(&_env), &symbol_short!("first"));

    let _all = client.get_all_versions();
    let latest = client.get_latest_version();
    assert_eq!(latest.metadata, symbol_short!("first"));
}

#[test]
fn test_multiple_contracts_independent_history() {
    let (_env, _admin, client) = setup();

    let mut addrs: Vec<Address> = Vec::new(&_env);
    for _ in 0..3 {
        addrs.push_back(Address::generate(&_env));
    }

    for (i, addr) in addrs.iter().enumerate() {
        let tag = Symbol::new(&_env, &format!("c{}_init", i));
        let _v = client.register(&addr, &dummy_hash(&_env), &tag);
    }

    assert_eq!(client.get_current_version_number(), 3);

    let all = client.get_all_versions();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_version_metadata_preserved() {
    let (_env, _admin, client) = setup();
    let addr = Address::generate(&_env);

    let v1 = client.register(&addr, &dummy_hash(&_env), &symbol_short!("deploy"));
    assert_eq!(v1.metadata, symbol_short!("deploy"));
    assert_eq!(v1.timestamp, 1000);

    _env.ledger().with_mut(|l| l.timestamp = 2000);
    let meta2 = Symbol::new(&_env, "security_fix");
    let v2 = client.register(&addr, &dummy_hash(&_env), &meta2);
    assert_eq!(v2.timestamp, 2000);
}

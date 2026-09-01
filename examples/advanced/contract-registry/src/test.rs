#![allow(deprecated)]
use super::*;
use soroban_sdk::{symbol_short, Env, Symbol, Vec};

#[test]
fn test_register_and_query() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ContractRegistry);
    let client = ContractRegistryClient::new(&env, &contract_id);

    let name = symbol_short!("reg1");
    let category = symbol_short!("finance");
    let version = symbol_short!("v1");
    let addr = contract_id.clone();

    // Register
    client.register(&name, &category, &version, &addr);

    // Query by name
    let md = client.get_by_name(&name);
    assert_eq!(md.name, name);
    assert_eq!(md.category, category);
    assert_eq!(md.version, version);
    assert_eq!(md.address, addr);

    // Listing by category
    let names: Vec<Symbol> = client.list_by_category(&category);
    assert_eq!(names.len(), 1);
    assert_eq!(names.get(0).unwrap(), name);

    // Categories list contains our category
    let cats: Vec<Symbol> = client.list_categories();
    assert!(cats.iter().any(|c| c == category));
}

#[test]
fn test_duplicate_register_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ContractRegistry);
    let client = ContractRegistryClient::new(&env, &contract_id);

    let name = symbol_short!("dup");
    let category = symbol_short!("util");
    let version = symbol_short!("v1");
    let addr = contract_id.clone();

    client.register(&name, &category, &version, &addr);

    let res = client.try_register(&name, &category, &version, &addr);
    assert!(res.is_err());
}

#[test]
fn test_count_tracks_registers() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ContractRegistry);
    let client = ContractRegistryClient::new(&env, &contract_id);

    assert_eq!(client.count(), 0);
    client.register(&symbol_short!("a"), &symbol_short!("cat"), &symbol_short!("v1"), &contract_id.clone());
    client.register(&symbol_short!("b"), &symbol_short!("cat"), &symbol_short!("v1"), &contract_id.clone());
    assert_eq!(client.count(), 2);
}

#[test]
fn test_deregister_removes_entry_and_updates_index() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ContractRegistry);
    let client = ContractRegistryClient::new(&env, &contract_id);

    let name = symbol_short!("gone");
    let category = symbol_short!("tmp");
    client.register(&name, &category, &symbol_short!("v1"), &contract_id.clone());
    assert_eq!(client.count(), 1);

    client.deregister(&name);
    assert_eq!(client.count(), 0);
    let candidates = client.list_by_category(&category);
    assert_eq!(candidates.len(), 0, "category index cleaned");
    let lookup = client.try_get_by_name(&name);
    assert!(lookup.is_err() || lookup.unwrap().is_err(), "entry removed");
}

#[test]
fn test_deregister_unknown_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ContractRegistry);
    let client = ContractRegistryClient::new(&env, &contract_id);

    let res = client.try_deregister(&symbol_short!("missing"));
    assert_eq!(res, Err(Ok(RegistryError::NotFound)));
}

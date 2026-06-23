use super::*;
use soroban_sdk::{symbol_short, vec, Address, Env, Symbol, Vec};

#[test]
fn test_register_and_query() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ContractRegistry);
    let client = ContractRegistryClient::new(&env, &contract_id);

    let name = symbol_short!("reg1");
    let category = symbol_short!("finance");
    let version = symbol_short!("v1");
    let addr = Address::from_contract_id(&env, &contract_id);

    // Register
    client
        .register(&name, &category, &version, &addr)
        .expect("register failed");

    // Query by name
    let md = client.get_by_name(&name).unwrap();
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
    assert!(cats.iter().any(|c| *c == category));
}

#[test]
fn test_duplicate_register_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ContractRegistry);
    let client = ContractRegistryClient::new(&env, &contract_id);

    let name = symbol_short!("dup");
    let category = symbol_short!("util");
    let version = symbol_short!("v1");
    let addr = Address::from_contract_id(&env, &contract_id);

    client
        .register(&name, &category, &version, &addr)
        .expect("first register failed");

    let res = client.register(&name, &category, &version, &addr);
    assert!(res.is_err());
}

//! Unit tests for the Event contract

#![cfg(test)]
use super::*;
use soroban_sdk::{symbol_short, testutils::Events as _, Env, Symbol, TryFromVal};

#[test]
fn test_emit_set_number() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);

    let client = ContractClient::new(&env, &contract_id);

    client.set_number(&1000);

    let events = env.events().all();
    assert!(!events.is_empty(), "one event should be emitted");
}

#[test]
fn test_emit_set_number_details() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);

    let client = ContractClient::new(&env, &contract_id);

    let number = 1000;
    client.set_number(&number);

    let events = env.events().all();

    let event = events.get(0).unwrap();
    let (_, _, data) = event;
    let payload: u32 = u32::try_from_val(&env, &data).unwrap();
    assert_eq!(payload, number, "data do not match")
}

#[test]
fn test_emit_increment_number_details() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);

    let client = ContractClient::new(&env, &contract_id);

    let number = 1000;
    client.set_number(&number);
    client.increment();

    let events = env.events().all();
    assert_eq!(events.len(), 2);

    let (_, topics, _) = events.get(1).unwrap();

    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();

    assert_eq!(t0, symbol_short!("number"));
    assert_eq!(t1, symbol_short!("inc"));
}

#[test]
fn test_emit_increment_number() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);

    let client = ContractClient::new(&env, &contract_id);

    let number = 1000;
    client.set_number(&number);
    client.increment();

    let events = env.events().all();

    let event = events.get(1).unwrap();
    let (_, _, data) = event;
    let payload: u32 = u32::try_from_val(&env, &data).unwrap();
    assert_eq!(payload, number + 1, "data do not match")
}

#[test]
fn test_emit_decrement_number() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);

    let client = ContractClient::new(&env, &contract_id);

    let number = 1000;
    client.set_number(&number);
    client.decrement();

    let events = env.events().all();

    let event = events.get(1).unwrap();
    let (_, _, data) = event;
    let payload: u32 = u32::try_from_val(&env, &data).unwrap();
    assert_eq!(payload, number - 1, "data do not match")
}

#[test]
fn test_emit_decrement_number_details() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);

    let client = ContractClient::new(&env, &contract_id);

    let number = 1000;
    client.set_number(&number);
    client.increment();
    client.increment();
    client.decrement();

    let events = env.events().all();
    assert_eq!(events.len(), 4);

    let (_, topics, _) = events.get(3).unwrap();

    let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let t1: Symbol = Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap();

    assert_eq!(t0, symbol_short!("number"));
    assert_eq!(t1, symbol_short!("dec"));
}

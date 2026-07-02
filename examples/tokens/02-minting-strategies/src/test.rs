//! Unit tests for the minting strategies token example.

use super::*;
use soroban_sdk::{symbol_short, testutils::{Address as _, Events as _}, Address, Env, Symbol, TryFromVal};
use soroban_validation::test_events::EventList;

#[test]
fn test_initialize_and_fixed_cap_minting() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let contract_id = env.register_contract(None, MintingStrategiesToken);
    let client = MintingStrategiesTokenClient::new(&env, &contract_id);

    client
        .try_initialize(&admin, &1000, &1000, &10, &100)
        .unwrap()
        .unwrap();

    assert_eq!(client.total_supply(), 0);
    assert_eq!(client.supply_cap(), Some(1000));
    assert_eq!(client.try_scheduled_available(), Err(Ok(MintingError::ScheduleNotStarted)));

    env.ledger().set_timestamp(1010);
    let minted = client.try_mint_with_cap(&admin, &alice, &400).unwrap().unwrap();
    assert_eq!(minted, 400);
    assert_eq!(client.balance(&alice), 400);
    assert_eq!(client.total_supply(), 400);
}

#[test]
fn test_fixed_cap_mint_rejects_excess() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let bob = Address::generate(&env);
    let contract_id = env.register_contract(None, MintingStrategiesToken);
    let client = MintingStrategiesTokenClient::new(&env, &contract_id);

    client
        .try_initialize(&admin, &500, &0, &10, &50)
        .unwrap()
        .unwrap();

    client.try_mint_with_cap(&admin, &bob, &450).unwrap().unwrap();
    assert_eq!(client.try_mint_with_cap(&admin, &bob, &100), Err(Ok(MintingError::SupplyCapExceeded)));
}

#[test]
fn test_unlimited_mint_ignores_supply_cap() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let carol = Address::generate(&env);
    let contract_id = env.register_contract(None, MintingStrategiesToken);
    let client = MintingStrategiesTokenClient::new(&env, &contract_id);

    client
        .try_initialize(&admin, &100, &0, &10, &50)
        .unwrap()
        .unwrap();

    let minted = client.try_mint_unlimited(&admin, &carol, &500).unwrap().unwrap();
    assert_eq!(minted, 500);
    assert_eq!(client.total_supply(), 500);
}

#[test]
fn test_scheduled_mint_releases_over_time() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let dave = Address::generate(&env);
    let contract_id = env.register_contract(None, MintingStrategiesToken);
    let client = MintingStrategiesTokenClient::new(&env, &contract_id);

    client
        .try_initialize(&admin, &0, &1000, &10, &200)
        .unwrap()
        .unwrap();

    env.ledger().set_timestamp(1010);
    assert_eq!(client.try_scheduled_available().unwrap().unwrap(), 200);

    let minted = client.try_mint_scheduled(&admin, &dave, &150).unwrap().unwrap();
    assert_eq!(minted, 150);
    assert_eq!(client.balance(&dave), 150);
    assert_eq!(client.total_supply(), 150);

    assert_eq!(client.try_scheduled_available().unwrap().unwrap(), 50);

    env.ledger().set_timestamp(1020);
    assert_eq!(client.try_scheduled_available().unwrap().unwrap(), 250);
}

#[test]
fn test_scheduled_mint_fails_before_start() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let eve = Address::generate(&env);
    let contract_id = env.register_contract(None, MintingStrategiesToken);
    let client = MintingStrategiesTokenClient::new(&env, &contract_id);

    client
        .try_initialize(&admin, &0, &2000, &10, &100)
        .unwrap()
        .unwrap();

    env.ledger().set_timestamp(1999);
    assert_eq!(client.try_mint_scheduled(&admin, &eve, &50), Err(Ok(MintingError::ScheduleNotStarted)));
}

#[test]
fn test_mint_emits_event_with_strategy_topic() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let frank = Address::generate(&env);
    let contract_id = env.register_contract(None, MintingStrategiesToken);
    let client = MintingStrategiesTokenClient::new(&env, &contract_id);

    client
        .try_initialize(&admin, &0, &0, &10, &100)
        .unwrap()
        .unwrap();

    client.try_mint_with_cap(&admin, &frank, &50).unwrap().unwrap();
    let events = EventList::new(&env, env.events().all());
    assert_eq!(events.len(), 1);
    let (_id, topics, data) = events.get(0).unwrap();
    assert_eq!(symbol_short!("mint"), Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap());
    assert_eq!(symbol_short!("fixed"), Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap());
    assert_eq!(frank, Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap());

    let payload: MintEventData = MintEventData::try_from_val(&env, &data).unwrap();
    assert_eq!(payload.amount, 50);
    assert_eq!(payload.strategy, symbol_short!("fixed"));
}

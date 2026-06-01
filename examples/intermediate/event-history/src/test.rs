//! Unit tests for the event history contract.

use super::*;
use soroban_sdk::{symbol_short, Address, Env};

fn setup() -> (Env, Address, EventHistoryClient) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, EventHistory);
    let client = EventHistoryClient::new(&env, &contract_id);
    client.initialize(&admin, &3).unwrap();
    (env, admin, client)
}

#[test]
fn test_append_and_paginate_history_entries() {
    let (mut env, _admin, client) = setup();
    env.mock_all_auths();

    let actor = Address::generate(&env);
    let action = symbol_short!("create");
    let details = symbol_short!("first");
    client.append_event(&actor, &action, &details).unwrap();

    env.ledger().set_timestamp(env.ledger().timestamp() + 10);
    let details2 = symbol_short!("second");
    client.append_event(&actor, &action, &details2).unwrap();

    env.ledger().set_timestamp(env.ledger().timestamp() + 10);
    let details3 = symbol_short!("third");
    client.append_event(&actor, &action, &details3).unwrap();

    let stats = client.history_stats();
    assert_eq!(stats.count, 3);
    assert_eq!(stats.max_entries, 3);

    let page = client.get_events(&0, &2);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().details, details);

    let all = client.get_events(&0, &10);
    assert_eq!(all.len(), 3);
}

#[test]
fn test_storage_limit_trims_oldest_entries() {
    let (mut env, _admin, client) = setup();
    env.mock_all_auths();

    let actor = Address::generate(&env);
    let action = symbol_short!("write");

    client.append_event(&actor, &action, &symbol_short!("first")).unwrap();
    client.append_event(&actor, &action, &symbol_short!("second")).unwrap();
    client.append_event(&actor, &action, &symbol_short!("third")).unwrap();

    let stats = client.history_stats();
    assert_eq!(stats.count, 3);
    assert_eq!(stats.start_index, 1);
    assert_eq!(stats.next_index, 3);

    let page = client.get_events(&0, &10);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().details, symbol_short!("second"));
}

#[test]
fn test_query_by_time_returns_matching_entries() {
    let (mut env, _admin, client) = setup();
    env.mock_all_auths();

    let actor = Address::generate(&env);
    let action = symbol_short!("time");

    let first_ts = env.ledger().timestamp();
    client.append_event(&actor, &action, &symbol_short!("one")).unwrap();
    env.ledger().set_timestamp(first_ts + 20);
    client.append_event(&actor, &action, &symbol_short!("two")).unwrap();
    env.ledger().set_timestamp(first_ts + 40);
    client.append_event(&actor, &action, &symbol_short!("three")).unwrap();

    let range = client.query_by_time(&(first_ts + 10), &(first_ts + 30), &10);
    assert_eq!(range.len(), 1);
    assert_eq!(range.get(0).unwrap().details, symbol_short!("two"));
}

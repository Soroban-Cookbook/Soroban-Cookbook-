#![allow(deprecated)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup() -> (Env, Address, SnapshotTriggerClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &10);
    (env, admin, client)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &10);

    let (freq, enabled, last, total) = client.get_config();
    assert_eq!(freq, 10);
    assert!(enabled);
    assert_eq!(last, 0);
    assert_eq!(total, 0);
}

#[test]
fn test_initialize_cannot_be_called_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &10);
    let result = client.try_initialize(&admin, &20);
    assert!(result.is_err());
}

#[test]
fn test_record_value_event_based() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.record_value(&user, &1000);
    assert_eq!(client.snapshot_count(&user), 1);

    client.record_value(&user, &2000);
    assert_eq!(client.snapshot_count(&user), 2);

    let snapshots = client.get_all_snapshots(&user);
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots.get_unchecked(0).value, 1000);
    assert_eq!(snapshots.get_unchecked(1).value, 2000);
}

#[test]
fn test_auto_snapshot_time_based_when_due() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.record_value(&user, &500);
    assert_eq!(client.snapshot_count(&user), 1);

    env.ledger().set_sequence_number(10);
    let result = client.try_auto_snapshot(&user);
    assert!(result.is_ok());
    assert_eq!(client.snapshot_count(&user), 2);
}

#[test]
fn test_auto_snapshot_skipped_when_not_due() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.record_value(&user, &500);
    env.ledger().set_sequence_number(5);
    let result = client.try_auto_snapshot(&user);
    assert!(result.is_err());
    assert_eq!(client.snapshot_count(&user), 1);
}

#[test]
fn test_auto_snapshot_disabled() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    client.set_enabled(&admin, &false);
    env.ledger().set_sequence_number(20);
    let result = client.try_auto_snapshot(&user);
    assert!(result.is_err());
}

#[test]
fn test_get_snapshot_by_index() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.record_value(&user, &100);
    client.record_value(&user, &200);
    client.record_value(&user, &300);

    let first = client.get_snapshot(&user, &0);
    assert_eq!(first.value, 100);

    let second = client.get_snapshot(&user, &1);
    assert_eq!(second.value, 200);

    let third = client.get_snapshot(&user, &2);
    assert_eq!(third.value, 300);
}

#[test]
fn test_get_snapshot_out_of_bounds() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    client.record_value(&user, &100);
    let result = client.try_get_snapshot(&user, &5);
    assert!(result.is_err());
}

#[test]
fn test_get_latest_snapshot() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.record_value(&user, &42);
    client.record_value(&user, &99);
    let latest = client.get_latest(&user);
    assert_eq!(latest.value, 99);
}

#[test]
fn test_get_latest_empty_history() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    let result = client.try_get_latest(&user);
    assert!(result.is_err());
}

#[test]
fn test_prune_old_snapshots() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    env.ledger().set_sequence_number(10);
    client.record_value(&user, &10);
    env.ledger().set_sequence_number(20);
    client.record_value(&user, &20);
    env.ledger().set_sequence_number(30);
    client.record_value(&user, &30);
    assert_eq!(client.snapshot_count(&user), 3);

    let pruned = client.prune(&admin, &user, &20);
    assert_eq!(pruned, 1);

    let remaining = client.get_all_snapshots(&user);
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining.get_unchecked(0).value, 20);
}

#[test]
fn test_prune_nothing_to_remove() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    env.ledger().set_sequence_number(10);
    client.record_value(&user, &10);
    env.ledger().set_sequence_number(20);
    client.record_value(&user, &20);

    let result = client.try_prune(&admin, &user, &5);
    assert!(result.is_err());
}

#[test]
fn test_set_frequency_updates_config() {
    let (env, admin, client) = setup();
    client.set_frequency(&admin, &25);
    let (freq, ..) = client.get_config();
    assert_eq!(freq, 25);
}

#[test]
fn test_unauthorized_admin_actions_fail() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    client.initialize(&admin, &10);

    let result = client.try_set_frequency(&attacker, &20);
    assert!(result.is_err());
}

#[test]
fn test_snapshot_timestamps_and_ledgers_recorded() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    env.ledger().set_sequence_number(100);
    env.ledger().set_timestamp(1000);
    client.record_value(&user, &777);

    let snap = client.get_snapshot(&user, &0);
    assert_eq!(snap.ledger, 100);
    assert_eq!(snap.timestamp, 1000);
    assert_eq!(snap.value, 777);
}

#[test]
fn test_get_config_returns_expected_values() {
    let (env, admin, client) = setup();
    let (freq, enabled, last, total) = client.get_config();
    assert_eq!(freq, 10);
    assert!(enabled);
    assert_eq!(last, 0);
    assert_eq!(total, 0);

    env.ledger().set_sequence_number(15);
    client.record_value(&admin, &500);
    let (.., total) = client.get_config();
    assert_eq!(total, 1);
}

#![allow(deprecated)]
//! Unit tests for the storage migration contract.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, Address, StorageMigrationClient<'static>) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(StorageMigration, ());
    let client = StorageMigrationClient::new(&env, &contract_id);
    client.initialize(&admin);
    (env, admin, client)
}

#[test]
fn test_explicit_v1_to_v2_migration_transforms_legacy_data() {
    let (env, _admin, client) = setup();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    client.add_user(&alice, &150);

    let processed = client.migrate_v1_to_v2(&2);
    assert_eq!(processed, 1);
    assert_eq!(client.get_version(), 2);

    let profile = client.profile(&alice).unwrap();
    assert_eq!(profile.balance, 150);
    assert_eq!(client.legacy_balance(&alice), 0);
}

#[test]
fn test_migrate_v1_to_v2_rejects_non_v1_versions() {
    let (env, _admin, client) = setup();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    client.add_user(&alice, &150);

    client.migrate_v1_to_v2(&1);
    let result = client.try_migrate_v1_to_v2(&1);
    assert!(result.is_err());
}

#[test]
fn test_prepare_and_execute_migration_batches() {
    let (env, _admin, client) = setup();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.add_user(&alice, &100);
    client.add_user(&bob, &200);

    client.prepare_migration(&2);
    let state = client.migration_state();
    assert!(matches!(state, MigrationState::Prepared(_, _)));
    assert!(matches!(state, MigrationState::Prepared(..)));

    let processed = client.migrate_batch(&1);
    assert_eq!(processed, 1);
    let state = client.migration_state();
    assert!(matches!(state, MigrationState::Prepared(_, 1)));
    assert!(matches!(state, MigrationState::Prepared(_, next_index) if next_index == 1));

    let processed = client.migrate_batch(&10);
    assert_eq!(processed, 1);
    assert_eq!(client.get_version(), 2);
    assert!(matches!(client.migration_state(), MigrationState::None));

    assert_eq!(client.profile(&alice).unwrap().balance, 100);
    assert_eq!(client.profile(&bob).unwrap().balance, 200);
    assert_eq!(client.legacy_balance(&alice), 0);
}

#[test]
fn test_cancel_migration_resets_state() {
    let (env, _admin, client) = setup();
    env.mock_all_auths();

    client.prepare_migration(&2);
    client.cancel_migration();

    assert!(matches!(client.migration_state(), MigrationState::None));
}

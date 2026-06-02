//! Unit tests for the storage migration contract.

use super::*;
use soroban_sdk::{Address, Env};

fn setup() -> (Env, Address, StorageMigrationClient) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, StorageMigration);
    let client = StorageMigrationClient::new(&env, &contract_id);
    client.initialize(&admin).unwrap();
    (env, admin, client)
}

#[test]
fn test_prepare_and_execute_migration_batches() {
    let (env, admin, client) = setup();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.add_user(&alice, &100).unwrap();
    client.add_user(&bob, &200).unwrap();

    client.prepare_migration(&2).unwrap();
    let state = client.migration_state();
    assert!(matches!(state, MigrationState::Prepared { .. }));

    let processed = client.migrate_batch(&1).unwrap();
    assert_eq!(processed, 1);
    let state = client.migration_state();
    assert!(matches!(state, MigrationState::Prepared { next_index, .. } if next_index == 1));

    let processed = client.migrate_batch(&10).unwrap();
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

    client.prepare_migration(&2).unwrap();
    client.cancel_migration().unwrap();

    assert!(matches!(client.migration_state(), MigrationState::None));
}

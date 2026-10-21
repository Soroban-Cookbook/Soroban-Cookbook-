#![cfg(test)]
#![allow(deprecated)]

extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};

fn setup() -> (Env, Address, ContractMigrationsClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);

    let admin = Address::generate(&env);
    let id = env.register(ContractMigrations, ());
    let client = ContractMigrationsClient::new(&env, &id);
    client.initialize(&admin);
    (env, admin, client)
}

#[test]
fn full_batched_migration_v1_to_v2() {
    let (env, _admin, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.add_user(&alice, &500);
    client.add_user(&bob, &5_000);
    client.add_user(&carol, &50_000);
    assert_eq!(client.get_version(), VERSION_V1);
    assert_eq!(client.user_count(), 3);

    assert_eq!(
        client.try_credit(&alice, &10),
        Err(Ok(MigrationError::MigrationRequired))
    );

    client.prepare_migration(&VERSION_V2);
    assert!(matches!(
        client.migration_state(),
        MigrationState::InProgress(VERSION_V2, 0)
    ));

    assert_eq!(client.balance_of(&alice), 500);

    assert_eq!(client.migrate_batch(&2), 2);
    assert!(matches!(
        client.migration_state(),
        MigrationState::InProgress(VERSION_V2, 2)
    ));
    assert!(client.get_account(&alice).is_some());
    assert!(client.get_account(&bob).is_some());
    assert!(client.get_account(&carol).is_none());
    assert_eq!(client.balance_of(&carol), 50_000);

    assert_eq!(client.migrate_batch(&2), 1);
    assert_eq!(client.migration_state(), MigrationState::None);
    assert_eq!(client.get_version(), VERSION_V2);

    let carol_acct = client.get_account(&carol).unwrap();
    assert_eq!(carol_acct.balance, 50_000);
    assert_eq!(carol_acct.tier, 2);
    assert_eq!(carol_acct.last_active, 1_700_000_000);

    let bob_acct = client.get_account(&bob).unwrap();
    assert_eq!(bob_acct.tier, 1);

    let credited = client.credit(&alice, &25);
    assert_eq!(credited.balance, 525);
}

#[test]
fn cancel_and_idempotent_guards() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    client.add_user(&user, &100);

    client.prepare_migration(&VERSION_V2);
    client.cancel_migration();
    assert_eq!(client.migration_state(), MigrationState::None);
    assert_eq!(
        client.try_migrate_batch(&1),
        Err(Ok(MigrationError::MigrationNotPrepared))
    );

    client.prepare_migration(&VERSION_V2);
    client.migrate_batch(&10);
    assert_eq!(
        client.try_prepare_migration(&VERSION_V2),
        Err(Ok(MigrationError::InvalidVersion))
    );
}

#[test]
fn upgrade_auth_passes_before_host_stub() {
    let (env, _admin, client) = setup();
    let hash = BytesN::from_array(&env, &[7u8; 32]);

    let result = client.try_upgrade(&hash);
    match result {
        Ok(_) => {}
        Err(Ok(e)) => {
            assert_ne!(e, MigrationError::Unauthorized);
            assert_ne!(e, MigrationError::NotInitialized);
        }
        Err(Err(_)) => {}
    }
}

#[test]
fn rejects_bad_batch_and_duplicate_init() {
    let (_env, admin, client) = setup();
    assert_eq!(
        client.try_initialize(&admin),
        Err(Ok(MigrationError::AlreadyInitialized))
    );

    client.prepare_migration(&VERSION_V2);
    assert_eq!(
        client.try_migrate_batch(&0),
        Err(Ok(MigrationError::InvalidBatchSize))
    );
    assert_eq!(
        client.try_migrate_batch(&101),
        Err(Ok(MigrationError::InvalidBatchSize))
    );
}

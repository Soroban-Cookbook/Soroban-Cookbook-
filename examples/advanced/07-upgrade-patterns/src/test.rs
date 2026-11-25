#![allow(deprecated)]
//! Tests for the upgrade-patterns example crate.
//!
//! Each section tests one module in isolation. The test environment does not
//! have a real WASM registry, so any call that reaches
//! `env.deployer().update_current_contract_wasm(...)` will produce a
//! host-level error. All guard logic (auth checks, version checks, init
//! guards) runs *before* that call and is therefore fully testable.
//!
//! The pattern used in the `upgrade_*` tests is borrowed from `31-proxy-admin`:
//! assert that the result is NOT one of our own error variants, then accept
//! any host-level error as proof that the deployer stub fired.

#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn dummy_hash(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

// ===========================================================================
// Pattern 1 — Direct Upgrade
// ===========================================================================

mod direct {
    use super::*;
    use crate::direct_upgrade::{DirectUpgradeContract, DirectUpgradeContractClient, UpgradeError};

    fn setup() -> (Env, Address, DirectUpgradeContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, DirectUpgradeContract);
        let client = DirectUpgradeContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, admin, client)
    }

    // ── Initialisation ───────────────────────────────────────────────────────

    #[test]
    fn initialize_stores_admin() {
        let (_env, admin, client) = setup();
        assert_eq!(client.admin(), admin);
    }

    #[test]
    fn double_initialize_is_rejected() {
        let (_env, admin, client) = setup();
        assert_eq!(
            client.try_initialize(&admin),
            Err(Ok(UpgradeError::AlreadyInitialized))
        );
    }

    #[test]
    fn uninitialised_admin_returns_error() {
        let env = Env::default();
        let id = env.register_contract(None, DirectUpgradeContract);
        let client = DirectUpgradeContractClient::new(&env, &id);
        assert_eq!(client.try_admin(), Err(Ok(UpgradeError::NotInitialized)));
    }

    // ── Auth guards ──────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn upgrade_without_auth_panics() {
        let env = Env::default();
        let id = env.register_contract(None, DirectUpgradeContract);
        let client = DirectUpgradeContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        env.mock_all_auths();
        client.initialize(&admin);
        // Strip all auths — the host should reject the call before our code runs.
        env.set_auths(&[]);
        client.upgrade(&dummy_hash(&env, 1));
    }

    #[test]
    fn upgrade_before_init_returns_not_initialised() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, DirectUpgradeContract);
        let client = DirectUpgradeContractClient::new(&env, &id);
        assert_eq!(
            client.try_upgrade(&dummy_hash(&env, 2)),
            Err(Ok(UpgradeError::NotInitialized))
        );
    }

    // ── Upgrade guard passes, deployer stub fires ────────────────────────────

    #[test]
    fn upgrade_guard_passes_deployer_fires() {
        // After auth and init checks pass, the only thing left to fail is the
        // deployer stub. Verify we do NOT get our own UpgradeError variants.
        let (env, _admin, client) = setup();
        let result = client.try_upgrade(&dummy_hash(&env, 3));
        match result {
            Ok(_) => {}
            Err(Ok(e)) => {
                assert_ne!(e, UpgradeError::Unauthorized);
                assert_ne!(e, UpgradeError::NotInitialized);
                assert_ne!(e, UpgradeError::AlreadyInitialized);
            }
            Err(Err(_)) => {} // host-level error from deployer stub — expected
        }
    }

    // ── Benchmarks ───────────────────────────────────────────────────────────

    #[cfg(test)]
    mod bench {
        extern crate std;
        use super::*;

        #[test]
        fn bench_initialize() {
            let env = Env::default();
            env.mock_all_auths();
            let id = env.register_contract(None, DirectUpgradeContract);
            let client = DirectUpgradeContractClient::new(&env, &id);
            let admin = Address::generate(&env);
            env.budget().reset_default();
            client.initialize(&admin);
            let cpu = env.budget().cpu_instruction_cost();
            let mem = env.budget().memory_bytes_cost();
            std::println!("[bench] direct_upgrade::initialize  cpu={cpu}  mem={mem}");
        }
    }
}

// ===========================================================================
// Pattern 2 — Versioned Upgrade & Migration
// ===========================================================================

mod versioned {
    use super::*;
    use crate::versioned_upgrade::{
        seed_v1_counter, VersionedError, VersionedUpgradeContract, VersionedUpgradeContractClient,
        CURRENT_VERSION, LEGACY_VERSION,
    };

    fn setup() -> (Env, Address, VersionedUpgradeContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, VersionedUpgradeContract);
        let client = VersionedUpgradeContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, admin, client)
    }

    // ── Initialisation ───────────────────────────────────────────────────────

    #[test]
    fn initialize_sets_legacy_version() {
        let (_env, _admin, client) = setup();
        assert_eq!(client.storage_version(), LEGACY_VERSION);
    }

    #[test]
    fn double_initialize_is_rejected() {
        let (_env, admin, client) = setup();
        assert_eq!(
            client.try_initialize(&admin),
            Err(Ok(VersionedError::AlreadyInitialized))
        );
    }

    // ── Migration ────────────────────────────────────────────────────────────

    #[test]
    fn migrate_transforms_v1_to_v2() {
        let (env, _admin, client) = setup();
        let contract_id = client.address.clone();

        // Seed the contract with a known v1 counter value so we can assert
        // the migration preserved it.
        env.as_contract(&contract_id, || seed_v1_counter(&env, 42));

        // Advance ledger time so `last_updated` is non-zero after migration.
        env.ledger().with_mut(|l| l.timestamp = 1_000);

        client.migrate();

        assert_eq!(client.storage_version(), CURRENT_VERSION);

        let counter = client.get_counter();
        assert_eq!(counter.val, 42, "migration must preserve the counter value");
        assert_eq!(
            counter.last_updated, 1_000,
            "migration must record the ledger timestamp"
        );
    }

    #[test]
    fn migrate_with_zero_counter_is_valid() {
        let (env, _admin, client) = setup();
        env.ledger().with_mut(|l| l.timestamp = 500);
        client.migrate();

        let counter = client.get_counter();
        assert_eq!(counter.val, 0);
        assert_eq!(counter.last_updated, 500);
    }

    #[test]
    fn double_migrate_is_rejected() {
        let (_env, _admin, client) = setup();
        client.migrate();
        assert_eq!(
            client.try_migrate(),
            Err(Ok(VersionedError::AlreadyMigrated))
        );
    }

    #[test]
    fn get_counter_before_migration_panics() {
        let (_env, _admin, client) = setup();
        // Storage is at LEGACY_VERSION — v2 entry point must refuse.
        let result = client.try_get_counter();
        assert!(
            result.is_err(),
            "get_counter must fail before migration runs"
        );
    }

    #[test]
    fn increment_before_migration_panics() {
        let (_env, _admin, client) = setup();
        let result = client.try_increment(&5i64);
        assert!(result.is_err(), "increment must fail before migration runs");
    }

    // ── Post-migration business logic ────────────────────────────────────────

    #[test]
    fn increment_after_migration_works() {
        let (env, _admin, client) = setup();
        env.ledger().with_mut(|l| l.timestamp = 100);
        client.migrate();

        env.ledger().with_mut(|l| l.timestamp = 200);
        let result = client.increment(&10i64);

        assert_eq!(result.val, 10);
        assert_eq!(result.last_updated, 200);
    }

    #[test]
    fn increment_accumulates_correctly() {
        let (env, _admin, client) = setup();
        env.ledger().with_mut(|l| l.timestamp = 0);
        client.migrate();

        client.increment(&5i64);
        client.increment(&3i64);
        let final_counter = client.increment(&2i64);

        assert_eq!(final_counter.val, 10, "5 + 3 + 2 = 10");
    }

    // ── Auth on migration ────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn migrate_without_auth_panics() {
        let env = Env::default();
        let id = env.register_contract(None, VersionedUpgradeContract);
        let client = VersionedUpgradeContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        env.mock_all_auths();
        client.initialize(&admin);
        env.set_auths(&[]);
        client.migrate();
    }

    // ── Upgrade guard passes, deployer stub fires ────────────────────────────

    #[test]
    fn upgrade_guard_passes_deployer_fires() {
        let (env, _admin, client) = setup();
        let result = client.try_upgrade(&dummy_hash(&env, 10));
        match result {
            Ok(_) => {}
            Err(Ok(e)) => {
                assert_ne!(e, VersionedError::Unauthorized);
                assert_ne!(e, VersionedError::NotInitialized);
            }
            Err(Err(_)) => {}
        }
    }

    // ── Benchmarks ───────────────────────────────────────────────────────────

    #[cfg(test)]
    mod bench {
        extern crate std;
        use super::*;
        use crate::versioned_upgrade::VersionedUpgradeContract;

        fn setup_bench() -> (Env, Address, VersionedUpgradeContractClient<'static>) {
            let env = Env::default();
            env.mock_all_auths();
            let id = env.register_contract(None, VersionedUpgradeContract);
            let client = VersionedUpgradeContractClient::new(&env, &id);
            let admin = Address::generate(&env);
            client.initialize(&admin);
            (env, admin, client)
        }

        #[test]
        fn bench_migrate() {
            let (env, _admin, client) = setup_bench();
            env.budget().reset_default();
            client.migrate();
            let cpu = env.budget().cpu_instruction_cost();
            let mem = env.budget().memory_bytes_cost();
            std::println!("[bench] versioned_upgrade::migrate  cpu={cpu}  mem={mem}");
        }

        #[test]
        fn bench_increment() {
            let (env, _admin, client) = setup_bench();
            client.migrate();
            env.budget().reset_default();
            client.increment(&1i64);
            let cpu = env.budget().cpu_instruction_cost();
            let mem = env.budget().memory_bytes_cost();
            std::println!("[bench] versioned_upgrade::increment  cpu={cpu}  mem={mem}");
        }
    }
}

// ===========================================================================
// Pattern 3 — Init Guard
// ===========================================================================

mod init_guard {
    use super::*;
    use crate::init_guard::{
        InitError, InitGuardContract, InitGuardContractClient, UPGRADE_INIT_VERSION,
    };

    fn setup() -> (Env, Address, InitGuardContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, InitGuardContract);
        let client = InitGuardContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, admin, client)
    }

    // ── Guard A: double-init prevention ──────────────────────────────────────

    #[test]
    fn initialize_sets_flag() {
        let (_env, _admin, client) = setup();
        assert!(client.is_initialized());
    }

    #[test]
    fn initialize_stores_admin() {
        let (_env, admin, client) = setup();
        assert_eq!(client.admin(), admin);
    }

    #[test]
    fn double_initialize_is_rejected() {
        let (_env, admin, client) = setup();
        assert_eq!(
            client.try_initialize(&admin),
            Err(Ok(InitError::AlreadyInitialized))
        );
    }

    #[test]
    fn double_initialize_cannot_replace_admin() {
        // Even if an attacker passes their own address, the guard fires before
        // any storage is written.
        let (env, original_admin, client) = setup();
        let attacker = Address::generate(&env);
        let _ = client.try_initialize(&attacker);
        // Admin must still be the original address.
        assert_eq!(client.admin(), original_admin);
    }

    #[test]
    fn is_initialized_returns_false_before_init() {
        let env = Env::default();
        let id = env.register_contract(None, InitGuardContract);
        let client = InitGuardContractClient::new(&env, &id);
        assert!(!client.is_initialized());
    }

    // ── Guard B: post-upgrade init ────────────────────────────────────────────

    #[test]
    fn post_upgrade_init_seeds_feature_flag() {
        let (_env, _admin, client) = setup();
        // Before post-upgrade init, the flag key is absent.
        assert!(client.feature_flag_v2().is_none());

        client.post_upgrade_init(&UPGRADE_INIT_VERSION);

        // After the call the flag is present and set to false.
        assert_eq!(client.feature_flag_v2(), Some(false));
        assert_eq!(client.setup_version(), UPGRADE_INIT_VERSION);
    }

    #[test]
    fn post_upgrade_init_is_idempotent() {
        let (_env, _admin, client) = setup();
        client.post_upgrade_init(&UPGRADE_INIT_VERSION);
        // A second call must return AlreadyRan, not panic.
        assert_eq!(
            client.try_post_upgrade_init(&UPGRADE_INIT_VERSION),
            Err(Ok(InitError::AlreadyRan))
        );
        // State must be unchanged.
        assert_eq!(client.feature_flag_v2(), Some(false));
    }

    #[test]
    fn post_upgrade_init_out_of_sequence_is_rejected() {
        let (_env, _admin, client) = setup();
        // stored setup_version = 1; calling with expected = 3 skips version 2.
        assert_eq!(
            client.try_post_upgrade_init(&3u32),
            Err(Ok(InitError::OutOfSequence))
        );
    }

    #[test]
    fn setup_version_starts_at_one() {
        let (_env, _admin, client) = setup();
        // No post-upgrade init has run; should default to 1.
        assert_eq!(client.setup_version(), 1u32);
    }

    // ── Auth guards ──────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn initialize_without_auth_panics() {
        let env = Env::default();
        // No mock_all_auths
        let id = env.register_contract(None, InitGuardContract);
        let client = InitGuardContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn post_upgrade_init_without_auth_panics() {
        let env = Env::default();
        let id = env.register_contract(None, InitGuardContract);
        let client = InitGuardContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        env.mock_all_auths();
        client.initialize(&admin);
        env.set_auths(&[]);
        client.post_upgrade_init(&UPGRADE_INIT_VERSION);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn upgrade_without_auth_panics() {
        let env = Env::default();
        let id = env.register_contract(None, InitGuardContract);
        let client = InitGuardContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        env.mock_all_auths();
        client.initialize(&admin);
        env.set_auths(&[]);
        client.upgrade(&dummy_hash(&env, 20));
    }

    // ── Upgrade guard passes, deployer stub fires ────────────────────────────

    #[test]
    fn upgrade_guard_passes_deployer_fires() {
        let (env, _admin, client) = setup();
        let result = client.try_upgrade(&dummy_hash(&env, 21));
        match result {
            Ok(_) => {}
            Err(Ok(e)) => {
                assert_ne!(e, InitError::Unauthorized);
                assert_ne!(e, InitError::NotInitialized);
            }
            Err(Err(_)) => {}
        }
    }

    // ── Benchmarks ───────────────────────────────────────────────────────────

    #[cfg(test)]
    mod bench {
        extern crate std;
        use super::*;

        fn setup_bench() -> (Env, Address, InitGuardContractClient<'static>) {
            let env = Env::default();
            env.mock_all_auths();
            let id = env.register_contract(None, InitGuardContract);
            let client = InitGuardContractClient::new(&env, &id);
            let admin = Address::generate(&env);
            client.initialize(&admin);
            (env, admin, client)
        }

        #[test]
        fn bench_initialize() {
            let env = Env::default();
            env.mock_all_auths();
            let id = env.register_contract(None, InitGuardContract);
            let client = InitGuardContractClient::new(&env, &id);
            let admin = Address::generate(&env);
            env.budget().reset_default();
            client.initialize(&admin);
            let cpu = env.budget().cpu_instruction_cost();
            let mem = env.budget().memory_bytes_cost();
            std::println!("[bench] init_guard::initialize  cpu={cpu}  mem={mem}");
        }

        #[test]
        fn bench_post_upgrade_init() {
            let (env, _admin, client) = setup_bench();
            env.budget().reset_default();
            client.post_upgrade_init(&UPGRADE_INIT_VERSION);
            let cpu = env.budget().cpu_instruction_cost();
            let mem = env.budget().memory_bytes_cost();
            std::println!("[bench] init_guard::post_upgrade_init  cpu={cpu}  mem={mem}");
        }
    }
}

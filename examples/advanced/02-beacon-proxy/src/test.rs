//! Integration tests for the Beacon Proxy pattern.
//!
//! ## Test coverage
//!
//! | # | Test | What it verifies |
//! |---|---|---|
//! | 1 | `test_beacon_init` | Beacon initialises with correct impl, version = 1 |
//! | 2 | `test_beacon_upgrade_increments_version` | Version counter increments on upgrade |
//! | 3 | `test_beacon_version_history_logged` | Version log entries are stored correctly |
//! | 4 | `test_beacon_upgrade_unauthorized` | Non-admin cannot upgrade (auth guard) |
//! | 5 | `test_beacon_double_init_panics` | Second `init` is rejected |
//! | 6 | `test_beacon_transfer_admin` | Admin rights can be transferred |
//! | 7 | `test_proxy_init_and_delegates` | Proxy resolves impl via Beacon, `add`/`sub` work |
//! | 8 | `test_upgrade_propagates_to_proxy` | Upgrading Beacon immediately changes proxy behaviour |
//! | 9 | `test_mul_available_after_upgrade` | `mul` (v2-only) is accessible after upgrade |
//! | 10 | `test_counter_persists_across_upgrade` | Impl state survives a Beacon upgrade |
//! | 11 | `test_proxy_set_beacon` | Proxy can be re-pointed to a new Beacon |
//! | 12 | `test_multiple_proxies_one_beacon` | Two proxies share one Beacon; one upgrade updates both |
//! | 13 | `test_beacon_get_version_entry` | `get_version_entry` returns correct historical record |
//! | 14 | `test_beacon_version_entry_not_found_panics` | Querying a missing version panics |

#![cfg(test)]

extern crate std;

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

use crate::{
    BeaconContract, BeaconContractClient, ImplV1, ImplV1Client, ImplV2, ImplV2Client,
    ProxyContract, ProxyContractClient,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Register all four contracts and return their addresses.
struct Contracts {
    beacon: Address,
    proxy: Address,
    impl_v1: Address,
    impl_v2: Address,
}

fn register_all(env: &Env) -> Contracts {
    Contracts {
        beacon: env.register(BeaconContract, ()),
        proxy: env.register(ProxyContract, ()),
        impl_v1: env.register(ImplV1, ()),
        impl_v2: env.register(ImplV2, ()),
    }
}

// ---------------------------------------------------------------------------
// Beacon tests
// ---------------------------------------------------------------------------

/// Test 1: Beacon initialises correctly.
#[test]
fn test_beacon_init() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);

    let admin = Address::generate(&env);
    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));

    assert_eq!(beacon.get_implementation(), c.impl_v1);
    assert_eq!(beacon.get_version(), 1u32);
    assert_eq!(beacon.get_admin(), admin);
}

/// Test 2: Upgrading the Beacon increments the version counter.
#[test]
fn test_beacon_upgrade_increments_version() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    assert_eq!(beacon.get_version(), 1u32);

    beacon.upgrade(&c.impl_v2, &symbol_short!("v2"));
    assert_eq!(beacon.get_version(), 2u32);
    assert_eq!(beacon.get_implementation(), c.impl_v2);
}

/// Test 3: Version history is logged for every upgrade.
#[test]
fn test_beacon_version_history_logged() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    beacon.upgrade(&c.impl_v2, &symbol_short!("v2"));

    let entry_v1 = beacon.get_version_entry(&1u32);
    let entry_v2 = beacon.get_version_entry(&2u32);

    assert_eq!(entry_v1.implementation, c.impl_v1);
    assert_eq!(entry_v1.label, symbol_short!("v1"));
    assert_eq!(entry_v2.implementation, c.impl_v2);
    assert_eq!(entry_v2.label, symbol_short!("v2"));
}

/// Test 4: Only the admin can upgrade the Beacon.
#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_beacon_upgrade_unauthorized() {
    let env = Env::default();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let admin = Address::generate(&env);

    // Init with mocked auth.
    env.mock_all_auths();
    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));

    // Strip all auths — upgrade should now fail.
    env.set_auths(&[]);
    beacon.upgrade(&c.impl_v2, &symbol_short!("v2"));
}

/// Test 5: Calling `init` a second time panics.
#[test]
#[should_panic(expected = "Already initialized")]
fn test_beacon_double_init_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    beacon.init(&admin, &c.impl_v2, &symbol_short!("v2")); // must panic
}

/// Test 6: Admin rights can be transferred to a new address.
#[test]
fn test_beacon_transfer_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    beacon.transfer_admin(&new_admin);

    assert_eq!(beacon.get_admin(), new_admin);
}

// ---------------------------------------------------------------------------
// Proxy tests
// ---------------------------------------------------------------------------

/// Test 7: Proxy initialises and delegates arithmetic to the impl via Beacon.
#[test]
fn test_proxy_init_and_delegates() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let proxy = ProxyContractClient::new(&env, &c.proxy);
    let admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    proxy.init(&admin, &c.beacon);

    // Proxy should resolve the implementation through the Beacon.
    assert_eq!(proxy.get_implementation(), c.impl_v1);

    // Delegated arithmetic.
    assert_eq!(proxy.add(&10i128, &5i128), 15i128);
    assert_eq!(proxy.sub(&10i128, &3i128), 7i128);
}

/// Test 8: Upgrading the Beacon immediately propagates to the Proxy.
///
/// This is the core property of the Beacon pattern: a single `upgrade` call
/// on the Beacon updates the behaviour of every proxy that references it.
#[test]
fn test_upgrade_propagates_to_proxy() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let proxy = ProxyContractClient::new(&env, &c.proxy);
    let admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    proxy.init(&admin, &c.beacon);

    // Before upgrade: impl is v1.
    assert_eq!(proxy.get_implementation(), c.impl_v1);

    // Upgrade Beacon to v2.
    beacon.upgrade(&c.impl_v2, &symbol_short!("v2"));

    // After upgrade: proxy automatically sees v2 without any proxy-level change.
    assert_eq!(proxy.get_implementation(), c.impl_v2);
    assert_eq!(beacon.get_version(), 2u32);
}

/// Test 9: `mul` (v2-only) is accessible through the proxy after the Beacon is upgraded.
#[test]
fn test_mul_available_after_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let proxy = ProxyContractClient::new(&env, &c.proxy);
    let admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    proxy.init(&admin, &c.beacon);

    // Upgrade to v2 which exposes `mul`.
    beacon.upgrade(&c.impl_v2, &symbol_short!("v2"));

    // Now `mul` should be available through the proxy.
    assert_eq!(proxy.mul(&6i128, &7i128), 42i128);
}

/// Test 10: Implementation state (counter) persists across a Beacon upgrade.
///
/// The state lives inside the implementation contract's own storage, not in
/// the proxy or beacon. Upgrading the Beacon to a new implementation address
/// means *starting with a fresh state* in the new contract — which is the
/// expected behaviour (similar to Ethereum's UUPS/Transparent proxy storage
/// separation via storage slots).
///
/// This test specifically verifies that the *new* v2 implementation starts at
/// 0 after the upgrade and can be independently incremented.
#[test]
fn test_counter_persists_across_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let proxy = ProxyContractClient::new(&env, &c.proxy);
    let impl_v1 = ImplV1Client::new(&env, &c.impl_v1);
    let impl_v2 = ImplV2Client::new(&env, &c.impl_v2);
    let admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    proxy.init(&admin, &c.beacon);

    // Increment counter in v1 three times through the proxy.
    proxy.increment();
    proxy.increment();
    proxy.increment();
    assert_eq!(proxy.get_counter(), 3u32);

    // v1's own storage should also reflect 3.
    assert_eq!(impl_v1.get_count(), 3u32);

    // Upgrade to v2.
    beacon.upgrade(&c.impl_v2, &symbol_short!("v2"));

    // v2 starts with its own independent counter at 0.
    assert_eq!(impl_v2.get_count(), 0u32);

    // Incrementing through the proxy now targets v2.
    proxy.increment();
    assert_eq!(proxy.get_counter(), 1u32);
    assert_eq!(impl_v2.get_count(), 1u32);

    // v1 counter is untouched.
    assert_eq!(impl_v1.get_count(), 3u32);
}

/// Test 11: Proxy can be re-pointed to a new Beacon (canary deployment pattern).
#[test]
fn test_proxy_set_beacon() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let admin = Address::generate(&env);

    // Two beacons: beacon_a → v1, beacon_b → v2.
    let beacon_b_addr = env.register(BeaconContract, ());
    let beacon_a = BeaconContractClient::new(&env, &c.beacon);
    let beacon_b = BeaconContractClient::new(&env, &beacon_b_addr);

    beacon_a.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    beacon_b.init(&admin, &c.impl_v2, &symbol_short!("v2"));

    // Proxy starts on beacon_a.
    let proxy = ProxyContractClient::new(&env, &c.proxy);
    proxy.init(&admin, &c.beacon);
    assert_eq!(proxy.get_implementation(), c.impl_v1);

    // Re-point the proxy to beacon_b.
    proxy.set_beacon(&beacon_b_addr);
    assert_eq!(proxy.get_beacon(), beacon_b_addr);
    assert_eq!(proxy.get_implementation(), c.impl_v2);
}

/// Test 12: Multiple proxies sharing one Beacon — a single upgrade updates all.
///
/// This is the primary advantage of the Beacon pattern over per-proxy upgrades.
#[test]
fn test_multiple_proxies_one_beacon() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let admin = Address::generate(&env);

    let beacon = BeaconContractClient::new(&env, &c.beacon);
    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));

    // Register two independent proxies, both bound to the same Beacon.
    let proxy_a_addr = env.register(ProxyContract, ());
    let proxy_b_addr = env.register(ProxyContract, ());
    let proxy_a = ProxyContractClient::new(&env, &proxy_a_addr);
    let proxy_b = ProxyContractClient::new(&env, &proxy_b_addr);

    proxy_a.init(&admin, &c.beacon);
    proxy_b.init(&admin, &c.beacon);

    // Both proxies see v1 before upgrade.
    assert_eq!(proxy_a.get_implementation(), c.impl_v1);
    assert_eq!(proxy_b.get_implementation(), c.impl_v1);

    // One Beacon upgrade propagates to *both* proxies simultaneously.
    beacon.upgrade(&c.impl_v2, &symbol_short!("v2"));

    assert_eq!(proxy_a.get_implementation(), c.impl_v2);
    assert_eq!(proxy_b.get_implementation(), c.impl_v2);

    // Both can now use mul (v2 feature).
    assert_eq!(proxy_a.mul(&3i128, &4i128), 12i128);
    assert_eq!(proxy_b.mul(&5i128, &5i128), 25i128);
}

/// Test 13: `get_version_entry` returns correct historical records.
#[test]
fn test_beacon_get_version_entry() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));
    beacon.upgrade(&c.impl_v2, &symbol_short!("v2"));

    // Version 1 should still point to impl_v1.
    let v1_entry = beacon.get_version_entry(&1u32);
    assert_eq!(v1_entry.implementation, c.impl_v1);
    assert_eq!(v1_entry.label, symbol_short!("v1"));

    // Version 2 should point to impl_v2.
    let v2_entry = beacon.get_version_entry(&2u32);
    assert_eq!(v2_entry.implementation, c.impl_v2);
    assert_eq!(v2_entry.label, symbol_short!("v2"));
}

/// Test 14: Querying a version that has never been registered panics.
#[test]
#[should_panic(expected = "Version not found")]
fn test_beacon_version_entry_not_found_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let c = register_all(&env);
    let beacon = BeaconContractClient::new(&env, &c.beacon);
    let admin = Address::generate(&env);

    beacon.init(&admin, &c.impl_v1, &symbol_short!("v1"));

    // Version 99 was never registered — must panic.
    beacon.get_version_entry(&99u32);
}

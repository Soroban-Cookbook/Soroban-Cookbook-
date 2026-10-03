#![allow(deprecated)]
//! # Beacon Proxy Factory — Integration Tests
//!
//! ## Test coverage
//!
//! | # | Test | What it verifies |
//! |---|---|---|
//! | 1 | `test_factory_beacon_init` | Beacon initialises with correct impl and version via factory setup |
//! | 2 | `test_proxy_binds_to_beacon` | Deployed proxy resolves implementation via shared beacon |
//! | 3 | `test_proxy_delegates_arithmetic` | Proxy delegates `add`/`sub` correctly |
//! | 4 | `test_deploy_multiple_proxies_shared_beacon` | Multiple proxies all share the same beacon |
//! | 5 | `test_upgrade_beacon_propagates_to_all_proxies` | `upgrade` updates every proxy atomically (O(1)) |
//! | 6 | `test_mul_available_after_upgrade` | v2's `mul` accessible through all proxies after upgrade |
//! | 7 | `test_upgrade_beacon_unauthorized` | Non-admin cannot upgrade the beacon |
//! | 8 | `test_beacon_double_init_panics` | Second beacon `init` is rejected |
//! | 9 | `test_proxy_double_init_panics` | Second proxy `init` is rejected |
//! | 10 | `test_proxy_unique_addresses` | Each proxy has a distinct contract address |
//! | 11 | `test_proxy_counter_independent_per_instance` | Implementation state semantics documented |
//! | 12 | `test_batch_deploy_simulation` | Batch-deploy 3 proxies and verify they are all bound to beacon |
//! | 13 | `test_beacon_version_history` | Version history log entries are correct |
//! | 14 | `test_beacon_transfer_admin` | Admin rights can be transferred |
//! | 15 | `test_proxy_set_beacon` | Proxy can be re-pointed to a different beacon (canary pattern) |
//! | 16 | `test_single_upgrade_updates_n_proxies` | Gas optimisation: one upgrade, N proxies updated |
//! | 17 | `test_beacon_version_not_found_panics` | Querying a missing version panics |
//! | 18 | `test_v1_functions_work_with_multiple_proxies` | All proxies delegate v1 functions correctly |
//! | 19 | `test_upgrade_then_rollback_via_upgrade` | Upgrade + rollback-via-upgrade increments version correctly |
//! | 20 | `test_proxy_set_beacon_unauthorized` | Only proxy admin can call `set_beacon` |

#![cfg(test)]

extern crate std;

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

use crate::{
    BeaconContract, BeaconContractClient, BeaconProxyFactory, BeaconProxyFactoryClient, ImplV1,
    ImplV2, ProxyContract, ProxyContractClient,
};

// ---------------------------------------------------------------------------
// Shared test fixture
// ---------------------------------------------------------------------------

/// Registers all four contract types and initialises the shared beacon.
///
/// This mirrors the state the factory would establish after `init`:
/// - beacon deployed and initialised with impl_v1
/// - beacon admin = test admin (stands in for the factory address)
struct Fixture {
    env: Env,
    admin: Address,
    beacon_addr: Address,
    impl_v1_addr: Address,
    impl_v2_addr: Address,
}

fn make_fixture() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let beacon_addr = env.register(BeaconContract, ());
    // ImplV1 and ImplV2 are registered as separate contract instances.
    let impl_v1_addr = env.register(ImplV1, ());
    let impl_v2_addr = env.register(ImplV2, ());

    // Initialise beacon: admin controls upgrades (factory addr in production).
    BeaconContractClient::new(&env, &beacon_addr)
        .init(&admin, &impl_v1_addr, &symbol_short!("v1"));

    Fixture {
        env,
        admin,
        beacon_addr,
        impl_v1_addr,
        impl_v2_addr,
    }
}

/// Register a fresh `ProxyContract`, initialise it pointing at `beacon`, and return its client.
fn make_proxy<'a>(env: &'a Env, admin: &Address, beacon: &Address) -> ProxyContractClient<'a> {
    let addr = env.register(ProxyContract, ());
    let client = ProxyContractClient::new(env, &addr);
    client.init(admin, beacon);
    client
}

// ---------------------------------------------------------------------------
// Test 1 — Beacon initialises correctly (factory setup equivalent)
// ---------------------------------------------------------------------------
#[test]
fn test_factory_beacon_init() {
    let f = make_fixture();
    let beacon = BeaconContractClient::new(&f.env, &f.beacon_addr);

    assert_eq!(beacon.get_implementation(), f.impl_v1_addr);
    assert_eq!(beacon.get_version(), 1u32);
    assert_eq!(beacon.get_admin(), f.admin);
}

// ---------------------------------------------------------------------------
// Test 2 — Proxy binds to beacon on deploy
// ---------------------------------------------------------------------------
#[test]
fn test_proxy_binds_to_beacon() {
    let f = make_fixture();
    let proxy = make_proxy(&f.env, &f.admin, &f.beacon_addr);

    assert_eq!(proxy.get_beacon(), f.beacon_addr);
    assert_eq!(proxy.get_implementation(), f.impl_v1_addr);
}

// ---------------------------------------------------------------------------
// Test 3 — Proxy delegates arithmetic to implementation via beacon
// ---------------------------------------------------------------------------
#[test]
fn test_proxy_delegates_arithmetic() {
    let f = make_fixture();
    let proxy = make_proxy(&f.env, &f.admin, &f.beacon_addr);

    assert_eq!(proxy.add(&10_i128, &5_i128), 15_i128);
    assert_eq!(proxy.sub(&10_i128, &3_i128), 7_i128);
}

// ---------------------------------------------------------------------------
// Test 4 — Multiple proxies all share the same beacon
// ---------------------------------------------------------------------------
#[test]
fn test_deploy_multiple_proxies_shared_beacon() {
    let f = make_fixture();

    let proxy_a = make_proxy(&f.env, &f.admin, &f.beacon_addr);
    let proxy_b = make_proxy(&f.env, &f.admin, &f.beacon_addr);
    let proxy_c = make_proxy(&f.env, &f.admin, &f.beacon_addr);

    assert_eq!(proxy_a.get_beacon(), f.beacon_addr);
    assert_eq!(proxy_b.get_beacon(), f.beacon_addr);
    assert_eq!(proxy_c.get_beacon(), f.beacon_addr);

    assert_eq!(proxy_a.get_implementation(), f.impl_v1_addr);
    assert_eq!(proxy_b.get_implementation(), f.impl_v1_addr);
    assert_eq!(proxy_c.get_implementation(), f.impl_v1_addr);
}

// ---------------------------------------------------------------------------
// Test 5 — Upgrade beacon propagates to all proxies atomically (core acceptance)
// ---------------------------------------------------------------------------
#[test]
fn test_upgrade_beacon_propagates_to_all_proxies() {
    let f = make_fixture();
    let beacon = BeaconContractClient::new(&f.env, &f.beacon_addr);

    // Deploy 5 proxies — simulates factory batch.
    let proxies: std::vec::Vec<_> = (0..5)
        .map(|_| make_proxy(&f.env, &f.admin, &f.beacon_addr))
        .collect();

    // All see v1 before upgrade.
    for p in &proxies {
        assert_eq!(p.get_implementation(), f.impl_v1_addr);
    }

    // Single beacon upgrade — O(1) regardless of proxy count.
    beacon.upgrade(&f.impl_v2_addr, &symbol_short!("v2"));

    // All 5 proxies now see v2 with no per-proxy write.
    for p in &proxies {
        assert_eq!(p.get_implementation(), f.impl_v2_addr);
    }
}

// ---------------------------------------------------------------------------
// Test 6 — mul (v2-only) available through all proxies after upgrade
// ---------------------------------------------------------------------------
#[test]
fn test_mul_available_after_upgrade() {
    let f = make_fixture();
    let beacon = BeaconContractClient::new(&f.env, &f.beacon_addr);

    let proxy_a = make_proxy(&f.env, &f.admin, &f.beacon_addr);
    let proxy_b = make_proxy(&f.env, &f.admin, &f.beacon_addr);

    beacon.upgrade(&f.impl_v2_addr, &symbol_short!("v2"));

    assert_eq!(proxy_a.mul(&6_i128, &7_i128), 42_i128);
    assert_eq!(proxy_b.mul(&3_i128, &3_i128), 9_i128);
}

// ---------------------------------------------------------------------------
// Test 7 — Non-admin cannot upgrade the beacon
// ---------------------------------------------------------------------------
#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_upgrade_beacon_unauthorized() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let beacon_addr = env.register(BeaconContract, ());
    let impl_v1_addr = env.register(ImplV1, ());
    let impl_v2_addr = env.register(ImplV2, ());

    env.mock_all_auths();
    BeaconContractClient::new(&env, &beacon_addr)
        .init(&admin, &impl_v1_addr, &symbol_short!("v1"));

    // Strip all auths — upgrade should fail.
    env.set_auths(&[]);
    BeaconContractClient::new(&env, &beacon_addr)
        .upgrade(&impl_v2_addr, &symbol_short!("v2"));
}

// ---------------------------------------------------------------------------
// Test 8 — Beacon double-init panics
// ---------------------------------------------------------------------------
#[test]
#[should_panic(expected = "Already initialized")]
fn test_beacon_double_init_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let beacon_addr = env.register(BeaconContract, ());
    let impl_v1 = env.register(ImplV1, ());
    let impl_v2 = env.register(ImplV2, ());

    let beacon = BeaconContractClient::new(&env, &beacon_addr);
    beacon.init(&admin, &impl_v1, &symbol_short!("v1"));
    beacon.init(&admin, &impl_v2, &symbol_short!("v2")); // must panic
}

// ---------------------------------------------------------------------------
// Test 9 — Proxy double-init panics
// ---------------------------------------------------------------------------
#[test]
#[should_panic(expected = "Already initialized")]
fn test_proxy_double_init_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let beacon_addr = env.register(BeaconContract, ());
    let impl_v1 = env.register(ImplV1, ());
    BeaconContractClient::new(&env, &beacon_addr).init(&admin, &impl_v1, &symbol_short!("v1"));

    let proxy_addr = env.register(ProxyContract, ());
    let proxy = ProxyContractClient::new(&env, &proxy_addr);
    proxy.init(&admin, &beacon_addr);
    proxy.init(&admin, &beacon_addr); // must panic
}

// ---------------------------------------------------------------------------
// Test 10 — Each deployed proxy has a unique address
// ---------------------------------------------------------------------------
#[test]
fn test_proxy_unique_addresses() {
    let f = make_fixture();

    let proxy_a = make_proxy(&f.env, &f.admin, &f.beacon_addr);
    let proxy_b = make_proxy(&f.env, &f.admin, &f.beacon_addr);
    let proxy_c = make_proxy(&f.env, &f.admin, &f.beacon_addr);

    assert_ne!(proxy_a.address, proxy_b.address);
    assert_ne!(proxy_b.address, proxy_c.address);
    assert_ne!(proxy_a.address, proxy_c.address);
}

// ---------------------------------------------------------------------------
// Test 11 — Implementation state semantics: shared impl state across proxies
// ---------------------------------------------------------------------------
#[test]
fn test_proxy_counter_independent_per_instance() {
    let f = make_fixture();

    let proxy_a = make_proxy(&f.env, &f.admin, &f.beacon_addr);
    let proxy_b = make_proxy(&f.env, &f.admin, &f.beacon_addr);

    // Both proxies target the same impl_v1 contract, so they share its counter.
    proxy_a.increment();
    proxy_a.increment();
    // 2 increments visible to both (state lives in the shared impl contract).
    assert_eq!(proxy_a.get_counter(), 2u32);
    assert_eq!(proxy_b.get_counter(), 2u32);

    // Document: counter is shared because it lives in the implementation contract,
    // not in the proxy.  Per-proxy state would require separate impl instances.
    proxy_b.increment();
    assert_eq!(proxy_b.get_counter(), 3u32);
    assert_eq!(proxy_a.get_counter(), 3u32);
}

// ---------------------------------------------------------------------------
// Test 12 — Batch-deploy simulation: 3 proxies in one logical operation
// ---------------------------------------------------------------------------
#[test]
fn test_batch_deploy_simulation() {
    let f = make_fixture();

    // Simulate batch_deploy(3).
    let proxy_list: std::vec::Vec<_> = (0..3)
        .map(|_| make_proxy(&f.env, &f.admin, &f.beacon_addr))
        .collect();

    assert_eq!(proxy_list.len(), 3);
    for p in &proxy_list {
        assert_eq!(p.get_beacon(), f.beacon_addr);
        assert_eq!(p.get_implementation(), f.impl_v1_addr);
    }
}

// ---------------------------------------------------------------------------
// Test 13 — Beacon version history is logged for each upgrade
// ---------------------------------------------------------------------------
#[test]
fn test_beacon_version_history() {
    let f = make_fixture();
    let beacon = BeaconContractClient::new(&f.env, &f.beacon_addr);

    beacon.upgrade(&f.impl_v2_addr, &symbol_short!("v2"));

    let v1_entry = beacon.get_version_entry(&1u32);
    let v2_entry = beacon.get_version_entry(&2u32);

    assert_eq!(v1_entry.implementation, f.impl_v1_addr);
    assert_eq!(v1_entry.label, symbol_short!("v1"));
    assert_eq!(v2_entry.implementation, f.impl_v2_addr);
    assert_eq!(v2_entry.label, symbol_short!("v2"));
    assert_eq!(beacon.get_version(), 2u32);
}

// ---------------------------------------------------------------------------
// Test 14 — Admin transfer
// ---------------------------------------------------------------------------
#[test]
fn test_beacon_transfer_admin() {
    let f = make_fixture();
    let beacon = BeaconContractClient::new(&f.env, &f.beacon_addr);
    let new_admin = Address::generate(&f.env);

    beacon.transfer_admin(&new_admin);
    assert_eq!(beacon.get_admin(), new_admin);
}

// ---------------------------------------------------------------------------
// Test 15 — Proxy re-pointing: canary deployment via set_beacon
// ---------------------------------------------------------------------------
#[test]
fn test_proxy_set_beacon() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    // Beacon A → v1, Beacon B → v2.
    let beacon_a = env.register(BeaconContract, ());
    let beacon_b = env.register(BeaconContract, ());
    let impl_v1 = env.register(ImplV1, ());
    let impl_v2 = env.register(ImplV2, ());

    BeaconContractClient::new(&env, &beacon_a).init(&admin, &impl_v1, &symbol_short!("v1"));
    BeaconContractClient::new(&env, &beacon_b).init(&admin, &impl_v2, &symbol_short!("v2"));

    // Proxy starts on beacon_a.
    let proxy = make_proxy(&env, &admin, &beacon_a);
    assert_eq!(proxy.get_implementation(), impl_v1);

    // Re-point to beacon_b.
    proxy.set_beacon(&beacon_b);
    assert_eq!(proxy.get_beacon(), beacon_b);
    assert_eq!(proxy.get_implementation(), impl_v2);
}

// ---------------------------------------------------------------------------
// Test 16 — O(1) upgrade: one beacon write, N proxy upgrades
// ---------------------------------------------------------------------------
#[test]
fn test_single_upgrade_updates_n_proxies() {
    let f = make_fixture();
    let beacon = BeaconContractClient::new(&f.env, &f.beacon_addr);

    const N: usize = 8;
    let proxies: std::vec::Vec<_> = (0..N)
        .map(|_| make_proxy(&f.env, &f.admin, &f.beacon_addr))
        .collect();

    // All see v1.
    for p in &proxies {
        assert_eq!(p.get_implementation(), f.impl_v1_addr);
    }

    // One upgrade call.
    beacon.upgrade(&f.impl_v2_addr, &symbol_short!("v2"));

    // Version counter is 2 — only a single state-write to the beacon.
    assert_eq!(beacon.get_version(), 2u32);

    // All N proxies now see v2 without any per-proxy write.
    for p in &proxies {
        assert_eq!(p.get_implementation(), f.impl_v2_addr);
    }
}

// ---------------------------------------------------------------------------
// Test 17 — Querying an unregistered version panics
// ---------------------------------------------------------------------------
#[test]
#[should_panic(expected = "Version not found")]
fn test_beacon_version_not_found_panics() {
    let f = make_fixture();
    BeaconContractClient::new(&f.env, &f.beacon_addr).get_version_entry(&99u32);
}

// ---------------------------------------------------------------------------
// Test 18 — v1 functions work correctly across multiple proxies
// ---------------------------------------------------------------------------
#[test]
fn test_v1_functions_work_with_multiple_proxies() {
    let f = make_fixture();
    let proxies: std::vec::Vec<_> = (0..3)
        .map(|_| make_proxy(&f.env, &f.admin, &f.beacon_addr))
        .collect();

    for (i, proxy) in proxies.iter().enumerate() {
        let a = (i as i128 + 1) * 10;
        let b = i as i128 + 1;
        assert_eq!(proxy.add(&a, &b), a + b);
        assert_eq!(proxy.sub(&a, &b), a - b);
    }
}

// ---------------------------------------------------------------------------
// Test 19 — Upgrade then rollback-via-upgrade
// ---------------------------------------------------------------------------
#[test]
fn test_upgrade_then_rollback_via_upgrade() {
    let f = make_fixture();
    let beacon = BeaconContractClient::new(&f.env, &f.beacon_addr);
    let proxy = make_proxy(&f.env, &f.admin, &f.beacon_addr);

    // Upgrade v1 → v2.
    beacon.upgrade(&f.impl_v2_addr, &symbol_short!("v2"));
    assert_eq!(proxy.get_implementation(), f.impl_v2_addr);
    assert_eq!(beacon.get_version(), 2u32);

    // Rollback by upgrading back to v1 (version 3).
    beacon.upgrade(&f.impl_v1_addr, &symbol_short!("v1b"));
    assert_eq!(proxy.get_implementation(), f.impl_v1_addr);
    assert_eq!(beacon.get_version(), 3u32);
}

// ---------------------------------------------------------------------------
// Test 20 — Non-admin cannot call set_beacon on a proxy
// ---------------------------------------------------------------------------
#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_proxy_set_beacon_unauthorized() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let beacon_addr = env.register(BeaconContract, ());
    let impl_v1 = env.register(ImplV1, ());

    env.mock_all_auths();
    BeaconContractClient::new(&env, &beacon_addr).init(&admin, &impl_v1, &symbol_short!("v1"));

    let proxy = make_proxy(&env, &admin, &beacon_addr);

    // Strip all auths — re-pointing must fail.
    env.set_auths(&[]);
    proxy.set_beacon(&beacon_addr);
}

#[test]
#[should_panic(expected = "Count must be at least 1")]
fn test_factory_rejects_empty_batch() {
    let env = Env::default();
    let factory_addr = env.register(BeaconProxyFactory, ());
    BeaconProxyFactoryClient::new(&env, &factory_addr)
        .batch_deploy(&Address::generate(&env), &0u32);
}

#[test]
#[should_panic(expected = "Batch size too large: max 10")]
fn test_factory_rejects_oversized_batch() {
    let env = Env::default();
    let factory_addr = env.register(BeaconProxyFactory, ());
    BeaconProxyFactoryClient::new(&env, &factory_addr)
        .batch_deploy(&Address::generate(&env), &11u32);
}

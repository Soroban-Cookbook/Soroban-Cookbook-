#![cfg(test)]

use crate::{BeaconManagementContract, BeaconManagementContractClient};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, Symbol};

fn setup() -> (Env, BeaconManagementContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(BeaconManagementContract, ());
    let client = BeaconManagementContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    (env, client, admin)
}

#[test]
fn registers_and_tracks_versions_for_a_beacon() {
    let (env, client, admin) = setup();
    let beacon_name: Symbol = symbol_short!("alpha");
    let implementation_v1 = Address::generate(&env);

    let version = client.register_beacon(&admin, &beacon_name, &implementation_v1);

    assert_eq!(version, 1);

    let beacon = client.get_beacon(&beacon_name);
    assert_eq!(beacon.current_version, 1);
    assert_eq!(beacon.current_implementation, implementation_v1);
    assert_eq!(beacon.history.len(), 1);
}

#[test]
fn upgrades_and_rolls_back_a_beacon_independently() {
    let (env, client, admin) = setup();
    let beacon_name: Symbol = symbol_short!("beta");
    let implementation_v1 = Address::generate(&env);
    let implementation_v2 = Address::generate(&env);

    client.register_beacon(&admin, &beacon_name, &implementation_v1);
    let upgraded_version = client.upgrade_beacon(&admin, &beacon_name, &implementation_v2);
    assert_eq!(upgraded_version, 2);

    let beacon = client.get_beacon(&beacon_name);
    assert_eq!(beacon.current_version, 2);
    assert_eq!(beacon.current_implementation, implementation_v2);

    let rolled_back_version = client.rollback_beacon(&admin, &beacon_name);
    assert_eq!(rolled_back_version, 1);

    let beacon = client.get_beacon(&beacon_name);
    assert_eq!(beacon.current_version, 1);
    assert_eq!(beacon.current_implementation, implementation_v1);
}

#[test]
fn supports_multiple_beacons_with_independent_histories() {
    let (env, client, admin) = setup();
    let alpha = symbol_short!("alpha");
    let beta = symbol_short!("beta");
    let alpha_v1 = Address::generate(&env);
    let alpha_v2 = Address::generate(&env);
    let beta_v1 = Address::generate(&env);

    client.register_beacon(&admin, &alpha, &alpha_v1);
    client.register_beacon(&admin, &beta, &beta_v1);
    client.upgrade_beacon(&admin, &alpha, &alpha_v2);

    let alpha_beacon = client.get_beacon(&alpha);
    let beta_beacon = client.get_beacon(&beta);

    assert_eq!(alpha_beacon.current_version, 2);
    assert_eq!(alpha_beacon.current_implementation, alpha_v2);
    assert_eq!(beta_beacon.current_version, 1);
    assert_eq!(beta_beacon.current_implementation, beta_v1);

    let names = client.list_beacons();
    assert_eq!(names.len(), 2);
}

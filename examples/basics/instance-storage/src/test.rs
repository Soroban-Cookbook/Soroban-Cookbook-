//! Tests for the instance-storage contract.
//!
//! Covers generic key/value helpers and two concrete use cases: a transaction
//! counter and runtime configuration.

use super::*;
use soroban_sdk::{symbol_short, Env};

#[test]
fn test_set_and_get_instance() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    let key = symbol_short!("mykey");
    assert_eq!(client.get_instance(&key), None, "Should be None before set");

    client.set_instance(&key, &42);
    assert_eq!(client.get_instance(&key), Some(42));
}

#[test]
fn test_set_instance_overwrites() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    let key = symbol_short!("val");
    client.set_instance(&key, &1);
    client.set_instance(&key, &99);
    assert_eq!(client.get_instance(&key), Some(99));
}

#[test]
fn test_different_keys_independent() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    let k1 = symbol_short!("k1");
    let k2 = symbol_short!("k2");

    client.set_instance(&k1, &10);
    client.set_instance(&k2, &20);

    assert_eq!(client.get_instance(&k1), Some(10));
    assert_eq!(client.get_instance(&k2), Some(20));
}

#[test]
fn test_counter_starts_at_zero() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    assert_eq!(client.get_counter(), 0, "Counter must default to 0");
}

#[test]
fn test_counter_increments() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    assert_eq!(client.increment_counter(), 1);
    assert_eq!(client.increment_counter(), 2);
    assert_eq!(client.increment_counter(), 3);
    assert_eq!(client.get_counter(), 3);
}

#[test]
fn test_counter_persists_across_calls() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    client.increment_counter();
    client.increment_counter();

    let client2 = InstanceStorageContractClient::new(&env, &id);
    assert_eq!(client2.get_counter(), 2);
}

#[test]
fn test_config_unset_returns_none() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    let key = symbol_short!("fee_bps");
    assert_eq!(client.get_config(&key), None);
}

#[test]
fn test_config_set_and_get() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    let fee_key = symbol_short!("fee_bps");
    let limit_key = symbol_short!("max_amt");

    client.set_config(&fee_key, &30);
    client.set_config(&limit_key, &1_000_000);

    assert_eq!(client.get_config(&fee_key), Some(30));
    assert_eq!(client.get_config(&limit_key), Some(1_000_000));
}

#[test]
fn test_config_update() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    let key = symbol_short!("fee_bps");
    client.set_config(&key, &30);
    client.set_config(&key, &50);
    assert_eq!(client.get_config(&key), Some(50));
}

#[test]
fn test_extend_ttl_does_not_corrupt_data() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    client.increment_counter();
    client.set_config(&symbol_short!("fee_bps"), &25);

    client.extend_ttl();

    assert_eq!(client.get_counter(), 1);
    assert_eq!(client.get_config(&symbol_short!("fee_bps")), Some(25));
}

#[test]
fn test_counter_and_config_coexist() {
    let env = Env::default();
    let id = env.register_contract(None, InstanceStorageContract);
    let client = InstanceStorageContractClient::new(&env, &id);

    client.increment_counter();
    client.set_config(&symbol_short!("rate"), &5);
    client.increment_counter();
    client.set_config(&symbol_short!("rate"), &10);
    client.increment_counter();

    assert_eq!(client.get_counter(), 3);
    assert_eq!(client.get_config(&symbol_short!("rate")), Some(10));
}

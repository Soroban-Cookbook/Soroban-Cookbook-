//! # Tests for Compressed Storage Example
//!
//! These tests verify that raw bytes and compressed bytes are both stored
//! correctly, and that decompression returns the original payload.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

#[test]
fn test_compression_round_trip() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CompressedStorageContract);
    let client = CompressedStorageContractClient::new(&env, &contract_id);

    let key = Address::generate(&env);
    let data = Bytes::from_slice(&env, b"aaaaabbbbcccccaaaaa");

    let raw_len = client.store_raw(&key, &data);
    let compressed_len = client.store_compressed(&key, &data);

    assert_eq!(raw_len, data.len());
    assert!(compressed_len < raw_len, "compression should reduce size for repeated bytes");

    assert_eq!(client.get_raw(&key), Some(data.clone()));
    assert_eq!(client.get_decompressed(&key), Some(data.clone()));
    assert_eq!(client.compare_stored_sizes(&key), (raw_len, compressed_len));
}

#[test]
fn test_compression_non_repeating_data() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CompressedStorageContract);
    let client = CompressedStorageContractClient::new(&env, &contract_id);

    let key = Address::generate(&env);
    let data = Bytes::from_slice(&env, b"abcdef0123456789");

    let raw_len = client.store_raw(&key, &data);
    let compressed_len = client.store_compressed(&key, &data);

    assert_eq!(raw_len, data.len());
    assert!(compressed_len >= raw_len, "small unrelated bytes may not benefit from RLE compression");
    assert_eq!(client.compare_stored_sizes(&key), (raw_len, compressed_len));
}

#[test]
fn test_decompressed_size_header() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CompressedStorageContract);
    let client = CompressedStorageContractClient::new(&env, &contract_id);

    let key = Address::generate(&env);
    let data = Bytes::from_slice(&env, b"zzzzzzzzzzzzzzzz");

    client.store_compressed(&key, data.clone());
    let decompressed = client.get_decompressed(&key).unwrap();

    assert_eq!(decompressed, data);
}

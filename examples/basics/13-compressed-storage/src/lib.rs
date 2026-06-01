#![no_std]
//! # Compressed Storage Example
//!
//! This contract demonstrates how to compress byte payloads before storing them
//! in Soroban persistent storage. It contrasts raw storage vs compressed storage
//! and shows why compression is beneficial only for certain data patterns.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env};

/// Typed storage keys for raw and compressed payloads.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    RawData(Address),
    CompressedData(Address),
}

/// Contract that stores raw and compressed payloads for comparison.
#[contract]
pub struct CompressedStorageContract;

#[contractimpl]
impl CompressedStorageContract {
    /// Store raw bytes in persistent storage.
    pub fn store_raw(env: Env, key: Address, data: Bytes) -> u32 {
        env.storage()
            .persistent()
            .set(&DataKey::RawData(key), &data);
        data.len()
    }

    /// Compress bytes and store the compressed payload in persistent storage.
    ///
    /// This example uses a simple run-length encoding (RLE) algorithm that is
    /// easy to implement in contract code and demonstrates the trade-offs of
    /// compression on-chain.
    pub fn store_compressed(env: Env, key: Address, data: Bytes) -> u32 {
        let compressed = Self::compress_bytes(&env, data);
        env.storage()
            .persistent()
            .set(&DataKey::CompressedData(key), &compressed);
        compressed.len()
    }

    /// Read raw bytes back from storage.
    pub fn get_raw(env: Env, key: Address) -> Option<Bytes> {
        env.storage().persistent().get(&DataKey::RawData(key))
    }

    /// Read compressed bytes from storage and decompress them before returning.
    pub fn get_decompressed(env: Env, key: Address) -> Option<Bytes> {
        env.storage()
            .persistent()
            .get(&DataKey::CompressedData(key))
            .map(|compressed: Bytes| Self::decompress_bytes(&env, compressed))
    }

    /// Compare stored payload sizes for raw vs compressed values.
    ///
    /// Returns `(raw_bytes_len, compressed_bytes_len)`.
    pub fn compare_stored_sizes(env: Env, key: Address) -> (u32, u32) {
        let raw_len = env
            .storage()
            .persistent()
            .get(&DataKey::RawData(key.clone()))
            .map(|raw: Bytes| raw.len())
            .unwrap_or(0);

        let compressed_len = env
            .storage()
            .persistent()
            .get(&DataKey::CompressedData(key))
            .map(|compressed: Bytes| compressed.len())
            .unwrap_or(0);

        (raw_len, compressed_len)
    }

    /// Helper: compress bytes using run-length encoding.
    fn compress_bytes(env: &Env, data: Bytes) -> Bytes {
        let mut out = Bytes::new(env);
        let original_len = (data.len() as u32).to_be_bytes();
        out.extend_from_slice(&original_len);

        let mut i = 0;
        while i < data.len() {
            let current = data.get(i).unwrap();
            let mut run_length: u8 = 1;

            while run_length < 255 && i + (run_length as usize) < data.len() {
                let next = data.get(i + run_length as usize).unwrap();
                if next != current {
                    break;
                }
                run_length += 1;
            }

            out.push_back(current);
            out.push_back(run_length);
            i += run_length as usize;
        }

        out
    }

    /// Helper: decompress run-length encoded bytes.
    fn decompress_bytes(env: &Env, compressed: Bytes) -> Bytes {
        let original_len = Self::read_u32(&compressed, 0) as usize;
        let mut out = Bytes::new(env);
        let mut i = 4;

        while i + 1 < compressed.len() {
            let value = compressed.get(i).unwrap();
            let run_length = compressed.get(i + 1).unwrap();
            let count = *run_length as usize;

            for _ in 0..count {
                out.push_back(*value);
            }
            i += 2;
        }

        // If the encoded payload is truncated, we still return what we can.
        assert!(out.len() == original_len, "decompressed length mismatch");
        out
    }

    fn read_u32(bytes: &Bytes, index: usize) -> u32 {
        let mut result = 0u32;
        for offset in 0..4 {
            result = (result << 8) | (*bytes.get(index + offset).unwrap() as u32);
        }
        result
    }
}

#[cfg(test)]
mod test;

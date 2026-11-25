//! # Off-chain Signature Verification
//!
//! A trusted signer approves an action off-chain ("credit `amount` to `to`")
//! and anyone can submit that approval on-chain. The contract checks the
//! signature itself instead of relying on `require_auth`, which is how you
//! accept signatures from keys that are not Stellar accounts: an ed25519 key
//! held by a backend or hardware wallet, or an Ethereum (secp256k1) key.
//!
//! Two schemes, one message format:
//!
//! | Scheme | Digest | Check |
//! |---|---|---|
//! | ed25519 | `sha256(payload)` | `env.crypto().ed25519_verify(pubkey, digest, sig)` (traps on failure) |
//! | secp256k1 (Ethereum) | `keccak256(payload)` | `env.crypto().secp256k1_recover(digest, sig, recovery_id)`, then compare the derived Ethereum address |
//!
//! The SDK has no `secp256k1_verify`: for secp256k1 you *recover* the public
//! key that produced the signature and compare it (or its Ethereum address)
//! with the one you trust. Recovery always yields *some* key, so the
//! comparison is the verification step. Never skip it.
//!
//! ## Replay protection
//!
//! `payload = domain_separator || scheme || to || amount || nonce` where
//!
//! * **domain separator** = `sha256("soroban-cookbook/off-chain-signatures/v1"
//!   || network_id || contract_address)`. A signature is only valid for this
//!   contract on this network, so it cannot be replayed on another deployment,
//!   on testnet vs. mainnet, or by another contract that trusts the same key.
//! * **scheme** byte: a signature produced for one verification path can
//!   never be fed to the other.
//! * **nonce**: each signer has a sequential nonce. A claim must use exactly
//!   the next nonce, which is then consumed, so every approval can be used
//!   once and only in order.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, xdr::ToXdr, Address, Bytes,
    BytesN, Env,
};

const DOMAIN_TAG: &[u8] = b"soroban-cookbook/off-chain-signatures/v1";

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// The claim did not use the signer's next nonce (replay or out of order).
    BadNonce = 1,
    /// The secp256k1 signature recovered to a different Ethereum address.
    WrongSigner = 2,
    /// Amount must be positive.
    InvalidAmount = 3,
}

/// Which signature scheme an approval uses. Part of the signed payload.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Scheme {
    Ed25519 = 0,
    Secp256k1 = 1,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Ed25519Signer,
    EthSigner,
    Nonce(Scheme),
    Credited(Address),
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claimed {
    #[topic]
    pub to: Address,
    pub scheme: Scheme,
    pub amount: i128,
    pub nonce: u64,
}

#[contract]
pub struct OffChainSignatures;

#[contractimpl]
impl OffChainSignatures {
    /// Fixes the trusted signers at deploy time (a constructor cannot be
    /// front-run like a separate `init` call).
    ///
    /// * `ed25519_signer` — 32-byte ed25519 public key.
    /// * `eth_signer` — 20-byte Ethereum address of the secp256k1 signer.
    pub fn __constructor(env: Env, ed25519_signer: BytesN<32>, eth_signer: BytesN<20>) {
        let s = env.storage().instance();
        s.set(&DataKey::Ed25519Signer, &ed25519_signer);
        s.set(&DataKey::EthSigner, &eth_signer);
    }

    /// The 32-byte domain separator signers must include in every payload.
    pub fn domain_separator(env: Env) -> BytesN<32> {
        let mut data = Bytes::from_slice(&env, DOMAIN_TAG);
        data.append(&env.ledger().network_id().into());
        data.append(&env.current_contract_address().to_xdr(&env));
        env.crypto().sha256(&data).into()
    }

    /// The nonce the next approval for `scheme` must carry.
    pub fn next_nonce(env: Env, scheme: Scheme) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::Nonce(scheme))
            .unwrap_or(0)
    }

    /// Total credited to `to` so far.
    pub fn credited(env: Env, to: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Credited(to))
            .unwrap_or(0)
    }

    /// The exact bytes a signer signs over (before hashing). Exposed so
    /// off-chain tooling can check its encoding against the contract's.
    pub fn payload(env: Env, scheme: Scheme, to: Address, amount: i128, nonce: u64) -> Bytes {
        let mut p: Bytes = Self::domain_separator(env.clone()).into();
        p.push_back(scheme as u8);
        p.append(&to.to_xdr(&env));
        p.extend_from_array(&amount.to_be_bytes());
        p.extend_from_array(&nonce.to_be_bytes());
        p
    }

    /// Claim an approval signed by the ed25519 signer over `sha256(payload)`.
    ///
    /// Traps (host error, not an `Error` variant) if the signature is invalid.
    pub fn claim_ed25519(
        env: Env,
        to: Address,
        amount: i128,
        nonce: u64,
        signature: BytesN<64>,
    ) -> Result<(), Error> {
        let payload = Self::payload(env.clone(), Scheme::Ed25519, to.clone(), amount, nonce);
        let digest: BytesN<32> = env.crypto().sha256(&payload).into();
        let signer: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::Ed25519Signer)
            .unwrap();
        env.crypto()
            .ed25519_verify(&signer, &digest.into(), &signature);

        consume(&env, Scheme::Ed25519, to, amount, nonce)
    }

    /// Claim an approval signed by the Ethereum signer over
    /// `keccak256(payload)`. `signature` is `r || s` (64 bytes) and
    /// `recovery_id` is `v - 27` (0 or 1).
    pub fn claim_secp256k1(
        env: Env,
        to: Address,
        amount: i128,
        nonce: u64,
        signature: BytesN<64>,
        recovery_id: u32,
    ) -> Result<(), Error> {
        let payload = Self::payload(env.clone(), Scheme::Secp256k1, to.clone(), amount, nonce);
        let digest = env.crypto().keccak256(&payload);
        let public_key = env
            .crypto()
            .secp256k1_recover(&digest, &signature, recovery_id);

        let expected: BytesN<20> = env.storage().instance().get(&DataKey::EthSigner).unwrap();
        if eth_address(&env, &public_key) != expected {
            return Err(Error::WrongSigner);
        }

        consume(&env, Scheme::Secp256k1, to, amount, nonce)
    }
}

/// Ethereum address = last 20 bytes of keccak256 of the uncompressed public
/// key without its 0x04 prefix.
fn eth_address(env: &Env, public_key: &BytesN<65>) -> BytesN<20> {
    let key: Bytes = public_key.clone().into();
    let hash: Bytes = env.crypto().keccak256(&key.slice(1..)).into();
    hash.slice(12..).try_into().unwrap()
}

/// Validates and consumes the nonce, then applies the approved action.
/// Runs only after the signature has been verified.
fn consume(env: &Env, scheme: Scheme, to: Address, amount: i128, nonce: u64) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    let expected = OffChainSignatures::next_nonce(env.clone(), scheme);
    if nonce != expected {
        return Err(Error::BadNonce);
    }
    env.storage()
        .instance()
        .set(&DataKey::Nonce(scheme), &(expected + 1));

    let key = DataKey::Credited(to.clone());
    let total: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    env.storage().persistent().set(&key, &(total + amount));

    Claimed {
        to,
        scheme,
        amount,
        nonce,
    }
    .publish(env);
    Ok(())
}

#[cfg(test)]
mod test;

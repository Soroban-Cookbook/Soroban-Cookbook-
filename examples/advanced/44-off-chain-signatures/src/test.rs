extern crate std;

use ed25519_dalek::{Signer, SigningKey as EdKey};
use k256::ecdsa::SigningKey as EthKey;
use sha3::{Digest, Keccak256};
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    xdr::ToXdr,
    Address, Bytes, BytesN, Env,
};

use crate::{Error, OffChainSignatures, OffChainSignaturesClient, Scheme};

// ── off-chain side ──────────────────────────────────────────────────────────
// Everything here mirrors what a backend or wallet would do. The payload is
// built independently of the contract so an encoding change on either side
// fails the tests.

fn payload(
    env: &Env,
    contract: &Address,
    scheme: Scheme,
    to: &Address,
    amount: i128,
    nonce: u64,
) -> std::vec::Vec<u8> {
    let mut domain = Bytes::from_slice(env, b"soroban-cookbook/off-chain-signatures/v1");
    domain.append(&env.ledger().network_id().into());
    domain.append(&contract.clone().to_xdr(env));
    let domain: BytesN<32> = env.crypto().sha256(&domain).into();

    let mut p = domain.to_array().to_vec();
    p.push(scheme as u8);
    let to_xdr = to.clone().to_xdr(env);
    p.extend(to_xdr.iter());
    p.extend_from_slice(&amount.to_be_bytes());
    p.extend_from_slice(&nonce.to_be_bytes());
    p
}

fn sign_ed25519(env: &Env, key: &EdKey, payload: &[u8]) -> BytesN<64> {
    let digest: BytesN<32> = env.crypto().sha256(&Bytes::from_slice(env, payload)).into();
    BytesN::from_array(env, &key.sign(&digest.to_array()).to_bytes())
}

fn sign_eth(env: &Env, key: &EthKey, payload: &[u8]) -> (BytesN<64>, u32) {
    let digest = Keccak256::digest(payload);
    let (sig, rec_id) = key.sign_prehash_recoverable(&digest).unwrap();
    (
        BytesN::from_array(env, &sig.to_bytes().into()),
        rec_id.to_byte() as u32,
    )
}

fn eth_address(key: &EthKey) -> [u8; 20] {
    let point = key.verifying_key().to_encoded_point(false);
    let hash = Keccak256::digest(&point.as_bytes()[1..]);
    hash[12..].try_into().unwrap()
}

// ── fixture ─────────────────────────────────────────────────────────────────

struct Setup {
    env: Env,
    client: OffChainSignaturesClient<'static>,
    ed: EdKey,
    eth: EthKey,
}

fn deploy(env: &Env, ed: &EdKey, eth: &EthKey) -> OffChainSignaturesClient<'static> {
    let id = env.register(
        OffChainSignatures,
        (
            BytesN::from_array(env, &ed.verifying_key().to_bytes()),
            BytesN::from_array(env, &eth_address(eth)),
        ),
    );
    OffChainSignaturesClient::new(env, &id)
}

fn setup() -> Setup {
    let env = Env::default();
    let ed = EdKey::from_bytes(&[7; 32]);
    let eth = EthKey::from_slice(&[9; 32]).unwrap();
    let client = deploy(&env, &ed, &eth);
    Setup {
        env,
        client,
        ed,
        eth,
    }
}

impl Setup {
    fn ed_claim(&self, to: &Address, amount: i128, nonce: u64) -> BytesN<64> {
        let p = payload(
            &self.env,
            &self.client.address,
            Scheme::Ed25519,
            to,
            amount,
            nonce,
        );
        sign_ed25519(&self.env, &self.ed, &p)
    }

    fn eth_claim(&self, to: &Address, amount: i128, nonce: u64) -> (BytesN<64>, u32) {
        let p = payload(
            &self.env,
            &self.client.address,
            Scheme::Secp256k1,
            to,
            amount,
            nonce,
        );
        sign_eth(&self.env, &self.eth, &p)
    }
}

// ── happy paths ─────────────────────────────────────────────────────────────

#[test]
fn payload_encoding_matches_off_chain_encoding() {
    let s = setup();
    let to = Address::generate(&s.env);
    let expected = payload(&s.env, &s.client.address, Scheme::Secp256k1, &to, 5, 3);
    let on_chain = s.client.payload(&Scheme::Secp256k1, &to, &5, &3);
    assert_eq!(on_chain, Bytes::from_slice(&s.env, &expected));
}

#[test]
fn ed25519_claim_credits_and_consumes_nonce() {
    let s = setup();
    let to = Address::generate(&s.env);

    s.client
        .claim_ed25519(&to, &100, &0, &s.ed_claim(&to, 100, 0));
    // events().all() covers only the most recent invocation, so check first.
    assert_eq!(s.env.events().all().events().len(), 1, "Claimed emitted");

    assert_eq!(s.client.credited(&to), 100);
    assert_eq!(s.client.next_nonce(&Scheme::Ed25519), 1);
    assert_eq!(s.client.next_nonce(&Scheme::Secp256k1), 0);
}

#[test]
fn secp256k1_claim_credits_and_consumes_nonce() {
    let s = setup();
    let to = Address::generate(&s.env);
    let (sig, rec) = s.eth_claim(&to, 250, 0);

    s.client.claim_secp256k1(&to, &250, &0, &sig, &rec);

    assert_eq!(s.client.credited(&to), 250);
    assert_eq!(s.client.next_nonce(&Scheme::Secp256k1), 1);
}

#[test]
fn sequential_claims_accumulate() {
    let s = setup();
    let to = Address::generate(&s.env);
    for nonce in 0..3 {
        s.client
            .claim_ed25519(&to, &10, &nonce, &s.ed_claim(&to, 10, nonce));
    }
    assert_eq!(s.client.credited(&to), 30);
    assert_eq!(s.client.next_nonce(&Scheme::Ed25519), 3);
}

// ── replay protection ───────────────────────────────────────────────────────

#[test]
fn replaying_an_ed25519_approval_fails() {
    let s = setup();
    let to = Address::generate(&s.env);
    let sig = s.ed_claim(&to, 100, 0);
    s.client.claim_ed25519(&to, &100, &0, &sig);

    assert_eq!(
        s.client.try_claim_ed25519(&to, &100, &0, &sig),
        Err(Ok(Error::BadNonce))
    );
    assert_eq!(s.client.credited(&to), 100);
}

#[test]
fn replaying_a_secp256k1_approval_fails() {
    let s = setup();
    let to = Address::generate(&s.env);
    let (sig, rec) = s.eth_claim(&to, 250, 0);
    s.client.claim_secp256k1(&to, &250, &0, &sig, &rec);

    assert_eq!(
        s.client.try_claim_secp256k1(&to, &250, &0, &sig, &rec),
        Err(Ok(Error::BadNonce))
    );
}

#[test]
fn future_nonce_is_rejected() {
    let s = setup();
    let to = Address::generate(&s.env);
    assert_eq!(
        s.client
            .try_claim_ed25519(&to, &1, &1, &s.ed_claim(&to, 1, 1)),
        Err(Ok(Error::BadNonce))
    );
}

#[test]
fn approval_for_another_deployment_is_rejected() {
    // Same signer keys, different contract => different domain separator.
    let s = setup();
    let other = deploy(&s.env, &s.ed, &s.eth);
    assert_ne!(other.domain_separator(), s.client.domain_separator());

    let to = Address::generate(&s.env);
    let sig = s.ed_claim(&to, 100, 0); // signed for `s.client`
    assert!(other.try_claim_ed25519(&to, &100, &0, &sig).is_err());

    let (sig, rec) = s.eth_claim(&to, 100, 0);
    assert_eq!(
        other.try_claim_secp256k1(&to, &100, &0, &sig, &rec),
        Err(Ok(Error::WrongSigner))
    );
    assert_eq!(other.credited(&to), 0);
}

#[test]
fn signature_cannot_cross_schemes() {
    // An ed25519 approval's bytes are signed with scheme=Ed25519; re-using the
    // same message on the secp256k1 path recovers an unrelated key.
    let s = setup();
    let to = Address::generate(&s.env);
    let p = payload(&s.env, &s.client.address, Scheme::Ed25519, &to, 5, 0);
    let (sig, rec) = sign_eth(&s.env, &s.eth, &p);
    assert_eq!(
        s.client.try_claim_secp256k1(&to, &5, &0, &sig, &rec),
        Err(Ok(Error::WrongSigner))
    );
}

// ── tampering / wrong signer ────────────────────────────────────────────────

#[test]
fn tampered_amount_or_recipient_is_rejected() {
    let s = setup();
    let to = Address::generate(&s.env);
    let attacker = Address::generate(&s.env);
    let sig = s.ed_claim(&to, 100, 0);

    assert!(s.client.try_claim_ed25519(&to, &1_000, &0, &sig).is_err());
    assert!(s
        .client
        .try_claim_ed25519(&attacker, &100, &0, &sig)
        .is_err());

    let (sig, rec) = s.eth_claim(&to, 100, 0);
    assert_eq!(
        s.client
            .try_claim_secp256k1(&attacker, &100, &0, &sig, &rec),
        Err(Ok(Error::WrongSigner))
    );
    assert_eq!(s.client.next_nonce(&Scheme::Ed25519), 0);
    assert_eq!(s.client.next_nonce(&Scheme::Secp256k1), 0);
}

#[test]
fn untrusted_signers_are_rejected() {
    let s = setup();
    let to = Address::generate(&s.env);
    let p_ed = payload(&s.env, &s.client.address, Scheme::Ed25519, &to, 5, 0);
    let rogue_ed = EdKey::from_bytes(&[1; 32]);
    assert!(s
        .client
        .try_claim_ed25519(&to, &5, &0, &sign_ed25519(&s.env, &rogue_ed, &p_ed))
        .is_err());

    let p_eth = payload(&s.env, &s.client.address, Scheme::Secp256k1, &to, 5, 0);
    let rogue_eth = EthKey::from_slice(&[2; 32]).unwrap();
    let (sig, rec) = sign_eth(&s.env, &rogue_eth, &p_eth);
    assert_eq!(
        s.client.try_claim_secp256k1(&to, &5, &0, &sig, &rec),
        Err(Ok(Error::WrongSigner))
    );
}

#[test]
fn non_positive_amount_is_rejected() {
    let s = setup();
    let to = Address::generate(&s.env);
    assert_eq!(
        s.client
            .try_claim_ed25519(&to, &0, &0, &s.ed_claim(&to, 0, 0)),
        Err(Ok(Error::InvalidAmount))
    );
}

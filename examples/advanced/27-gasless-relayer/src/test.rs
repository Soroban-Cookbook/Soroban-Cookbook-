#![cfg(test)]
#![allow(deprecated)]

extern crate std;

use super::*;
use ed25519_dalek::{Signer as Ed25519Signer, SigningKey};
use soroban_sdk::{testutils::Address as _, xdr::ToXdr, Env};

fn generate_keypair(env: &Env) -> (SigningKey, BytesN<32>) {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let pubkey = signer.verifying_key().to_bytes();
    (signer, BytesN::<32>::from_array(env, &pubkey))
}

fn sign_tx(env: &Env, signer: &SigningKey, tx: &MetaTx) -> BytesN<64> {
    let message = tx.to_xdr(env);
    let message_hash = env.crypto().sha256(&message).to_bytes();
    let sig = signer.sign(&message_hash.to_array()).to_bytes();
    BytesN::<64>::from_array(env, &sig)
}

#[test]
fn test_initialize_and_fund() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(GaslessRelayerContract, ());
    let client = GaslessRelayerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let (_, pubkey) = generate_keypair(&env);

    assert_eq!(client.try_initialize(&admin), Ok(Ok(())));
    assert_eq!(client.try_add_trusted_relayer(&admin), Ok(Ok(())));
    assert_eq!(client.try_register_signer(&owner, &pubkey), Ok(Ok(())));
    assert_eq!(client.try_fund(&owner, &100), Ok(Ok(())));
}

#[test]
fn test_relay_transfer_with_valid_signature() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(GaslessRelayerContract, ());
    let client = GaslessRelayerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let relayer = Address::generate(&env);
    let (signer, pubkey) = generate_keypair(&env);

    assert_eq!(client.try_initialize(&admin), Ok(Ok(())));
    assert_eq!(client.try_add_trusted_relayer(&relayer), Ok(Ok(())));
    assert_eq!(client.try_register_signer(&owner, &pubkey), Ok(Ok(())));
    assert_eq!(client.try_fund(&owner, &100), Ok(Ok(())));

    let tx = MetaTx {
        from: owner.clone(),
        to: recipient.clone(),
        amount: 25,
        nonce: 1,
        deadline: 1_000_000,
    };

    let signature = sign_tx(&env, &signer, &tx);
    assert_eq!(
        client.try_relay_transfer(&relayer, &tx, &signature),
        Ok(Ok(()))
    );
}

#[test]
fn test_replay_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(GaslessRelayerContract, ());
    let client = GaslessRelayerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let relayer = Address::generate(&env);
    let (signer, pubkey) = generate_keypair(&env);

    assert_eq!(client.try_initialize(&admin), Ok(Ok(())));
    assert_eq!(client.try_add_trusted_relayer(&relayer), Ok(Ok(())));
    assert_eq!(client.try_register_signer(&owner, &pubkey), Ok(Ok(())));
    assert_eq!(client.try_fund(&owner, &100), Ok(Ok(())));

    let tx = MetaTx {
        from: owner.clone(),
        to: recipient.clone(),
        amount: 10,
        nonce: 1,
        deadline: 1_000_000,
    };

    let signature = sign_tx(&env, &signer, &tx);
    assert_eq!(
        client.try_relay_transfer(&relayer, &tx, &signature),
        Ok(Ok(()))
    );
    assert_eq!(
        client.try_relay_transfer(&relayer, &tx, &signature),
        Err(Ok(RelayerError::InvalidNonce))
    );
}

#[test]
fn test_invalid_signature_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(GaslessRelayerContract, ());
    let client = GaslessRelayerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let relayer = Address::generate(&env);
    let (signer, pubkey) = generate_keypair(&env);

    assert_eq!(client.try_initialize(&admin), Ok(Ok(())));
    assert_eq!(client.try_add_trusted_relayer(&relayer), Ok(Ok(())));
    assert_eq!(client.try_register_signer(&owner, &pubkey), Ok(Ok(())));
    assert_eq!(client.try_fund(&owner, &100), Ok(Ok(())));

    let tx = MetaTx {
        from: owner.clone(),
        to: recipient.clone(),
        amount: 10,
        nonce: 1,
        deadline: 1_000_000,
    };

    let signature = sign_tx(&env, &signer, &tx);

    let bad_tx = MetaTx {
        from: owner.clone(),
        to: recipient.clone(),
        amount: 11,
        nonce: 1,
        deadline: 1_000_000,
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.relay_transfer(&relayer, &bad_tx, &signature);
    }));
    assert!(result.is_err());
}

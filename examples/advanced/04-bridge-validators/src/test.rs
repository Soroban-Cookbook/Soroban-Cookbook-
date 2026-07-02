#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, ed25519::Sign},
    Bytes, Env,
};

fn generate_keypair() -> (soroban_sdk::testutils::ed25519::Signer, BytesN<32>) {
    let env = Env::default();
    let signer = soroban_sdk::testutils::ed25519::Signer::generate(&env);
    let pubkey = signer.public.clone();
    (signer, pubkey.into())
}

#[test]
fn test_init() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BridgeValidators);
    let client = BridgeValidatorsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100);

    let res = client.try_init(&admin, &100);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_add_validator() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, BridgeValidators);
    let client = BridgeValidatorsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin, &100);

    let (_, pubkey) = generate_keypair();
    client.add_validator(&pubkey, &50);

    // Cannot add twice
    let res = client.try_add_validator(&pubkey, &50);
    assert_eq!(res, Err(Ok(Error::ValidatorExists)));
}

#[test]
fn test_remove_validator() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, BridgeValidators);
    let client = BridgeValidatorsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin, &100);

    let (_, pubkey) = generate_keypair();
    client.add_validator(&pubkey, &50);
    client.remove_validator(&pubkey);

    let res = client.try_remove_validator(&pubkey);
    assert_eq!(res, Err(Ok(Error::ValidatorNotFound)));
}

#[test]
fn test_slash_validator() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, BridgeValidators);
    let client = BridgeValidatorsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin, &100);

    let (_, pubkey) = generate_keypair();
    client.add_validator(&pubkey, &50);
    client.slash_validator(&pubkey);
    
    // Slashing makes power 0 and active false. We can test this indirectly by trying to process message
}

#[test]
fn test_process_message_threshold_met() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, BridgeValidators);
    let client = BridgeValidatorsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin, &100);

    let (signer1, pub1) = generate_keypair();
    let (signer2, pub2) = generate_keypair();
    
    client.add_validator(&pub1, &60);
    client.add_validator(&pub2, &50);

    let message = Bytes::from_slice(&env, b"message to sign");
    let msg_hash = env.crypto().sha256(&message);
    
    let sig1 = signer1.sign(message.clone());
    let sig2 = signer2.sign(message.clone());
    
    let mut sigs = Map::new(&env);
    sigs.set(pub1, sig1.into());
    sigs.set(pub2, sig2.into());
    
    let res = client.process_message(&msg_hash, &sigs);
    assert_eq!(res, true);
    
    let res2 = client.try_process_message(&msg_hash, &sigs);
    assert_eq!(res2, Err(Ok(Error::MessageAlreadyProcessed)));
}

#[test]
fn test_process_message_threshold_not_met() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, BridgeValidators);
    let client = BridgeValidatorsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin, &100);

    let (signer1, pub1) = generate_keypair();
    let (_, pub2) = generate_keypair();
    
    client.add_validator(&pub1, &60);
    client.add_validator(&pub2, &50);

    let message = Bytes::from_slice(&env, b"message to sign");
    let msg_hash = env.crypto().sha256(&message);
    
    let sig1 = signer1.sign(message.clone());
    
    let mut sigs = Map::new(&env);
    sigs.set(pub1, sig1.into());
    
    let res = client.try_process_message(&msg_hash, &sigs);
    assert_eq!(res, Err(Ok(Error::ThresholdNotMet)));
}

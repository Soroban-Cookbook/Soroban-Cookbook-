#![cfg(test)]
#![allow(deprecated)]

extern crate std;

use super::*;
use ed25519_dalek::{Signer as Ed25519Signer, SigningKey};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    xdr::ToXdr,
    Bytes, Env,
};

fn generate_keypair(env: &Env) -> (SigningKey, BytesN<32>) {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let pubkey = signer.verifying_key().to_bytes();
    (signer, BytesN::<32>::from_array(env, &pubkey))
}

fn sign_meta_tx(env: &Env, signer: &SigningKey, tx: &MetaTx) -> BytesN<64> {
    let message = tx.to_xdr(env);
    let message_hash = env.crypto().sha256(&message).to_bytes();
    let sig = signer.sign(&message_hash.to_array()).to_bytes();
    BytesN::<64>::from_array(env, &sig)
}

fn setup() -> (
    Env,
    TrustedForwarderClient<'static>,
    SimpleRecipientClient<'static>,
    Address,
    Address,
    Address,
    (SigningKey, BytesN<32>),
) {
    let env = Env::default();
    env.mock_all_auths();

    let forwarder_id = env.register(TrustedForwarder, ());
    let recipient_id = env.register(SimpleRecipient, ());
    let forwarder = TrustedForwarderClient::new(&env, &forwarder_id);
    let recipient = SimpleRecipientClient::new(&env, &recipient_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let relayer = Address::generate(&env);
    let (signer, pubkey) = generate_keypair(&env);

    forwarder.initialize(&admin, &10);
    recipient.initialize(&forwarder_id);

    forwarder.register_signer(&user, &pubkey);
    forwarder.fund(&user, &1000);

    (
        env,
        forwarder,
        recipient,
        user,
        relayer,
        admin,
        (signer, pubkey),
    )
}

fn make_tx(
    env: &Env,
    from: &Address,
    to: &Address,
    data: &[u8],
    nonce: u64,
    fee: i128,
    deadline: u64,
) -> MetaTx {
    MetaTx {
        from: from.clone(),
        to: to.clone(),
        data: Bytes::from_slice(env, data),
        nonce,
        fee,
        deadline,
    }
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TrustedForwarder, ());
    let client = TrustedForwarderClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    assert_eq!(client.try_initialize(&admin, &10), Ok(Ok(())));
    assert_eq!(client.get_fee(), 10);
}

#[test]
fn test_initialize_rejects_duplicate() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TrustedForwarder, ());
    let client = TrustedForwarderClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    assert_eq!(client.try_initialize(&admin, &10), Ok(Ok(())));
    assert_eq!(
        client.try_initialize(&admin, &10),
        Err(Ok(ForwarderError::AlreadyInitialized))
    );
}

#[test]
fn test_forward_success() {
    let (env, forwarder, recipient, user, relayer, _admin, (signer, _)) = setup();

    let data = b"hello";
    let tx = make_tx(&env, &user, &recipient.address, data, 1, 10, 1_000_000);
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(forwarder.try_forward(&tx, &signature, &relayer), Ok(Ok(())));

    let stored = recipient.get_stored_value();
    assert_eq!(stored, Bytes::from_slice(&env, data));

    assert_eq!(recipient.get_last_sender(), Some(user.clone()));
}

#[test]
fn test_forward_relayer_paid() {
    let (env, forwarder, recipient, user, relayer, _admin, (signer, _)) = setup();

    assert_eq!(forwarder.balance(&user), 1000);
    assert_eq!(forwarder.balance(&relayer), 0);
    let tx = make_tx(&env, &user, &recipient.address, b"data", 1, 10, 1_000_000);
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(forwarder.try_forward(&tx, &signature, &relayer), Ok(Ok(())));

    assert_eq!(forwarder.balance(&user), 990);
    assert_eq!(forwarder.balance(&relayer), 10);
}

#[test]
fn test_forward_rejects_invalid_signature() {
    let (env, forwarder, _recipient, user, relayer, _admin, (_signer, _)) = setup();
    let bad_signer = SigningKey::from_bytes(&[99u8; 32]);

    let tx = make_tx(&env, &user, &forwarder.address, b"data", 1, 10, 1_000_000);
    let signature = sign_meta_tx(&env, &bad_signer, &tx);

    let result = forwarder.try_forward(&tx, &signature, &relayer);
    assert!(result.is_err());
}

#[test]
fn test_forward_rejects_expired_deadline() {
    let (env, forwarder, recipient, user, relayer, _admin, (signer, _)) = setup();

    let tx = make_tx(&env, &user, &recipient.address, b"data", 1, 10, 500);
    env.ledger().with_mut(|l| l.timestamp = 501);
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(
        forwarder.try_forward(&tx, &signature, &relayer),
        Err(Ok(ForwarderError::Expired))
    );
}

#[test]
fn test_forward_rejects_bad_nonce() {
    let (env, forwarder, _recipient, user, relayer, _admin, (signer, _)) = setup();

    let tx = make_tx(&env, &user, &forwarder.address, b"data", 99, 10, 1_000_000);
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(
        forwarder.try_forward(&tx, &signature, &relayer),
        Err(Ok(ForwarderError::InvalidNonce))
    );
}

#[test]
fn test_forward_rejects_replay() {
    let (env, forwarder, recipient, user, relayer, _admin, (signer, _)) = setup();

    let tx = make_tx(&env, &user, &recipient.address, b"data", 1, 10, 1_000_000);
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(forwarder.try_forward(&tx, &signature, &relayer), Ok(Ok(())));

    assert_eq!(
        forwarder.try_forward(&tx, &signature, &relayer),
        Err(Ok(ForwarderError::InvalidNonce))
    );
}

#[test]
fn test_forward_rejects_insufficient_balance() {
    let (env, forwarder, _recipient, user, relayer, _admin, (signer, _)) = setup();
    forwarder.withdraw(&user, &995);
    assert_eq!(forwarder.balance(&user), 5);

    let tx = make_tx(&env, &user, &forwarder.address, b"data", 1, 10, 1_000_000);
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(
        forwarder.try_forward(&tx, &signature, &relayer),
        Err(Ok(ForwarderError::InsufficientBalance))
    );
}

#[test]
fn test_forward_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TrustedForwarder, ());
    let forwarder = TrustedForwarderClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let relayer = Address::generate(&env);
    let (signer, pubkey) = generate_keypair(&env);

    forwarder.register_signer(&user, &pubkey);
    forwarder.fund(&user, &100);

    let tx = make_tx(&env, &user, &forwarder.address, b"data", 1, 10, 1_000_000);
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(
        forwarder.try_forward(&tx, &signature, &relayer),
        Err(Ok(ForwarderError::NotInitialized))
    );
}

#[test]
fn test_nonce_monotonic() {
    let (env, forwarder, recipient, user, relayer, _admin, (signer, _)) = setup();

    assert_eq!(forwarder.next_nonce(&user), 1);

    let tx1 = make_tx(&env, &user, &recipient.address, b"first", 1, 10, 1_000_000);
    let sig1 = sign_meta_tx(&env, &signer, &tx1);
    forwarder.forward(&tx1, &sig1, &relayer);

    assert_eq!(forwarder.next_nonce(&user), 2);

    let tx2 = make_tx(&env, &user, &recipient.address, b"second", 2, 10, 1_000_000);
    let sig2 = sign_meta_tx(&env, &signer, &tx2);
    forwarder.forward(&tx2, &sig2, &relayer);

    assert_eq!(forwarder.next_nonce(&user), 3);
}

#[test]
fn test_fund_and_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TrustedForwarder, ());
    let client = TrustedForwarderClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &10);

    assert_eq!(client.balance(&user), 0);

    client.fund(&user, &500);
    assert_eq!(client.balance(&user), 500);

    client.withdraw(&user, &200);
    assert_eq!(client.balance(&user), 300);

    assert_eq!(
        client.try_withdraw(&user, &999),
        Err(Ok(ForwarderError::InsufficientBalance))
    );
}

#[test]
fn test_set_fee_admin_only() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TrustedForwarder, ());
    let client = TrustedForwarderClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &10);
    assert_eq!(client.get_fee(), 10);

    client.set_fee(&25);
    assert_eq!(client.get_fee(), 25);
}

#[test]
fn test_register_signer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TrustedForwarder, ());
    let client = TrustedForwarderClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (_signer, pubkey) = generate_keypair(&env);

    client.initialize(&admin, &10);
    assert_eq!(client.try_register_signer(&user, &pubkey), Ok(Ok(())));
}

#[test]
fn test_set_fee_requires_auth() {
    let env = Env::default();
    let contract_id = env.register(TrustedForwarder, ());
    let client = TrustedForwarderClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &10);
    env.set_auths(&[]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_fee(&100);
    }));
    assert!(result.is_err());
}

#[test]
fn test_recipient_rejects_direct_call() {
    let env = Env::default();

    let forwarder_id = env.register(TrustedForwarder, ());
    let recipient_id = env.register(SimpleRecipient, ());
    let recipient = SimpleRecipientClient::new(&env, &recipient_id);

    env.mock_all_auths();
    recipient.initialize(&forwarder_id);
    env.set_auths(&[]);

    let attacker = Address::generate(&env);
    let data = Bytes::from_slice(&env, b"hack");

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        recipient.forwarded_call(&attacker, &data);
    }));
    assert!(result.is_err());
}

#[test]
fn test_multiple_forwarders_independent_nonces() {
    let env = Env::default();
    env.mock_all_auths();

    let forwarder_id = env.register(TrustedForwarder, ());
    let forwarder = TrustedForwarderClient::new(&env, &forwarder_id);
    let recipient_id = env.register(SimpleRecipient, ());
    let recipient = SimpleRecipientClient::new(&env, &recipient_id);

    let admin = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let relayer = Address::generate(&env);
    let (signer_a, pubkey_a) = generate_keypair(&env);
    let signer_b = SigningKey::from_bytes(&[42u8; 32]);
    let pubkey_b_bytes = signer_b.verifying_key().to_bytes();
    let pubkey_b = BytesN::<32>::from_array(&env, &pubkey_b_bytes);

    forwarder.initialize(&admin, &10);
    recipient.initialize(&forwarder_id);

    forwarder.register_signer(&user_a, &pubkey_a);
    forwarder.register_signer(&user_b, &pubkey_b);
    forwarder.fund(&user_a, &1000);
    forwarder.fund(&user_b, &1000);

    let tx_a = make_tx(
        &env,
        &user_a,
        &recipient.address,
        b"from_a",
        1,
        10,
        1_000_000,
    );
    let sig_a = sign_meta_tx(&env, &signer_a, &tx_a);

    let tx_b = make_tx(
        &env,
        &user_b,
        &recipient.address,
        b"from_b",
        1,
        10,
        1_000_000,
    );
    let sig_b = sign_meta_tx(&env, &signer_b, &tx_b);

    assert_eq!(forwarder.try_forward(&tx_a, &sig_a, &relayer), Ok(Ok(())));
    assert_eq!(forwarder.try_forward(&tx_b, &sig_b, &relayer), Ok(Ok(())));

    assert_eq!(forwarder.next_nonce(&user_a), 2);
    assert_eq!(forwarder.next_nonce(&user_b), 2);
}

#[test]
fn test_empty_data_rejected() {
    let (env, forwarder, _recipient, user, relayer, _admin, (signer, _)) = setup();

    let tx = MetaTx {
        from: user.clone(),
        to: forwarder.address.clone(),
        data: Bytes::new(&env),
        nonce: 1,
        fee: 10,
        deadline: 1_000_000,
    };
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(
        forwarder.try_forward(&tx, &signature, &relayer),
        Err(Ok(ForwarderError::InvalidAmount))
    );
}

#[test]
fn test_forward_with_zero_fee() {
    let (env, forwarder, recipient, user, relayer, _admin, (signer, _)) = setup();
    forwarder.set_fee(&0);
    forwarder.withdraw(&user, &990);
    assert_eq!(forwarder.balance(&user), 10);

    let tx = make_tx(&env, &user, &recipient.address, b"free", 1, 0, 1_000_000);
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(forwarder.try_forward(&tx, &signature, &relayer), Ok(Ok(())));

    let stored = recipient.get_stored_value();
    assert_eq!(stored, Bytes::from_slice(&env, b"free"));
}

#[test]
fn test_forward_deadline_boundary() {
    let env = Env::default();
    env.mock_all_auths();

    let forwarder_id = env.register(TrustedForwarder, ());
    let forwarder = TrustedForwarderClient::new(&env, &forwarder_id);
    let recipient_id = env.register(SimpleRecipient, ());
    let recipient = SimpleRecipientClient::new(&env, &recipient_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let relayer = Address::generate(&env);
    let (signer, pubkey) = generate_keypair(&env);

    forwarder.initialize(&admin, &10);
    recipient.initialize(&forwarder_id);
    forwarder.register_signer(&user, &pubkey);
    forwarder.fund(&user, &1000);

    let current_ts = env.ledger().timestamp();

    let tx = make_tx(
        &env,
        &user,
        &recipient.address,
        b"exactly_now",
        1,
        10,
        current_ts,
    );
    let signature = sign_meta_tx(&env, &signer, &tx);

    assert_eq!(forwarder.try_forward(&tx, &signature, &relayer), Ok(Ok(())));
}

#[test]
fn test_recipient_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let recipient_id = env.register(SimpleRecipient, ());
    let recipient = SimpleRecipientClient::new(&env, &recipient_id);
    let sender = Address::generate(&env);
    let data = Bytes::from_slice(&env, b"test");

    let result = recipient.try_forwarded_call(&sender, &data);
    assert_eq!(result, Err(Ok(RecipientError::NotInitialized)));
}

extern crate std;

use ed25519_dalek::{Signer, SigningKey};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, Ledger},
    token::{StellarAssetClient, TokenClient},
    xdr::ToXdr,
    Address, Bytes, BytesN, Env, IntoVal, Symbol,
};

use crate::{PaymentChannel, PaymentChannelClient};

const EXPIRY_OFFSET: u64 = 1000;

struct Setup {
    env: Env,
    channel: PaymentChannelClient<'static>,
    token: TokenClient<'static>,
    addr_a: Address,
    addr_b: Address,
    kp_a: SigningKey,
    kp_b: SigningKey,
}

fn pubkey(env: &Env, kp: &SigningKey) -> BytesN<32> {
    BytesN::from_array(env, &kp.verifying_key().to_bytes())
}

/// Mirrors `build_message` in lib.rs independently, so a change to the signed
/// format on either side makes the signature tests fail.
fn state_message(
    env: &Env,
    channel: &Address,
    balance_a: i128,
    balance_b: i128,
    seq: u32,
) -> Bytes {
    let mut msg = channel.clone().to_xdr(env);
    msg.extend_from_array(&balance_a.to_be_bytes());
    msg.extend_from_array(&balance_b.to_be_bytes());
    msg.extend_from_array(&seq.to_be_bytes());
    msg
}

fn sign(env: &Env, kp: &SigningKey, msg: &Bytes) -> BytesN<64> {
    let mut buf = std::vec![0u8; msg.len() as usize];
    msg.copy_into_slice(&mut buf);
    BytesN::from_array(env, &kp.sign(&buf).to_bytes())
}

fn setup() -> Setup {
    let env = Env::default();
    env.mock_all_auths();

    let kp_a = SigningKey::from_bytes(&[1u8; 32]);
    let kp_b = SigningKey::from_bytes(&[2u8; 32]);
    let addr_a = Address::generate(&env);
    let addr_b = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
    let asset = StellarAssetClient::new(&env, &sac.address());
    asset.mint(&addr_a, &1000);
    asset.mint(&addr_b, &1000);

    let channel = PaymentChannelClient::new(&env, &env.register(PaymentChannel, ()));
    let expiry = env.ledger().timestamp() + EXPIRY_OFFSET;
    channel.init(
        &sac.address(),
        &addr_a,
        &addr_b,
        &pubkey(&env, &kp_a),
        &pubkey(&env, &kp_b),
        &expiry,
    );

    Setup {
        token: TokenClient::new(&env, &sac.address()),
        env,
        channel,
        addr_a,
        addr_b,
        kp_a,
        kp_b,
    }
}
#![cfg(test)]

use ed25519_dalek::{Signer, SigningKey};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient as TokenAssetClient};
use soroban_sdk::{contract, contractimpl, symbol_short, xdr::ToXdr, Address, Bytes, BytesN, Env, IntoVal};

use crate::{PaymentChannel, PaymentChannelClient};

#[contract]
pub struct TestToken;

#[contractimpl]
impl TestToken {
    pub fn init(env: Env, admin: Address) {
        env.storage().instance().set(&symbol_short!("admin"), &admin);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();
        let bal = env.storage().persistent().get::<Address, i128>(&to).unwrap_or(0);
        env.storage().persistent().set(&to, &(bal + amount));
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_bal = env.storage().persistent().get::<Address, i128>(&from).unwrap_or(0);
        let to_bal = env.storage().persistent().get::<Address, i128>(&to).unwrap_or(0);
        assert!(from_bal >= amount, "insufficient balance");
        env.storage().persistent().set(&from, &(from_bal - amount));
        env.storage().persistent().set(&to, &(to_bal + amount));
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage().persistent().get::<Address, i128>(&id).unwrap_or(0)
    }
}

fn pubkey_to_bytesn(env: &Env, kp: &SigningKey) -> BytesN<32> {
    BytesN::from_array(env, &kp.verifying_key().to_bytes())
}

fn make_state_message(env: &Env, contract_id: &Address, balance_a: i128, balance_b: i128, sequence: u32) -> Bytes {
    let mut msg = contract_id.to_xdr(env);
    msg.append(&Bytes::from_slice(env, &balance_a.to_be_bytes()));
    msg.append(&Bytes::from_slice(env, &balance_b.to_be_bytes()));
    msg.append(&Bytes::from_slice(env, &sequence.to_be_bytes()));
    msg
}

fn sign_message(env: &Env, signer: &SigningKey, message: &Bytes) -> BytesN<64> {
    let message_bytes = message.iter().collect::<std::vec::Vec<_>>();
    BytesN::from_array(env, &signer.sign(&message_bytes).to_bytes())
}

fn setup() -> (Env, Address, Address, Address, SigningKey, SigningKey, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let kp_a = SigningKey::from_bytes(&[1u8; 32]);
    let kp_b = SigningKey::from_bytes(&[2u8; 32]);
    let pub_a = pubkey_to_bytesn(&env, &kp_a);
    let pub_b = pubkey_to_bytesn(&env, &kp_b);
    let addr_a = Address::generate(&env);
    let addr_b = Address::generate(&env);

    // Deploy token
    let token_id = env.register(TestToken, ());
    let token_admin = Address::generate(&env);
    let test_token_client = TestTokenClient::new(&env, &token_id);
    test_token_client.init(&token_admin);

/// Deposit 100 from A and 50 from B.
fn fund(s: &Setup) {
    s.channel.deposit(&s.addr_a, &100);
    s.channel.deposit(&s.addr_b, &50);
}

/// Submit a state co-signed by both participants.
fn submit(s: &Setup, balance_a: i128, balance_b: i128, seq: u32) {
    let msg = state_message(&s.env, &s.channel.address, balance_a, balance_b, seq);
    s.channel.submit_state(
        &s.addr_a,
        &balance_a,
        &balance_b,
        &seq,
        &sign(&s.env, &s.kp_a, &msg),
        &sign(&s.env, &s.kp_b, &msg),
    );
}
    // Deploy payment channel
    let pc_id = env.register(PaymentChannel, ());
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let expiry = env.ledger().timestamp() + 1000;
    pc_client.init(&token_id, &addr_a, &addr_b, &pub_a, &pub_b, &expiry);

fn last_auth_is(
    s: &Setup,
    signer: &Address,
    function: &str,
    args: soroban_sdk::Vec<soroban_sdk::Val>,
) {
    let auths = s.env.auths();
    let (addr, invocation) = auths.first().expect("no auth recorded");
    assert_eq!(addr, signer);
    assert_eq!(
        invocation.function,
        AuthorizedFunction::Contract((
            s.channel.address.clone(),
            Symbol::new(&s.env, function),
            args,
        ))
    );
}

// ---------------------------------------------------------------------------
// Happy paths
// ---------------------------------------------------------------------------

#[test]
fn test_deposit_and_close() {
    let s = setup();
    fund(&s);
    let (env, pc_id, addr_a, addr_b, _kp_a, _kp_b, token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let token_client = TokenClient::new(&env, &token_id);

    let info = s.channel.get_info();
    assert_eq!(info.balance_a, 100);
    assert_eq!(info.balance_b, 50);
    assert_eq!(s.token.balance(&s.addr_a), 900);
    assert_eq!(s.token.balance(&s.addr_b), 950);
    assert_eq!(s.token.balance(&s.channel.address), 150);

    s.channel.close(&s.addr_a);

    assert_eq!(s.token.balance(&s.addr_a), 1000);
    assert_eq!(s.token.balance(&s.addr_b), 1000);
    assert_eq!(s.token.balance(&s.channel.address), 0);
    assert!(s.channel.get_info().is_closed);
    assert_eq!(token_client.balance(&addr_a), 900);
    assert_eq!(token_client.balance(&addr_b), 950);
    assert_eq!(token_client.balance(&pc_id), 150);

    pc_client.close(&addr_a);

    assert_eq!(token_client.balance(&addr_a), 1000);
    assert_eq!(token_client.balance(&addr_b), 1000);
    assert_eq!(token_client.balance(&pc_id), 0);
}

#[test]
fn test_bidirectional_payment() {
    let s = setup();
    fund(&s);

    // A pays B 30, then B pays A 10.
    submit(&s, 70, 80, 1);
    submit(&s, 80, 70, 2);

    let info = s.channel.get_info();
    assert_eq!((info.balance_a, info.balance_b, info.sequence), (80, 70, 2));
    // A sends 30 to B
    let new_a: i128 = 70;
    let new_b: i128 = 80;
    let seq: u32 = 1;
    let msg = make_state_message(&env, &pc_id, new_a, new_b, seq);
    let sig_a_bytes = sign_message(&env, &kp_a, &msg);
    let sig_b_bytes = sign_message(&env, &kp_b, &msg);

    s.channel.close(&s.addr_b);

    assert_eq!(s.token.balance(&s.addr_a), 980);
    assert_eq!(s.token.balance(&s.addr_b), 1020);
}

// ---------------------------------------------------------------------------
// Authorization
// ---------------------------------------------------------------------------

#[test]
fn test_init_requires_both_participants_auth() {
    let s = setup();
    // `setup` consumed the init auths; re-run init on a fresh channel.
    let channel = PaymentChannelClient::new(&s.env, &s.env.register(PaymentChannel, ()));
    channel.init(
        &s.token.address,
        &s.addr_a,
        &s.addr_b,
        &pubkey(&s.env, &s.kp_a),
        &pubkey(&s.env, &s.kp_b),
        &(s.env.ledger().timestamp() + EXPIRY_OFFSET),
    );
    let signers: std::vec::Vec<Address> = s.env.auths().into_iter().map(|(a, _)| a).collect();
    assert_eq!(signers, std::vec![s.addr_a.clone(), s.addr_b.clone()]);
}

#[test]
fn test_deposit_requires_depositor_auth() {
    let s = setup();
    s.channel.deposit(&s.addr_a, &100);
    last_auth_is(
        &s,
        &s.addr_a,
        "deposit",
        (s.addr_a.clone(), 100i128).into_val(&s.env),
    );
}
    assert_eq!(token_client.balance(&addr_a), 970);
    assert_eq!(token_client.balance(&addr_b), 1030);
}

#[test]
#[should_panic]
fn test_invalid_signature() {
    let (env, pc_id, addr_a, addr_b, _kp_a, kp_b, _token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);

#[test]
fn test_submit_state_requires_submitter_auth() {
    let s = setup();
    fund(&s);
    submit(&s, 70, 80, 1);
    let (addr, invocation) = s.env.auths().first().cloned().unwrap();
    assert_eq!(addr, s.addr_a);
    match invocation.function {
        AuthorizedFunction::Contract((contract, function, _)) => {
            assert_eq!(contract, s.channel.address);
            assert_eq!(function, Symbol::new(&s.env, "submit_state"));
        }
        other => panic!("unexpected auth: {other:?}"),
    }
}

#[test]
fn test_close_requires_closer_auth() {
    let s = setup();
    fund(&s);
    s.channel.close(&s.addr_b);
    last_auth_is(&s, &s.addr_b, "close", (s.addr_b.clone(),).into_val(&s.env));
}

#[test]
fn test_deposit_without_auth_is_rejected() {
    let s = setup();
    s.env.set_auths(&[]);
    assert!(s.channel.try_deposit(&s.addr_a, &100).is_err());
    assert_eq!(s.token.balance(&s.addr_a), 1000);
}

#[test]
fn test_close_without_auth_is_rejected() {
    let s = setup();
    fund(&s);
    s.env.set_auths(&[]);
    assert!(s.channel.try_close(&s.addr_a).is_err());
    assert!(!s.channel.get_info().is_closed);
}

// ---------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------
    let new_a: i128 = 70;
    let new_b: i128 = 80;
    let seq: u32 = 1;
    let msg = make_state_message(&env, &pc_id, new_a, new_b, seq);
    // Sign with a wrong key
    let wrong_kp = SigningKey::from_bytes(&[3u8; 32]);
    let sig_bad_bytes = sign_message(&env, &wrong_kp, &msg);
    let sig_b_bytes = sign_message(&env, &kp_b, &msg);

#[test]
#[should_panic]
fn test_invalid_signature() {
    let s = setup();
    fund(&s);

    let msg = state_message(&s.env, &s.channel.address, 70, 80, 1);
    let wrong = SigningKey::from_bytes(&[3u8; 32]);
    s.channel.submit_state(
        &s.addr_a,
        &70,
        &80,
        &1,
        &sign(&s.env, &wrong, &msg),
        &sign(&s.env, &s.kp_b, &msg),
    );
}

#[test]
#[should_panic]
fn test_signature_for_other_state_rejected() {
    let s = setup();
    fund(&s);

    // Both sign (70, 80) but the submitter claims (10, 140).
    let msg = state_message(&s.env, &s.channel.address, 70, 80, 1);
    s.channel.submit_state(
        &s.addr_a,
        &10,
        &140,
        &1,
        &sign(&s.env, &s.kp_a, &msg),
        &sign(&s.env, &s.kp_b, &msg),
    );
}

#[test]
#[should_panic(expected = "sequence must increase")]
fn test_sequence_must_increase() {
    let s = setup();
    fund(&s);
    submit(&s, 70, 80, 1);
    submit(&s, 80, 70, 1);
}

#[test]
#[should_panic(expected = "balance mismatch")]
fn test_state_must_preserve_total() {
    let s = setup();
    fund(&s);
    submit(&s, 100, 100, 1);
}

#[test]
#[should_panic(expected = "not a participant")]
fn test_non_participant_cannot_deposit() {
    let s = setup();
    let outsider = Address::generate(&s.env);
    StellarAssetClient::new(&s.env, &s.token.address).mint(&outsider, &100);
    s.channel.deposit(&outsider, &100);
}

#[test]
#[should_panic(expected = "not a participant")]
fn test_non_participant_cannot_close() {
    let s = setup();
    fund(&s);
    s.channel.close(&Address::generate(&s.env));
}

#[test]
#[should_panic(expected = "channel expired")]
fn test_deposit_after_expiry() {
    let s = setup();
    s.env.ledger().with_mut(|l| l.timestamp += EXPIRY_OFFSET);
    s.channel.deposit(&s.addr_a, &100);
}

#[test]
#[should_panic(expected = "channel closed")]
fn test_cannot_close_twice() {
    let s = setup();
    fund(&s);
    s.channel.close(&s.addr_a);
    s.channel.close(&s.addr_b);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_cannot_reinitialize() {
    let s = setup();
    s.channel.init(
        &s.token.address,
        &s.addr_a,
        &s.addr_b,
        &pubkey(&s.env, &s.kp_a),
        &pubkey(&s.env, &s.kp_b),
        &(s.env.ledger().timestamp() + EXPIRY_OFFSET),
    );
    let (env, pc_id, addr_a, addr_b, kp_a, kp_b, _token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);

    pc_client.deposit(&addr_a, &100);
    pc_client.deposit(&addr_b, &50);

    let new_a: i128 = 70;
    let new_b: i128 = 80;
    let seq: u32 = 1;
    let msg = make_state_message(&env, &pc_id, new_a, new_b, seq);
    let sig_a_bytes = sign_message(&env, &kp_a, &msg);
    let sig_b_bytes = sign_message(&env, &kp_b, &msg);
    pc_client.submit_state(&addr_a, &new_a, &new_b, &seq, &sig_a_bytes, &sig_b_bytes);

    // Try to submit an older sequence
    pc_client.submit_state(&addr_a, &new_a, &new_b, &0, &sig_a_bytes, &sig_b_bytes);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_init_without_auth_fails() {
    let env = Env::default();
    let kp_a = SigningKey::from_bytes(&[1u8; 32]);
    let kp_b = SigningKey::from_bytes(&[2u8; 32]);
    let pub_a = pubkey_to_bytesn(&env, &kp_a);
    let pub_b = pubkey_to_bytesn(&env, &kp_b);
    let addr_a = Address::generate(&env);
    let addr_b = Address::generate(&env);

    let token_id = env.register(TestToken, ());
    let pc_id = env.register(PaymentChannel, ());
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let expiry = env.ledger().timestamp() + 1000;

    env.mock_all_auths();
    env.set_auths(&[]); // Strip all auths so neither participant authorizes

    pc_client.init(&token_id, &addr_a, &addr_b, &pub_a, &pub_b, &expiry);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_init_without_participant_b_auth_fails() {
    let env = Env::default();
    let kp_a = SigningKey::from_bytes(&[1u8; 32]);
    let kp_b = SigningKey::from_bytes(&[2u8; 32]);
    let pub_a = pubkey_to_bytesn(&env, &kp_a);
    let pub_b = pubkey_to_bytesn(&env, &kp_b);
    let addr_a = Address::generate(&env);
    let addr_b = Address::generate(&env);

    let token_id = env.register(TestToken, ());
    let pc_id = env.register(PaymentChannel, ());
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let expiry = env.ledger().timestamp() + 1000;

    // Only participant A authorizes
    env.mock_auths(&[
        soroban_sdk::testutils::MockAuth {
            address: &addr_a,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &pc_id,
                fn_name: "init",
                args: (&token_id, &addr_a, &addr_b, &pub_a, &pub_b, expiry).into_val(&env),
                sub_invokes: &[],
            },
        },
    ]);

    pc_client.init(&token_id, &addr_a, &addr_b, &pub_a, &pub_b, &expiry);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_init_without_participant_a_auth_fails() {
    let env = Env::default();
    let kp_a = SigningKey::from_bytes(&[1u8; 32]);
    let kp_b = SigningKey::from_bytes(&[2u8; 32]);
    let pub_a = pubkey_to_bytesn(&env, &kp_a);
    let pub_b = pubkey_to_bytesn(&env, &kp_b);
    let addr_a = Address::generate(&env);
    let addr_b = Address::generate(&env);

    let token_id = env.register(TestToken, ());
    let pc_id = env.register(PaymentChannel, ());
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let expiry = env.ledger().timestamp() + 1000;

    // Only participant B authorizes
    env.mock_auths(&[
        soroban_sdk::testutils::MockAuth {
            address: &addr_b,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &pc_id,
                fn_name: "init",
                args: (&token_id, &addr_a, &addr_b, &pub_a, &pub_b, expiry).into_val(&env),
                sub_invokes: &[],
            },
        },
    ]);

    pc_client.init(&token_id, &addr_a, &addr_b, &pub_a, &pub_b, &expiry);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_init_already_initialized_fails() {
    let (env, pc_id, addr_a, addr_b, kp_a, kp_b, token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let pub_a = pubkey_to_bytesn(&env, &kp_a);
    let pub_b = pubkey_to_bytesn(&env, &kp_b);
    let expiry = env.ledger().timestamp() + 2000;

    pc_client.init(&token_id, &addr_a, &addr_b, &pub_a, &pub_b, &expiry);
}

#[test]
#[should_panic(expected = "not a participant")]
fn test_deposit_by_non_participant_fails() {
    let (env, pc_id, _addr_a, _addr_b, _kp_a, _kp_b, token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let outsider = Address::generate(&env);

    let token_asset = TokenAssetClient::new(&env, &token_id);
    token_asset.mint(&outsider, &1000);

    pc_client.deposit(&outsider, &100);
}

#[test]
#[should_panic(expected = "not a participant")]
fn test_close_by_non_participant_fails() {
    let (env, pc_id, _addr_a, _addr_b, _kp_a, _kp_b, _token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let outsider = Address::generate(&env);

    pc_client.close(&outsider);
}

#[test]
#[should_panic(expected = "channel expired")]
fn test_deposit_after_expiry_fails() {
    let (env, pc_id, addr_a, _addr_b, _kp_a, _kp_b, _token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);

    // Fast forward ledger timestamp past expiry (expiry was timestamp + 1000)
    env.ledger().with_mut(|l| l.timestamp += 2000);

    pc_client.deposit(&addr_a, &100);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_deposit_zero_fails() {
    let (env, pc_id, addr_a, _addr_b, _kp_a, _kp_b, _token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);

    pc_client.deposit(&addr_a, &0);
}

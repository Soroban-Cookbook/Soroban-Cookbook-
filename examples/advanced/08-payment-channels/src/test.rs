#![cfg(test)]

use ed25519_dalek::Signer;
use soroban_sdk::contract::{contract, contractimpl};
use soroban_sdk::testutils::ed25519::Ed25519;
use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, Symbol};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient as TokenAssetClient};

use crate::{ChannelInfo, PaymentChannel, PaymentChannelClient};

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

    pub fn balance_of(env: Env, id: Address) -> i128 {
        env.storage().persistent().get::<Address, i128>(&id).unwrap_or(0)
    }
}

fn pubkey_to_bytesn(env: &Env, kp: &Ed25519) -> BytesN<32> {
    BytesN::from_array(env, &kp.verifying_key().to_bytes())
}

fn make_state_message(env: &Env, contract_id: &Address, balance_a: i128, balance_b: i128, sequence: u32) -> Bytes {
    let mut msg = Bytes::new(env);
    let contract_bytes = contract_id.as_contract().unwrap();
    for byte in contract_bytes.iter() {
        msg.push(byte);
    }
    for byte in balance_a.to_be_bytes().iter() {
        msg.push(*byte);
    }
    for byte in balance_b.to_be_bytes().iter() {
        msg.push(*byte);
    }
    for byte in sequence.to_be_bytes().iter() {
        msg.push(*byte);
    }
    msg
}

fn setup() -> (Env, Address, Address, Address, Ed25519, Ed25519, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let kp_a = Ed25519::from_seed(&[1u8; 32]);
    let kp_b = Ed25519::from_seed(&[2u8; 32]);
    let pub_a = pubkey_to_bytesn(&env, &kp_a);
    let pub_b = pubkey_to_bytesn(&env, &kp_b);
    let addr_a = Address::from_ed25519(&pub_a);
    let addr_b = Address::from_ed25519(&pub_b);

    // Deploy token
    let token_id = env.register_contract(None, TestToken);
    let token_admin = Address::generate(&env);
    env.invoke_contract::<()>(&token_id, "init", (token_admin.clone(),));

    let token_asset = TokenAssetClient::new(&env, &token_id);
    token_asset.mint(&addr_a, &1000);
    token_asset.mint(&addr_b, &1000);

    // Deploy payment channel
    let pc_id = env.register_contract(None, PaymentChannel);
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let expiry = env.ledger().timestamp() + 1000;
    pc_client.init(&token_id, &pub_a, &pub_b, &expiry);

    (env, pc_id, addr_a, addr_b, kp_a, kp_b, token_id)
}

#[test]
fn test_deposit_and_close() {
    let (env, pc_id, addr_a, addr_b, kp_a, kp_b, token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let token_client = TokenClient::new(&env, &token_id);

    pc_client.deposit(&addr_a, &100);
    pc_client.deposit(&addr_b, &50);

    let info = pc_client.get_info();
    assert_eq!(info.balance_a, 100);
    assert_eq!(info.balance_b, 50);

    assert_eq!(token_client.balance_of(&addr_a), 900);
    assert_eq!(token_client.balance_of(&addr_b), 950);
    assert_eq!(token_client.balance_of(&pc_id), 150);

    pc_client.close(&addr_a);

    assert_eq!(token_client.balance_of(&addr_a), 1000);
    assert_eq!(token_client.balance_of(&addr_b), 1000);
    assert_eq!(token_client.balance_of(&pc_id), 0);
}

#[test]
fn test_bidirectional_payment() {
    let (env, pc_id, addr_a, addr_b, kp_a, kp_b, token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);
    let token_client = TokenClient::new(&env, &token_id);

    pc_client.deposit(&addr_a, &100);
    pc_client.deposit(&addr_b, &50);

    // A sends 30 to B
    let new_a: i128 = 70;
    let new_b: i128 = 80;
    let seq: u32 = 1;
    let msg = make_state_message(&env, &pc_id, new_a, new_b, seq);
    let sig_a = kp_a.sign(&msg);
    let sig_b = kp_b.sign(&msg);
    let sig_a_bytes = BytesN::from_array(&env, &sig_a.to_bytes());
    let sig_b_bytes = BytesN::from_array(&env, &sig_b.to_bytes());

    pc_client.submit_state(&addr_a, &new_a, &new_b, &seq, &sig_a_bytes, &sig_b_bytes);

    let info = pc_client.get_info();
    assert_eq!(info.balance_a, 70);
    assert_eq!(info.balance_b, 80);

    pc_client.close(&addr_b);

    assert_eq!(token_client.balance_of(&addr_a), 970);
    assert_eq!(token_client.balance_of(&addr_b), 1030);
}

#[test]
#[should_panic(expected = "invalid signature A")]
fn test_invalid_signature() {
    let (env, pc_id, addr_a, addr_b, kp_a, kp_b, token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);

    pc_client.deposit(&addr_a, &100);
    pc_client.deposit(&addr_b, &50);

    let new_a: i128 = 70;
    let new_b: i128 = 80;
    let seq: u32 = 1;
    let msg = make_state_message(&env, &pc_id, new_a, new_b, seq);
    // Sign with a wrong key
    let wrong_kp = Ed25519::from_seed(&[3u8; 32]);
    let sig_bad = wrong_kp.sign(&msg);
    let sig_bad_bytes = BytesN::from_array(&env, &sig_bad.to_bytes());
    let sig_b = kp_b.sign(&msg);
    let sig_b_bytes = BytesN::from_array(&env, &sig_b.to_bytes());

    pc_client.submit_state(&addr_a, &new_a, &new_b, &seq, &sig_bad_bytes, &sig_b_bytes);
}

#[test]
#[should_panic(expected = "sequence must increase")]
fn test_sequence_must_increase() {
    let (env, pc_id, addr_a, addr_b, kp_a, kp_b, token_id) = setup();
    let pc_client = PaymentChannelClient::new(&env, &pc_id);

    pc_client.deposit(&addr_a, &100);
    pc_client.deposit(&addr_b, &50);

    let new_a: i128 = 70;
    let new_b: i128 = 80;
    let seq: u32 = 1;
    let msg = make_state_message(&env, &pc_id, new_a, new_b, seq);
    let sig_a = kp_a.sign(&msg);
    let sig_b = kp_b.sign(&msg);
    let sig_a_bytes = BytesN::from_array(&env, &sig_a.to_bytes());
    let sig_b_bytes = BytesN::from_array(&env, &sig_b.to_bytes());
    pc_client.submit_state(&addr_a, &new_a, &new_b, &seq, &sig_a_bytes, &sig_b_bytes);

    // Try to submit an older sequence
    pc_client.submit_state(&addr_a, &new_a, &new_b, &0, &sig_a_bytes, &sig_b_bytes);
}

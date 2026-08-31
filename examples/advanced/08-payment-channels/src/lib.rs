#![no_std]

use soroban_sdk::contract::{contract, contractimpl};
use soroban_sdk::token;
use soroban_sdk::{Address, Bytes, BytesN, Env, Symbol};

const TOKEN: Symbol = Symbol::new("TOKEN");
const PUB_A: Symbol = Symbol::new("PUB_A");
const PUB_B: Symbol = Symbol::new("PUB_B");
const PART_A: Symbol = Symbol::new("PART_A");
const PART_B: Symbol = Symbol::new("PART_B");
const EXPIRY: Symbol = Symbol::new("EXPIRY");
const BAL_A: Symbol = Symbol::new("BAL_A");
const BAL_B: Symbol = Symbol::new("BAL_B");
const SEQ: Symbol = Symbol::new("SEQ");
const CLOSED: Symbol = Symbol::new("CLOSED");

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ChannelInfo {
    pub token: Address,
    pub participant_a: Address,
    pub participant_b: Address,
    pub balance_a: i128,
    pub balance_b: i128,
    pub sequence: u32,
    pub expiry: u64,
    pub is_closed: bool,
}

#[contract]
pub struct PaymentChannel;

fn get_participant_a(env: &Env) -> Address {
    env.storage().instance().get(&PART_A).unwrap()
}
fn get_participant_b(env: &Env) -> Address {
    env.storage().instance().get(&PART_B).unwrap()
}
fn get_token(env: &Env) -> Address {
    env.storage().instance().get(&TOKEN).unwrap()
}
fn get_balance_a(env: &Env) -> i128 {
    env.storage().instance().get(&BAL_A).unwrap()
}
fn get_balance_b(env: &Env) -> i128 {
    env.storage().instance().get(&BAL_B).unwrap()
}
fn get_sequence(env: &Env) -> u32 {
    env.storage().instance().get(&SEQ).unwrap()
}
fn get_expiry(env: &Env) -> u64 {
    env.storage().instance().get(&EXPIRY).unwrap()
}
fn is_closed(env: &Env) -> bool {
    env.storage().instance().get(&CLOSED).unwrap_or(false)
}

fn build_message(env: &Env, balance_a: &i128, balance_b: &i128, sequence: &u32) -> Bytes {
    let mut msg = Bytes::new(env);
    let contract_bytes = env.current_contract_address().as_contract().unwrap();
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

#[contractimpl]
impl PaymentChannel {
    pub fn init(env: Env, token: Address, pubkey_a: BytesN<32>, pubkey_b: BytesN<32>, expiry: u64) {
        assert!(!env.storage().instance().has(&TOKEN), "already initialized");
        let participant_a = Address::from_ed25519(&pubkey_a);
        let participant_b = Address::from_ed25519(&pubkey_b);
        env.storage().instance().set(&TOKEN, &token);
        env.storage().instance().set(&PUB_A, &pubkey_a);
        env.storage().instance().set(&PUB_B, &pubkey_b);
        env.storage().instance().set(&PART_A, &participant_a);
        env.storage().instance().set(&PART_B, &participant_b);
        env.storage().instance().set(&EXPIRY, &expiry);
        env.storage().instance().set(&BAL_A, &0_i128);
        env.storage().instance().set(&BAL_B, &0_i128);
        env.storage().instance().set(&SEQ, &0_u32);
        env.storage().instance().set(&CLOSED, &false);
    }

    pub fn deposit(env: Env, from: Address, amount: i128) {
        assert!(is_closed(&env) == false, "channel closed");
        assert!(env.ledger().timestamp() < get_expiry(&env), "channel expired");
        assert!(amount > 0, "amount must be positive");
        from.require_auth();
        let token = get_token(&env);
        token::Client::new(&env, &token).transfer(&from, &env.current_contract_address(), &amount);
        let participant_a = get_participant_a(&env);
        let participant_b = get_participant_b(&env);
        if from == participant_a {
            let bal = get_balance_a(&env);
            env.storage().instance().set(&BAL_A, &(bal + amount));
        } else if from == participant_b {
            let bal = get_balance_b(&env);
            env.storage().instance().set(&BAL_B, &(bal + amount));
        } else {
            panic!("not a participant");
        }
    }

    pub fn submit_state(
        env: Env,
        from: Address,
        new_balance_a: i128,
        new_balance_b: i128,
        sequence: u32,
        sig_a: BytesN<64>,
        sig_b: BytesN<64>,
    ) {
        assert!(!is_closed(&env), "channel closed");
        assert!(env.ledger().timestamp() < get_expiry(&env), "channel expired");
        let participant_a = get_participant_a(&env);
        let participant_b = get_participant_b(&env);
        if from != participant_a && from != participant_b {
            panic!("not a participant");
        }
        let stored_seq = get_sequence(&env);
        assert!(sequence > stored_seq, "sequence must increase");
        let cur_a = get_balance_a(&env);
        let cur_b = get_balance_b(&env);
        let total = cur_a + cur_b;
        assert!(new_balance_a >= 0 && new_balance_b >= 0, "negative balance");
        assert!(new_balance_a + new_balance_b == total, "balance mismatch");
        let msg = build_message(&env, &new_balance_a, &new_balance_b, &sequence);
        let pk_a: BytesN<32> = env.storage().instance().get(&PUB_A).unwrap();
        let pk_b: BytesN<32> = env.storage().instance().get(&PUB_B).unwrap();
        assert!(env.verify_sig_ed25519(&msg, &sig_a, &pk_a), "invalid signature A");
        assert!(env.verify_sig_ed25519(&msg, &sig_b, &pk_b), "invalid signature B");
        env.storage().instance().set(&BAL_A, &new_balance_a);
        env.storage().instance().set(&BAL_B, &new_balance_b);
        env.storage().instance().set(&SEQ, &sequence);
    }

    pub fn close(env: Env, from: Address) {
        assert!(!is_closed(&env), "channel closed");
        from.require_auth();
        let participant_a = get_participant_a(&env);
        let participant_b = get_participant_b(&env);
        if from != participant_a && from != participant_b {
            panic!("not a participant");
        }
        let token = get_token(&env);
        let balance_a = get_balance_a(&env);
        let balance_b = get_balance_b(&env);
        if balance_a > 0 {
            token::Client::new(&env, &token).transfer(&env.current_contract_address(), &participant_a, &balance_a);
        }
        if balance_b > 0 {
            token::Client::new(&env, &token).transfer(&env.current_contract_address(), &participant_b, &balance_b);
        }
        env.storage().instance().set(&CLOSED, &true);
        env.storage().instance().set(&BAL_A, &0_i128);
        env.storage().instance().set(&BAL_B, &0_i128);
    }

    pub fn get_info(env: Env) -> ChannelInfo {
        ChannelInfo {
            token: get_token(&env),
            participant_a: get_participant_a(&env),
            participant_b: get_participant_b(&env),
            balance_a: get_balance_a(&env),
            balance_b: get_balance_b(&env),
            sequence: get_sequence(&env),
            expiry: get_expiry(&env),
            is_closed: is_closed(&env),
        }
    }
}

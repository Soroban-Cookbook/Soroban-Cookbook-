#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, xdr::ToXdr, Address, Bytes, BytesN, Env, Symbol};
use soroban_sdk::token;

const TOKEN: Symbol = symbol_short!("TOKEN");
const PUB_A: Symbol = symbol_short!("PUB_A");
const PUB_B: Symbol = symbol_short!("PUB_B");
const PART_A: Symbol = symbol_short!("PART_A");
const PART_B: Symbol = symbol_short!("PART_B");
const EXPIRY: Symbol = symbol_short!("EXPIRY");
const BAL_A: Symbol = symbol_short!("BAL_A");
const BAL_B: Symbol = symbol_short!("BAL_B");
const SEQ: Symbol = symbol_short!("SEQ");
const CLOSED: Symbol = symbol_short!("CLOSED");

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
    env.storage().instance().get(&DataKey::ParticipantA).unwrap()
}
fn get_participant_b(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::ParticipantB).unwrap()
}
fn get_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}
fn get_balance_a(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::BalanceA).unwrap()
}
fn get_balance_b(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::BalanceB).unwrap()
}
fn get_sequence(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Sequence).unwrap()
}
fn get_expiry(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::Expiry).unwrap()
}
fn is_closed(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Closed)
        .unwrap_or(false)
}

fn build_message(env: &Env, balance_a: &i128, balance_b: &i128, sequence: &u32) -> Bytes {
    let mut msg = env.current_contract_address().to_xdr(env);
    msg.append(&Bytes::from_slice(env, &balance_a.to_be_bytes()));
    msg.append(&Bytes::from_slice(env, &balance_b.to_be_bytes()));
    msg.append(&Bytes::from_slice(env, &sequence.to_be_bytes()));
    msg
}

#[contractimpl]
impl PaymentChannel {
    pub fn init(
        env: Env,
        token: Address,
        participant_a: Address,
        participant_b: Address,
        pubkey_a: BytesN<32>,
        pubkey_b: BytesN<32>,
        expiry: u64,
    ) {
        assert!(!env.storage().instance().has(&TOKEN), "already initialized");
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
        assert!(!is_closed(&env), "channel closed");
        assert!(env.ledger().timestamp() < get_expiry(&env), "channel expired");
        assert!(amount > 0, "amount must be positive");
        from.require_auth();
        let token = get_token(&env);
        token::Client::new(&env, &token).transfer(&from, &env.current_contract_address(), &amount);
        let participant_a = get_participant_a(&env);
        let participant_b = get_participant_b(&env);
        if from == participant_a {
            let bal = get_balance_a(&env);
            env.storage()
                .instance()
                .set(&DataKey::BalanceA, &(bal + amount));
        } else if from == participant_b {
            let bal = get_balance_b(&env);
            env.storage()
                .instance()
                .set(&DataKey::BalanceB, &(bal + amount));
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
        env.crypto().ed25519_verify(&pk_a, &msg, &sig_a);
        env.crypto().ed25519_verify(&pk_b, &msg, &sig_b);
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
            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &participant_a,
                &balance_a,
            );
        }
        if balance_b > 0 {
            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &participant_b,
                &balance_b,
            );
        }
        env.storage().instance().set(&DataKey::Closed, &true);
        env.storage()
            .instance()
            .set(&DataKey::BalanceA, &0_i128);
        env.storage()
            .instance()
            .set(&DataKey::BalanceB, &0_i128);
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

#[cfg(test)]
mod test;

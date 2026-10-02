//! # Payment Channel
//!
//! A bidirectional payment channel between two participants.
//!
//! 1. `init` fixes the token, the two participants, the ed25519 keys each
//!    participant uses to sign off-chain state, and an expiry timestamp.
//! 2. Participants `deposit` tokens into the channel.
//! 3. Off-chain, they exchange state updates `(balance_a, balance_b, sequence)`
//!    signed by **both** keys. Any participant can `submit_state` the latest one.
//! 4. `close` pays each participant their balance and closes the channel.
//!
//! ## Security notes
//!
//! * Every state-changing entry point calls `require_auth` on the acting
//!   participant (`init` requires both).
//! * Signed messages are bound to this contract's address, so a state signed
//!   for one channel cannot be replayed on another.
//! * Sequence numbers must strictly increase, so older states cannot replace
//!   newer ones.
//! * A new state must redistribute exactly the deposited total, never more.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, xdr::ToXdr, Address, Bytes, BytesN,
    Env, Symbol,
};

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

fn assert_participant(env: &Env, from: &Address) {
    if *from != get_participant_a(env) && *from != get_participant_b(env) {
        panic!("not a participant");
    }
}

/// The message both participants sign off-chain:
/// `contract_address_xdr || balance_a (be) || balance_b (be) || sequence (be)`.
fn build_message(env: &Env, balance_a: i128, balance_b: i128, sequence: u32) -> Bytes {
    let mut msg = env.current_contract_address().to_xdr(env);
    msg.extend_from_array(&balance_a.to_be_bytes());
    msg.extend_from_array(&balance_b.to_be_bytes());
    msg.extend_from_array(&sequence.to_be_bytes());
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
        assert!(participant_a != participant_b, "participants must differ");
        assert!(expiry > env.ledger().timestamp(), "expiry in the past");
        participant_a.require_auth();
        participant_b.require_auth();

        let storage = env.storage().instance();
        storage.set(&TOKEN, &token);
        storage.set(&PUB_A, &pubkey_a);
        storage.set(&PUB_B, &pubkey_b);
        storage.set(&PART_A, &participant_a);
        storage.set(&PART_B, &participant_b);
        storage.set(&EXPIRY, &expiry);
        storage.set(&BAL_A, &0_i128);
        storage.set(&BAL_B, &0_i128);
        storage.set(&SEQ, &0_u32);
        storage.set(&CLOSED, &false);
    }

    pub fn deposit(env: Env, from: Address, amount: i128) {
        assert!(!is_closed(&env), "channel closed");
        assert!(
            env.ledger().timestamp() < get_expiry(&env),
            "channel expired"
        );
        assert!(amount > 0, "amount must be positive");
        from.require_auth();
        assert_participant(&env, &from);

        let key = if from == get_participant_a(&env) {
            BAL_A
        } else {
            BAL_B
        };
        let bal: i128 = env.storage().instance().get(&key).unwrap();
        let new_bal = bal.checked_add(amount).expect("balance overflow");

        token::Client::new(&env, &get_token(&env)).transfer(
            &from,
            env.current_contract_address(),
            &amount,
        );
        env.storage().instance().set(&key, &new_bal);
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
        assert!(
            env.ledger().timestamp() < get_expiry(&env),
            "channel expired"
        );
        from.require_auth();
        assert_participant(&env, &from);
        assert!(sequence > get_sequence(&env), "sequence must increase");
        assert!(new_balance_a >= 0 && new_balance_b >= 0, "negative balance");
        let total = get_balance_a(&env) + get_balance_b(&env);
        assert!(
            new_balance_a.checked_add(new_balance_b) == Some(total),
            "balance mismatch"
        );

        // Both signatures are required; `ed25519_verify` traps on failure.
        let msg = build_message(&env, new_balance_a, new_balance_b, sequence);
        let pk_a: BytesN<32> = env.storage().instance().get(&PUB_A).unwrap();
        let pk_b: BytesN<32> = env.storage().instance().get(&PUB_B).unwrap();
        env.crypto().ed25519_verify(&pk_a, &msg, &sig_a);
        env.crypto().ed25519_verify(&pk_b, &msg, &sig_b);

        let storage = env.storage().instance();
        storage.set(&BAL_A, &new_balance_a);
        storage.set(&BAL_B, &new_balance_b);
        storage.set(&SEQ, &sequence);
    }

    pub fn close(env: Env, from: Address) {
        assert!(!is_closed(&env), "channel closed");
        from.require_auth();
        assert_participant(&env, &from);

        let token = token::Client::new(&env, &get_token(&env));
        let this = env.current_contract_address();
        let balance_a = get_balance_a(&env);
        let balance_b = get_balance_b(&env);

        // Effects before interactions.
        let storage = env.storage().instance();
        storage.set(&CLOSED, &true);
        storage.set(&BAL_A, &0_i128);
        storage.set(&BAL_B, &0_i128);

        if balance_a > 0 {
            token.transfer(&this, get_participant_a(&env), &balance_a);
        }
        if balance_b > 0 {
            token.transfer(&this, get_participant_b(&env), &balance_b);
        }
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

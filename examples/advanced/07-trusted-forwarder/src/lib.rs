#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, xdr::ToXdr, Address, Bytes,
    BytesN, Env, Symbol, Val, Vec,
};

const CONTRACT_NS: Symbol = symbol_short!("fwd");
const ACTION_FORWARD: Symbol = symbol_short!("forward");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ForwarderError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidNonce = 3,
    InvalidSignature = 4,
    Expired = 5,
    InsufficientBalance = 6,
    ForwardFailed = 7,
    InvalidAmount = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetaTx {
    pub from: Address,
    pub to: Address,
    pub data: Bytes,
    pub nonce: u64,
    pub fee: i128,
    pub deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForwardEvent {
    pub from: Address,
    pub to: Address,
    pub nonce: u64,
    pub fee: i128,
    pub timestamp: u64,
}

#[contracttype]
pub enum ForwarderDataKey {
    Admin,
    Initialized,
    Fee,
    Balance(Address),
    Nonce(Address),
    UsedNonce(Address, u64),
    SignerPubkey(Address),
}

#[contract]
pub struct TrustedForwarder;

#[contractimpl]
impl TrustedForwarder {
    pub fn initialize(env: Env, admin: Address, fee: i128) -> Result<(), ForwarderError> {
        if env.storage().instance().has(&ForwarderDataKey::Initialized) {
            return Err(ForwarderError::AlreadyInitialized);
        }
        env.storage()
            .instance()
            .set(&ForwarderDataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&ForwarderDataKey::Admin, &admin);
        env.storage().instance().set(&ForwarderDataKey::Fee, &fee);
        Ok(())
    }

    pub fn set_fee(env: Env, fee: i128) -> Result<(), ForwarderError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ForwarderDataKey::Admin)
            .unwrap();
        admin.require_auth();
        env.storage().instance().set(&ForwarderDataKey::Fee, &fee);
        Ok(())
    }

    pub fn register_signer(
        env: Env,
        owner: Address,
        pubkey: BytesN<32>,
    ) -> Result<(), ForwarderError> {
        owner.require_auth();
        env.storage()
            .instance()
            .set(&ForwarderDataKey::SignerPubkey(owner), &pubkey);
        Ok(())
    }

    pub fn fund(env: Env, owner: Address, amount: i128) -> Result<(), ForwarderError> {
        owner.require_auth();
        if amount <= 0 {
            return Err(ForwarderError::InvalidAmount);
        }
        let current: i128 = env
            .storage()
            .instance()
            .get(&ForwarderDataKey::Balance(owner.clone()))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&ForwarderDataKey::Balance(owner), &(current + amount));
        Ok(())
    }

    pub fn withdraw(env: Env, owner: Address, amount: i128) -> Result<(), ForwarderError> {
        owner.require_auth();
        if amount <= 0 {
            return Err(ForwarderError::InvalidAmount);
        }
        let current: i128 = env
            .storage()
            .instance()
            .get(&ForwarderDataKey::Balance(owner.clone()))
            .unwrap_or(0);
        if current < amount {
            return Err(ForwarderError::InsufficientBalance);
        }
        env.storage()
            .instance()
            .set(&ForwarderDataKey::Balance(owner), &(current - amount));
        Ok(())
    }

    pub fn forward(
        env: Env,
        tx: MetaTx,
        signature: BytesN<64>,
        relayer: Address,
    ) -> Result<(), ForwarderError> {
        if !env.storage().instance().has(&ForwarderDataKey::Initialized) {
            return Err(ForwarderError::NotInitialized);
        }

        if tx.fee < 0 || tx.data.is_empty() {
            return Err(ForwarderError::InvalidAmount);
        }

        if tx.deadline < env.ledger().timestamp() {
            return Err(ForwarderError::Expired);
        }

        let nonce_key = ForwarderDataKey::Nonce(tx.from.clone());
        let current_nonce: u64 = env.storage().instance().get(&nonce_key).unwrap_or(0);
        if tx.nonce != current_nonce + 1 {
            return Err(ForwarderError::InvalidNonce);
        }

        let used_key = ForwarderDataKey::UsedNonce(tx.from.clone(), tx.nonce);
        if env.storage().instance().has(&used_key) {
            return Err(ForwarderError::InvalidNonce);
        }

        let pubkey: BytesN<32> = env
            .storage()
            .instance()
            .get(&ForwarderDataKey::SignerPubkey(tx.from.clone()))
            .ok_or(ForwarderError::InvalidSignature)?;

        let message = tx.clone().to_xdr(&env);
        let message_hash = env.crypto().sha256(&message).to_bytes();
        let message_bytes: Bytes = message_hash.into();
        env.crypto()
            .ed25519_verify(&pubkey, &message_bytes, &signature);

        let balance: i128 = env
            .storage()
            .instance()
            .get(&ForwarderDataKey::Balance(tx.from.clone()))
            .unwrap_or(0);
        if balance < tx.fee {
            return Err(ForwarderError::InsufficientBalance);
        }

        let relayer_balance: i128 = env
            .storage()
            .instance()
            .get(&ForwarderDataKey::Balance(relayer.clone()))
            .unwrap_or(0);
        env.storage().instance().set(
            &ForwarderDataKey::Balance(tx.from.clone()),
            &(balance - tx.fee),
        );
        env.storage().instance().set(
            &ForwarderDataKey::Balance(relayer.clone()),
            &(relayer_balance + tx.fee),
        );

        env.storage().instance().set(&nonce_key, &tx.nonce);
        env.storage().instance().set(&used_key, &true);

        let forward_fn = Symbol::new(&env, "forwarded_call");
        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(tx.from.clone().to_val());
        args.push_back(tx.data.clone().to_val());
        let _result: Val = env.invoke_contract(&tx.to, &forward_fn, args);

        #[allow(deprecated)]
        env.events().publish(
            (CONTRACT_NS, ACTION_FORWARD, tx.from.clone(), tx.to.clone()),
            ForwardEvent {
                from: tx.from,
                to: tx.to,
                nonce: tx.nonce,
                fee: tx.fee,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    pub fn next_nonce(env: Env, owner: Address) -> u64 {
        let key = ForwarderDataKey::Nonce(owner);
        env.storage().instance().get(&key).unwrap_or(0) + 1
    }

    pub fn balance(env: Env, owner: Address) -> i128 {
        env.storage()
            .instance()
            .get(&ForwarderDataKey::Balance(owner))
            .unwrap_or(0)
    }

    pub fn get_fee(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&ForwarderDataKey::Fee)
            .unwrap_or(0)
    }
}

// Demo recipient used by host tests. Gated off wasm so its `initialize` export
// does not collide with TrustedForwarder (SDK 26 removed contractimpl export=false).
#[cfg(any(test, not(target_family = "wasm")))]
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RecipientError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    UnauthorizedCaller = 3,
}

#[cfg(any(test, not(target_family = "wasm")))]
#[contracttype]
pub enum RecipientDataKey {
    TrustedForwarder,
    Initialized,
    StoredValue,
    LastSender,
}

#[cfg(any(test, not(target_family = "wasm")))]
#[contract]
pub struct SimpleRecipient;

#[cfg(any(test, not(target_family = "wasm")))]
#[contractimpl]
impl SimpleRecipient {
    pub fn initialize(env: Env, forwarder: Address) -> Result<(), RecipientError> {
        if env.storage().instance().has(&RecipientDataKey::Initialized) {
            return Err(RecipientError::AlreadyInitialized);
        }
        env.storage()
            .instance()
            .set(&RecipientDataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&RecipientDataKey::TrustedForwarder, &forwarder);
        Ok(())
    }

    pub fn forwarded_call(env: Env, sender: Address, data: Bytes) -> Result<(), RecipientError> {
        if !env.storage().instance().has(&RecipientDataKey::Initialized) {
            return Err(RecipientError::NotInitialized);
        }

        let trusted: Address = env
            .storage()
            .instance()
            .get(&RecipientDataKey::TrustedForwarder)
            .unwrap();
        trusted.require_auth();

        env.storage()
            .instance()
            .set(&RecipientDataKey::LastSender, &sender);
        env.storage()
            .instance()
            .set(&RecipientDataKey::StoredValue, &data);

        Ok(())
    }

    pub fn get_stored_value(env: Env) -> Bytes {
        env.storage()
            .instance()
            .get(&RecipientDataKey::StoredValue)
            .unwrap_or(Bytes::new(&env))
    }

    pub fn get_last_sender(env: Env) -> Option<Address> {
        env.storage().instance().get(&RecipientDataKey::LastSender)
    }
}

mod test;

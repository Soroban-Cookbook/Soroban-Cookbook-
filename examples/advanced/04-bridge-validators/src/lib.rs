#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Map, Symbol, Vec,
    crypto::Hash,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    InvalidThreshold = 3,
    ValidatorExists = 4,
    ValidatorNotFound = 5,
    DuplicateSignature = 6,
    ThresholdNotMet = 7,
    InvalidSignature = 8,
    MessageAlreadyProcessed = 9,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Threshold,
    ValidatorCount,
    Validator(BytesN<32>),
    Processed(BytesN<32>), // message hash -> bool
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Validator {
    pub pubkey: BytesN<32>,
    pub power: u32,
    pub active: bool,
}

#[contract]
pub struct BridgeValidators;

#[contractimpl]
impl BridgeValidators {
    pub fn init(env: Env, admin: Address, threshold: u32) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        if threshold == 0 {
            return Err(Error::InvalidThreshold);
        }
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
        env.storage().instance().set(&DataKey::ValidatorCount, &0u32);
        
        Ok(())
    }

    pub fn add_validator(env: Env, pubkey: BytesN<32>, power: u32) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Validator(pubkey.clone())) {
            return Err(Error::ValidatorExists);
        }

        let validator = Validator {
            pubkey: pubkey.clone(),
            power,
            active: true,
        };

        env.storage().persistent().set(&DataKey::Validator(pubkey.clone()), &validator);
        
        let mut count: u32 = env.storage().instance().get(&DataKey::ValidatorCount).unwrap();
        count += 1;
        env.storage().instance().set(&DataKey::ValidatorCount, &count);

        Ok(())
    }

    pub fn remove_validator(env: Env, pubkey: BytesN<32>) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if !env.storage().persistent().has(&DataKey::Validator(pubkey.clone())) {
            return Err(Error::ValidatorNotFound);
        }

        env.storage().persistent().remove(&DataKey::Validator(pubkey.clone()));
        
        let mut count: u32 = env.storage().instance().get(&DataKey::ValidatorCount).unwrap();
        count -= 1;
        env.storage().instance().set(&DataKey::ValidatorCount, &count);

        Ok(())
    }

    pub fn slash_validator(env: Env, pubkey: BytesN<32>) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut validator: Validator = env
            .storage()
            .persistent()
            .get(&DataKey::Validator(pubkey.clone()))
            .ok_or(Error::ValidatorNotFound)?;

        validator.active = false;
        validator.power = 0;
        env.storage().persistent().set(&DataKey::Validator(pubkey), &validator);

        Ok(())
    }

    pub fn set_threshold(env: Env, new_threshold: u32) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if new_threshold == 0 {
            return Err(Error::InvalidThreshold);
        }

        env.storage().instance().set(&DataKey::Threshold, &new_threshold);
        Ok(())
    }

    pub fn process_message(
        env: Env,
        message_hash: BytesN<32>,
        signatures: Map<BytesN<32>, BytesN<64>>, // map of pubkey to signature
    ) -> Result<bool, Error> {
        if env.storage().persistent().has(&DataKey::Processed(message_hash.clone())) {
            return Err(Error::MessageAlreadyProcessed);
        }

        let threshold: u32 = env.storage().instance().get(&DataKey::Threshold).unwrap();
        let mut total_power = 0;

        for (pubkey, sig) in signatures.iter() {
            let validator: Validator = env
                .storage()
                .persistent()
                .get(&DataKey::Validator(pubkey.clone()))
                .ok_or(Error::ValidatorNotFound)?;
                
            if !validator.active {
                continue;
            }

            // Verify signature. Using env.crypto().ed25519_verify
            env.crypto().ed25519_verify(&pubkey, &message_hash.clone().into(), &sig);
            
            total_power += validator.power;
        }

        if total_power < threshold {
            return Err(Error::ThresholdNotMet);
        }

        env.storage().persistent().set(&DataKey::Processed(message_hash.clone()), &true);
        
        // Emit event
        env.events().publish((Symbol::new(&env, "processed"), message_hash), ());

        Ok(true)
    }
}

mod test;

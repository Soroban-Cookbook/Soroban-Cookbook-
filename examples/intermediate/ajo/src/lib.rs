//! # Ajo Template Contract
//!
//! Rotating savings pool template deployed by [`ajo_factory::AjoFactory`].

#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AjoError {
    AlreadyInitialized = 1,
}

#[contract]
pub struct Ajo;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AjoDataKey {
    Amount,
    MaxMembers,
    Creator,
}

#[contractimpl]
impl Ajo {
    /// Initialize a new Ajo instance.
    pub fn initialize(
        env: Env,
        amount: i128,
        max_members: u32,
        creator: Address,
    ) -> Result<(), AjoError> {
        if env.storage().instance().has(&AjoDataKey::Creator) {
            return Err(AjoError::AlreadyInitialized);
        }

        env.storage().instance().set(&AjoDataKey::Amount, &amount);
        env.storage()
            .instance()
            .set(&AjoDataKey::MaxMembers, &max_members);
        env.storage().instance().set(&AjoDataKey::Creator, &creator);

        Ok(())
    }

    pub fn get_creator(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&AjoDataKey::Creator)
            .expect("Not initialized")
    }

    pub fn get_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&AjoDataKey::Amount)
            .expect("Not initialized")
    }
}

#[cfg(test)]
mod test;

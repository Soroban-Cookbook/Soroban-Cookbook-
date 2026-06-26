#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SecurityError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidCaller = 8,
}

#[contracttype]
pub enum FacetKey {
    Diamond,
}

#[contract]
pub struct FacetMultiplierContract;

#[contractimpl]
impl FacetMultiplierContract {
    /// Initialize the Multiplier Facet with the Diamond Proxy address
    pub fn init_multiplier(env: Env, diamond: Address) -> Result<(), SecurityError> {
        if env.storage().instance().has(&FacetKey::Diamond) {
            return Err(SecurityError::AlreadyInitialized);
        }
        env.storage().instance().set(&FacetKey::Diamond, &diamond);
        Ok(())
    }

    /// Verify support for registered methods
    pub fn support(_env: Env, functions: Vec<Symbol>) -> bool {
        let expected = symbol_short!("multiply");
        for func in functions.iter() {
            if func == expected {
                return true;
            }
        }
        false
    }

    /// Sample Multiply method. Can only be invoked via Diamond Proxy.
    pub fn multiply(env: Env, caller: Address, a: i128, b: i128) -> Result<i128, SecurityError> {
        caller.require_auth();

        let diamond: Address = env
            .storage()
            .instance()
            .get(&FacetKey::Diamond)
            .ok_or(SecurityError::NotInitialized)?;

        if caller != diamond {
            return Err(SecurityError::InvalidCaller);
        }

        // Write directly to Facet's own storage
        let key = symbol_short!("count");
        let count: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        let new_count = count + 1;
        env.storage().persistent().set(&key, &new_count);

        Ok(a * b)
    }
}

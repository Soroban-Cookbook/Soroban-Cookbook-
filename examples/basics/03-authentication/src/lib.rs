#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, contracterror, symbol_short, Address, Env, Symbol, Vec};

#[contract]
pub struct AuthContract;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuthError {
    Unauthorized = 1,
    NotAdmin = 2,
    AlreadyInitialized = 3,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance(Address),
    Allowance(Address, Address),
}

#[contractimpl]
impl AuthContract {
    /// Basic authentication check
    pub fn check_auth(_env: Env, user: Address) -> bool {
        user.require_auth();
        true
    }

    /// Initialize contract with admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), AuthError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AuthError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Admin-only function
    pub fn admin_action(env: Env, admin: Address, value: u32) -> Result<u32, AuthError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AuthError::NotAdmin)?;
        
        if admin != stored_admin {
            return Err(AuthError::NotAdmin);
        }
        
        Ok(value * 2)
    }

    /// Transfer with authentication
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
        from.require_auth();
        
        let from_balance: i128 = env.storage().persistent().get(&DataKey::Balance(from.clone())).unwrap_or(0);
        let to_balance: i128 = env.storage().persistent().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        
        env.storage().persistent().set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage().persistent().set(&DataKey::Balance(to), &(to_balance + amount));
        
        Ok(())
    }

    /// Set balance (admin only)
    pub fn set_balance(env: Env, admin: Address, user: Address, amount: i128) -> Result<(), AuthError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AuthError::NotAdmin)?;
        
        if admin != stored_admin {
            return Err(AuthError::NotAdmin);
        }
        
        env.storage().persistent().set(&DataKey::Balance(user), &amount);
        Ok(())
    }

    /// Get balance
    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Balance(user)).unwrap_or(0)
    }

    /// Approve allowance
    pub fn approve(env: Env, from: Address, spender: Address, amount: i128) -> Result<(), AuthError> {
        from.require_auth();
        env.storage().persistent().set(&DataKey::Allowance(from, spender), &amount);
        Ok(())
    }

    /// Transfer from allowance
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
        spender.require_auth();
        
        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(0);
        
        if allowance < amount {
            return Err(AuthError::Unauthorized);
        }
        
        let from_balance: i128 = env.storage().persistent().get(&DataKey::Balance(from.clone())).unwrap_or(0);
        let to_balance: i128 = env.storage().persistent().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        
        env.storage().persistent().set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage().persistent().set(&DataKey::Balance(to), &(to_balance + amount));
        env.storage().persistent().set(&DataKey::Allowance(from, spender), &(allowance - amount));
        
        Ok(())
    }

    /// Multi-signature operation
    pub fn multi_sig_action(env: Env, signers: Vec<Address>, value: u32) -> u32 {
        for signer in signers.iter() {
            signer.require_auth();
        }
        value + signers.len()
    }

    /// Emit event with authentication
    pub fn emit_event(env: Env, user: Address, message: Symbol) {
        user.require_auth();
        env.events().publish((symbol_short!("event"), user), message);
    }
}

mod test;

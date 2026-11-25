//! # Treasury Contract
//!
//! A simple treasury contract that holds tokens and allows authorized
//! withdrawals. Demonstrates being called by another contract.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TreasuryError {
    NotAuthorized = 1,
    InsufficientBalance = 2,
    AlreadyInitialized = 3,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance,
    TotalWithdrawn,
}

#[contract]
pub struct Treasury;

#[contractimpl]
impl Treasury {
    /// Initialize the treasury with an admin address
    pub fn initialize(env: Env, admin: Address) -> Result<(), TreasuryError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(TreasuryError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Balance, &0i128);
        env.storage().instance().set(&DataKey::TotalWithdrawn, &0i128);

        Ok(())
    }

    /// Deposit funds into the treasury
    pub fn deposit(env: Env, depositor: Address, amount: i128) -> Result<(), TreasuryError> {
        depositor.require_auth();

        let current_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Balance)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::Balance, &(current_balance + amount));

        env.events().publish(
            (soroban_sdk::symbol_short!("deposit"),),
            (depositor, amount),
        );

        Ok(())
    }

    /// Withdraw funds from treasury (requires admin authorization)
    pub fn withdraw(
        env: Env,
        admin: Address,
        recipient: Address,
        amount: i128,
    ) -> Result<(), TreasuryError> {
        // Verify the caller is the admin
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(TreasuryError::NotAuthorized)?;

        if admin != stored_admin {
            return Err(TreasuryError::NotAuthorized);
        }

        admin.require_auth();

        let current_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Balance)
            .unwrap_or(0);

        if current_balance < amount {
            return Err(TreasuryError::InsufficientBalance);
        }

        let total_withdrawn: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWithdrawn)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::Balance, &(current_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::TotalWithdrawn, &(total_withdrawn + amount));

        env.events().publish(
            (soroban_sdk::symbol_short!("withdraw"),),
            (recipient, amount),
        );

        Ok(())
    }

    /// Get current balance
    pub fn balance(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Balance)
            .unwrap_or(0)
    }

    /// Get total amount withdrawn
    pub fn total_withdrawn(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalWithdrawn)
            .unwrap_or(0)
    }

    /// Get admin address
    pub fn admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }
}

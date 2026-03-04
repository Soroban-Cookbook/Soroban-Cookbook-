//! # Authentication Patterns Contract
//!
//! Demonstrates core address-authentication patterns using Soroban's
//! `require_auth()` function.
//!
//! ## What `require_auth()` does
//!
//! - Verifies that the given `Address` has signed / authorized this invocation.
//! - Works for user accounts (ed25519 keypairs) and contract addresses alike.
//! - Protects against replays -- the host records the nonce automatically.
//! - Is essential for any state-mutating operation in multi-user contracts.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, Env, Symbol,
    Vec,
};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

/// Keys used in contract storage.
///
/// * `Admin`              -- the privileged admin address (instance storage).
/// * `Balance(Address)`   -- per-account token balance (persistent storage).
/// * `Allowance(from, spender)` -- spend allowance (persistent storage).
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance(Address),
    Allowance(Address, Address),
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuthError {
    /// The caller does not have the required permission.
    Unauthorized = 1,
    /// The operation requires an admin that has not been set, or the provided
    /// address does not match the stored admin.
    NotAdmin = 2,
    /// `initialize()` has already been called.
    AlreadyInitialized = 3,
    /// The sender does not have enough balance to complete the transfer.
    InsufficientBalance = 4,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AuthContract;

#[contractimpl]
impl AuthContract {
    // ==================== INITIALIZATION ====================

    /// Initializes the contract with the given admin address.
    ///
    /// Must be called exactly once. Subsequent calls return
    /// `AlreadyInitialized` so the admin cannot be hijacked after deployment.
    pub fn initialize(env: Env, admin: Address) -> Result<(), AuthError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AuthError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Returns the current admin address, if set.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    // ==================== ADMIN-ONLY PATTERNS ====================

    /// Demonstrates an admin-only gate.
    ///
    /// Pattern:
    /// 1. `require_auth` on the caller.
    /// 2. Load the stored admin and compare -- prevents anyone from passing a
    ///    random `Address` that they happen to control.
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

    /// Admin-only balance setter (e.g. for minting in tests or bootstrapping).
    pub fn set_balance(
        env: Env,
        admin: Address,
        user: Address,
        amount: i128,
    ) -> Result<(), AuthError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AuthError::NotAdmin)?;

        if admin != stored_admin {
            return Err(AuthError::NotAdmin);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(user), &amount);
        Ok(())
    }

    // ==================== SINGLE-ADDRESS AUTH PATTERN ====================

    /// Transfer tokens from `from` to `to`.
    ///
    /// Security:
    /// - `from.require_auth()` ensures only the owner can debit their account.
    /// - The balance check prevents the sender from going negative.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
        from.require_auth();

        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        if amount <= 0 || from_balance < amount {
            return Err(AuthError::InsufficientBalance);
        }

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_balance + amount));

        Ok(())
    }

    // ==================== ALLOWANCE PATTERN ====================

    /// Approve `spender` to transfer up to `amount` on behalf of `from`.
    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
    ) -> Result<(), AuthError> {
        from.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(from, spender), &amount);
        Ok(())
    }

    /// Transfer `amount` from `from` to `to` using the `spender` allowance.
    ///
    /// Security:
    /// - `spender.require_auth()` -- the spender must authorize the spend.
    /// - Allowance is checked BEFORE modifying balances.
    /// - `from_balance` is checked so the sender cannot go negative.
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), AuthError> {
        spender.require_auth();

        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(0);

        if allowance < amount {
            return Err(AuthError::Unauthorized);
        }

        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        if from_balance < amount {
            return Err(AuthError::InsufficientBalance);
        }

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_balance + amount));
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(from, spender), &(allowance - amount));

        Ok(())
    }

    // ==================== QUERY ====================

    /// Returns the balance for `user` (0 if never set).
    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(user))
            .unwrap_or(0)
    }

    // ==================== MULTI-SIG PATTERN ====================

    /// Demonstrates N-of-N multi-sig: every signer in the list must authorize.
    ///
    /// The Soroban host collects authorizations before invoking the contract, so
    /// order does not matter. This function simply iterates the list calling
    /// `require_auth()` on each -- the host verifies all of them atomically.
    pub fn multi_sig_action(_env: Env, signers: Vec<Address>, value: u32) -> u32 {
        for signer in signers.iter() {
            signer.require_auth();
        }
        value + signers.len()
    }

    // ==================== SECURE OPERATION ====================

    /// Demonstrates authenticated operation with typed error return.
    ///
    /// Pattern: auth first, then validate, then execute.
    pub fn secure_operation(
        env: Env,
        user: Address,
        operation: Symbol,
    ) -> Result<Vec<Symbol>, AuthError> {
        user.require_auth();

        if operation == symbol_short!("invalid") {
            return Err(AuthError::Unauthorized);
        }

        Ok(vec![&env, symbol_short!("success"), operation])
    }

    // ==================== EVENTS WITH AUTH ====================

    /// Emit a named event after verifying the caller.
    pub fn emit_event(env: Env, user: Address, message: Symbol) {
        user.require_auth();
        env.events()
            .publish((symbol_short!("event"), user), message);
    }
}

#[cfg(test)]
#[cfg(test)]
mod test;

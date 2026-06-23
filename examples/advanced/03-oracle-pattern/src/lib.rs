//! # Basic Oracle Pattern
//!
//! Demonstrates how external data can be submitted by an authorized updater,
//! stored with a timestamp, and queried with freshness validation.
//!
//! ## Trust Model
//!
//! - This is a **single-source oracle**: the contract trusts whoever is set as
//!   the authorized updater. There is no decentralized consensus or multi-signer
//!   aggregation.
//! - Freshness checks reduce stale-data risk but do **not** prove correctness
//!   of the submitted value.
//! - For production use, consider extending with multi-signer validation,
//!   multiple independent sources, or signed payload verification.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum OracleError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller is not the authorized updater.
    NotAuthorized = 3,
    /// No data has been submitted yet.
    NoData = 4,
    /// The stored data is older than the configured max age.
    StaleData = 5,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The admin who can rotate the updater.
    Admin,
    /// The address authorized to submit data.
    Updater,
    /// The latest submitted value.
    Value,
    /// Ledger timestamp of the last submission.
    Timestamp,
    /// Maximum age (seconds) before data is considered stale.
    MaxAge,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const CONTRACT_NS: Symbol = symbol_short!("oracle");
const ACTION_SUBMIT: Symbol = symbol_short!("submit");
const ACTION_ADMIN: Symbol = symbol_short!("admin");

/// Event payload for data submissions.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmitEventData {
    pub value: i128,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct OracleContract;

#[contractimpl]
impl OracleContract {
    /// Initialize the oracle with an admin, an authorized updater, and a
    /// maximum data age (in seconds).
    pub fn initialize(
        env: Env,
        admin: Address,
        updater: Address,
        max_age: u64,
    ) -> Result<(), OracleError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(OracleError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Updater, &updater);
        env.storage().instance().set(&DataKey::MaxAge, &max_age);
        Ok(())
    }

    /// Submit a new data value. Only the authorized updater may call this.
    pub fn submit(env: Env, updater: Address, value: i128) -> Result<(), OracleError> {
        let stored_updater: Address = env
            .storage()
            .instance()
            .get(&DataKey::Updater)
            .ok_or(OracleError::NotInitialized)?;

        if updater != stored_updater {
            return Err(OracleError::NotAuthorized);
        }
        updater.require_auth();

        let now = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Value, &value);
        env.storage().instance().set(&DataKey::Timestamp, &now);

        env.events().publish(
            (CONTRACT_NS, ACTION_SUBMIT, updater),
            SubmitEventData {
                value,
                timestamp: now,
            },
        );

        Ok(())
    }

    /// Return the latest value regardless of freshness.
    pub fn get_value(env: Env) -> Result<i128, OracleError> {
        env.storage()
            .instance()
            .get(&DataKey::Value)
            .ok_or(OracleError::NoData)
    }

    /// Return the timestamp of the latest submission.
    pub fn get_timestamp(env: Env) -> Result<u64, OracleError> {
        env.storage()
            .instance()
            .get(&DataKey::Timestamp)
            .ok_or(OracleError::NoData)
    }

    /// Return `true` if the stored data is within the configured max age.
    pub fn is_fresh(env: Env) -> Result<bool, OracleError> {
        let ts: u64 = env
            .storage()
            .instance()
            .get(&DataKey::Timestamp)
            .ok_or(OracleError::NoData)?;
        let max_age: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MaxAge)
            .ok_or(OracleError::NotInitialized)?;
        let age = env.ledger().timestamp().saturating_sub(ts);
        Ok(age <= max_age)
    }

    /// Return the latest value only if it is fresh; otherwise error.
    pub fn get_value_strict(env: Env) -> Result<i128, OracleError> {
        let ts: u64 = env
            .storage()
            .instance()
            .get(&DataKey::Timestamp)
            .ok_or(OracleError::NoData)?;
        let max_age: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MaxAge)
            .ok_or(OracleError::NotInitialized)?;
        let age = env.ledger().timestamp().saturating_sub(ts);
        if age > max_age {
            return Err(OracleError::StaleData);
        }
        env.storage()
            .instance()
            .get(&DataKey::Value)
            .ok_or(OracleError::NoData)
    }

    /// Rotate the authorized updater. Only the admin may call this.
    pub fn set_updater(env: Env, new_updater: Address) -> Result<(), OracleError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(OracleError::NotInitialized)?;
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Updater, &new_updater);

        env.events()
            .publish((CONTRACT_NS, ACTION_ADMIN, admin), symbol_short!("rotate"));

        Ok(())
    }
}

#[cfg(test)]
mod test;

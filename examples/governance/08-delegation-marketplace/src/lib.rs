//! # Delegation Marketplace
//!
//! A marketplace for renting and listing voting power. Token holders who
//! do not wish to participate in governance can list their voting power for
//! rent; other accounts can rent that power and use it to influence proposals.
//!
//! ## Key Concepts
//!
//! - **Offer** – a delegation offer created by a `delegator` who lists a
//!   `voting_power` amount at a `price_per_unit` fee.
//! - **Renting** – a `renter` pays `units * price_per_unit` tokens to the
//!   delegator and receives `units` of delegated voting power until the
//!   delegation expires.
//! - **Incentive** – delegators earn fees; renters gain voting influence
//!   without holding governance tokens themselves.
//!
//! ## Storage Layout
//!
//! | Key | Type | Storage |
//! |-----|------|---------|
//! | `Offer(delegator)` | `DelegationOffer` | Persistent |
//! | `Delegation(renter, delegator)` | `ActiveDelegation` | Persistent |
//! | `Balance(address)` | `i128` | Persistent |
//!
//! ## Patterns reused from basics
//!
//! - Auth: `address.require_auth()` before every state-mutating call
//!   (see `examples/basics/03-authentication`).
//! - Events: `(namespace, action, primary, secondary)` topic layout
//!   (see `examples/basics/04-events`).

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MarketplaceError {
    /// An offer for this delegator already exists.
    OfferAlreadyExists = 1,
    /// No offer found for the given delegator.
    OfferNotFound = 2,
    /// The requested units exceed what the offer has available.
    InsufficientVotingPower = 3,
    /// The renter does not have enough token balance to pay the fee.
    InsufficientBalance = 4,
    /// A delegation from this renter to this delegator already exists.
    DelegationAlreadyExists = 5,
    /// No active delegation found.
    DelegationNotFound = 6,
    /// price_per_unit or voting_power must be greater than zero.
    InvalidAmount = 7,
    /// The delegation has not yet expired.
    DelegationNotExpired = 8,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// An open offer to rent out voting power.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationOffer {
    /// Owner of the voting power.
    pub delegator: Address,
    /// Total units of voting power available for rent.
    pub voting_power: u64,
    /// Fee charged per unit of voting power per rental.
    pub price_per_unit: i128,
}

/// An active delegation from a renter to a delegator.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveDelegation {
    /// Address that rented the voting power.
    pub renter: Address,
    /// Address that provided the voting power.
    pub delegator: Address,
    /// Units of voting power rented.
    pub units: u64,
    /// Ledger timestamp at which this delegation expires.
    pub expires_at: u64,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Delegation offer keyed by the delegator address.
    Offer(Address),
    /// Active delegation keyed by (renter, delegator).
    Delegation(Address, Address),
    /// Token balance for an address (simplified; real impl uses SEP-41).
    Balance(Address),
}

// ---------------------------------------------------------------------------
// Event topics
// ---------------------------------------------------------------------------

const NS: Symbol = symbol_short!("deleg_mkt");
const EVT_LIST: Symbol = symbol_short!("listed");
const EVT_CANCEL: Symbol = symbol_short!("cancel");
const EVT_RENT: Symbol = symbol_short!("rented");
const EVT_EXPIRE: Symbol = symbol_short!("expired");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct DelegationMarketplace;

#[contractimpl]
impl DelegationMarketplace {
    // -----------------------------------------------------------------------
    // Offer management
    // -----------------------------------------------------------------------

    /// List `voting_power` units at `price_per_unit` fee.
    ///
    /// The delegator must authorize this call. Only one open offer per
    /// delegator is allowed at a time.
    pub fn list_offer(
        env: Env,
        delegator: Address,
        voting_power: u64,
        price_per_unit: i128,
    ) -> Result<(), MarketplaceError> {
        delegator.require_auth();

        if voting_power == 0 || price_per_unit <= 0 {
            return Err(MarketplaceError::InvalidAmount);
        }

        let key = DataKey::Offer(delegator.clone());
        if env.storage().persistent().has(&key) {
            return Err(MarketplaceError::OfferAlreadyExists);
        }

        let offer = DelegationOffer {
            delegator: delegator.clone(),
            voting_power,
            price_per_unit,
        };
        env.storage().persistent().set(&key, &offer);

        env.events().publish(
            (NS, EVT_LIST, delegator),
            (voting_power, price_per_unit),
        );

        Ok(())
    }

    /// Cancel an open offer. Any remaining units are reclaimed.
    ///
    /// Only the delegator who created the offer can cancel it.
    pub fn cancel_offer(
        env: Env,
        delegator: Address,
    ) -> Result<(), MarketplaceError> {
        delegator.require_auth();

        let key = DataKey::Offer(delegator.clone());
        if !env.storage().persistent().has(&key) {
            return Err(MarketplaceError::OfferNotFound);
        }

        env.storage().persistent().remove(&key);

        env.events().publish((NS, EVT_CANCEL, delegator), ());

        Ok(())
    }

    /// Return the open offer for `delegator`, or `None` if not found.
    pub fn get_offer(env: Env, delegator: Address) -> Option<DelegationOffer> {
        env.storage()
            .persistent()
            .get(&DataKey::Offer(delegator))
    }

    // -----------------------------------------------------------------------
    // Renting
    // -----------------------------------------------------------------------

    /// Rent `units` of voting power from `delegator` for `duration` seconds.
    ///
    /// The renter pays `units * price_per_unit` tokens to the delegator.
    /// The offer's available `voting_power` is reduced by `units`.
    /// An `ActiveDelegation` record is stored for the renter.
    pub fn rent_voting_power(
        env: Env,
        renter: Address,
        delegator: Address,
        units: u64,
        duration: u64,
    ) -> Result<(), MarketplaceError> {
        renter.require_auth();

        if units == 0 || duration == 0 {
            return Err(MarketplaceError::InvalidAmount);
        }

        // Load offer
        let offer_key = DataKey::Offer(delegator.clone());
        let mut offer: DelegationOffer = env
            .storage()
            .persistent()
            .get(&offer_key)
            .ok_or(MarketplaceError::OfferNotFound)?;

        if offer.voting_power < units {
            return Err(MarketplaceError::InsufficientVotingPower);
        }

        // Prevent duplicate delegation
        let delegation_key = DataKey::Delegation(renter.clone(), delegator.clone());
        if env.storage().persistent().has(&delegation_key) {
            return Err(MarketplaceError::DelegationAlreadyExists);
        }

        // Calculate and deduct fee
        let fee: i128 = (units as i128)
            .checked_mul(offer.price_per_unit)
            .unwrap_or(i128::MAX);

        let renter_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(renter.clone()))
            .unwrap_or(0);

        if renter_balance < fee {
            return Err(MarketplaceError::InsufficientBalance);
        }

        let delegator_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(delegator.clone()))
            .unwrap_or(0);

        // Transfer fee: renter → delegator
        env.storage()
            .persistent()
            .set(&DataKey::Balance(renter.clone()), &(renter_balance - fee));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(delegator.clone()), &(delegator_balance + fee));

        // Reduce available voting power in offer
        offer.voting_power -= units;
        if offer.voting_power == 0 {
            env.storage().persistent().remove(&offer_key);
        } else {
            env.storage().persistent().set(&offer_key, &offer);
        }

        // Record delegation
        let expires_at = env.ledger().timestamp() + duration;
        let delegation = ActiveDelegation {
            renter: renter.clone(),
            delegator: delegator.clone(),
            units,
            expires_at,
        };
        env.storage().persistent().set(&delegation_key, &delegation);

        env.events().publish(
            (NS, EVT_RENT, renter, delegator),
            (units, fee, expires_at),
        );

        Ok(())
    }

    /// Return the active delegation for (`renter`, `delegator`), or `None`.
    pub fn get_delegation(
        env: Env,
        renter: Address,
        delegator: Address,
    ) -> Option<ActiveDelegation> {
        env.storage()
            .persistent()
            .get(&DataKey::Delegation(renter, delegator))
    }

    // -----------------------------------------------------------------------
    // Expiry
    // -----------------------------------------------------------------------

    /// Remove an expired delegation and return the voting power units to the
    /// delegator's offer (if the offer still exists).
    ///
    /// Anyone can call this to clean up expired delegations.
    pub fn expire_delegation(
        env: Env,
        renter: Address,
        delegator: Address,
    ) -> Result<(), MarketplaceError> {
        let delegation_key = DataKey::Delegation(renter.clone(), delegator.clone());
        let delegation: ActiveDelegation = env
            .storage()
            .persistent()
            .get(&delegation_key)
            .ok_or(MarketplaceError::DelegationNotFound)?;

        if env.ledger().timestamp() < delegation.expires_at {
            return Err(MarketplaceError::DelegationNotExpired);
        }

        let returned_units = delegation.units;

        // Remove the delegation record
        env.storage().persistent().remove(&delegation_key);

        // Return voting power to offer if it still exists
        let offer_key = DataKey::Offer(delegator.clone());
        if let Some(mut offer) = env
            .storage()
            .persistent()
            .get::<DataKey, DelegationOffer>(&offer_key)
        {
            offer.voting_power += returned_units;
            env.storage().persistent().set(&offer_key, &offer);
        }

        env.events().publish(
            (NS, EVT_EXPIRE, renter, delegator),
            returned_units,
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Balance helpers (simplified token bookkeeping for tests)
    // -----------------------------------------------------------------------

    /// Credit `amount` tokens to `account` (test helper / mint substitute).
    pub fn fund_account(env: Env, account: Address, amount: i128) {
        let current: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(account.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(account), &(current + amount));
    }

    /// Return the token balance for `account`.
    pub fn get_balance(env: Env, account: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(account))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;

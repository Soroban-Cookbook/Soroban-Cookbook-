//! # Oracle consumer — shared feed interface and validation helpers
//!
//! Oracle *providers* publish data; oracle *consumers* have to decide whether
//! that data is safe to act on. This crate holds what all consumers in
//! [`12-oracle-consumer`](../) share: the feed interface they call, the error
//! type they return, and the three validation checks they apply.
//!
//! It is an interface crate — it defines no contract of its own and is never
//! deployed. The deployable examples are:
//!
//! | Crate | Data usage pattern | Use it when |
//! |-------|--------------------|-------------|
//! | `guarded-oracle-consumer` | Pull, validate, cache, serve from cache | One trusted feed; reads must be cheap and deterministic |
//! | `aggregating-oracle-consumer` | Fan out to N feeds, drop bad ones, require a quorum, take the median | No single feed is trusted enough on its own |
//! | `settlement-oracle-consumer` | Gate a state change behind a deviation circuit breaker | A price drives value transfer and a bad tick is expensive |
//!
//! ## Best practices these helpers encode
//!
//! - Never trust a raw feed read: check freshness, sanity bounds, and that the
//!   timestamp is not in the future before using a value.
//! - Use checked arithmetic on every price calculation; prices are attacker
//!   influenced inputs.
//! - Return a typed error for each distinct rejection reason, so callers and
//!   monitoring can tell "the feed is quiet" from "the feed is lying".

#![no_std]

use soroban_sdk::{contractclient, contracterror, contracttype, Env, Symbol};

#[cfg(feature = "testutils")]
pub mod testutils;

// ---------------------------------------------------------------------------
// Shared feed interface
// ---------------------------------------------------------------------------

/// A single price observation reported by a feed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quote {
    /// Price scaled by [`PRICE_SCALE`].
    pub price: i128,
    /// Ledger timestamp at which the feed observed the price.
    pub timestamp: u64,
}

/// Minimal interface a price feed must expose to be consumable here.
///
/// Feeds that expose a different shape — for example the separate
/// `get_value` / `get_timestamp` calls of
/// [`03-oracle-pattern`](../../03-oracle-pattern/) — can be wrapped in a small
/// adapter contract that implements this trait.
#[contractclient(name = "PriceFeedClient")]
pub trait PriceFeed {
    /// Return the latest quote for `asset`.
    fn quote(env: Env, asset: Symbol) -> Quote;
}

/// Fixed-point scale used for all prices: 1.0 is represented as `10_000_000`.
pub const PRICE_SCALE: i128 = 10_000_000;

/// Basis-point denominator (100% = 10_000 bps).
pub const BPS_DENOMINATOR: i128 = 10_000;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors shared by every consumer in this example.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ConsumerError {
    /// `initialize` was called more than once.
    AlreadyInitialized = 1,
    /// The contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller is not the stored admin.
    Unauthorized = 3,
    /// The supplied configuration is not usable (empty feed set, zero window,
    /// inverted price bounds, ...).
    InvalidConfig = 4,
    /// The freshest acceptable quote is older than the configured window.
    StaleData = 5,
    /// The quote falls outside the configured sanity bounds, or is not
    /// positive.
    PriceOutOfBounds = 6,
    /// Fewer feeds returned a usable quote than the configured quorum.
    QuorumNotMet = 7,
    /// No validated value has ever been cached.
    NoCachedValue = 8,
    /// The new price moved further from the last accepted price than the
    /// configured limit allows.
    DeviationTooLarge = 9,
    /// The circuit breaker is open; the admin must reset it.
    CircuitOpen = 10,
    /// A price calculation overflowed.
    ArithmeticOverflow = 11,
    /// The feed is already registered, or is not registered.
    FeedNotFound = 12,
}

// ---------------------------------------------------------------------------
// Shared validation helpers
// ---------------------------------------------------------------------------

/// Reject quotes that are stale or dated in the future.
///
/// A future timestamp is treated as an error rather than as "very fresh": it
/// means the feed's clock disagrees with the ledger, and trusting it would let
/// a misconfigured feed keep a stale value alive forever.
pub fn check_freshness(env: &Env, quote: &Quote, max_age: u64) -> Result<(), ConsumerError> {
    let now = env.ledger().timestamp();
    if quote.timestamp > now {
        return Err(ConsumerError::StaleData);
    }
    if now - quote.timestamp > max_age {
        return Err(ConsumerError::StaleData);
    }
    Ok(())
}

/// Reject prices outside `[min_price, max_price]`, and any non-positive price.
pub fn check_bounds(price: i128, min_price: i128, max_price: i128) -> Result<(), ConsumerError> {
    if price <= 0 || price < min_price || price > max_price {
        return Err(ConsumerError::PriceOutOfBounds);
    }
    Ok(())
}

/// Absolute deviation of `new_price` from `reference`, in basis points.
///
/// Returns [`ConsumerError::ArithmeticOverflow`] rather than wrapping, and
/// [`ConsumerError::PriceOutOfBounds`] when `reference` is not positive.
pub fn deviation_bps(reference: i128, new_price: i128) -> Result<i128, ConsumerError> {
    if reference <= 0 {
        return Err(ConsumerError::PriceOutOfBounds);
    }
    let delta = new_price
        .checked_sub(reference)
        .ok_or(ConsumerError::ArithmeticOverflow)?
        .checked_abs()
        .ok_or(ConsumerError::ArithmeticOverflow)?;
    delta
        .checked_mul(BPS_DENOMINATOR)
        .ok_or(ConsumerError::ArithmeticOverflow)?
        .checked_div(reference)
        .ok_or(ConsumerError::ArithmeticOverflow)
}

#[cfg(test)]
mod test;

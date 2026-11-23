//! # Consumer 1 — validate once, serve from cache
//!
//! The simplest safe way to consume a single feed:
//!
//! 1. A permissionless `refresh` pulls a quote, validates it, and caches it.
//! 2. Business logic reads `price`, which never makes a cross-contract call
//!    and never returns a value older than `max_age`.
//! 3. If the feed goes quiet, `price` starts failing. `price_or_last_known`
//!    is the *explicit* degraded path, bounded by `fallback_max_age`.
//!
//! Splitting the write path from the read path keeps reads cheap and makes the
//! failure mode obvious: callers choose whether they can tolerate stale data
//! instead of silently receiving it.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

use oracle_consumer_common::{
    check_bounds, check_freshness, ConsumerError, PriceFeedClient, Quote,
};

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum GuardKey {
    Admin,
    Config,
    /// Last quote that passed every validation check.
    Cached,
}

/// Consumer-side policy applied to every quote from the feed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardConfig {
    /// Address of the price feed contract.
    pub feed: Address,
    /// Asset symbol passed to the feed.
    pub asset: Symbol,
    /// Maximum age, in seconds, for a quote to be used normally.
    pub max_age: u64,
    /// Maximum age, in seconds, for the degraded `price_or_last_known` path.
    /// Must be at least `max_age`.
    pub fallback_max_age: u64,
    /// Lower sanity bound (exclusive of zero and negatives).
    pub min_price: i128,
    /// Upper sanity bound.
    pub max_price: i128,
}

const NS: Symbol = symbol_short!("guarded");
const EV_REFRESH: Symbol = symbol_short!("refresh");
const EV_FEED: Symbol = symbol_short!("feed");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct GuardedConsumer;

#[contractimpl]
impl GuardedConsumer {
    /// Store the admin and the consumption policy.
    ///
    /// Rejects configurations that could never validate a quote: a zero
    /// freshness window, a fallback window shorter than the normal one, or
    /// inverted / non-positive price bounds.
    pub fn initialize(env: Env, admin: Address, config: GuardConfig) -> Result<(), ConsumerError> {
        if env.storage().instance().has(&GuardKey::Admin) {
            return Err(ConsumerError::AlreadyInitialized);
        }
        if config.max_age == 0
            || config.fallback_max_age < config.max_age
            || config.min_price <= 0
            || config.max_price < config.min_price
        {
            return Err(ConsumerError::InvalidConfig);
        }

        admin.require_auth();
        env.storage().instance().set(&GuardKey::Admin, &admin);
        env.storage().instance().set(&GuardKey::Config, &config);
        Ok(())
    }

    /// Pull a quote from the feed, validate it, and cache it on success.
    ///
    /// Permissionless on purpose: anybody may pay to keep the cache warm, and
    /// a caller cannot influence *what* gets cached — only whether a
    /// validated value is written.
    pub fn refresh(env: Env) -> Result<i128, ConsumerError> {
        let config = read_config(&env)?;

        let quote = PriceFeedClient::new(&env, &config.feed).quote(&config.asset);
        check_freshness(&env, &quote, config.max_age)?;
        check_bounds(quote.price, config.min_price, config.max_price)?;

        env.storage().instance().set(&GuardKey::Cached, &quote);
        env.events()
            .publish((NS, EV_REFRESH, config.asset), quote.clone());

        Ok(quote.price)
    }

    /// Return the cached price, or [`ConsumerError::StaleData`] if the cache
    /// is older than `max_age`. This is the entry point business logic should
    /// use.
    pub fn price(env: Env) -> Result<i128, ConsumerError> {
        let config = read_config(&env)?;
        let quote = read_cached(&env)?;
        check_freshness(&env, &quote, config.max_age)?;
        Ok(quote.price)
    }

    /// Degraded read: return the cached price even if it is past `max_age`,
    /// as long as it is within `fallback_max_age`.
    ///
    /// Callers opt into stale data by choosing this entry point; nothing here
    /// falls back implicitly.
    pub fn price_or_last_known(env: Env) -> Result<i128, ConsumerError> {
        let config = read_config(&env)?;
        let quote = read_cached(&env)?;
        check_freshness(&env, &quote, config.fallback_max_age)?;
        Ok(quote.price)
    }

    /// Point the consumer at a different feed. Admin only.
    ///
    /// The cache is cleared: a value validated against the old feed says
    /// nothing about the new one.
    pub fn set_feed(env: Env, new_feed: Address) -> Result<(), ConsumerError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        let mut config = read_config(&env)?;
        config.feed = new_feed.clone();
        env.storage().instance().set(&GuardKey::Config, &config);
        env.storage().instance().remove(&GuardKey::Cached);

        env.events().publish((NS, EV_FEED, admin), new_feed);
        Ok(())
    }

    /// The last quote that passed validation, if any.
    pub fn cached(env: Env) -> Option<Quote> {
        env.storage().instance().get(&GuardKey::Cached)
    }

    pub fn config(env: Env) -> Result<GuardConfig, ConsumerError> {
        read_config(&env)
    }

    pub fn admin(env: Env) -> Result<Address, ConsumerError> {
        read_admin(&env)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_admin(env: &Env) -> Result<Address, ConsumerError> {
    env.storage()
        .instance()
        .get(&GuardKey::Admin)
        .ok_or(ConsumerError::NotInitialized)
}

fn read_config(env: &Env) -> Result<GuardConfig, ConsumerError> {
    env.storage()
        .instance()
        .get(&GuardKey::Config)
        .ok_or(ConsumerError::NotInitialized)
}

fn read_cached(env: &Env) -> Result<Quote, ConsumerError> {
    env.storage()
        .instance()
        .get(&GuardKey::Cached)
        .ok_or(ConsumerError::NoCachedValue)
}

#[cfg(test)]
mod test;

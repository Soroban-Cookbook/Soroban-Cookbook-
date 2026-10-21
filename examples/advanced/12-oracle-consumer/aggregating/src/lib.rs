//! # Consumer 2 — quorum across redundant feeds
//!
//! When no single provider is trusted enough, read several and reduce them to
//! one number:
//!
//! 1. Fan out to every registered feed with `try_quote`, so a feed that traps,
//!    whose instance entry has expired, or that no longer exposes the expected
//!    function is skipped instead of reverting the whole call.
//! 2. Drop quotes that are stale or outside the sanity bounds.
//! 3. Require at least `min_responses` survivors, otherwise fail loudly.
//! 4. Return the **median** — one compromised feed cannot move it, whereas it
//!    can move a mean arbitrarily far.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

use oracle_consumer_common::{check_bounds, check_freshness, ConsumerError, PriceFeedClient};

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum AggKey {
    Admin,
    Config,
    Feeds,
}

/// Aggregation policy.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggConfig {
    /// Asset symbol passed to every feed.
    pub asset: Symbol,
    /// Maximum quote age, in seconds.
    pub max_age: u64,
    /// Minimum number of feeds that must return a usable quote.
    pub min_responses: u32,
    /// Lower sanity bound (exclusive of zero and negatives).
    pub min_price: i128,
    /// Upper sanity bound.
    pub max_price: i128,
}

const NS: Symbol = symbol_short!("aggr");
const EV_ADD: Symbol = symbol_short!("add_feed");
const EV_REMOVE: Symbol = symbol_short!("rm_feed");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AggregatingConsumer;

#[contractimpl]
impl AggregatingConsumer {
    /// Store the admin, the initial feed set, and the aggregation policy.
    ///
    /// A quorum larger than the feed set can never be met, so it is rejected
    /// at initialization rather than at read time.
    pub fn initialize(
        env: Env,
        admin: Address,
        feeds: Vec<Address>,
        config: AggConfig,
    ) -> Result<(), ConsumerError> {
        if env.storage().instance().has(&AggKey::Admin) {
            return Err(ConsumerError::AlreadyInitialized);
        }
        if config.max_age == 0
            || config.min_responses == 0
            || config.min_price <= 0
            || config.max_price < config.min_price
        {
            return Err(ConsumerError::InvalidConfig);
        }
        if feeds.len() < config.min_responses {
            return Err(ConsumerError::InvalidConfig);
        }

        env.storage().instance().set(&AggKey::Admin, &admin);
        env.storage().instance().set(&AggKey::Feeds, &feeds);
        env.storage().instance().set(&AggKey::Config, &config);
        Ok(())
    }

    /// Every quote that survived the freshness and bounds checks, in feed
    /// registration order. Useful for monitoring which providers are healthy.
    pub fn usable_prices(env: Env) -> Result<Vec<i128>, ConsumerError> {
        let config = read_config(&env)?;
        let feeds = read_feeds(&env)?;

        let mut prices = Vec::new(&env);
        for feed in feeds.iter() {
            // `try_quote` keeps one broken provider from reverting the read.
            let Ok(Ok(quote)) = PriceFeedClient::new(&env, &feed).try_quote(&config.asset) else {
                continue;
            };
            if check_freshness(&env, &quote, config.max_age).is_err() {
                continue;
            }
            if check_bounds(quote.price, config.min_price, config.max_price).is_err() {
                continue;
            }
            prices.push_back(quote.price);
        }
        Ok(prices)
    }

    /// Median of the usable quotes, provided the quorum is met.
    pub fn median_price(env: Env) -> Result<i128, ConsumerError> {
        let config = read_config(&env)?;
        let prices = Self::usable_prices(env.clone())?;
        if prices.len() < config.min_responses {
            return Err(ConsumerError::QuorumNotMet);
        }
        median(prices)
    }

    /// Register an additional feed. Admin only.
    pub fn add_feed(env: Env, feed: Address) -> Result<(), ConsumerError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        let mut feeds = read_feeds(&env)?;
        if feeds.contains(&feed) {
            return Err(ConsumerError::InvalidConfig);
        }
        feeds.push_back(feed.clone());
        env.storage().instance().set(&AggKey::Feeds, &feeds);

        env.events().publish((NS, EV_ADD, admin), feed);
        Ok(())
    }

    /// Remove a feed. Admin only.
    ///
    /// Removing a feed may not drop the set below the configured quorum —
    /// otherwise every subsequent read would fail.
    pub fn remove_feed(env: Env, feed: Address) -> Result<(), ConsumerError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        let config = read_config(&env)?;
        let feeds = read_feeds(&env)?;
        let index = feeds
            .first_index_of(&feed)
            .ok_or(ConsumerError::FeedNotFound)?;
        if feeds.len() - 1 < config.min_responses {
            return Err(ConsumerError::InvalidConfig);
        }

        let mut feeds = feeds;
        feeds.remove(index);
        env.storage().instance().set(&AggKey::Feeds, &feeds);

        env.events().publish((NS, EV_REMOVE, admin), feed);
        Ok(())
    }

    pub fn feeds(env: Env) -> Result<Vec<Address>, ConsumerError> {
        read_feeds(&env)
    }

    pub fn config(env: Env) -> Result<AggConfig, ConsumerError> {
        read_config(&env)
    }

    pub fn admin(env: Env) -> Result<Address, ConsumerError> {
        read_admin(&env)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Median of a non-empty vector. Even-length inputs average the two middle
/// values with checked arithmetic.
fn median(prices: Vec<i128>) -> Result<i128, ConsumerError> {
    let len = prices.len();
    if len == 0 {
        return Err(ConsumerError::QuorumNotMet);
    }
    let sorted = sorted(prices);
    let mid = len / 2;
    if len % 2 == 1 {
        return sorted.get(mid).ok_or(ConsumerError::QuorumNotMet);
    }
    let lower = sorted.get(mid - 1).ok_or(ConsumerError::QuorumNotMet)?;
    let upper = sorted.get(mid).ok_or(ConsumerError::QuorumNotMet)?;
    lower
        .checked_add(upper)
        .ok_or(ConsumerError::ArithmeticOverflow)?
        .checked_div(2)
        .ok_or(ConsumerError::ArithmeticOverflow)
}

/// Insertion sort. Feed sets are small (single digits), so the simplest
/// in-place algorithm is also the cheapest one in ledger terms.
fn sorted(values: Vec<i128>) -> Vec<i128> {
    let mut values = values;
    let len = values.len();
    for i in 1..len {
        let Some(key) = values.get(i) else { continue };
        let mut j = i;
        while j > 0 {
            let Some(prev) = values.get(j - 1) else { break };
            if prev <= key {
                break;
            }
            values.set(j, prev);
            j -= 1;
        }
        values.set(j, key);
    }
    values
}

fn read_admin(env: &Env) -> Result<Address, ConsumerError> {
    env.storage()
        .instance()
        .get(&AggKey::Admin)
        .ok_or(ConsumerError::NotInitialized)
}

fn read_config(env: &Env) -> Result<AggConfig, ConsumerError> {
    env.storage()
        .instance()
        .get(&AggKey::Config)
        .ok_or(ConsumerError::NotInitialized)
}

fn read_feeds(env: &Env) -> Result<Vec<Address>, ConsumerError> {
    env.storage()
        .instance()
        .get(&AggKey::Feeds)
        .ok_or(ConsumerError::NotInitialized)
}

#[cfg(test)]
mod test;

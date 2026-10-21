//! # Consumer 3 — gate a state change behind a circuit breaker
//!
//! The previous two consumers only *read* a price. This one lets a price move
//! value, which raises the stakes: a single bad tick mints credit that cannot
//! be un-minted.
//!
//! On top of the usual freshness and bounds checks, every settlement is
//! compared against the last accepted price. A jump larger than
//! `max_deviation_bps` is rejected, and a circuit breaker can be tripped to
//! block all further settlement until an admin re-anchors the reference price
//! by hand. A feed that produced one impossible tick has lost its
//! credibility, and the next tick back inside the band should not silently
//! resume trading.
//!
//! ## Pitfall: you cannot both fail and remember
//!
//! An invocation that returns an error has **all of its storage writes rolled
//! back**. So `settle` cannot reject a bad tick *and* record that it saw one —
//! the breaker flag would vanish along with the error. Tripping therefore
//! lives in [`SettlementConsumer::trip_if_deviated`], a permissionless
//! keeper call that returns `Ok` and so keeps its write.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

use oracle_consumer_common::{
    check_bounds, check_freshness, deviation_bps, ConsumerError, PriceFeedClient, PRICE_SCALE,
};

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum SettleKey {
    Admin,
    Config,
    /// Last price accepted by a successful settlement.
    LastPrice,
    /// `true` while the circuit breaker is tripped.
    Breaker,
    /// Settled value credited to an account.
    Credit(Address),
}

/// Settlement policy.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettleConfig {
    /// Address of the price feed contract.
    pub feed: Address,
    /// Asset symbol passed to the feed.
    pub asset: Symbol,
    /// Maximum quote age, in seconds.
    pub max_age: u64,
    /// Largest tolerated move from the last accepted price, in basis points.
    pub max_deviation_bps: i128,
    /// Lower sanity bound (exclusive of zero and negatives).
    pub min_price: i128,
    /// Upper sanity bound.
    pub max_price: i128,
}

const NS: Symbol = symbol_short!("settle");
const EV_SETTLE: Symbol = symbol_short!("settled");
const EV_TRIP: Symbol = symbol_short!("tripped");
const EV_RESET: Symbol = symbol_short!("reset");

/// TTL bump applied to per-account credit entries: ~1 day low-water mark,
/// ~7 days target, matching the other advanced examples.
const CREDIT_TTL_THRESHOLD: u32 = 17_280;
const CREDIT_TTL_EXTEND: u32 = 120_960;

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct SettlementConsumer;

#[contractimpl]
impl SettlementConsumer {
    /// Store the admin and the settlement policy.
    pub fn initialize(env: Env, admin: Address, config: SettleConfig) -> Result<(), ConsumerError> {
        if env.storage().instance().has(&SettleKey::Admin) {
            return Err(ConsumerError::AlreadyInitialized);
        }
        if config.max_age == 0
            || config.max_deviation_bps <= 0
            || config.min_price <= 0
            || config.max_price < config.min_price
        {
            return Err(ConsumerError::InvalidConfig);
        }

        env.storage().instance().set(&SettleKey::Admin, &admin);
        env.storage().instance().set(&SettleKey::Config, &config);
        env.storage().instance().set(&SettleKey::Breaker, &false);
        Ok(())
    }

    /// Value `amount` units of the asset at the current price and credit the
    /// result to `account`.
    ///
    /// Fails — and trips the breaker — when the feed moves further than
    /// `max_deviation_bps` from the last accepted price.
    pub fn settle(env: Env, account: Address, amount: i128) -> Result<i128, ConsumerError> {
        account.require_auth();

        if Self::is_open(env.clone()) {
            return Err(ConsumerError::CircuitOpen);
        }
        if amount <= 0 {
            return Err(ConsumerError::InvalidConfig);
        }
        let config = read_config(&env)?;

        let quote = PriceFeedClient::new(&env, &config.feed).quote(&config.asset);
        check_freshness(&env, &quote, config.max_age)?;
        check_bounds(quote.price, config.min_price, config.max_price)?;

        // Anchored comparison against the last accepted price. Rejecting is
        // all this path can do — see the module docs on rolled-back writes.
        if let Some(last) = read_last_price(&env) {
            if deviation_bps(last, quote.price)? > config.max_deviation_bps {
                return Err(ConsumerError::DeviationTooLarge);
            }
        }

        let value = amount
            .checked_mul(quote.price)
            .ok_or(ConsumerError::ArithmeticOverflow)?
            .checked_div(PRICE_SCALE)
            .ok_or(ConsumerError::ArithmeticOverflow)?;

        let key = SettleKey::Credit(account.clone());
        let credit = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(0_i128)
            .checked_add(value)
            .ok_or(ConsumerError::ArithmeticOverflow)?;
        env.storage().persistent().set(&key, &credit);
        env.storage()
            .persistent()
            .extend_ttl(&key, CREDIT_TTL_THRESHOLD, CREDIT_TTL_EXTEND);

        env.storage()
            .instance()
            .set(&SettleKey::LastPrice, &quote.price);

        env.events()
            .publish((NS, EV_SETTLE, account), (amount, quote.price, value));
        Ok(value)
    }

    /// Trip the breaker if the feed currently reports a tick this contract
    /// would refuse to settle at. Returns `true` if the breaker is open
    /// afterwards.
    ///
    /// Permissionless: any keeper watching the feed may call it, and a caller
    /// cannot choose the outcome — only whether the check runs. Because this
    /// call succeeds, its write survives; `settle` cannot do the same job on
    /// its error path.
    ///
    /// A merely *stale* quote is not grounds for tripping: feeds go quiet for
    /// benign reasons, and `settle` already refuses stale data on its own.
    pub fn trip_if_deviated(env: Env) -> Result<bool, ConsumerError> {
        let config = read_config(&env)?;
        if Self::is_open(env.clone()) {
            return Ok(true);
        }
        let Some(last) = read_last_price(&env) else {
            return Ok(false);
        };

        let quote = PriceFeedClient::new(&env, &config.feed).quote(&config.asset);
        if check_freshness(&env, &quote, config.max_age).is_err() {
            return Ok(false);
        }

        let out_of_bounds = check_bounds(quote.price, config.min_price, config.max_price).is_err();
        let deviated =
            !out_of_bounds && deviation_bps(last, quote.price)? > config.max_deviation_bps;
        if !out_of_bounds && !deviated {
            return Ok(false);
        }

        env.storage().instance().set(&SettleKey::Breaker, &true);
        env.events()
            .publish((NS, EV_TRIP, config.asset), (last, quote.price));
        Ok(true)
    }

    /// Close the breaker and re-anchor the reference price. Admin only.
    ///
    /// The admin must state the price they are re-anchoring to, so that
    /// recovery is an explicit, auditable decision rather than a side effect
    /// of whatever the feed happens to report next.
    pub fn reset_breaker(env: Env, acknowledged_price: i128) -> Result<(), ConsumerError> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        let config = read_config(&env)?;
        check_bounds(acknowledged_price, config.min_price, config.max_price)?;

        env.storage().instance().set(&SettleKey::Breaker, &false);
        env.storage()
            .instance()
            .set(&SettleKey::LastPrice, &acknowledged_price);

        env.events()
            .publish((NS, EV_RESET, admin), acknowledged_price);
        Ok(())
    }

    /// `true` while the circuit breaker is tripped.
    pub fn is_open(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&SettleKey::Breaker)
            .unwrap_or(false)
    }

    /// Last price accepted by a successful settlement, if any.
    pub fn last_price(env: Env) -> Option<i128> {
        read_last_price(&env)
    }

    /// Total settled value credited to `account`.
    pub fn credit_of(env: Env, account: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&SettleKey::Credit(account))
            .unwrap_or(0)
    }

    pub fn config(env: Env) -> Result<SettleConfig, ConsumerError> {
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
        .get(&SettleKey::Admin)
        .ok_or(ConsumerError::NotInitialized)
}

fn read_config(env: &Env) -> Result<SettleConfig, ConsumerError> {
    env.storage()
        .instance()
        .get(&SettleKey::Config)
        .ok_or(ConsumerError::NotInitialized)
}

fn read_last_price(env: &Env) -> Option<i128> {
    env.storage().instance().get(&SettleKey::LastPrice)
}

#[cfg(test)]
mod test;

//! # Price Oracle
//!
//! A secure and reusable price feed oracle for Soroban.
//! Supports multiple updaters, median aggregation, TWAP calculation, and stale price detection.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum OracleError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAsset = 4,
    InvalidPrice = 5,
    NoData = 6,
    StaleData = 7,
    ArithmeticOverflow = 8,
    EmptyAggregation = 9,
    InvalidTimestamp = 10,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetConfig {
    pub max_age: u64,
    pub twap_window: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceUpdateEvent {
    pub asset: Symbol,
    pub price: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Updaters, // Vec<Address>
    AssetConfig(Symbol),
    LatestPrice(Symbol),
    PriceHistory(Symbol), // Vec<PriceData>
    UpdaterPrice(Symbol, Address),
}

#[contract]
pub struct PriceOracleContract;

#[contractimpl]
impl PriceOracleContract {
    /// Initialize the oracle with an admin.
    pub fn initialize(env: Env, admin: Address) -> Result<(), OracleError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(OracleError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Updaters, &Vec::<Address>::new(&env));
        Ok(())
    }

    /// Set the configuration for an asset. Only admin.
    pub fn set_asset_config(
        env: Env,
        admin: Address,
        asset: Symbol,
        config: AssetConfig,
    ) -> Result<(), OracleError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(OracleError::NotInitialized)?;
        if admin != stored_admin {
            return Err(OracleError::Unauthorized);
        }
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::AssetConfig(asset), &config);
        Ok(())
    }

    /// Add an authorized updater. Only admin.
    pub fn add_updater(env: Env, admin: Address, updater: Address) -> Result<(), OracleError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(OracleError::NotInitialized)?;
        if admin != stored_admin {
            return Err(OracleError::Unauthorized);
        }
        admin.require_auth();

        let updaters: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Updaters)
            .unwrap_or_else(|| Vec::new(&env));

        if !updaters.contains(&updater) {
            let mut new_updaters = updaters;
            new_updaters.push_back(updater);
            env.storage().instance().set(&DataKey::Updaters, &new_updaters);
        }

        Ok(())
    }

    /// Remove an authorized updater. Only admin.
    pub fn remove_updater(env: Env, admin: Address, updater: Address) -> Result<(), OracleError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(OracleError::NotInitialized)?;
        if admin != stored_admin {
            return Err(OracleError::Unauthorized);
        }
        admin.require_auth();

        let updaters: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Updaters)
            .ok_or(OracleError::NotInitialized)?;

        let mut new_updaters = Vec::new(&env);
        for u in updaters.iter() {
            if u != updater {
                new_updaters.push_back(u);
            }
        }
        env.storage().instance().set(&DataKey::Updaters, &new_updaters);

        Ok(())
    }

    /// Submit a new price for an asset. Only authorized updaters.
    pub fn submit_prices(
        env: Env,
        updater: Address,
        prices: Vec<(Symbol, i128)>,
    ) -> Result<(), OracleError> {
        let updaters: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Updaters)
            .ok_or(OracleError::NotInitialized)?;

        if !updaters.contains(&updater) {
            return Err(OracleError::Unauthorized);
        }
        updater.require_auth();

        let now = env.ledger().timestamp();

        for item in prices.iter() {
            let (asset, price) = item;
            if price <= 0 {
                return Err(OracleError::InvalidPrice);
            }

            let asset_clone = asset.clone();

            // Check if asset is configured
            if !env.storage().instance().has(&DataKey::AssetConfig(asset_clone.clone())) {
                return Err(OracleError::InvalidAsset);
            }

            // Store price in history for this updater
            env.storage().temporary().set(&DataKey::UpdaterPrice(asset_clone.clone(), updater.clone()), &PriceData {
                price,
                timestamp: now,
            });

            // Perform aggregation (Median)
            let aggregated_price = aggregate_median(&env, asset_clone.clone())?;

            // Update latest price
            env.storage().instance().set(&DataKey::LatestPrice(asset_clone.clone()), &PriceData {
                price: aggregated_price,
                timestamp: now,
            });

            // Add to history for TWAP
            let history: Vec<PriceData> = env
                .storage()
                .persistent()
                .get(&DataKey::PriceHistory(asset_clone.clone()))
                .unwrap_or_else(|| Vec::new(&env));

            let mut new_history = history;
            new_history.push_back(PriceData {
                price: aggregated_price,
                timestamp: now,
            });

            // Prune history based on twap_window
            let config: AssetConfig = env.storage().instance().get(&DataKey::AssetConfig(asset_clone.clone())).unwrap();
            let mut pruned_history = Vec::new(&env);
            for p in new_history.iter() {
                if p.timestamp >= now.saturating_sub(config.twap_window) {
                    pruned_history.push_back(p);
                }
            }

            env.storage().persistent().set(&DataKey::PriceHistory(asset_clone.clone()), &pruned_history);

            #[allow(deprecated)]
            env.events().publish(
                (symbol_short!("oracle"), symbol_short!("update"), asset),
                PriceUpdateEvent {
                    asset: asset_clone,
                    price: aggregated_price,
                },
            );
        }

        Ok(())
    }

    /// Return the latest price for an asset.
    pub fn get_price(env: Env, asset: Symbol) -> Result<PriceData, OracleError> {
        env.storage()
            .instance()
            .get(&DataKey::LatestPrice(asset))
            .ok_or(OracleError::NoData)
    }

    /// Return the latest price only if it is fresh; otherwise error.
    pub fn get_price_strict(env: Env, asset: Symbol) -> Result<i128, OracleError> {
        let data: PriceData = env
            .storage()
            .instance()
            .get(&DataKey::LatestPrice(asset.clone()))
            .ok_or(OracleError::NoData)?;

        let config: AssetConfig = env
            .storage()
            .instance()
            .get(&DataKey::AssetConfig(asset))
            .ok_or(OracleError::InvalidAsset)?;

        let age = env.ledger().timestamp().saturating_sub(data.timestamp);
        if age > config.max_age {
            return Err(OracleError::StaleData);
        }

        Ok(data.price)
    }

    /// Calculate TWAP for an asset over the configured window.
    pub fn get_twap(env: Env, asset: Symbol) -> Result<i128, OracleError> {
        let history: Vec<PriceData> = env
            .storage()
            .persistent()
            .get(&DataKey::PriceHistory(asset.clone()))
            .ok_or(OracleError::NoData)?;

        if history.is_empty() {
            return Err(OracleError::NoData);
        }

        let config: AssetConfig = env
            .storage()
            .instance()
            .get(&DataKey::AssetConfig(asset))
            .ok_or(OracleError::InvalidAsset)?;

        let now = env.ledger().timestamp();
        let window_start = now.saturating_sub(config.twap_window);

        let mut total_weighted_price = 0i128;
        let mut total_time = 0u64;

        let mut prev_price = history.get(0).unwrap().price;
        let mut prev_ts = history.get(0).unwrap().timestamp;

        // If the first observation is after window_start, we assume the price was the same
        // from window_start until the first observation.
        if prev_ts > window_start {
            let duration = prev_ts.saturating_sub(window_start);
            let weight = duration as i128;
            total_weighted_price = total_weighted_price
                .checked_add(prev_price.checked_mul(weight).ok_or(OracleError::ArithmeticOverflow)?)
                .ok_or(OracleError::ArithmeticOverflow)?;
            total_time = total_time.saturating_add(duration);
        }

        for i in 1..history.len() {
            let p = history.get(i).unwrap();

            // Current price 'p.price' was active from 'prev_ts' to 'p.timestamp'
            // Wait, usually it's the OTHER way around: 'prev_price' was active from 'prev_ts' to 'p.timestamp'
            let duration = p.timestamp.saturating_sub(core::cmp::max(prev_ts, window_start));
            if duration > 0 {
                let weight = duration as i128;
                total_weighted_price = total_weighted_price
                    .checked_add(prev_price.checked_mul(weight).ok_or(OracleError::ArithmeticOverflow)?)
                    .ok_or(OracleError::ArithmeticOverflow)?;
                total_time = total_time.saturating_add(duration);
            }

            prev_price = p.price;
            prev_ts = p.timestamp;
        }

        // Add the weight of the last price until 'now'
        if prev_ts < now {
            let duration = now.saturating_sub(core::cmp::max(prev_ts, window_start));
            let weight = duration as i128;
            total_weighted_price = total_weighted_price
                .checked_add(prev_price.checked_mul(weight).ok_or(OracleError::ArithmeticOverflow)?)
                .ok_or(OracleError::ArithmeticOverflow)?;
            total_time = total_time.saturating_add(duration);
        }

        if total_time == 0 {
            // If no time has passed in the window, return the latest price
            return Ok(history.get(history.len() - 1).unwrap().price);
        }

        total_weighted_price
            .checked_div(total_time as i128)
            .ok_or(OracleError::ArithmeticOverflow)
    }
}

fn aggregate_median(env: &Env, asset: Symbol) -> Result<i128, OracleError> {
    let updaters: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::Updaters)
        .ok_or(OracleError::NotInitialized)?;

    let mut prices = Vec::new(env);
    let now = env.ledger().timestamp();
    let config: AssetConfig = env.storage().instance().get(&DataKey::AssetConfig(asset.clone())).unwrap();

    for updater in updaters.iter() {
        if let Some(data) = env.storage().temporary().get::<_, PriceData>(&DataKey::UpdaterPrice(asset.clone(), updater)) {
            // Only include non-stale prices in aggregation
            if now.saturating_sub(data.timestamp) <= config.max_age {
                prices.push_back(data.price);
            }
        }
    }

    if prices.is_empty() {
        return Err(OracleError::EmptyAggregation);
    }

    // Sort prices to find median
    let mut prices_vec: Vec<i128> = prices;

    // Bubble sort for simplicity in a no_std environment without easy alloc access
    let n = prices_vec.len();
    for i in 0..n {
        for j in 0..n - 1 - i {
            let p1 = prices_vec.get(j).unwrap();
            let p2 = prices_vec.get(j + 1).unwrap();
            if p1 > p2 {
                prices_vec.set(j, p2);
                prices_vec.set(j + 1, p1);
            }
        }
    }

    if n % 2 == 1 {
        Ok(prices_vec.get(n / 2).unwrap())
    } else {
        let mid1 = prices_vec.get(n / 2 - 1).unwrap();
        let mid2 = prices_vec.get(n / 2).unwrap();
        mid1.checked_add(mid2)
            .ok_or(OracleError::ArithmeticOverflow)?
            .checked_div(2)
            .ok_or(OracleError::ArithmeticOverflow)
    }
}

#[cfg(test)]
mod test;

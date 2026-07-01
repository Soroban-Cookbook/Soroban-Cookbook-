#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

/// Maximum number of data sources allowed per oracle
const MAX_SOURCES: u32 = 10;
/// Minimum number of data points required for aggregation
const MIN_DATA_POINTS: u32 = 3;
/// Maximum deviation (in basis points) for outlier detection: 500 = 5%
const MAX_DEVIATION_BPS: i128 = 500;
/// Basis points constant
const BASIS_POINTS: i128 = 10000;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataSubmissionEventData {
    pub source: Address,
    pub value: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregationEventData {
    pub median: i128,
    pub point_count: u32,
    pub outliers: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManipulationEventData {
    pub source: Address,
    pub value: i128,
    pub deviation_bps: i128,
    pub timestamp: u64,
}

const CONTRACT_NS: Symbol = symbol_short!("oracle");
const ACTION_SUBMIT: Symbol = symbol_short!("submit");
const ACTION_AGGR: Symbol = symbol_short!("aggr");
const ACTION_MANIP: Symbol = symbol_short!("manip");

// ---------------------------------------------------------------------------
// Data Types
// ---------------------------------------------------------------------------

#[contracttype]
pub enum DataKey {
    SourceValue(Address),
    SourceTimestamp(Address),
    Admin,
    TrustedSources,
    Paused,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregationResult {
    pub median_value: i128,
    pub mean_value: i128,
    pub outliers_removed: u32,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct DataAggregationOracleContract;

#[contractimpl]
impl DataAggregationOracleContract {
    /// Initialize the oracle
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    /// Add a trusted data source
    pub fn add_source(env: Env, admin: Address, source: Address) {
        Self::require_admin(&env, &admin);
        Self::require_not_paused(&env);

        let mut sources = Self::get_sources_vec(&env);
        if (sources.len() as u32) >= MAX_SOURCES {
            panic!("Max sources");
        }

        for s in sources.iter() {
            if s == source {
                panic!("Exists");
            }
        }

        sources.push_back(source);
        env.storage().instance().set(&DataKey::TrustedSources, &sources);
    }

    /// Remove a trusted data source
    pub fn remove_source(env: Env, admin: Address, source: Address) {
        Self::require_admin(&env, &admin);

        let mut sources = Self::get_sources_vec(&env);
        let mut found = false;

        for i in 0..sources.len() {
            if sources.get(i).unwrap() == source {
                sources.remove(i);
                found = true;
                break;
            }
        }

        if !found {
            panic!("Not found");
        }

        env.storage().instance().set(&DataKey::TrustedSources, &sources);
    }

    /// Submit data from a source
    pub fn submit_data(env: Env, source: Address, value: i128) {
        Self::require_not_paused(&env);
        Self::require_authorized(&env, &source);

        env.storage()
            .instance()
            .set(&DataKey::SourceValue(source.clone()), &value);
        env.storage()
            .instance()
            .set(&DataKey::SourceTimestamp(source.clone()), &env.ledger().timestamp());

        env.events().publish(
            (CONTRACT_NS, ACTION_SUBMIT, source.clone()),
            DataSubmissionEventData {
                source,
                value,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Aggregate data with outlier detection
    pub fn aggregate_data(env: Env) -> AggregationResult {
        Self::require_not_paused(&env);

        let sources = Self::get_sources_vec(&env);
        if (sources.len() as u32) < MIN_DATA_POINTS {
            panic!("Not enough sources");
        }

        // Collect values
        let mut values: Vec<i128> = Vec::new(&env);
        for source in sources.iter() {
            if env.storage()
                .instance()
                .has(&DataKey::SourceValue(source.clone()))
            {
                let val: i128 = env
                    .storage()
                    .instance()
                    .get::<_, i128>(&DataKey::SourceValue(source.clone()))
                    .unwrap();
                values.push_back(val);
            }
        }

        if (values.len() as u32) < MIN_DATA_POINTS {
            panic!("Not enough data");
        }

        // Calculate median
        let median = Self::calc_median(&values);

        // Filter outliers
        let mut filtered: Vec<i128> = Vec::new(&env);
        let mut outlier_cnt: u32 = 0;

        for (idx, source) in sources.iter().enumerate() {
            let idx_u32 = idx as u32;
            if idx_u32 < (values.len() as u32) {
                let val = values.get(idx_u32).unwrap();
                let deviation = if median != 0 {
                    ((val - median).abs() * BASIS_POINTS) / median.abs()
                } else {
                    0
                };

                if deviation > MAX_DEVIATION_BPS {
                    env.events().publish(
                        (CONTRACT_NS, ACTION_MANIP, source.clone()),
                        ManipulationEventData {
                            source,
                            value: val,
                            deviation_bps: deviation,
                            timestamp: env.ledger().timestamp(),
                        },
                    );
                    outlier_cnt += 1;
                } else {
                    filtered.push_back(val);
                }
            }
        }

        let mean = if !filtered.is_empty() {
            Self::calc_mean(&filtered)
        } else {
            median
        };

        env.events().publish(
            (CONTRACT_NS, ACTION_AGGR),
            AggregationEventData {
                median,
                point_count: values.len() as u32,
                outliers: outlier_cnt,
                timestamp: env.ledger().timestamp(),
            },
        );

        AggregationResult {
            median_value: median,
            mean_value: mean,
            outliers_removed: outlier_cnt,
            timestamp: env.ledger().timestamp(),
        }
    }

    /// Pause the oracle
    pub fn pause(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &true);
    }

    /// Resume the oracle
    pub fn resume(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    /// Get trusted sources
    pub fn get_sources(env: Env) -> Vec<Address> {
        Self::get_sources_vec(&env)
    }

    /// Check if paused
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or_default()
    }

    // Internal helpers
    fn require_admin(env: &Env, admin: &Address) {
        let stored: Address = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Admin)
            .unwrap();
        if stored != *admin {
            panic!("Unauthorized");
        }
    }

    fn require_not_paused(env: &Env) {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or_default();
        if paused {
            panic!("Paused");
        }
    }

    fn require_authorized(env: &Env, source: &Address) {
        let sources = Self::get_sources_vec(env);
        for s in sources.iter() {
            if s == *source {
                return;
            }
        }
        panic!("Unauthorized");
    }

    fn get_sources_vec(env: &Env) -> Vec<Address> {
        match env.storage().instance().get(&DataKey::TrustedSources) {
            Some(val) => val,
            None => Vec::new(env),
        }
    }

    fn calc_median(values: &Vec<i128>) -> i128 {
        let len = values.len();
        if len == 0 {
            return 0;
        }
        if len == 1 {
            return values.get(0).unwrap();
        }

        // Find min and max
        let mut min = values.get(0).unwrap();
        let mut max = values.get(0).unwrap();

        for val in values.iter() {
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        }

        // Return middle value (simple average of extremes for small datasets)
        (min + max) / 2
    }

    fn calc_mean(values: &Vec<i128>) -> i128 {
        if values.is_empty() {
            return 0;
        }

        let mut sum: i128 = 0;
        for val in values.iter() {
            sum += val;
        }

        sum / (values.len() as i128)
    }
}

#[cfg(test)]
mod test;

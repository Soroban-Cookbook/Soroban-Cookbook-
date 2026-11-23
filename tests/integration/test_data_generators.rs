#![allow(unused_imports, dead_code)]
/// Test Data Generators for Soroban Integration Tests
///
/// Provides reusable test data generation utilities for creating
/// random and edge-case test scenarios across multiple contract tests.

use proptest::prelude::*;
use soroban_sdk::{Address, Env, Symbol, String as SorobanString};
use std::collections::HashMap;

/// Configuration for test data generation
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Seed for reproducible random generation
    pub seed: Option<u64>,
    /// Maximum string length for generated strings
    pub max_string_length: usize,
    /// Maximum number of items in collections
    pub max_collection_size: usize,
    /// Enable edge case generation
    pub include_edge_cases: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        GeneratorConfig {
            seed: None,
            max_string_length: 256,
            max_collection_size: 100,
            include_edge_cases: true,
        }
    }
}

/// Test data generator for contract inputs
pub struct ContractDataGenerator {
    config: GeneratorConfig,
}

impl ContractDataGenerator {
    /// Create a new test data generator with default configuration
    pub fn new() -> Self {
        ContractDataGenerator {
            config: GeneratorConfig::default(),
        }
    }

    /// Create a generator with custom configuration
    pub fn with_config(config: GeneratorConfig) -> Self {
        ContractDataGenerator { config }
    }

    /// Generate random u128 values
    pub fn generate_u128_strategy() -> impl Strategy<Value = u128> {
        0u128..u128::MAX
    }

    /// Generate random u64 values
    pub fn generate_u64_strategy() -> impl Strategy<Value = u64> {
        0u64..u64::MAX
    }

    /// Generate random i128 values
    pub fn generate_i128_strategy() -> impl Strategy<Value = i128> {
        any::<i128>()
    }

    /// Generate random boolean values
    pub fn generate_bool_strategy() -> impl Strategy<Value = bool> {
        any::<bool>()
    }

    /// Generate random byte arrays
    pub fn generate_bytes_strategy(max_len: usize) -> impl Strategy<Value = Vec<u8>> {
        prop::collection::vec(0u8..255, 0..max_len)
    }

    /// Generate edge case u128 values (boundaries, zero, max, etc.)
    pub fn generate_u128_edge_cases() -> Vec<u128> {
        vec![
            0,
            1,
            u128::MAX / 2,
            u128::MAX - 1,
            u128::MAX,
            100,
            1000,
            1_000_000,
        ]
    }

    /// Generate edge case u64 values
    pub fn generate_u64_edge_cases() -> Vec<u64> {
        vec![
            0,
            1,
            u64::MAX / 2,
            u64::MAX - 1,
            u64::MAX,
            100,
            1000,
            1_000_000,
        ]
    }

    /// Generate edge case i128 values
    pub fn generate_i128_edge_cases() -> Vec<i128> {
        vec![
            0,
            1,
            -1,
            i128::MAX,
            i128::MIN,
            i128::MAX / 2,
            i128::MIN / 2,
            100,
            -100,
        ]
    }

    /// Generate amounts with realistic DeFi values
    pub fn generate_defi_amounts() -> Vec<u128> {
        vec![
            1_000_000,           // 1 token (6 decimals)
            10_000_000,          // 10 tokens
            100_000_000,         // 100 tokens
            1_000_000_000,       // 1000 tokens
            1_000_000_000_000,   // 1M tokens
            1_000_000_000_000_000, // 1B tokens
        ]
    }

    /// Generate blockchain addresses for testing
    pub fn generate_address_scenarios() -> Vec<String> {
        vec![
            "GBRPYHIL2CI3FV4BMSXIzsqgccdtuj74RTYCXSZKBCTGVL7MGPHD5Q2C".to_string(),
            "GBZXN7PIRZGNMHGA7MUQCSOMZU32UXEIOTZ4YC5ZTSXILTZXJJWJBhenderson".to_string(),
            "GCZST3XVCDTUJ76RTYCXSZKBCTGVL7MGPHD5Q2CBRPYHIL2CI3FV4B".to_string(),
        ]
    }

    /// Generate rate/fee values for testing
    pub fn generate_rate_values() -> Vec<u128> {
        vec![
            0,            // 0%
            100,          // 0.1%
            500,          // 0.5%
            1000,         // 1%
            5000,         // 5%
            10000,        // 10%
        ]
    }

    /// Generate duration values (in seconds)
    pub fn generate_duration_values() -> Vec<u64> {
        vec![
            0,               // immediate
            60,              // 1 minute
            3600,            // 1 hour
            86400,           // 1 day
            604800,          // 1 week
            2592000,         // 30 days
            31536000,        // 1 year
        ]
    }

    /// Generate collection scenarios
    pub fn generate_collection_scenarios() -> Vec<Vec<u128>> {
        vec![
            vec![],                           // empty
            vec![1],                          // single element
            vec![1, 2, 3, 4, 5],             // small collection
            (1..=50).collect(),              // medium collection
            (1..=100).collect(),             // large collection
        ]
    }

    /// Generate authorization scenarios
    pub fn generate_auth_scenarios() -> Vec<&'static str> {
        vec![
            "owner",
            "admin",
            "user",
            "unauthorized",
            "expired",
            "revoked",
        ]
    }
}

/// Scenario-based test data generator for realistic testing
pub struct ScenarioDataGenerator;

impl ScenarioDataGenerator {
    /// Generate a complete DeFi swap scenario
    pub fn generate_swap_scenario() -> SwapScenario {
        SwapScenario {
            from_amount: 1_000_000,
            min_out_amount: 900_000,
            max_price: Some(1_100_000),
            fee_percentage: 500, // 0.5%
        }
    }

    /// Generate a staking scenario
    pub fn generate_staking_scenario() -> StakingScenario {
        StakingScenario {
            initial_stake: 10_000_000,
            lock_period: 2592000, // 30 days
            reward_rate: 1000,    // 1%
            min_stake_amount: 1_000_000,
        }
    }

    /// Generate a lending scenario
    pub fn generate_lending_scenario() -> LendingScenario {
        LendingScenario {
            borrow_amount: 5_000_000,
            collateral_amount: 10_000_000,
            interest_rate: 500,  // 0.5%
            loan_duration: 604800, // 1 week
        }
    }

    /// Generate a governance scenario
    pub fn generate_governance_scenario() -> GovernanceScenario {
        GovernanceScenario {
            proposal_threshold: 100_000,
            voting_period: 259200,  // 3 days
            execution_delay: 86400, // 1 day
            quorum_percentage: 4000, // 40%
        }
    }

    /// Generate realistic token metadata
    pub fn generate_token_metadata() -> TokenMetadata {
        TokenMetadata {
            name: "Test Token".to_string(),
            symbol: "TST".to_string(),
            decimals: 6,
            initial_supply: 1_000_000_000_000_000,
        }
    }

    /// Generate NFT metadata
    pub fn generate_nft_metadata() -> NftMetadata {
        NftMetadata {
            name: "Test NFT".to_string(),
            symbol: "TNFT".to_string(),
            uri: "https://example.com/metadata/1".to_string(),
            royalty_percentage: 500, // 5%
        }
    }

    /// Generate contract initialization parameters
    pub fn generate_contract_init_params() -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("admin".to_string(), "GBZXN7PIRZGNMHGA7".to_string());
        params.insert("initial_supply".to_string(), "1000000".to_string());
        params.insert("paused".to_string(), "false".to_string());
        params
    }

    /// Generate access control scenarios
    pub fn generate_access_control_scenarios() -> Vec<AccessControlScenario> {
        vec![
            AccessControlScenario {
                caller: "owner".to_string(),
                action: "admin_action".to_string(),
                should_succeed: true,
            },
            AccessControlScenario {
                caller: "user".to_string(),
                action: "admin_action".to_string(),
                should_succeed: false,
            },
            AccessControlScenario {
                caller: "owner".to_string(),
                action: "user_action".to_string(),
                should_succeed: true,
            },
        ]
    }
}

/// DeFi swap test scenario
#[derive(Debug, Clone)]
pub struct SwapScenario {
    pub from_amount: u128,
    pub min_out_amount: u128,
    pub max_price: Option<u128>,
    pub fee_percentage: u128,
}

/// Staking test scenario
#[derive(Debug, Clone)]
pub struct StakingScenario {
    pub initial_stake: u128,
    pub lock_period: u64,
    pub reward_rate: u128,
    pub min_stake_amount: u128,
}

/// Lending test scenario
#[derive(Debug, Clone)]
pub struct LendingScenario {
    pub borrow_amount: u128,
    pub collateral_amount: u128,
    pub interest_rate: u128,
    pub loan_duration: u64,
}

/// Governance test scenario
#[derive(Debug, Clone)]
pub struct GovernanceScenario {
    pub proposal_threshold: u128,
    pub voting_period: u64,
    pub execution_delay: u64,
    pub quorum_percentage: u128,
}

/// Token metadata for testing
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub initial_supply: u128,
}

/// NFT metadata for testing
#[derive(Debug, Clone)]
pub struct NftMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub royalty_percentage: u128,
}

/// Access control test scenario
#[derive(Debug, Clone)]
pub struct AccessControlScenario {
    pub caller: String,
    pub action: String,
    pub should_succeed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_data_generator_creation() {
        let generator = ContractDataGenerator::new();
        assert_eq!(
            generator.config.max_string_length,
            256
        );
        assert!(generator.config.include_edge_cases);
    }

    #[test]
    fn test_u128_edge_cases() {
        let edge_cases = ContractDataGenerator::generate_u128_edge_cases();
        assert_eq!(edge_cases[0], 0);
        assert_eq!(edge_cases[edge_cases.len() - 1], 1_000_000);
        assert!(edge_cases.contains(&u128::MAX));
    }

    #[test]
    fn test_defi_amounts_generation() {
        let amounts = ContractDataGenerator::generate_defi_amounts();
        assert!(amounts.len() > 0);
        assert!(amounts.is_sorted());
    }

    #[test]
    fn test_scenario_generator_swap() {
        let scenario = ScenarioDataGenerator::generate_swap_scenario();
        assert!(scenario.from_amount > 0);
        assert!(scenario.min_out_amount <= scenario.from_amount);
    }

    #[test]
    fn test_scenario_generator_staking() {
        let scenario = ScenarioDataGenerator::generate_staking_scenario();
        assert!(scenario.initial_stake > 0);
        assert!(scenario.lock_period > 0);
    }

    #[test]
    fn test_token_metadata_generation() {
        let metadata = ScenarioDataGenerator::generate_token_metadata();
        assert!(!metadata.name.is_empty());
        assert_eq!(metadata.decimals, 6);
    }

    #[test]
    fn test_access_control_scenarios() {
        let scenarios = ScenarioDataGenerator::generate_access_control_scenarios();
        assert_eq!(scenarios.len(), 3);
        assert!(scenarios[0].should_succeed);
        assert!(!scenarios[1].should_succeed);
    }
}

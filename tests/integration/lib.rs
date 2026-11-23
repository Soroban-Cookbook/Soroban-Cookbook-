/// Soroban Integration Test Library
///
/// This library provides shared modules for integration testing including
/// test data generators, parallel test utilities, and common test infrastructure.

pub mod test_data_generators;
pub mod parallel_tests;

pub use test_data_generators::{
    ContractDataGenerator, ScenarioDataGenerator, GeneratorConfig,
    SwapScenario, StakingScenario, LendingScenario, GovernanceScenario,
    TokenMetadata, NftMetadata, AccessControlScenario,
};

#[allow(unused_imports)]
pub use parallel_tests::*;

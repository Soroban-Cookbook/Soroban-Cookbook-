//! # Integration Test Framework
//!
//! Demonstrates the comprehensive test framework architecture for Soroban
//! Cookbook contracts.  Introduces reusable helpers, mock contracts,
//! setup/teardown fixtures, and patterns for testing governance + cross-contract
//! scenarios.
//!
//! ## Architecture
//!
//! ```text
//! tests/
//! ├── framework_tests.rs    ← this file (framework demo tests)
//! ├── integration_tests.rs  ← existing cross-contract tests
//! ├── helpers/
//! │   └── mod.rs            ← reusable test utilities & fixtures
//! └── mocks/
//!     └── mod.rs            ← lightweight mock contracts
//! ```

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

mod helpers;
mod mocks;

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, Env, IntoVal, String, Symbol, Vec,
};

// ===========================================================================
// Section 1: Tests Using Helpers & Fixtures
// ===========================================================================

#[test]
fn test_fixture_auth_setup() {
    let env = helpers::setup_env();
    let fixture = helpers::AuthFixture::setup(&env, 3, 500);

    // Verify all users are funded
    for user in &fixture.users {
        helpers::assert_balance(&env, &fixture.contract_id, user, 500);
    }
}

#[test]
fn test_fixture_events_counter() {
    let env = helpers::setup_env();
    let events = helpers::EventsFixture::setup(&env);

    events.increment(&env);
    events.increment(&env);
    events.increment(&env);

    assert_eq!(events.get_count(&env), 3);
}

#[test]
fn test_fixture_storage_patterns() {
    let env = helpers::setup_env();
    let storage = helpers::StorageFixture::setup(&env);

    storage.set_persistent(&env, symbol_short!("key1"), 42);
    assert_eq!(
        storage.get_persistent(&env, symbol_short!("key1")),
        Some(42)
    );

    storage.set_persistent(&env, symbol_short!("key1"), 100);
    assert_eq!(
        storage.get_persistent(&env, symbol_short!("key1")),
        Some(100)
    );
}

#[test]
fn test_combined_fixtures_workflow() {
    let env = helpers::setup_env();
    let auth = helpers::AuthFixture::setup(&env, 2, 1000);
    let events = helpers::EventsFixture::setup(&env);
    let storage = helpers::StorageFixture::setup(&env);

    // Transfer between users via auth contract
    env.invoke_contract::<()>(
        &auth.contract_id,
        &symbol_short!("transfer"),
        Vec::from_array(
            &env,
            [
                auth.users[0].clone().into_val(&env),
                auth.users[1].clone().into_val(&env),
                200i128.into_val(&env),
            ],
        ),
    );
    events.increment(&env);

    helpers::assert_balance(&env, &auth.contract_id, &auth.users[0], 800);
    helpers::assert_balance(&env, &auth.contract_id, &auth.users[1], 1200);

    // Track in storage
    storage.set_persistent(&env, symbol_short!("tx_count"), 1);
    assert_eq!(
        storage.get_persistent(&env, symbol_short!("tx_count")),
        Some(1)
    );
    assert_eq!(events.get_count(&env), 1);
}

// ===========================================================================
// Section 2: Tests Using Mock Contracts
// ===========================================================================

#[test]
fn test_mock_token_mint_and_transfer() {
    let env = helpers::setup_env();

    #[allow(deprecated)]
    let token_id = env.register_contract(None, mocks::MockToken);
    let client = mocks::MockTokenClient::new(&env, &token_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.mint(&alice, &1000);
    assert_eq!(client.balance(&alice), 1000);
    assert_eq!(client.supply(), 1000);

    client.transfer(&alice, &bob, &300);
    assert_eq!(client.balance(&alice), 700);
    assert_eq!(client.balance(&bob), 300);
    assert_eq!(client.supply(), 1000);
}

#[test]
fn test_mock_oracle_price_feed() {
    let env = helpers::setup_env();

    #[allow(deprecated)]
    let oracle_id = env.register_contract(None, mocks::MockOracle);
    let client = mocks::MockOracleClient::new(&env, &oracle_id);

    let admin = Address::generate(&env);
    let xlm = Symbol::new(&env, "XLM");
    let btc = Symbol::new(&env, "BTC");

    client.init(&admin);
    client.set_price(&admin, &xlm, &100_000);
    client.set_price(&admin, &btc, &50_000_000_000);

    assert_eq!(client.get_price(&xlm), Some(100_000));
    assert_eq!(client.get_price(&btc), Some(50_000_000_000));
}

#[test]
fn test_mock_timelock_lock_and_unlock() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);

    #[allow(deprecated)]
    let tl_id = env.register_contract(None, mocks::MockTimelock);
    let client = mocks::MockTimelockClient::new(&env, &tl_id);

    let key = symbol_short!("proposal");
    client.lock(&key, &2000);

    // Still locked at t=1000
    assert!(client.is_locked(&key));
    assert!(!client.unlock(&key));

    // Advance past unlock time
    env.ledger().set_timestamp(2001);

    assert!(!client.is_locked(&key));
    assert!(client.unlock(&key));
}

// ===========================================================================
// Section 3: Governance Integration Tests (voting contract)
// ===========================================================================

#[test]
fn test_governance_voting_lifecycle() {
    let env = helpers::setup_env();

    #[allow(deprecated)]
    let voting_id = env.register_contract(None, simple_voting::VotingContract);
    let client = simple_voting::VotingContractClient::new(&env, &voting_id);

    let admin = Address::generate(&env);
    let voters = helpers::generate_addresses(&env, 5);

    // Initialize
    client.initialize(&admin);

    // Create proposal with future deadline
    env.ledger().set_timestamp(100);
    let prop_id = client.create_prop(&admin, &String::from_str(&env, "funding"), &500u64);
    assert_eq!(prop_id, 1);

    // All 5 voters cast votes: 3 For, 1 Against, 1 Abstain
    client.cast_vote(&voters[0], &prop_id, &simple_voting::VoteChoice::For);
    client.cast_vote(&voters[1], &prop_id, &simple_voting::VoteChoice::For);
    client.cast_vote(&voters[2], &prop_id, &simple_voting::VoteChoice::For);
    client.cast_vote(&voters[3], &prop_id, &simple_voting::VoteChoice::Against);
    client.cast_vote(&voters[4], &prop_id, &simple_voting::VoteChoice::Abstain);

    // Tally
    let (yes, no, abstain) = client.tally(&prop_id);
    assert_eq!(yes, 3);
    assert_eq!(no, 1);
    assert_eq!(abstain, 1);

    // Advance past deadline and execute
    env.ledger().set_timestamp(501);
    let status = client.execute(&prop_id);
    assert_eq!(status, simple_voting::ProposalStatus::Passed);
}

#[test]
fn test_governance_with_events_tracking() {
    let env = helpers::setup_env();
    let events = helpers::EventsFixture::setup(&env);

    #[allow(deprecated)]
    let voting_id = env.register_contract(None, simple_voting::VotingContract);
    let client = simple_voting::VotingContractClient::new(&env, &voting_id);

    let admin = Address::generate(&env);
    let voter = Address::generate(&env);

    client.initialize(&admin);
    events.increment(&env); // Track init

    env.ledger().set_timestamp(100);
    client.create_prop(&admin, &String::from_str(&env, "upgrade"), &300u64);
    events.increment(&env); // Track proposal

    client.cast_vote(&voter, &1u32, &simple_voting::VoteChoice::For);
    events.increment(&env); // Track vote

    assert_eq!(events.get_count(&env), 3);
}

#[test]
fn test_governance_multi_proposal_concurrent() {
    let env = helpers::setup_env();

    #[allow(deprecated)]
    let voting_id = env.register_contract(None, simple_voting::VotingContract);
    let client = simple_voting::VotingContractClient::new(&env, &voting_id);

    let admin = Address::generate(&env);
    let voters = helpers::generate_addresses(&env, 3);

    client.initialize(&admin);
    env.ledger().set_timestamp(100);

    // Create two proposals with different deadlines
    let p1 = client.create_prop(&admin, &String::from_str(&env, "propA"), &200u64);
    let p2 = client.create_prop(&admin, &String::from_str(&env, "propB"), &400u64);

    // Voters split across proposals
    client.cast_vote(&voters[0], &p1, &simple_voting::VoteChoice::For);
    client.cast_vote(&voters[1], &p1, &simple_voting::VoteChoice::Against);
    client.cast_vote(&voters[2], &p1, &simple_voting::VoteChoice::Against);

    client.cast_vote(&voters[0], &p2, &simple_voting::VoteChoice::For);
    client.cast_vote(&voters[1], &p2, &simple_voting::VoteChoice::For);

    // Execute p1 after its deadline
    env.ledger().set_timestamp(201);
    let s1 = client.execute(&p1);
    assert_eq!(s1, simple_voting::ProposalStatus::Rejected); // 1 For, 2 Against

    // p2 still active
    let prop2 = client.get_prop(&p2);
    assert_eq!(prop2.status, simple_voting::ProposalStatus::Active);

    // Execute p2 after its deadline
    env.ledger().set_timestamp(401);
    let s2 = client.execute(&p2);
    assert_eq!(s2, simple_voting::ProposalStatus::Passed); // 2 For, 0 Against
}

#[test]
fn test_governance_timelock_gated_execution() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(100);

    #[allow(deprecated)]
    let voting_id = env.register_contract(None, simple_voting::VotingContract);
    let client = simple_voting::VotingContractClient::new(&env, &voting_id);

    #[allow(deprecated)]
    let tl_id = env.register_contract(None, mocks::MockTimelock);
    let tl_client = mocks::MockTimelockClient::new(&env, &tl_id);

    let admin = Address::generate(&env);
    let voter = Address::generate(&env);

    // Set up voting
    client.initialize(&admin);
    let prop_id = client.create_prop(&admin, &String::from_str(&env, "timelk"), &200u64);
    client.cast_vote(&voter, &prop_id, &simple_voting::VoteChoice::For);

    // Lock the execution behind a timelock (300)
    tl_client.lock(&symbol_short!("exec"), &300);

    // Advance past vote deadline but still timelocked
    env.ledger().set_timestamp(250);
    assert!(tl_client.is_locked(&symbol_short!("exec")));

    // Advance past timelock
    env.ledger().set_timestamp(301);
    assert!(!tl_client.is_locked(&symbol_short!("exec")));

    // Now execute the proposal
    let status = client.execute(&prop_id);
    assert_eq!(status, simple_voting::ProposalStatus::Passed);
}

// ===========================================================================
// Section 4: Setup/Teardown Patterns
// ===========================================================================

/// Demonstrates a test that verifies storage cleanup/reset behavior.
#[test]
fn test_setup_teardown_storage_reset() {
    let env = helpers::setup_env();
    let storage = helpers::StorageFixture::setup(&env);

    // Setup: populate storage
    storage.set_persistent(&env, symbol_short!("count"), 10);
    assert_eq!(
        storage.get_persistent(&env, symbol_short!("count")),
        Some(10)
    );

    // Simulate teardown: remove the key
    env.invoke_contract::<()>(
        &storage.contract_id,
        &Symbol::new(&env, "remove_persistent"),
        Vec::from_array(&env, [symbol_short!("count").into_val(&env)]),
    );

    // Verify cleanup
    let has: bool = env.invoke_contract(
        &storage.contract_id,
        &Symbol::new(&env, "has_persistent"),
        Vec::from_array(&env, [symbol_short!("count").into_val(&env)]),
    );
    assert!(!has);
}

/// Demonstrates that each test environment is fully isolated — no state leaks.
#[test]
fn test_environment_isolation() {
    // Test A: set a value
    {
        let env = helpers::setup_env();
        let storage = helpers::StorageFixture::setup(&env);
        storage.set_persistent(&env, symbol_short!("shared"), 999);
        assert_eq!(
            storage.get_persistent(&env, symbol_short!("shared")),
            Some(999)
        );
    }

    // Test B: fresh env sees no prior state
    {
        let env = helpers::setup_env();
        let storage = helpers::StorageFixture::setup(&env);
        assert_eq!(storage.get_persistent(&env, symbol_short!("shared")), None);
    }
}

/// Demonstrates timestamp-based setup for deadline-sensitive tests.
#[test]
fn test_timestamp_setup_helper() {
    let env = helpers::setup_env_with_timestamp(5000);
    assert_eq!(env.ledger().timestamp(), 5000);

    // Advance
    env.ledger().set_timestamp(6000);
    assert_eq!(env.ledger().timestamp(), 6000);
}

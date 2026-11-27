#![allow(deprecated)]
//! # Multi-Contract Testing Examples
//!
//! Comprehensive tests demonstrating Soroban's multi-contract testing patterns:
//! - Registering multiple mock contracts in the same environment
//! - Cross-contract calls between contracts
//! - State verification across contract boundaries
//! - Using mock_all_auths() for automatic authorization
//! - Using mock_auths() for targeted authorization control

use crate::governance::{Governance, GovernanceClient, GovernanceError};
use crate::treasury::{Treasury, TreasuryClient, TreasuryError};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, IntoVal, Symbol,
};

// ══════════════════════════════════════════════════════════════════════════════
// BASIC MULTI-CONTRACT SETUP
// ══════════════════════════════════════════════════════════════════════════════

/// Test registering two contracts in the same test environment
#[test]
fn test_register_multiple_contracts() {
    let env = Env::default();

    // Register both contracts in the same environment
    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);

    // Create clients for both contracts
    let treasury = TreasuryClient::new(&env, &treasury_id);
    let governance = GovernanceClient::new(&env, &governance_id);

    // Verify both contracts are initialized independently
    env.mock_all_auths();

    let admin = Address::generate(&env);
    treasury.initialize(&admin);

    let voters = vec![&env, Address::generate(&env)];
    governance.initialize(&treasury_id, &voters, &1);

    // Both contracts should be functional
    assert_eq!(treasury.balance(), 0);
    assert_eq!(treasury.admin(), Some(admin));
}

// ══════════════════════════════════════════════════════════════════════════════
// CROSS-CONTRACT CALLS WITH STATE VERIFICATION
// ══════════════════════════════════════════════════════════════════════════════

/// Test cross-contract call with state verification on both sides
#[test]
fn test_cross_contract_call_with_state_verification() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup: Register both contracts
    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);

    let treasury = TreasuryClient::new(&env, &treasury_id);
    let governance = GovernanceClient::new(&env, &governance_id);

    // Initialize treasury with governance as admin
    treasury.initialize(&governance_id);

    // Setup voters
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let voters = vec![&env, voter1.clone(), voter2.clone()];

    governance.initialize(&treasury_id, &voters, &2);

    // Deposit funds into treasury
    let depositor = Address::generate(&env);
    treasury.deposit(&depositor, &1000);

    // Verify treasury state before cross-contract call
    assert_eq!(treasury.balance(), 1000);
    assert_eq!(treasury.total_withdrawn(), 0);

    // Create proposal in governance
    let recipient = Address::generate(&env);
    let proposal_id = governance.propose(&voter1, &recipient, &300);

    // Verify governance state
    assert_eq!(governance.vote_count(&proposal_id), 0);
    assert!(!governance.is_executed(&proposal_id));

    // Vote on proposal
    governance.vote(&proposal_id, &voter1);
    governance.vote(&proposal_id, &voter2);

    // Verify votes were recorded
    assert_eq!(governance.vote_count(&proposal_id), 2);

    // Execute proposal - this triggers cross-contract call
    governance.execute(&proposal_id, &voter1);

    // Verify state on BOTH contracts after cross-contract call
    // Treasury state should be updated
    assert_eq!(treasury.balance(), 700); // 1000 - 300
    assert_eq!(treasury.total_withdrawn(), 300);

    // Governance state should be updated
    assert!(governance.is_executed(&proposal_id));
    let proposal = governance.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
    assert_eq!(proposal.amount, 300);
}

/// Test that cross-contract call failures don't corrupt state
#[test]
fn test_cross_contract_call_failure_handling() {
    let env = Env::default();
    env.mock_all_auths();

    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);

    let treasury = TreasuryClient::new(&env, &treasury_id);
    let governance = GovernanceClient::new(&env, &governance_id);

    // Initialize with governance as treasury admin
    treasury.initialize(&governance_id);

    let voter1 = Address::generate(&env);
    let voters = vec![&env, voter1.clone()];
    governance.initialize(&treasury_id, &voters, &1);

    // Create proposal for more than treasury has
    let recipient = Address::generate(&env);
    let proposal_id = governance.propose(&voter1, &recipient, &1000);
    governance.vote(&proposal_id, &voter1);

    // Attempt to execute - should fail due to insufficient balance
    let result = governance.try_execute(&proposal_id, &voter1);
    assert!(result.is_err());

    // Verify states: proposal should NOT be marked executed
    assert!(!governance.is_executed(&proposal_id));

    // Treasury state should be unchanged
    assert_eq!(treasury.balance(), 0);
    assert_eq!(treasury.total_withdrawn(), 0);
}

// ══════════════════════════════════════════════════════════════════════════════
// MOCK_ALL_AUTHS() PATTERN
// ══════════════════════════════════════════════════════════════════════════════

/// Demonstrate mock_all_auths() - automatically mocks all authorization checks
#[test]
fn test_mock_all_auths_pattern() {
    let env = Env::default();

    // mock_all_auths() allows any address to pass require_auth() checks
    env.mock_all_auths();

    let treasury_id = env.register_contract(None, Treasury);
    let treasury = TreasuryClient::new(&env, &treasury_id);

    let admin = Address::generate(&env);
    treasury.initialize(&admin);

    // With mock_all_auths(), we can call methods requiring auth
    // without setting up specific auth contexts
    let depositor = Address::generate(&env);
    treasury.deposit(&depositor, &500);

    let recipient = Address::generate(&env);
    treasury.withdraw(&admin, &recipient, &200);

    assert_eq!(treasury.balance(), 300);
}

/// Test complex multi-step workflow with mock_all_auths()
#[test]
fn test_mock_all_auths_complex_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);

    let treasury = TreasuryClient::new(&env, &treasury_id);
    let governance = GovernanceClient::new(&env, &governance_id);

    treasury.initialize(&governance_id);

    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let voter3 = Address::generate(&env);
    let voters = vec![&env, voter1.clone(), voter2.clone(), voter3.clone()];

    governance.initialize(&treasury_id, &voters, &2);

    // Fund treasury
    let depositor = Address::generate(&env);
    treasury.deposit(&depositor, &5000);

    // Create multiple proposals
    let recipient1 = Address::generate(&env);
    let proposal_id1 = governance.propose(&voter1, &recipient1, &1000);

    let recipient2 = Address::generate(&env);
    let proposal_id2 = governance.propose(&voter2, &recipient2, &1500);

    // Vote and execute first proposal
    governance.vote(&proposal_id1, &voter1);
    governance.vote(&proposal_id1, &voter2);
    governance.execute(&proposal_id1, &voter1);

    // Vote and execute second proposal
    governance.vote(&proposal_id2, &voter2);
    governance.vote(&proposal_id2, &voter3);
    governance.execute(&proposal_id2, &voter2);

    // Verify final state across both contracts
    assert_eq!(treasury.balance(), 2500); // 5000 - 1000 - 1500
    assert_eq!(treasury.total_withdrawn(), 2500);
    assert!(governance.is_executed(&proposal_id1));
    assert!(governance.is_executed(&proposal_id2));
}

// ══════════════════════════════════════════════════════════════════════════════
// MOCK_AUTHS() PATTERN - TARGETED AUTHORIZATION
// ══════════════════════════════════════════════════════════════════════════════

/// Demonstrate targeted mock_auths() for specific authorization control
#[test]
fn test_targeted_mock_auths() {
    let env = Env::default();

    let treasury_id = env.register_contract(None, Treasury);
    let treasury = TreasuryClient::new(&env, &treasury_id);

    let admin = Address::generate(&env);

    // Mock authorization for ONLY the initialize call
    env.mock_auths(&[]);
    treasury.initialize(&admin);

    // Now mock authorization for a specific depositor
    let depositor = Address::generate(&env);
    env.mock_auths(&[soroban_sdk::MockAuth {
        address: &depositor,
        invoke: &soroban_sdk::MockAuthInvoke {
            contract: &treasury_id,
            fn_name: "deposit",
            args: (depositor.clone(), 1000i128).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    treasury.deposit(&depositor, &1000);
    assert_eq!(treasury.balance(), 1000);

    // Mock authorization for admin withdraw
    let recipient = Address::generate(&env);
    env.mock_auths(&[soroban_sdk::MockAuth {
        address: &admin,
        invoke: &soroban_sdk::MockAuthInvoke {
            contract: &treasury_id,
            fn_name: "withdraw",
            args: (admin.clone(), recipient.clone(), 500i128).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    treasury.withdraw(&admin, &recipient, &500);
    assert_eq!(treasury.balance(), 500);
}

/// Test cross-contract authorization with targeted mocks
#[test]
fn test_cross_contract_with_targeted_auth() {
    let env = Env::default();

    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);

    let treasury = TreasuryClient::new(&env, &treasury_id);
    let governance = GovernanceClient::new(&env, &governance_id);

    // Initialize both contracts
    env.mock_all_auths();
    treasury.initialize(&governance_id);

    let voter1 = Address::generate(&env);
    let voters = vec![&env, voter1.clone()];
    governance.initialize(&treasury_id, &voters, &1);

    // Fund treasury
    let depositor = Address::generate(&env);
    treasury.deposit(&depositor, &2000);

    // Create and vote on proposal using mock_all_auths
    let recipient = Address::generate(&env);
    let proposal_id = governance.propose(&voter1, &recipient, &800);
    governance.vote(&proposal_id, &voter1);

    // For execution, use targeted auth to be more precise
    env.mock_auths(&[soroban_sdk::MockAuth {
        address: &voter1,
        invoke: &soroban_sdk::MockAuthInvoke {
            contract: &governance_id,
            fn_name: "execute",
            args: (proposal_id, voter1.clone()).into_val(&env),
            sub_invokes: &[soroban_sdk::MockAuth {
                address: &governance_id,
                invoke: &soroban_sdk::MockAuthInvoke {
                    contract: &treasury_id,
                    fn_name: "withdraw",
                    args: (governance_id.clone(), recipient.clone(), 800i128).into_val(&env),
                    sub_invokes: &[],
                },
            }],
        },
    }]);

    governance.execute(&proposal_id, &voter1);

    // Verify state on both contracts
    assert_eq!(treasury.balance(), 1200);
    assert!(governance.is_executed(&proposal_id));
}

// ══════════════════════════════════════════════════════════════════════════════
// AUTHORIZATION VERIFICATION
// ══════════════════════════════════════════════════════════════════════════════

/// Test that we can verify authorization calls were made correctly
#[test]
fn test_verify_authorization_calls() {
    let env = Env::default();
    env.mock_all_auths();

    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);

    let treasury = TreasuryClient::new(&env, &treasury_id);
    let governance = GovernanceClient::new(&env, &governance_id);

    treasury.initialize(&governance_id);

    let voter1 = Address::generate(&env);
    let voters = vec![&env, voter1.clone()];
    governance.initialize(&treasury_id, &voters, &1);

    let depositor = Address::generate(&env);
    treasury.deposit(&depositor, &1000);

    let recipient = Address::generate(&env);
    let proposal_id = governance.propose(&voter1, &recipient, &400);
    governance.vote(&proposal_id, &voter1);
    governance.execute(&proposal_id, &voter1);

    // Verify the authorization structure for the execute call
    let auth_snapshot = env.auths();

    // The voter1 should have authorized the execute call
    assert!(!auth_snapshot.is_empty());

    // Verify the execute call authorized properly
    let execute_auth = auth_snapshot
        .iter()
        .find(|(addr, _)| addr == &voter1)
        .expect("Should have auth for voter1");

    // Check that the invocation tree is correct
    assert_eq!(
        execute_auth.1.first().unwrap().function,
        AuthorizedFunction::Contract((
            governance_id.clone(),
            Symbol::new(&env, "execute"),
            (proposal_id, voter1.clone()).into_val(&env),
        ))
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// EDGE CASES AND ERROR SCENARIOS
// ══════════════════════════════════════════════════════════════════════════════

/// Test unauthorized cross-contract call attempts
#[test]
fn test_unauthorized_cross_contract_call() {
    let env = Env::default();
    env.mock_all_auths();

    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);

    let treasury = TreasuryClient::new(&env, &treasury_id);
    let governance = GovernanceClient::new(&env, &governance_id);

    // Initialize treasury with a DIFFERENT admin (not governance)
    let other_admin = Address::generate(&env);
    treasury.initialize(&other_admin);

    let voter1 = Address::generate(&env);
    let voters = vec![&env, voter1.clone()];
    governance.initialize(&treasury_id, &voters, &1);

    // Fund treasury
    let depositor = Address::generate(&env);
    treasury.deposit(&depositor, &1000);

    // Create and approve proposal
    let recipient = Address::generate(&env);
    let proposal_id = governance.propose(&voter1, &recipient, &500);
    governance.vote(&proposal_id, &voter1);

    // Execute should fail because governance is not the treasury admin
    let result = governance.try_execute(&proposal_id, &voter1);
    assert_eq!(result, Err(Ok(GovernanceError::TreasuryCallFailed)));

    // Verify treasury state unchanged
    assert_eq!(treasury.balance(), 1000);
}

/// Test multiple voters and threshold requirements
#[test]
fn test_multi_voter_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);

    let treasury = TreasuryClient::new(&env, &treasury_id);
    let governance = GovernanceClient::new(&env, &governance_id);

    treasury.initialize(&governance_id);

    // Create 5 voters with threshold of 3
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let voter3 = Address::generate(&env);
    let voter4 = Address::generate(&env);
    let voter5 = Address::generate(&env);
    let voters = vec![
        &env,
        voter1.clone(),
        voter2.clone(),
        voter3.clone(),
        voter4.clone(),
        voter5.clone(),
    ];

    governance.initialize(&treasury_id, &voters, &3);

    // Fund treasury
    let depositor = Address::generate(&env);
    treasury.deposit(&depositor, &10000);

    // Create proposal
    let recipient = Address::generate(&env);
    let proposal_id = governance.propose(&voter1, &recipient, &2000);

    // Try to execute with insufficient votes
    governance.vote(&proposal_id, &voter1);
    governance.vote(&proposal_id, &voter2);

    let result = governance.try_execute(&proposal_id, &voter1);
    assert_eq!(result, Err(Ok(GovernanceError::InsufficientVotes)));

    // Add third vote and execute
    governance.vote(&proposal_id, &voter3);
    governance.execute(&proposal_id, &voter1);

    // Verify execution
    assert_eq!(treasury.balance(), 8000);
    assert!(governance.is_executed(&proposal_id));
}

/// Test that double voting is prevented
#[test]
fn test_double_voting_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);

    let treasury = TreasuryClient::new(&env, &treasury_id);
    let governance = GovernanceClient::new(&env, &governance_id);

    treasury.initialize(&governance_id);

    let voter1 = Address::generate(&env);
    let voters = vec![&env, voter1.clone()];
    governance.initialize(&treasury_id, &voters, &1);

    let recipient = Address::generate(&env);
    let proposal_id = governance.propose(&voter1, &recipient, &100);

    governance.vote(&proposal_id, &voter1);

    // Attempt to vote again
    let result = governance.try_vote(&proposal_id, &voter1);
    assert_eq!(result, Err(Ok(GovernanceError::AlreadyVoted)));

    // Verify vote count is still 1
    assert_eq!(governance.vote_count(&proposal_id), 1);
}

/// Test contract state isolation
#[test]
fn test_contract_state_isolation() {
    let env = Env::default();
    env.mock_all_auths();

    // Create two separate treasury contracts
    let treasury1_id = env.register_contract(None, Treasury);
    let treasury2_id = env.register_contract(None, Treasury);

    let treasury1 = TreasuryClient::new(&env, &treasury1_id);
    let treasury2 = TreasuryClient::new(&env, &treasury2_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    treasury1.initialize(&admin1);
    treasury2.initialize(&admin2);

    // Deposit different amounts
    let depositor = Address::generate(&env);
    treasury1.deposit(&depositor, &1000);
    treasury2.deposit(&depositor, &2000);

    // Verify state is isolated
    assert_eq!(treasury1.balance(), 1000);
    assert_eq!(treasury2.balance(), 2000);
    assert_eq!(treasury1.admin(), Some(admin1.clone()));
    assert_eq!(treasury2.admin(), Some(admin2));

    // Withdraw from one shouldn't affect the other
    let recipient = Address::generate(&env);
    treasury1.withdraw(&admin1, &recipient, &500);

    assert_eq!(treasury1.balance(), 500);
    assert_eq!(treasury2.balance(), 2000); // Unchanged
}

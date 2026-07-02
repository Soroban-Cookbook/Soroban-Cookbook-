//! Governance Integration Tests (issue #587)
//!
//! Covers proposal-lifecycle, simple-voting, and voting-time-constraints
//! contracts.  Treasury and delegation scenarios use lightweight mock
//! contracts defined below because no dedicated treasury/delegation contract
//! exists in the repo yet.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger as _},
    Address, Env, IntoVal, String, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Mock: Treasury
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
enum TreasuryKey {
    Balance,
}

#[contract]
pub struct MockTreasury;

#[contractimpl]
impl MockTreasury {
    pub fn deposit(env: Env, amount: i128) {
        let bal: i128 = env
            .storage()
            .instance()
            .get(&TreasuryKey::Balance)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&TreasuryKey::Balance, &(bal + amount));
    }

    pub fn withdraw(env: Env, amount: i128) {
        let bal: i128 = env
            .storage()
            .instance()
            .get(&TreasuryKey::Balance)
            .unwrap_or(0);
        assert!(bal >= amount, "insufficient treasury balance");
        env.storage()
            .instance()
            .set(&TreasuryKey::Balance, &(bal - amount));
    }

    pub fn balance(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&TreasuryKey::Balance)
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Mock: Delegation Registry
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
enum DelegKey {
    Delegate(Address),
    Weight(Address),
}

#[contract]
pub struct MockDelegation;

#[contractimpl]
impl MockDelegation {
    pub fn delegate(env: Env, delegator: Address, delegate: Address, weight: i128) {
        delegator.require_auth();
        env.storage()
            .persistent()
            .set(&DelegKey::Delegate(delegator.clone()), &delegate);
        env.storage()
            .persistent()
            .set(&DelegKey::Weight(delegator), &weight);
    }

    pub fn undelegate(env: Env, delegator: Address) {
        delegator.require_auth();
        env.storage()
            .persistent()
            .remove(&DelegKey::Delegate(delegator.clone()));
        env.storage()
            .persistent()
            .remove(&DelegKey::Weight(delegator));
    }

    pub fn get_delegate(env: Env, delegator: Address) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DelegKey::Delegate(delegator))
    }

    pub fn get_weight(env: Env, delegator: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DelegKey::Weight(delegator))
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn new_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

/// Register + initialise proposal-lifecycle. Returns (contract_id, admin).
fn setup_gov(env: &Env, min_quorum: i128) -> (Address, Address) {
    let id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let admin = Address::generate(env);
    proposal_lifecycle::ProposalLifecycleContractClient::new(env, &id)
        .initialize(&admin, &min_quorum);
    (id, admin)
}

/// Register + initialise simple-voting. Returns (contract_id, admin).
fn setup_sv(env: &Env) -> (Address, Address) {
    let id = env.register_contract(None, simple_voting::VotingContract);
    let admin = Address::generate(env);
    simple_voting::VotingContractClient::new(env, &id).initialize(&admin);
    (id, admin)
}

/// Register + initialise voting-time-constraints. Returns (contract_id, admin).
fn setup_vtc(env: &Env, voting_period: u64, grace_period: u64) -> (Address, Address) {
    let id = env.register_contract(None, voting_time_constraints::VotingContract);
    let admin = Address::generate(env);
    voting_time_constraints::VotingContractClient::new(env, &id)
        .initialize(&admin, &voting_period, &grace_period);
    (id, admin)
}

/// Register a treasury mock and return its id.
fn setup_treasury(env: &Env) -> Address {
    env.register_contract(None, MockTreasury)
}

/// Register a delegation registry mock and return its id.
fn setup_delegation(env: &Env) -> Address {
    env.register_contract(None, MockDelegation)
}

// ===========================================================================
// 1. Proposal Creation
// ===========================================================================

#[test]
fn test_proposal_creation() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Fund community"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    assert_eq!(pid, 0, "first proposal ID must be 0");
}

// ===========================================================================
// 2. Proposal Validation — not-initialized error
// ===========================================================================

#[test]
fn test_proposal_validation_not_initialized() {
    let env = new_env();
    // Register without calling initialize.
    let id = env.register_contract(None, proposal_lifecycle::ProposalLifecycleContract);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &id);
    let proposer = Address::generate(&env);

    let result = client.try_create_proposal(
        &proposer,
        &String::from_str(&env, "x"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    assert!(result.is_err(), "must fail before initialization");
}

// ===========================================================================
// 3. Successful Voting
// ===========================================================================

#[test]
fn test_successful_voting() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Upgrade"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    client.submit_proposal(&proposer, &pid, &10u32, &10u32);

    let voter = Address::generate(&env);
    client.vote(&voter, &pid, &true, &1i128);

    let proposal = client.get_proposal(&pid);
    assert_eq!(proposal.votes_yes, 1);
    assert_eq!(proposal.votes_no, 0);
}

// ===========================================================================
// 4. Duplicate Vote Prevention
// ===========================================================================

#[test]
fn test_duplicate_vote_prevention() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Dup"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    client.submit_proposal(&proposer, &pid, &10u32, &10u32);

    let voter = Address::generate(&env);
    client.vote(&voter, &pid, &true, &1i128);

    let result = client.try_vote(&voter, &pid, &false, &1i128);
    assert!(result.is_err(), "second vote from same address must fail");
}

// ===========================================================================
// 5. Voting Deadline Enforcement
// ===========================================================================

#[test]
fn test_voting_deadline_enforcement() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Deadline"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    client.submit_proposal(&proposer, &pid, &5u32, &10u32);

    // Advance ledger past the voting window.
    env.ledger().set_sequence_number(env.ledger().sequence() + 6);

    let voter = Address::generate(&env);
    let result = client.try_vote(&voter, &pid, &true, &1i128);
    assert!(result.is_err(), "vote after deadline must be rejected");
}

// ===========================================================================
// 6. Proposal Execution
// ===========================================================================

#[test]
fn test_proposal_execution() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let treasury_id = setup_treasury(&env);
    let tc = MockTreasuryClient::new(&env, &treasury_id);
    tc.deposit(&200i128);

    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);
    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Withdraw 50"),
        &treasury_id,
        &Symbol::new(&env, "withdraw"),
        &Vec::from_array(&env, [50i128.into_val(&env)]),
    );
    client.submit_proposal(&proposer, &pid, &5u32, &10u32);

    let voter = Address::generate(&env);
    client.vote(&voter, &pid, &true, &5i128);

    // Advance past voting window.
    env.ledger().set_sequence_number(env.ledger().sequence() + 6);

    let executor = Address::generate(&env);
    client.execute_proposal(&executor, &pid);

    assert_eq!(tc.balance(), 150i128);
}

// ===========================================================================
// 7. Proposal Rejection
// ===========================================================================

#[test]
fn test_proposal_rejection() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Rejected"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    client.submit_proposal(&proposer, &pid, &5u32, &10u32);

    let voter = Address::generate(&env);
    client.vote(&voter, &pid, &false, &5i128);

    env.ledger().set_sequence_number(env.ledger().sequence() + 6);

    let state = client.get_proposal_state(&pid);
    assert_eq!(state, proposal_lifecycle::ProposalState::Failed);
}

// ===========================================================================
// 8. Treasury Deposit
// ===========================================================================

#[test]
fn test_treasury_deposit() {
    let env = new_env();
    let treasury_id = setup_treasury(&env);
    let tc = MockTreasuryClient::new(&env, &treasury_id);
    tc.deposit(&500i128);
    assert_eq!(tc.balance(), 500i128);
}

// ===========================================================================
// 9. Treasury Withdrawal via Governance
// ===========================================================================

#[test]
fn test_treasury_withdrawal_via_governance() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let treasury_id = setup_treasury(&env);
    let tc = MockTreasuryClient::new(&env, &treasury_id);
    tc.deposit(&300i128);

    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);
    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Withdraw 100"),
        &treasury_id,
        &Symbol::new(&env, "withdraw"),
        &Vec::from_array(&env, [100i128.into_val(&env)]),
    );
    client.submit_proposal(&proposer, &pid, &5u32, &10u32);

    let voter = Address::generate(&env);
    client.vote(&voter, &pid, &true, &3i128);

    env.ledger().set_sequence_number(env.ledger().sequence() + 6);
    client.execute_proposal(&proposer, &pid);

    assert_eq!(tc.balance(), 200i128);
}

// ===========================================================================
// 10. Treasury Authorization Failure (over-withdrawal)
// ===========================================================================

#[test]
#[should_panic]
fn test_treasury_auth_failure_over_withdrawal() {
    let env = new_env();
    let treasury_id = setup_treasury(&env);
    let tc = MockTreasuryClient::new(&env, &treasury_id);
    tc.deposit(&10i128);
    // Withdraw more than balance — contract asserts and panics.
    tc.withdraw(&100i128);
}

// ===========================================================================
// 11. Invalid Proposer — cannot cancel another's proposal
// ===========================================================================

#[test]
fn test_invalid_proposer_cancel() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "My proposal"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );

    let stranger = Address::generate(&env);
    let result = client.try_cancel_proposal(&stranger, &pid);
    assert!(result.is_err(), "stranger must not cancel another's proposal");
}

// ===========================================================================
// 12. Invalid Executor — cannot execute while voting is still open
// ===========================================================================

#[test]
fn test_invalid_executor_while_active() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Active exec"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    client.submit_proposal(&proposer, &pid, &10u32, &10u32);

    let voter = Address::generate(&env);
    client.vote(&voter, &pid, &true, &5i128);

    // Still within the voting window.
    let executor = Address::generate(&env);
    let result = client.try_execute_proposal(&executor, &pid);
    assert!(result.is_err(), "execution while voting is open must fail");
}

// ===========================================================================
// 13. Multiple Proposals — tracked independently
// ===========================================================================

#[test]
fn test_multiple_proposals() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid_a = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Proposal A"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    let pid_b = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Proposal B"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );

    assert_eq!(pid_a, 0);
    assert_eq!(pid_b, 1);
    assert_eq!(
        client.get_proposal_state(&pid_a),
        proposal_lifecycle::ProposalState::Draft
    );
    assert_eq!(
        client.get_proposal_state(&pid_b),
        proposal_lifecycle::ProposalState::Draft
    );
}

// ===========================================================================
// 14. Quorum Not Met — proposal fails even with yes-votes below threshold
// ===========================================================================

#[test]
fn test_quorum_not_met_fails_proposal() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 10); // quorum = 10
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Low quorum"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    client.submit_proposal(&proposer, &pid, &5u32, &10u32);

    // Only 2 total votes — below quorum of 10.
    let voter = Address::generate(&env);
    client.vote(&voter, &pid, &true, &2i128);

    env.ledger().set_sequence_number(env.ledger().sequence() + 6);

    let state = client.get_proposal_state(&pid);
    assert_eq!(state, proposal_lifecycle::ProposalState::Failed);
}

// ===========================================================================
// 15. Simple-Voting: Proposal Creation
// ===========================================================================

#[test]
fn test_sv_proposal_creation() {
    let env = new_env();
    env.ledger().set_timestamp(1_000);
    let (sv, admin) = setup_sv(&env);
    let client = simple_voting::VotingContractClient::new(&env, &sv);

    let pid = client.create_prop(&admin, &String::from_str(&env, "Fund v2"), &2_000u64);
    assert_eq!(pid, 1u32);

    let (f, a, ab) = client.tally(&pid);
    assert_eq!((f, a, ab), (0, 0, 0));
}

// ===========================================================================
// 16. Simple-Voting: Cast Vote
// ===========================================================================

#[test]
fn test_sv_cast_vote() {
    let env = new_env();
    env.ledger().set_timestamp(1_000);
    let (sv, admin) = setup_sv(&env);
    let client = simple_voting::VotingContractClient::new(&env, &sv);

    let pid = client.create_prop(&admin, &String::from_str(&env, "Vote test"), &5_000u64);

    let voter = Address::generate(&env);
    client.cast_vote(&voter, &pid, &simple_voting::VoteChoice::For);

    let (f, a, ab) = client.tally(&pid);
    assert_eq!(f, 1);
    assert_eq!(a, 0);
    assert_eq!(ab, 0);
    assert!(client.has_voted(&voter, &pid));
}

// ===========================================================================
// 17. Simple-Voting: Duplicate Vote Prevented
// ===========================================================================

#[test]
fn test_sv_duplicate_vote() {
    let env = new_env();
    env.ledger().set_timestamp(1_000);
    let (sv, admin) = setup_sv(&env);
    let client = simple_voting::VotingContractClient::new(&env, &sv);

    let pid = client.create_prop(&admin, &String::from_str(&env, "Dup"), &5_000u64);

    let voter = Address::generate(&env);
    client.cast_vote(&voter, &pid, &simple_voting::VoteChoice::For);

    let result = client.try_cast_vote(&voter, &pid, &simple_voting::VoteChoice::Against);
    assert!(result.is_err(), "second vote must be rejected");
}

// ===========================================================================
// 18. Simple-Voting: Deadline Enforcement
// ===========================================================================

#[test]
fn test_sv_deadline_enforcement() {
    let env = new_env();
    env.ledger().set_timestamp(1_000);
    let (sv, admin) = setup_sv(&env);
    let client = simple_voting::VotingContractClient::new(&env, &sv);

    let pid = client.create_prop(&admin, &String::from_str(&env, "Deadline"), &2_000u64);

    // Advance past deadline.
    env.ledger().set_timestamp(3_000);
    let voter = Address::generate(&env);
    let result = client.try_cast_vote(&voter, &pid, &simple_voting::VoteChoice::For);
    assert!(result.is_err(), "vote after deadline must fail");
}

// ===========================================================================
// 19. Simple-Voting: Proposal Passes
// ===========================================================================

#[test]
fn test_sv_proposal_passes() {
    let env = new_env();
    env.ledger().set_timestamp(1_000);
    let (sv, admin) = setup_sv(&env);
    let client = simple_voting::VotingContractClient::new(&env, &sv);

    let pid = client.create_prop(&admin, &String::from_str(&env, "Pass"), &5_000u64);

    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    client.cast_vote(&v1, &pid, &simple_voting::VoteChoice::For);
    client.cast_vote(&v2, &pid, &simple_voting::VoteChoice::For);

    env.ledger().set_timestamp(6_000);
    let status = client.execute(&pid);
    assert_eq!(status, simple_voting::ProposalStatus::Passed);
}

// ===========================================================================
// 20. Simple-Voting: Proposal Rejected
// ===========================================================================

#[test]
fn test_sv_proposal_rejected() {
    let env = new_env();
    env.ledger().set_timestamp(1_000);
    let (sv, admin) = setup_sv(&env);
    let client = simple_voting::VotingContractClient::new(&env, &sv);

    let pid = client.create_prop(&admin, &String::from_str(&env, "Reject"), &5_000u64);

    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    client.cast_vote(&v1, &pid, &simple_voting::VoteChoice::Against);
    client.cast_vote(&v2, &pid, &simple_voting::VoteChoice::Against);

    env.ledger().set_timestamp(6_000);
    let status = client.execute(&pid);
    assert_eq!(status, simple_voting::ProposalStatus::Rejected);
}

// ===========================================================================
// 21. Delegation Creation
// ===========================================================================

#[test]
fn test_delegation_creation() {
    let env = new_env();
    let reg = setup_delegation(&env);
    let client = MockDelegationClient::new(&env, &reg);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    client.delegate(&delegator, &delegate, &10i128);

    assert_eq!(client.get_delegate(&delegator), Some(delegate));
    assert_eq!(client.get_weight(&delegator), 10i128);
}

// ===========================================================================
// 22. Delegation Removal
// ===========================================================================

#[test]
fn test_delegation_removal() {
    let env = new_env();
    let reg = setup_delegation(&env);
    let client = MockDelegationClient::new(&env, &reg);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    client.delegate(&delegator, &delegate, &10i128);
    client.undelegate(&delegator);

    assert_eq!(client.get_delegate(&delegator), None);
    assert_eq!(client.get_weight(&delegator), 0i128);
}

// ===========================================================================
// 23. Delegated Voting
// ===========================================================================

#[test]
fn test_delegated_voting() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let reg = setup_delegation(&env);
    let reg_client = MockDelegationClient::new(&env, &reg);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    reg_client.delegate(&delegator, &delegate, &8i128);

    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);
    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Delegated vote"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    client.submit_proposal(&proposer, &pid, &10u32, &10u32);

    // Delegate votes with the delegated weight.
    let weight = reg_client.get_weight(&delegator);
    client.vote(&delegate, &pid, &true, &weight);

    let proposal = client.get_proposal(&pid);
    assert_eq!(proposal.votes_yes, 8i128);
}

// ===========================================================================
// 24. Delegation Edge Case — zero-weight delegation
// ===========================================================================

#[test]
fn test_delegation_zero_weight() {
    let env = new_env();
    let reg = setup_delegation(&env);
    let client = MockDelegationClient::new(&env, &reg);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    client.delegate(&delegator, &delegate, &0i128);

    assert_eq!(client.get_weight(&delegator), 0i128);
    assert_eq!(client.get_delegate(&delegator), Some(delegate));
}

// ===========================================================================
// 25. Cross-Contract Governance Interaction
// ===========================================================================

#[test]
fn test_cross_contract_governance_interaction() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 0); // quorum=0
    let treasury_id = setup_treasury(&env);
    let tc = MockTreasuryClient::new(&env, &treasury_id);
    tc.deposit(&1_000i128);

    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);
    let pid = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Cross-contract"),
        &treasury_id,
        &Symbol::new(&env, "withdraw"),
        &Vec::from_array(&env, [300i128.into_val(&env)]),
    );
    client.submit_proposal(&proposer, &pid, &5u32, &10u32);

    let voter = Address::generate(&env);
    client.vote(&voter, &pid, &true, &1i128);

    env.ledger().set_sequence_number(env.ledger().sequence() + 6);
    client.execute_proposal(&proposer, &pid);

    assert_eq!(tc.balance(), 700i128);
}

// ===========================================================================
// 26. VTC: Proposal Creation
// ===========================================================================

#[test]
fn test_vtc_proposal_creation() {
    let env = new_env();
    env.ledger().set_timestamp(1_000);
    let (vtc, admin) = setup_vtc(&env, 3_600, 1_800);
    let client = voting_time_constraints::VotingContractClient::new(&env, &vtc);
    let pid = Symbol::new(&env, "prop1");

    client.create_proposal(&admin, &pid, &2u32);

    let proposal = client.get_proposal(&pid);
    assert_eq!(proposal.state, voting_time_constraints::ProposalState::Active);
    assert_eq!(proposal.quorum_threshold, 2);
}

// ===========================================================================
// 27. VTC: Vote Rejected After Deadline
// ===========================================================================

#[test]
fn test_vtc_vote_after_deadline_rejected() {
    let env = new_env();
    env.ledger().set_timestamp(1_000);
    let (vtc, admin) = setup_vtc(&env, 3_600, 1_800);
    let client = voting_time_constraints::VotingContractClient::new(&env, &vtc);
    let pid = Symbol::new(&env, "prop2");

    client.create_proposal(&admin, &pid, &1u32);

    // Jump past voting deadline (1_000 + 3_600 = 4_600).
    env.ledger().set_timestamp(5_000);

    let voter = Address::generate(&env);
    let result = client.try_vote(&voter, &pid, &true);
    assert!(result.is_err(), "vote after deadline must fail");
}

// ===========================================================================
// 28. VTC: Full Lifecycle — create → vote → finalize → execute
// ===========================================================================

#[test]
fn test_vtc_full_lifecycle() {
    let env = new_env();
    env.ledger().set_timestamp(1_000);
    let (vtc, admin) = setup_vtc(&env, 3_600, 1_800);
    let client = voting_time_constraints::VotingContractClient::new(&env, &vtc);
    let pid = Symbol::new(&env, "e2e");

    client.create_proposal(&admin, &pid, &1u32);

    let voter = Address::generate(&env);
    client.vote(&voter, &pid, &true);

    // Advance past voting deadline.
    env.ledger().set_timestamp(5_000);
    client.finalize(&pid);

    // Advance past grace period (4_600 + 1_800 = 6_400).
    env.ledger().set_timestamp(7_000);
    client.finalize(&pid);

    client.execute(&admin, &pid);

    let proposal = client.get_proposal(&pid);
    assert_eq!(
        proposal.state,
        voting_time_constraints::ProposalState::Executed
    );
}

// ===========================================================================
// 29. Concurrent Governance Operations
// ===========================================================================

#[test]
fn test_concurrent_governance_operations() {
    let env = new_env();
    let (gov, proposer) = setup_gov(&env, 1);
    let target = setup_treasury(&env);
    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    let pid_a = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Concurrent A"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );
    let pid_b = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Concurrent B"),
        &target,
        &Symbol::new(&env, "balance"),
        &Vec::new(&env),
    );

    client.submit_proposal(&proposer, &pid_a, &10u32, &10u32);
    client.submit_proposal(&proposer, &pid_b, &10u32, &10u32);

    let voter_a = Address::generate(&env);
    let voter_b = Address::generate(&env);
    client.vote(&voter_a, &pid_a, &true, &3i128);
    client.vote(&voter_b, &pid_b, &false, &3i128);

    env.ledger().set_sequence_number(env.ledger().sequence() + 11);

    assert_eq!(
        client.get_proposal_state(&pid_a),
        proposal_lifecycle::ProposalState::Passed
    );
    assert_eq!(
        client.get_proposal_state(&pid_b),
        proposal_lifecycle::ProposalState::Failed
    );
}

// ===========================================================================
// 30. End-to-End Governance Workflow
// ===========================================================================

#[test]
fn test_end_to_end_governance_workflow() {
    let env = new_env();
    let (gov, admin) = setup_gov(&env, 2);
    let treasury_id = setup_treasury(&env);
    let tc = MockTreasuryClient::new(&env, &treasury_id);
    tc.deposit(&500i128);

    let client = proposal_lifecycle::ProposalLifecycleContractClient::new(&env, &gov);

    // 1. Create proposal.
    let pid = client.create_proposal(
        &admin,
        &String::from_str(&env, "Community fund"),
        &treasury_id,
        &Symbol::new(&env, "withdraw"),
        &Vec::from_array(&env, [100i128.into_val(&env)]),
    );

    // 2. Submit for voting.
    client.submit_proposal(&admin, &pid, &10u32, &10u32);

    // 3. Community votes.
    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    let v3 = Address::generate(&env);
    client.vote(&v1, &pid, &true, &2i128);
    client.vote(&v2, &pid, &true, &2i128);
    client.vote(&v3, &pid, &false, &1i128);

    // 4. Advance past voting window.
    env.ledger().set_sequence_number(env.ledger().sequence() + 11);

    // 5. State must be Passed.
    assert_eq!(
        client.get_proposal_state(&pid),
        proposal_lifecycle::ProposalState::Passed
    );

    // 6. Execute — triggers treasury withdrawal.
    client.execute_proposal(&admin, &pid);

    // 7. Verify treasury reduced and proposal Executed.
    assert_eq!(tc.balance(), 400i128);
    assert_eq!(
        client.get_proposal_state(&pid),
        proposal_lifecycle::ProposalState::Executed
    );
}

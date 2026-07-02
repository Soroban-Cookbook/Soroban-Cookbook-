#![cfg(test)]

use soroban_sdk::{symbol_short, testutils::Address as _, testutils::Ledger as _, Address, Env};

use crate::{ProposalState, VotingContract, VotingContractClient, VotingError};

fn setup(env: &Env) -> (VotingContractClient<'_>, Address) {
    let contract_id = env.register(VotingContract, ());
    let client = VotingContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    // voting_period = 100s, grace_period = 50s
    client.initialize(&admin, &100u64, &50u64);
    (client, admin)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
fn test_double_initialize_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let result = client.try_initialize(&admin, &100u64, &50u64);
    assert_eq!(result, Err(Ok(VotingError::AlreadyInitialized)));
}

// ---------------------------------------------------------------------------
// Proposal creation
// ---------------------------------------------------------------------------

#[test]
fn test_create_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let creator = Address::generate(&env);
    let pid = symbol_short!("prop1");

    client.create_proposal(&creator, &pid, &3u32);

    let proposal = client.get_proposal(&pid);
    assert_eq!(proposal.state, ProposalState::Active);
    assert_eq!(proposal.quorum_threshold, 3);
    assert_eq!(proposal.votes_for, 0);
    assert_eq!(proposal.votes_against, 0);
}

#[test]
fn test_duplicate_proposal_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let creator = Address::generate(&env);
    let pid = symbol_short!("dup");

    client.create_proposal(&creator, &pid, &1u32);
    let result = client.try_create_proposal(&creator, &pid, &1u32);
    assert_eq!(result, Err(Ok(VotingError::InvalidState)));
}

// ---------------------------------------------------------------------------
// Voting period
// ---------------------------------------------------------------------------

#[test]
fn test_vote_for_and_against() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let creator = Address::generate(&env);
    let voter_a = Address::generate(&env);
    let voter_b = Address::generate(&env);
    let pid = symbol_short!("vote1");

    client.create_proposal(&creator, &pid, &1u32);
    client.vote(&voter_a, &pid, &true);
    client.vote(&voter_b, &pid, &false);

    let p = client.get_proposal(&pid);
    assert_eq!(p.votes_for, 1);
    assert_eq!(p.votes_against, 1);
    assert!(client.has_voted(&pid, &voter_a));
    assert!(client.has_voted(&pid, &voter_b));
}

#[test]
fn test_double_vote_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let creator = Address::generate(&env);
    let voter = Address::generate(&env);
    let pid = symbol_short!("dblvote");

    client.create_proposal(&creator, &pid, &1u32);
    client.vote(&voter, &pid, &true);
    let result = client.try_vote(&voter, &pid, &true);
    assert_eq!(result, Err(Ok(VotingError::AlreadyVoted)));
}

// ---------------------------------------------------------------------------
// Proposal deadline
// ---------------------------------------------------------------------------

#[test]
fn test_vote_after_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let creator = Address::generate(&env);
    let voter = Address::generate(&env);
    let pid = symbol_short!("late");

    client.create_proposal(&creator, &pid, &1u32);

    // Advance time past the 100s voting period.
    env.ledger().with_mut(|l| l.timestamp += 200);

    let result = client.try_vote(&voter, &pid, &true);
    assert_eq!(result, Err(Ok(VotingError::VotingClosed)));
}

#[test]
fn test_finalize_active_to_grace_after_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let creator = Address::generate(&env);
    let pid = symbol_short!("fin1");

    client.create_proposal(&creator, &pid, &1u32);

    // Past voting deadline, still within grace period.
    env.ledger().with_mut(|l| l.timestamp += 110);

    let state = client.finalize(&pid);
    assert_eq!(state, ProposalState::GracePeriod);
}

// ---------------------------------------------------------------------------
// Grace period
// ---------------------------------------------------------------------------

#[test]
fn test_finalize_grace_to_executable() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let creator = Address::generate(&env);
    let pid = symbol_short!("fin2");

    client.create_proposal(&creator, &pid, &1u32);

    // voting_period = 100s, grace_period = 50s → executable after 150s.
    env.ledger().with_mut(|l| l.timestamp += 160);

    // First call: Active → GracePeriod (deadline passed)
    let s1 = client.finalize(&pid);
    assert_eq!(s1, ProposalState::GracePeriod);
    // Second call: GracePeriod → Executable (grace period elapsed)
    let s2 = client.finalize(&pid);
    assert_eq!(s2, ProposalState::Executable);
}

#[test]
fn test_execute_before_grace_ends_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let creator = Address::generate(&env);
    let pid = symbol_short!("early");

    client.create_proposal(&creator, &pid, &1u32);

    // Past deadline but still in grace period.
    env.ledger().with_mut(|l| l.timestamp += 110);
    client.finalize(&pid);

    let result = client.try_execute(&admin, &pid);
    assert_eq!(result, Err(Ok(VotingError::InvalidState)));
}

// ---------------------------------------------------------------------------
// Early closure
// ---------------------------------------------------------------------------

#[test]
fn test_early_closure_with_quorum() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let creator = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let pid = symbol_short!("early2");

    client.create_proposal(&creator, &pid, &2u32);
    client.vote(&voter1, &pid, &true);
    client.vote(&voter2, &pid, &true);

    // Close early — quorum of 2 met.
    client.close_early(&admin, &pid);

    let p = client.get_proposal(&pid);
    assert_eq!(p.state, ProposalState::GracePeriod);
}

#[test]
fn test_early_closure_without_quorum_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let creator = Address::generate(&env);
    let voter = Address::generate(&env);
    let pid = symbol_short!("noq");

    client.create_proposal(&creator, &pid, &3u32);
    client.vote(&voter, &pid, &true);

    let result = client.try_close_early(&admin, &pid);
    assert_eq!(result, Err(Ok(VotingError::QuorumNotMet)));
}

#[test]
fn test_early_closure_non_admin_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let creator = Address::generate(&env);
    let intruder = Address::generate(&env);
    let pid = symbol_short!("noauth");

    client.create_proposal(&creator, &pid, &1u32);
    client.vote(&creator, &pid, &true);

    let result = client.try_close_early(&intruder, &pid);
    assert_eq!(result, Err(Ok(VotingError::Unauthorized)));
}

// ---------------------------------------------------------------------------
// Full lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_full_proposal_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let creator = Address::generate(&env);
    let voter = Address::generate(&env);
    let pid = symbol_short!("full");

    // Create → vote → wait → finalize → execute
    client.create_proposal(&creator, &pid, &1u32);
    client.vote(&voter, &pid, &true);

    // Past both voting period and grace period.
    env.ledger().with_mut(|l| l.timestamp += 160);

    // First finalize: Active → GracePeriod
    let s1 = client.finalize(&pid);
    assert_eq!(s1, ProposalState::GracePeriod);
    // Second finalize: GracePeriod → Executable
    let state = client.finalize(&pid);
    assert_eq!(state, ProposalState::Executable);

    client.execute(&admin, &pid);

    let p = client.get_proposal(&pid);
    assert_eq!(p.state, ProposalState::Executed);
}

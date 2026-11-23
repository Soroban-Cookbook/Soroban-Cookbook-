//! Storage exhaustion / limits security tests (#1486).
//!
//! Contracts that maintain unbounded collections (e.g., per-proposal vote
//! records, per-user delegation lists) must handle or gracefully reject inputs
//! that would push those collections to Soroban's ledger-entry size limits.
//!
//! These tests probe:
//!   1. The governance vote store: creating many proposals and verifying that
//!      proposal-count storage remains consistent.
//!   2. The delegation list: adding many delegators to a single delegatee and
//!      confirming the voting-power calculation remains correct.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, String,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn title(env: &Env, n: u32) -> String {
    // Build a short title string — using a fixed label avoids dynamic
    // formatting which is not available in no_std.
    String::from_str(env, "P")
}

fn setup_voting(env: &Env) -> (simple_voting::VotingContractClient<'_>, Address) {
    let id = env.register_contract(None, simple_voting::VotingContract);
    let client = simple_voting::VotingContractClient::new(env, &id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&admin);
    (client, admin)
}

fn setup_delegation(env: &Env) -> (vote_delegation::VoteDelegationContractClient<'_>, Address) {
    let id = env.register_contract(None, vote_delegation::VoteDelegationContract);
    let client = vote_delegation::VoteDelegationContractClient::new(env, &id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.init(&admin);
    (client, admin)
}

// ---------------------------------------------------------------------------
// Storage limit test 1: Many proposals — prop_count must stay accurate
// ---------------------------------------------------------------------------

/// Creating a large number of proposals must keep `prop_count` accurate and
/// never produce a stale or overflowed counter.
#[test]
fn many_proposals_prop_count_stays_accurate() {
    let env = Env::default();
    env.ledger().set_timestamp(1_000);
    let (client, admin) = setup_voting(&env);

    const PROPOSAL_COUNT: u32 = 50;
    for i in 0..PROPOSAL_COUNT {
        // Each proposal gets a unique far-future deadline to avoid BadDeadline.
        client.create_prop(&admin, &title(&env, i), &(2_000 + i as u64 * 10));
    }

    assert_eq!(
        client.prop_count(),
        PROPOSAL_COUNT,
        "prop_count must equal the number of created proposals"
    );
}

/// Each voter's vote record must be stored independently. Casting many votes
/// on the same proposal must not corrupt the tally or cause key collisions.
#[test]
fn many_votes_on_one_proposal_tally_is_correct() {
    let env = Env::default();
    env.ledger().set_timestamp(1_000);
    let (client, admin) = setup_voting(&env);

    let id = client.create_prop(&admin, &title(&env, 0), &100_000);

    const VOTER_COUNT: u32 = 50;
    for _ in 0..VOTER_COUNT {
        let voter = Address::generate(&env);
        client.cast_vote(&voter, &id, &simple_voting::VoteChoice::For);
    }

    let (votes_for, votes_against, votes_abstain) = client.tally(&id);
    assert_eq!(votes_for, VOTER_COUNT);
    assert_eq!(votes_against, 0);
    assert_eq!(votes_abstain, 0);
}

// ---------------------------------------------------------------------------
// Storage limit test 2: Many delegators — voting power stays correct
// ---------------------------------------------------------------------------

/// Adding many delegators to a single delegatee must keep the voting-power
/// sum accurate and not cause the delegators list to diverge.
#[test]
fn many_delegators_voting_power_accumulates_correctly() {
    let env = Env::default();
    let (client, admin) = setup_delegation(&env);

    // The delegate who will accumulate power.
    let delegatee = Address::generate(&env);
    client.set_balance(&admin, &delegatee, &100);

    const DELEGATOR_COUNT: i128 = 20;
    for _ in 0..DELEGATOR_COUNT {
        let delegator = Address::generate(&env);
        client.set_balance(&admin, &delegator, &10);
        client.delegate(&delegator, &delegatee);
    }

    // Expected: delegatee's own balance (100) + 20 * 10 = 300.
    let power = client.get_voting_power(&delegatee);
    assert_eq!(
        power,
        100 + DELEGATOR_COUNT * 10,
        "voting power must equal delegatee balance plus all delegator balances"
    );
}

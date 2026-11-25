#![allow(deprecated)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env, String,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(env: &Env) -> (VotingContractClient<'_>, Address) {
    let contract_id = env.register_contract(None, VotingContract);
    let client = VotingContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&admin);
    (client, admin)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_sets_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_init_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
}

// ---------------------------------------------------------------------------
// Proposal creation
// ---------------------------------------------------------------------------

#[test]
fn test_create_proposal() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);

    let title = String::from_str(&env, "Fund development");
    let id = client.create_prop(&admin, &title, &500);
    assert_eq!(id, 1);

    let prop = client.get_prop(&1);
    assert_eq!(prop.id, 1);
    assert_eq!(prop.deadline, 500);
    assert_eq!(prop.votes_for, 0);
    assert_eq!(prop.votes_against, 0);
    assert_eq!(prop.votes_abstain, 0);
    assert_eq!(prop.status, ProposalStatus::Active);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_non_admin_cannot_create_proposal() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, _admin) = setup(&env);
    let attacker = Address::generate(&env);
    let title = String::from_str(&env, "Bad proposal");
    client.create_prop(&attacker, &title, &500);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_invalid_deadline() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Past deadline");
    client.create_prop(&admin, &title, &50);
}

// ---------------------------------------------------------------------------
// Voting
// ---------------------------------------------------------------------------

#[test]
fn test_cast_vote_for() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Test vote");
    client.create_prop(&admin, &title, &500);

    let voter = Address::generate(&env);
    client.cast_vote(&voter, &1, &VoteChoice::For);

    assert!(client.has_voted(&voter, &1));
    assert_eq!(client.get_vote(&voter, &1), Some(VoteChoice::For));
}

#[test]
fn test_cast_vote_against() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Test against");
    client.create_prop(&admin, &title, &500);

    let voter = Address::generate(&env);
    client.cast_vote(&voter, &1, &VoteChoice::Against);

    assert_eq!(client.get_vote(&voter, &1), Some(VoteChoice::Against));
}

#[test]
fn test_cast_vote_abstain() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Test abstain");
    client.create_prop(&admin, &title, &500);

    let voter = Address::generate(&env);
    client.cast_vote(&voter, &1, &VoteChoice::Abstain);

    assert_eq!(client.get_vote(&voter, &1), Some(VoteChoice::Abstain));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_cannot_vote_twice() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Double vote");
    client.create_prop(&admin, &title, &500);

    let voter = Address::generate(&env);
    client.cast_vote(&voter, &1, &VoteChoice::For);
    client.cast_vote(&voter, &1, &VoteChoice::Against);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_cannot_vote_after_deadline() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Expired");
    client.create_prop(&admin, &title, &200);

    env.ledger().set_timestamp(300);
    let voter = Address::generate(&env);
    client.cast_vote(&voter, &1, &VoteChoice::For);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_vote_nonexistent_proposal() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, _admin) = setup(&env);

    let voter = Address::generate(&env);
    client.cast_vote(&voter, &99, &VoteChoice::For);
}

// ---------------------------------------------------------------------------
// Tallying
// ---------------------------------------------------------------------------

#[test]
fn test_tally_votes() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Tally test");
    client.create_prop(&admin, &title, &500);

    // 3 For, 1 Against, 1 Abstain
    for _ in 0..3 {
        let voter = Address::generate(&env);
        client.cast_vote(&voter, &1, &VoteChoice::For);
    }
    let against = Address::generate(&env);
    client.cast_vote(&against, &1, &VoteChoice::Against);
    let abstain = Address::generate(&env);
    client.cast_vote(&abstain, &1, &VoteChoice::Abstain);

    let (f, a, ab) = client.tally(&1);
    assert_eq!(f, 3);
    assert_eq!(a, 1);
    assert_eq!(ab, 1);
}

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

#[test]
fn test_execute_passed() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Pass test");
    client.create_prop(&admin, &title, &200);

    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    client.cast_vote(&v1, &1, &VoteChoice::For);
    client.cast_vote(&v2, &1, &VoteChoice::For);

    env.ledger().set_timestamp(300);
    let status = client.execute(&1);
    assert_eq!(status, ProposalStatus::Passed);

    let prop = client.get_prop(&1);
    assert_eq!(prop.status, ProposalStatus::Passed);
}

#[test]
fn test_execute_rejected() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Reject test");
    client.create_prop(&admin, &title, &200);

    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    client.cast_vote(&v1, &1, &VoteChoice::Against);
    client.cast_vote(&v2, &1, &VoteChoice::Against);

    env.ledger().set_timestamp(300);
    let status = client.execute(&1);
    assert_eq!(status, ProposalStatus::Rejected);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_cannot_execute_before_deadline() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Early exec");
    client.create_prop(&admin, &title, &500);
    client.execute(&1);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_cannot_execute_twice() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Double exec");
    client.create_prop(&admin, &title, &200);

    env.ledger().set_timestamp(300);
    client.execute(&1);
    client.execute(&1);
}

// ---------------------------------------------------------------------------
// Multiple proposals
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_proposals() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);

    let t1 = String::from_str(&env, "Proposal A");
    let t2 = String::from_str(&env, "Proposal B");
    let id1 = client.create_prop(&admin, &t1, &500);
    let id2 = client.create_prop(&admin, &t2, &600);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(client.prop_count(), 2);
}

#[test]
fn test_vote_on_separate_proposals() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);

    let t1 = String::from_str(&env, "Prop 1");
    let t2 = String::from_str(&env, "Prop 2");
    client.create_prop(&admin, &t1, &500);
    client.create_prop(&admin, &t2, &500);

    let voter = Address::generate(&env);
    client.cast_vote(&voter, &1, &VoteChoice::For);
    client.cast_vote(&voter, &2, &VoteChoice::Against);

    assert_eq!(client.get_vote(&voter, &1), Some(VoteChoice::For));
    assert_eq!(client.get_vote(&voter, &2), Some(VoteChoice::Against));
}

#[test]
fn test_tie_results_in_rejected() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, admin) = setup(&env);
    let title = String::from_str(&env, "Tie vote");
    client.create_prop(&admin, &title, &200);

    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    client.cast_vote(&v1, &1, &VoteChoice::For);
    client.cast_vote(&v2, &1, &VoteChoice::Against);

    env.ledger().set_timestamp(300);
    let status = client.execute(&1);
    assert_eq!(status, ProposalStatus::Rejected);
}

// ---------------------------------------------------------------------------
// Property-based tests (#1484)
// ---------------------------------------------------------------------------
//
// Properties verified:
//   1. The final tally always equals the number of votes cast for each choice.
//   2. Double-vote rejection is enforced regardless of vote order.
//   3. `prop_count` always equals the number of successful `create_prop` calls.

#[cfg(test)]
mod proptest_voting {
    use super::*;
    use proptest::prelude::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Env, String,
    };

    /// Helpers ------------------------------------------------------------------

    fn make_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);
        env
    }

    fn make_client(env: &Env) -> (VotingContractClient<'_>, Address) {
        let id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(env, &id);
        let admin = Address::generate(env);
        client.initialize(&admin);
        (client, admin)
    }

    fn choice_from_u8(env: &Env, v: u8) -> (VoteChoice, usize) {
        match v % 3 {
            0 => (VoteChoice::For, 0),
            1 => (VoteChoice::Against, 1),
            _ => (VoteChoice::Abstain, 2),
        }
    }

    fn title(env: &Env, s: &str) -> soroban_sdk::String {
        soroban_sdk::String::from_str(env, s)
    }

    /// Property 1 ---------------------------------------------------------------
    /// The vote tally for every choice is exactly the count of voters who chose it.
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(32))]
        #[test]
        fn tally_equals_sum_of_individual_votes(
            choices in prop::collection::vec(0u8..3, 1..=20usize)
        ) {
            let env = make_env();
            let (client, admin) = make_client(&env);
            client.create_prop(&admin, &title(&env, "P"), &2_000);

            let mut counts = [0u32; 3];
            for raw in &choices {
                let voter = Address::generate(&env);
                let (choice, idx) = choice_from_u8(&env, *raw);
                client.cast_vote(&voter, &1, &choice);
                counts[idx] += 1;
            }

            let (f, a, ab) = client.tally(&1);
            prop_assert_eq!(f,  counts[0]);
            prop_assert_eq!(a,  counts[1]);
            prop_assert_eq!(ab, counts[2]);
        }
    }

    /// Property 2 ---------------------------------------------------------------
    /// A second vote from the same address always fails, regardless of which
    /// choice is submitted first or second.
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(32))]
        #[test]
        fn double_vote_always_rejected(
            first  in 0u8..3,
            second in 0u8..3,
        ) {
            let env = make_env();
            let (client, admin) = make_client(&env);
            client.create_prop(&admin, &title(&env, "P"), &2_000);
            let voter = Address::generate(&env);

            let (c1, _) = choice_from_u8(&env, first);
            let (c2, _) = choice_from_u8(&env, second);

            client.cast_vote(&voter, &1, &c1);
            let res = client.try_cast_vote(&voter, &1, &c2);
            prop_assert!(res.is_err(), "expected AlreadyVoted but got Ok");
        }
    }

    /// Property 3 ---------------------------------------------------------------
    /// `prop_count` always equals the number of proposals successfully created
    /// regardless of how many are created.
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(16))]
        #[test]
        fn prop_count_tracks_created_proposals(n in 1u32..=15) {
            let env = make_env();
            let (client, admin) = make_client(&env);
            for i in 0..n {
                let t = soroban_sdk::String::from_str(&env, "P");
                // Each proposal gets a unique far-future deadline.
                client.create_prop(&admin, &t, &(2_000 + i as u64 * 10));
            }
            prop_assert_eq!(client.prop_count(), n);
        }
    }
}

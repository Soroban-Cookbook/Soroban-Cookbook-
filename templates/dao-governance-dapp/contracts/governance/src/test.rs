#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env, String};

#[test]
fn test_propose_vote_and_execute() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(DAOGovernanceContract, ());
    let client = DAOGovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let target_grantee = Address::generate(&env);

    // Initialize: Quorum = 100 votes, voting period = 10 ledgers
    client.initialize(&admin, &100, &10);

    let desc = String::from_str(&env, "Community Grant for Developer Tooling");
    let prop_id = client.propose(&proposer, &desc, &target_grantee, &5000);
    assert_eq!(prop_id, 1);

    // Cast votes
    client.vote(&voter1, &1, &true, &80);
    client.vote(&voter2, &1, &true, &50);

    let proposal = client.get_proposal(&1);
    assert_eq!(proposal.votes_for, 130);
    assert_eq!(proposal.votes_against, 0);

    // Advance ledger beyond voting period (sequence > end_ledger)
    env.ledger().set_sequence_number(20);

    // Execute proposal
    client.execute(&admin, &1);

    let executed_prop = client.get_proposal(&1);
    assert_eq!(executed_prop.status, ProposalStatus::Executed);
}

#[test]
fn test_defeated_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(DAOGovernanceContract, ());
    let client = DAOGovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let target = Address::generate(&env);

    client.initialize(&admin, &100, &10);

    let desc = String::from_str(&env, "Defeated Proposal");
    let prop_id = client.propose(&proposer, &desc, &target, &1000);

    // Vote against
    client.vote(&voter, &prop_id, &false, &150);

    env.ledger().set_sequence_number(20);

    // Execution should fail with ProposalNotPassed
    assert!(client.try_execute(&admin, &prop_id).is_err());
}

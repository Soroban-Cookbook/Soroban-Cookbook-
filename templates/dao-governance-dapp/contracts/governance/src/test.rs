#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env, String,
};

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

    // One vote per address. Quorum is two votes. A positive amount is not executable.
    client.initialize(&admin, &2, &10);

    let desc = String::from_str(&env, "Community Grant for Developer Tooling");
    let prop_id = client.propose(&proposer, &desc, &target_grantee, &0);
    assert_eq!(prop_id, 1);

    // Cast votes
    client.vote(&voter1, &1, &true, &1);
    client.vote(&voter2, &1, &true, &1);

    let proposal = client.get_proposal(&1);
    assert_eq!(proposal.votes_for, 2);
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

    client.initialize(&admin, &1, &10);

    let desc = String::from_str(&env, "Defeated Proposal");
    let prop_id = client.propose(&proposer, &desc, &target, &0);

    // Vote against
    client.vote(&voter, &prop_id, &false, &1);

    env.ledger().set_sequence_number(20);

    // Execution should fail with ProposalNotPassed
    assert!(client.try_execute(&admin, &prop_id).is_err());
}

#[test]
fn test_execute_rejects_unfunded_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(DAOGovernanceContract, ());
    let client = DAOGovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let target = Address::generate(&env);

    client.initialize(&admin, &1, &10);
    let desc = String::from_str(&env, "Unfunded transfer");
    let prop_id = client.propose(&proposer, &desc, &target, &5000);
    client.vote(&voter, &prop_id, &true, &1);

    env.ledger().set_sequence_number(20);

    assert_eq!(
        client.try_execute(&admin, &prop_id),
        Err(Ok(GovernanceError::InvalidAmount))
    );
    assert_eq!(
        client.get_proposal(&prop_id).status,
        ProposalStatus::Active
    );
}

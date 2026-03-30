use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone(), signer3.clone()];

    client.initialize(&2, &signers);
}

#[test]
#[should_panic(expected = "Invalid threshold")]
fn test_initialize_invalid_threshold() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signers = vec![&env, signer1];

    client.initialize(&0, &signers);
}

#[test]
fn test_create_and_approve_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone(), signer3.clone()];

    client.initialize(&2, &signers);

    let proposal_id = client.create_proposal(&signer1);
    assert_eq!(proposal_id, 0);

    client.approve(&proposal_id, &signer1);
    client.approve(&proposal_id, &signer2);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.approvals.len(), 2);
    assert!(!proposal.executed);
}

#[test]
#[should_panic(expected = "Already approved")]
fn test_double_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone()];

    client.initialize(&2, &signers);
    let proposal_id = client.create_proposal(&signer1);

    client.approve(&proposal_id, &signer1);
    client.approve(&proposal_id, &signer1);
}

#[test]
fn test_execute_with_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone(), signer3.clone()];

    client.initialize(&2, &signers);
    let proposal_id = client.create_proposal(&signer1);

    client.approve(&proposal_id, &signer1);
    client.approve(&proposal_id, &signer2);

    let result = client.execute(&proposal_id, &signer1);
    assert!(result);

    let proposal = client.get_proposal(&proposal_id);
    assert!(proposal.executed);
}

#[test]
#[should_panic(expected = "Threshold not met")]
fn test_execute_without_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone(), signer3.clone()];

    client.initialize(&3, &signers);
    let proposal_id = client.create_proposal(&signer1);

    client.approve(&proposal_id, &signer1);
    client.approve(&proposal_id, &signer2);

    client.execute(&proposal_id, &signer1);
}

#[test]
#[should_panic(expected = "Already executed")]
fn test_double_execute() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone()];

    client.initialize(&2, &signers);
    let proposal_id = client.create_proposal(&signer1);

    client.approve(&proposal_id, &signer1);
    client.approve(&proposal_id, &signer2);

    client.execute(&proposal_id, &signer1);
    client.execute(&proposal_id, &signer1);
}

#[test]
fn test_multi_auth_action() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone()];

    let result = client.multi_auth_action(&signers);
    assert!(result);
}

#[test]
fn test_require_all_signers() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone(), signer3.clone()];

    client.initialize(&2, &signers);

    let result = client.require_all_signers();
    assert!(result);
}

#[test]
#[should_panic(expected = "Not an authorized signer")]
fn test_unauthorized_signer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone()];

    client.initialize(&2, &signers);
    client.create_proposal(&unauthorized);
}

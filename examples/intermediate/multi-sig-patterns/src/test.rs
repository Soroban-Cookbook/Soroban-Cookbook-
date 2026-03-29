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
    
    // Test re-initialization fails
    let result = client.try_initialize(&2, &signers);
    assert_eq!(result, Err(Ok(AuthError::AlreadyInitialized)));
}

#[test]
fn test_initialize_invalid_threshold() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signers = vec![&env, signer1];

    let result = client.try_initialize(&0, &signers);
    assert_eq!(result, Err(Ok(AuthError::InvalidThreshold)));
    
    let result = client.try_initialize(&2, &signers);
    assert_eq!(result, Err(Ok(AuthError::InvalidThreshold)));
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
fn test_unauthorized_signer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signers = vec![&env, signer1.clone()];
    client.initialize(&1, &signers);

    let attacker = Address::generate(&env);
    
    let result = client.try_create_proposal(&attacker);
    assert_eq!(result, Err(Ok(AuthError::NotAuthorized)));
    
    let proposal_id = client.create_proposal(&signer1);
    let result = client.try_approve(&proposal_id, &attacker);
    assert_eq!(result, Err(Ok(AuthError::NotAuthorized)));
}

#[test]
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
    let result = client.try_approve(&proposal_id, &signer1);
    assert_eq!(result, Err(Ok(AuthError::AlreadyApproved)));
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

    // Test execution before threshold
    let result = client.try_execute(&proposal_id, &signer1);
    assert_eq!(result, Err(Ok(AuthError::ThresholdNotMet)));

    client.approve(&proposal_id, &signer1);
    client.approve(&proposal_id, &signer2);

    let result = client.execute(&proposal_id, &signer1);
    assert!(result);

    let proposal = client.get_proposal(&proposal_id);
    assert!(proposal.executed);
    
    // Test execution after execution
    let result = client.try_execute(&proposal_id, &signer1);
    assert_eq!(result, Err(Ok(AuthError::AlreadyExecuted)));
}

#[test]
fn test_proposal_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signers = vec![&env, signer1.clone()];
    client.initialize(&1, &signers);

    let result = client.try_approve(&999, &signer1);
    assert_eq!(result, Err(Ok(AuthError::ProposalNotFound)));
}

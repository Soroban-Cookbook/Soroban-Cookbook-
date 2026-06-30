#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, token,
};

fn setup_test(
    env: &Env,
) -> (
    VestingContractClient<'_>,
    Address,
    Address,
    token::StellarAssetClient<'_>,
) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin).address();
    let token_client = token::StellarAssetClient::new(env, &token_address);

    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(env, &contract_id);

    client.initialize(&admin, &token_address);

    (client, admin, token_address, token_client)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, admin, token_address, _) = setup_test(&env);

    // Test that it's initialized correctly by trying to initialize again
    let result = client.try_initialize(&admin, &token_address);
    assert_eq!(result.err(), Some(Ok(VestingError::AlreadyInitialized)));
}

#[test]
fn test_create_schedule() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);

    let schedule = client.get_schedule(&beneficiary).unwrap();
    assert_eq!(schedule.beneficiary, beneficiary);
    assert_eq!(schedule.total_allocation, 1000);
    assert_eq!(schedule.vesting_duration, 100);
}

#[test]
fn test_create_schedule_unauthorized() {
    let env = Env::default();
    let (client, _admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);
    let wrong_admin = Address::generate(&env);

    env.mock_all_auths();
    let result = client.try_create_schedule(&wrong_admin, &beneficiary, &1000, &0, &0, &100);
    assert_eq!(result.err(), Some(Ok(VestingError::Unauthorized)));
}

#[test]
fn test_create_duplicate_schedule() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);
    let result = client.try_create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);
    assert_eq!(result.err(), Some(Ok(VestingError::ScheduleAlreadyExists)));
}

#[test]
fn test_invalid_schedule_params() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    // Zero allocation
    let result = client.try_create_schedule(&admin, &beneficiary, &0, &0, &0, &100);
    assert_eq!(result.err(), Some(Ok(VestingError::InvalidSchedule)));

    // Zero duration
    let result = client.try_create_schedule(&admin, &beneficiary, &1000, &0, &0, &0);
    assert_eq!(result.err(), Some(Ok(VestingError::InvalidSchedule)));
}

#[test]
fn test_cliff_enforcement() {
    let env = Env::default();
    let (client, admin, _token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    let start_time = 0;
    let cliff_duration = 50;
    let vesting_duration = 100;
    let total_allocation = 1000;

    env.mock_all_auths();
    client.create_schedule(
        &admin,
        &beneficiary,
        &total_allocation,
        &start_time,
        &cliff_duration,
        &vesting_duration,
    );

    // Fund the contract
    token_client.mint(&client.address, &total_allocation);

    env.ledger().with_mut(|li| li.timestamp = 49);

    let vested = client.get_vested_amount(&beneficiary);
    assert_eq!(vested, 0);

    let result = client.try_claim(&beneficiary);
    assert_eq!(result.err(), Some(Ok(VestingError::ClaimBeforeCliff)));
}

#[test]
fn test_linear_vesting_calculation() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);

    env.ledger().with_mut(|li| li.timestamp = 25);
    assert_eq!(client.get_vested_amount(&beneficiary), 250);

    env.ledger().with_mut(|li| li.timestamp = 50);
    assert_eq!(client.get_vested_amount(&beneficiary), 500);

    env.ledger().with_mut(|li| li.timestamp = 75);
    assert_eq!(client.get_vested_amount(&beneficiary), 750);
}

#[test]
fn test_partial_claim() {
    let env = Env::default();
    let (client, admin, token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);
    token_client.mint(&client.address, &1000);

    env.ledger().with_mut(|li| li.timestamp = 50);
    client.claim(&beneficiary);

    let token_bal = token::Client::new(&env, &token_address).balance(&beneficiary);
    assert_eq!(token_bal, 500);

    let schedule = client.get_schedule(&beneficiary).unwrap();
    assert_eq!(schedule.claimed_amount, 500);
}

#[test]
fn test_multiple_claims() {
    let env = Env::default();
    let (client, admin, token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);
    token_client.mint(&client.address, &1000);

    env.ledger().with_mut(|li| li.timestamp = 25);
    client.claim(&beneficiary);

    env.ledger().with_mut(|li| li.timestamp = 50);
    client.claim(&beneficiary);

    let token_bal = token::Client::new(&env, &token_address).balance(&beneficiary);
    assert_eq!(token_bal, 500);
}

#[test]
fn test_full_vesting() {
    let env = Env::default();
    let (client, admin, token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);
    token_client.mint(&client.address, &1000);

    env.ledger().with_mut(|li| li.timestamp = 100);
    assert_eq!(client.get_vested_amount(&beneficiary), 1000);

    client.claim(&beneficiary);
    let token_bal = token::Client::new(&env, &token_address).balance(&beneficiary);
    assert_eq!(token_bal, 1000);
}

#[test]
fn test_claim_after_completion() {
    let env = Env::default();
    let (client, admin, _token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);
    token_client.mint(&client.address, &1000);

    env.ledger().with_mut(|li| li.timestamp = 150);
    assert_eq!(client.get_vested_amount(&beneficiary), 1000);

    client.claim(&beneficiary);

    let result = client.try_claim(&beneficiary);
    assert_eq!(result.err(), Some(Ok(VestingError::NothingToClaim)));
}

#[test]
fn test_claim_auth_failure() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);
    let attacker = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);

    env.ledger().with_mut(|li| li.timestamp = 50);

    env.mock_all_auths();
    // Use set_auths to mock a call from attacker
    env.set_auths(&[MockAuth {
        address: &attacker,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "claim",
            args: (&beneficiary,).into_val(&env),
            sub_invokes: &[],
        },
    }
    .into()]);

    let result = client.try_claim(&beneficiary);
    // beneficiary.require_auth() should fail because only attacker authorized
    assert!(result.is_err());
}

#[test]
fn test_claim_with_mock_auth() {
    let env = Env::default();
    let (client, admin, _token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100);
    token_client.mint(&client.address, &1000);

    env.ledger().with_mut(|li| li.timestamp = 50);

    client.claim(&beneficiary);
}

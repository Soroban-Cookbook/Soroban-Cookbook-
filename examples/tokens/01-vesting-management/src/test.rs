#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token, Address, Env, IntoVal,
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
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let token_client = token::StellarAssetClient::new(env, &token_address);

    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(env, &contract_id);

    client.initialize(&admin, &Some(token_address.clone()));

    (client, admin, token_address, token_client)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, admin, token_address, _) = setup_test(&env);

    // Test that trying to initialize again fails
    let result = client.try_initialize(&admin, &Some(token_address));
    assert_eq!(result.err(), Some(Ok(VestingError::AlreadyInitialized)));
}

#[test]
fn test_create_schedule_admin_only() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100, &None);

    let schedule = client.get_schedule(&beneficiary).unwrap();
    assert_eq!(schedule.beneficiary, beneficiary);
    assert_eq!(schedule.total_amount, 1000);
    assert_eq!(schedule.vesting_duration, 100);

    // Non-admin attempt should fail with Unauthorized
    let wrong_admin = Address::generate(&env);
    let result = client.try_create_schedule(
        &wrong_admin,
        &Address::generate(&env),
        &1000,
        &0,
        &0,
        &100,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(VestingError::Unauthorized)));
}

#[test]
fn test_duplicate_schedule_error() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100, &None);

    // Attempting to create duplicate schedule for the same beneficiary should fail
    let result = client.try_create_schedule(&admin, &beneficiary, &1000, &0, &0, &100, &None);
    assert_eq!(result.err(), Some(Ok(VestingError::ScheduleAlreadyExists)));
}

#[test]
fn test_invalid_schedule_parameters() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    // Zero/Negative allocation
    let result = client.try_create_schedule(&admin, &beneficiary, &0, &0, &0, &100, &None);
    assert_eq!(result.err(), Some(Ok(VestingError::InvalidSchedule)));

    // Zero duration
    let result = client.try_create_schedule(&admin, &beneficiary, &1000, &0, &0, &0, &None);
    assert_eq!(result.err(), Some(Ok(VestingError::InvalidSchedule)));

    // Cliff duration > vesting duration
    let result = client.try_create_schedule(&admin, &beneficiary, &1000, &0, &150, &100, &None);
    assert_eq!(result.err(), Some(Ok(VestingError::InvalidSchedule)));
}

#[test]
fn test_cliff_enforcement() {
    let env = Env::default();
    let (client, admin, _, token_client) = setup_test(&env);
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
        &None,
    );

    // Fund contract
    token_client.mint(&client.address, &total_allocation);

    env.ledger().with_mut(|li| li.timestamp = 49);

    assert_eq!(client.vested_amount(&beneficiary), 0);
    assert_eq!(client.releasable_amount(&beneficiary), 0);

    let result = client.try_claim(&beneficiary);
    assert_eq!(result.err(), Some(Ok(VestingError::CliffNotReached)));
}

#[test]
fn test_linear_vesting_calculation() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100, &None);

    env.ledger().with_mut(|li| li.timestamp = 25);
    assert_eq!(client.vested_amount(&beneficiary), 250);
    assert_eq!(client.releasable_amount(&beneficiary), 250);

    env.ledger().with_mut(|li| li.timestamp = 50);
    assert_eq!(client.vested_amount(&beneficiary), 500);
    assert_eq!(client.releasable_amount(&beneficiary), 500);

    env.ledger().with_mut(|li| li.timestamp = 75);
    assert_eq!(client.vested_amount(&beneficiary), 750);
    assert_eq!(client.releasable_amount(&beneficiary), 750);
}

#[test]
fn test_multiple_claims() {
    let env = Env::default();
    let (client, admin, token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100, &None);
    token_client.mint(&client.address, &1000);

    env.ledger().with_mut(|li| li.timestamp = 25);
    client.claim(&beneficiary);

    let token_bal = token::Client::new(&env, &token_address).balance(&beneficiary);
    assert_eq!(token_bal, 250);
    assert_eq!(client.releasable_amount(&beneficiary), 0);

    env.ledger().with_mut(|li| li.timestamp = 50);
    assert_eq!(client.releasable_amount(&beneficiary), 250);
    client.claim(&beneficiary);

    let token_bal_2 = token::Client::new(&env, &token_address).balance(&beneficiary);
    assert_eq!(token_bal_2, 500);
}

#[test]
fn test_full_vesting() {
    let env = Env::default();
    let (client, admin, token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100, &None);
    token_client.mint(&client.address, &1000);

    env.ledger().with_mut(|li| li.timestamp = 150);
    assert_eq!(client.vested_amount(&beneficiary), 1000);

    client.claim(&beneficiary);
    let token_bal = token::Client::new(&env, &token_address).balance(&beneficiary);
    assert_eq!(token_bal, 1000);

    // Try claiming again after fully released should return NothingToClaim
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
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100, &None);

    env.ledger().with_mut(|li| li.timestamp = 50);

    // Set authorization mock specifically for the attacker instead of beneficiary
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
    assert!(result.is_err());
}

#[test]
fn test_multiple_beneficiaries_isolation() {
    let env = Env::default();
    let (client, admin, _, token_client) = setup_test(&env);

    let b1 = Address::generate(&env);
    let b2 = Address::generate(&env);
    let b3 = Address::generate(&env);

    env.mock_all_auths();
    // Schedule 1: 1000 tokens, 100s duration, no cliff
    client.create_schedule(&admin, &b1, &1000, &0, &0, &100, &None);
    // Schedule 2: 5000 tokens, 200s duration, 50s cliff
    client.create_schedule(&admin, &b2, &5000, &0, &50, &200, &None);
    // Schedule 3: 2000 tokens, 50s duration, no cliff
    client.create_schedule(&admin, &b3, &2000, &10, &0, &50, &None);

    token_client.mint(&client.address, &8000);

    // Advance to t = 30
    env.ledger().with_mut(|li| li.timestamp = 30);

    // b1: 30% of 1000 = 300 vested
    assert_eq!(client.vested_amount(&b1), 300);
    // b2: under cliff (30 < 50) = 0 vested
    assert_eq!(client.vested_amount(&b2), 0);
    // b3: (30 - 10) / 50 = 40% of 2000 = 800 vested
    assert_eq!(client.vested_amount(&b3), 800);

    // Claim for b1 and b3
    client.claim(&b1);
    client.claim(&b3);

    assert_eq!(client.get_schedule(&b1).unwrap().released_amount, 300);
    assert_eq!(client.get_schedule(&b3).unwrap().released_amount, 800);
    assert_eq!(client.get_schedule(&b2).unwrap().released_amount, 0);
}

#[test]
fn test_revoke_schedule_mid_vesting() {
    let env = Env::default();
    let (client, admin, token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &0, &100, &None);
    token_client.mint(&client.address, &1000);

    // Mid-vesting at t = 40. Vested is 400. Unvested is 600.
    env.ledger().with_mut(|li| li.timestamp = 40);

    // Revoke schedule
    client.revoke(&admin, &beneficiary);

    // Unvested 600 should be returned to admin
    let admin_bal = token::Client::new(&env, &token_address).balance(&admin);
    assert_eq!(admin_bal, 600);

    // Schedule should be marked as revoked
    let schedule = client.get_schedule(&beneficiary).unwrap();
    assert!(schedule.revoked);
    assert_eq!(schedule.total_amount, 400); // capped at 400

    // Vested amount should be locked at 400, even if time advances
    env.ledger().with_mut(|li| li.timestamp = 80);
    assert_eq!(client.vested_amount(&beneficiary), 400);
    assert_eq!(client.releasable_amount(&beneficiary), 400);

    // Beneficiary can claim their vested-but-unclaimed tokens (400)
    client.claim(&beneficiary);
    let ben_bal = token::Client::new(&env, &token_address).balance(&beneficiary);
    assert_eq!(ben_bal, 400);
    assert_eq!(client.releasable_amount(&beneficiary), 0);

    // Non-admin cannot revoke
    let non_admin = Address::generate(&env);
    let result = client.try_revoke(&non_admin, &beneficiary);
    assert_eq!(result.err(), Some(Ok(VestingError::Unauthorized)));

    // Already-revoked schedule cannot be revoked again
    let result2 = client.try_revoke(&admin, &beneficiary);
    assert_eq!(result2.err(), Some(Ok(VestingError::AlreadyRevoked)));
}

#[test]
fn test_revoke_before_cliff() {
    let env = Env::default();
    let (client, admin, token_address, token_client) = setup_test(&env);
    let beneficiary = Address::generate(&env);

    env.mock_all_auths();
    client.create_schedule(&admin, &beneficiary, &1000, &0, &50, &100, &None);
    token_client.mint(&client.address, &1000);

    // Revoke at t = 30 (before cliff of 50). Vested is 0. Unvested is 1000.
    env.ledger().with_mut(|li| li.timestamp = 30);
    client.revoke(&admin, &beneficiary);

    // All 1000 tokens should be returned to admin
    let admin_bal = token::Client::new(&env, &token_address).balance(&admin);
    assert_eq!(admin_bal, 1000);

    // Beneficiary has 0 claimable tokens
    env.ledger().with_mut(|li| li.timestamp = 80);
    assert_eq!(client.vested_amount(&beneficiary), 0);
    let result = client.try_claim(&beneficiary);
    assert_eq!(result.err(), Some(Ok(VestingError::NothingToClaim)));
}

#[test]
fn test_transfer_admin() {
    let env = Env::default();
    let (client, admin, _, _) = setup_test(&env);
    let new_admin = Address::generate(&env);

    env.mock_all_auths();
    client.transfer_admin(&admin, &new_admin);

    assert_eq!(client.get_admin(), Some(new_admin.clone()));

    // Old admin can no longer create a schedule
    let beneficiary = Address::generate(&env);
    let result = client.try_create_schedule(&admin, &beneficiary, &1000, &0, &0, &100, &None);
    assert_eq!(result.err(), Some(Ok(VestingError::Unauthorized)));

    // New admin can create a schedule
    client.create_schedule(&new_admin, &beneficiary, &1000, &0, &0, &100, &None);
    assert!(client.get_schedule(&beneficiary).is_some());
}

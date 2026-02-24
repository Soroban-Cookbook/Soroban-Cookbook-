//! Unit tests for Authentication & Custom Authorization contract.
//!
//! Tests cover:
//! - Initialization and admin setup
//! - Role-based access control (grant, revoke, check)
//! - Admin-only and moderator-level actions
//! - Time-lock restrictions
//! - Cooldown enforcement
//! - Contract state gating (Active / Paused / Frozen)

#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup_initialized_contract() -> (Env, Address, Address, AuthContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, contract_id, admin, client)
}

// ---------------------------------------------------------------------------
// 1. Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize() {
    let (_env, _contract_id, admin, client) = setup_initialized_contract();

    assert_eq!(client.get_role(&admin), 0); // 0 = Admin
    assert!(client.has_role(&admin, &Role::Admin));
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_initialize_twice_panics() {
    let (env, _contract_id, _admin, client) = setup_initialized_contract();
    let second_admin = Address::generate(&env);
    client.initialize(&second_admin);
}

// ---------------------------------------------------------------------------
// 2. Role management
// ---------------------------------------------------------------------------

#[test]
fn test_grant_and_check_role() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Moderator);

    assert_eq!(client.get_role(&user), 1); // 1 = Moderator
    assert!(client.has_role(&user, &Role::Moderator));
    assert!(!client.has_role(&user, &Role::Admin));
}

#[test]
fn test_revoke_role() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::User);
    assert!(client.has_role(&user, &Role::User));

    client.revoke_role(&admin, &user);
    assert!(!client.has_role(&user, &Role::User));
}

// ---------------------------------------------------------------------------
// 3. Admin actions
// ---------------------------------------------------------------------------

#[test]
fn test_admin_action_success() {
    let (_env, _contract_id, admin, client) = setup_initialized_contract();

    let result = client.admin_action(&admin, &50);
    assert_eq!(result, 100); // value * 2
}

#[test]
#[should_panic(expected = "Insufficient role")]
fn test_admin_action_non_admin_panics() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();
    let user = Address::generate(&env);
    client.grant_role(&admin, &user, &Role::User);

    client.admin_action(&user, &50);
}

// ---------------------------------------------------------------------------
// 4. Moderator actions
// ---------------------------------------------------------------------------

#[test]
fn test_moderator_action_by_admin() {
    let (_env, _contract_id, admin, client) = setup_initialized_contract();

    let result = client.moderator_action(&admin, &50);
    assert_eq!(result, 150); // value + 100
}

#[test]
fn test_moderator_action_by_moderator() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();
    let moderator = Address::generate(&env);
    client.grant_role(&admin, &moderator, &Role::Moderator);

    let result = client.moderator_action(&moderator, &50);
    assert_eq!(result, 150); // value + 100
}

#[test]
#[should_panic(expected = "Insufficient role")]
fn test_moderator_action_by_user_panics() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();
    let user = Address::generate(&env);
    client.grant_role(&admin, &user, &Role::User);

    client.moderator_action(&user, &50);
}

// ---------------------------------------------------------------------------
// 5. Time-lock restrictions
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Action is time-locked")]
fn test_time_lock_blocks_action() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();

    client.set_time_lock(&admin, &1000);

    env.ledger().with_mut(|li| {
        li.timestamp = 500;
    });

    client.time_locked_action(&admin);
}

#[test]
fn test_time_lock_allows_after_unlock() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();

    client.set_time_lock(&admin, &1000);

    env.ledger().with_mut(|li| {
        li.timestamp = 1001;
    });

    let result = client.time_locked_action(&admin);
    assert_eq!(result, 1001);
}

// ---------------------------------------------------------------------------
// 6. Cooldown enforcement
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Cooldown period not elapsed")]
fn test_cooldown_enforced() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();

    client.set_cooldown(&admin, &100);

    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });
    client.cooldown_action(&admin);

    env.ledger().with_mut(|li| {
        li.timestamp = 250; // only 50s elapsed, need 100
    });
    client.cooldown_action(&admin);
}

#[test]
fn test_cooldown_allows_after_period() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();

    client.set_cooldown(&admin, &100);

    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });
    let first = client.cooldown_action(&admin);
    assert_eq!(first, 200);

    env.ledger().with_mut(|li| {
        li.timestamp = 301; // 101s elapsed, cooldown of 100 satisfied
    });
    let second = client.cooldown_action(&admin);
    assert_eq!(second, 301);
}

// ---------------------------------------------------------------------------
// 7. Contract state gating
// ---------------------------------------------------------------------------

#[test]
fn test_state_active_allows_action() {
    let (env, _contract_id, admin, client) = setup_initialized_contract();

    client.set_state(&admin, &ContractState::Active);

    env.ledger().with_mut(|li| {
        li.timestamp = 500;
    });

    let result = client.active_only_action(&admin);
    assert_eq!(result, 500);
}

#[test]
fn test_get_state_returns_correct_value() {
    let (_env, _contract_id, admin, client) = setup_initialized_contract();

    client.set_state(&admin, &ContractState::Paused);
    assert_eq!(client.get_state(), 1); // 1 = Paused

    client.set_state(&admin, &ContractState::Active);
    assert_eq!(client.get_state(), 0); // 0 = Active

    client.set_state(&admin, &ContractState::Frozen);
    assert_eq!(client.get_state(), 2); // 2 = Frozen
}

#[test]
#[should_panic(expected = "Contract is not active")]
fn test_state_paused_blocks_action() {
    let (_env, _contract_id, admin, client) = setup_initialized_contract();

    client.set_state(&admin, &ContractState::Paused);
    client.active_only_action(&admin);
}

#[test]
#[should_panic(expected = "Contract is not active")]
fn test_state_frozen_blocks_action() {
    let (_env, _contract_id, admin, client) = setup_initialized_contract();

    client.set_state(&admin, &ContractState::Frozen);
    client.active_only_action(&admin);
}

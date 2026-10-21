#![allow(deprecated)]
//! Tests for the Pausable Permissions contract.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, Vec,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_client(env: &Env) -> PausablePermissionsClient<'_> {
    env.mock_all_auths();
    let id = env.register_contract(None, PausablePermissions);
    PausablePermissionsClient::new(env, &id)
}

fn setup(env: &Env) -> (PausablePermissionsClient<'_>, Address) {
    let client = make_client(env);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn initialize_sets_admin_and_starts_unpaused() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    assert!(!client.is_paused());
}

#[test]
fn initialize_twice_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let res = client.try_initialize(&admin);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        PauseError::AlreadyInitialized
    );
}

// ---------------------------------------------------------------------------
// Mechanism 1 — Pauser Role
// ---------------------------------------------------------------------------

#[test]
fn set_pauser_assigns_role() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let pauser = Address::generate(&env);

    client.set_pauser(&admin, &pauser);
    assert_eq!(client.get_pauser(), Some(pauser));
}

#[test]
fn set_pauser_fails_for_non_admin() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let attacker = Address::generate(&env);
    let pauser = Address::generate(&env);

    let res = client.try_set_pauser(&attacker, &pauser);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::NotAdmin);
}

#[test]
fn pause_as_pauser_pauses_contract() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let pauser = Address::generate(&env);

    client.set_pauser(&admin, &pauser);
    client.pause_as_pauser(&pauser);

    assert!(client.is_paused());
}

#[test]
fn pause_as_pauser_fails_for_wrong_address() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let pauser = Address::generate(&env);
    let attacker = Address::generate(&env);

    client.set_pauser(&admin, &pauser);

    let res = client.try_pause_as_pauser(&attacker);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::NotPauser);
}

#[test]
fn pause_as_pauser_fails_when_already_paused() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let pauser = Address::generate(&env);

    client.set_pauser(&admin, &pauser);
    client.pause_as_pauser(&pauser);

    let res = client.try_pause_as_pauser(&pauser);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::ContractPaused);
}

#[test]
fn unpause_by_admin_clears_pause() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let pauser = Address::generate(&env);

    client.set_pauser(&admin, &pauser);
    client.pause_as_pauser(&pauser);
    assert!(client.is_paused());

    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
fn unpause_fails_when_not_paused() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let res = client.try_unpause(&admin);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::AlreadyInState);
}

#[test]
fn unpause_fails_for_non_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let pauser = Address::generate(&env);

    client.set_pauser(&admin, &pauser);
    client.pause_as_pauser(&pauser);

    let res = client.try_unpause(&pauser);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::NotAdmin);
}

// ---------------------------------------------------------------------------
// Mechanism 2 — Multi-Sig Pause
// ---------------------------------------------------------------------------

#[test]
fn set_guardians_stores_config() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let g1 = Address::generate(&env);
    let g2 = Address::generate(&env);
    let g3 = Address::generate(&env);

    let guardians = Vec::from_array(&env, [g1.clone(), g2.clone(), g3.clone()]);
    client.set_guardians(&admin, &guardians, &2u32);

    assert_eq!(client.get_threshold(), 2);
    assert_eq!(client.get_guardians(), guardians);
}

#[test]
fn set_guardians_fails_when_threshold_exceeds_count() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let g1 = Address::generate(&env);
    let guardians = Vec::from_array(&env, [g1.clone()]);

    let res = client.try_set_guardians(&admin, &guardians, &5u32);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::InvalidConfig);
}

#[test]
fn set_guardians_fails_with_zero_threshold() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let g1 = Address::generate(&env);
    let guardians = Vec::from_array(&env, [g1.clone()]);

    let res = client.try_set_guardians(&admin, &guardians, &0u32);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::InvalidConfig);
}

#[test]
fn single_guardian_vote_with_threshold_1_pauses() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let g1 = Address::generate(&env);

    let guardians = Vec::from_array(&env, [g1.clone()]);
    client.set_guardians(&admin, &guardians, &1u32);

    client.guardian_vote_pause(&g1);
    assert!(client.is_paused());
}

#[test]
fn multi_guardian_vote_pauses_at_threshold() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let g1 = Address::generate(&env);
    let g2 = Address::generate(&env);
    let g3 = Address::generate(&env);

    let guardians = Vec::from_array(&env, [g1.clone(), g2.clone(), g3.clone()]);
    client.set_guardians(&admin, &guardians, &2u32);

    // First vote — not yet paused.
    client.guardian_vote_pause(&g1);
    assert!(!client.is_paused());
    assert_eq!(client.pending_votes(), 1);

    // Second vote — threshold met, now paused.
    client.guardian_vote_pause(&g2);
    assert!(client.is_paused());
    // Votes reset after triggering pause.
    assert_eq!(client.pending_votes(), 0);
}

#[test]
fn guardian_vote_fails_for_non_guardian() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let g1 = Address::generate(&env);
    let outsider = Address::generate(&env);

    let guardians = Vec::from_array(&env, [g1.clone()]);
    client.set_guardians(&admin, &guardians, &1u32);

    let res = client.try_guardian_vote_pause(&outsider);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::NotGuardian);
}

#[test]
fn guardian_vote_fails_on_double_vote() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let g1 = Address::generate(&env);
    let g2 = Address::generate(&env);

    let guardians = Vec::from_array(&env, [g1.clone(), g2.clone()]);
    client.set_guardians(&admin, &guardians, &2u32);

    client.guardian_vote_pause(&g1);

    let res = client.try_guardian_vote_pause(&g1);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::AlreadyVoted);
}

// ---------------------------------------------------------------------------
// Mechanism 3 — Time-Limited Pause
// ---------------------------------------------------------------------------

#[test]
fn pause_for_sets_expiry_and_pauses() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let (client, admin) = setup(&env);

    client.pause_for(&admin, &3600u64);

    assert!(client.is_paused());
    assert_eq!(client.pause_expires_at(), 4600); // 1000 + 3600
}

#[test]
fn pause_auto_lifts_after_expiry() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let (client, admin) = setup(&env);

    client.pause_for(&admin, &100u64);
    assert!(client.is_paused());

    // Advance time past expiry.
    env.ledger().with_mut(|li| li.timestamp = 1200);

    assert!(!client.is_paused());
    // assert_not_paused should now succeed.
    client.assert_not_paused();
}

#[test]
fn pause_for_fails_with_zero_duration() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let res = client.try_pause_for(&admin, &0u64);
    assert_eq!(
        res.err().unwrap().ok().unwrap(),
        PauseError::InvalidDuration
    );
}

#[test]
fn pause_for_allowed_for_pauser() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let (client, admin) = setup(&env);
    let pauser = Address::generate(&env);

    client.set_pauser(&admin, &pauser);
    client.pause_for(&pauser, &500u64);

    assert!(client.is_paused());
    assert_eq!(client.pause_expires_at(), 1500);
}

#[test]
fn pause_for_fails_for_non_admin_non_pauser() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let attacker = Address::generate(&env);

    let res = client.try_pause_for(&attacker, &500u64);
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::NotPauser);
}

// ---------------------------------------------------------------------------
// assert_not_paused
// ---------------------------------------------------------------------------

#[test]
fn assert_not_paused_succeeds_when_unpaused() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    // Should not panic.
    client.assert_not_paused();
}

#[test]
fn assert_not_paused_fails_when_paused() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let pauser = Address::generate(&env);
    client.set_pauser(&admin, &pauser);
    client.pause_as_pauser(&pauser);

    let res = client.try_assert_not_paused();
    assert_eq!(res.err().unwrap().ok().unwrap(), PauseError::ContractPaused);
}

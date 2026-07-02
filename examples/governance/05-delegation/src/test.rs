#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::Address as _,
    Address, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Spin up a fresh environment with a registered contract and an admin.
fn setup(env: &Env) -> (DelegationContractClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, DelegationContract);
    let client = DelegationContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

/// Shorthand for a Global-scope delegation.
fn global() -> DelegationScope {
    DelegationScope::Global
}

/// Shorthand for a Topic-scope delegation.
fn topic(sym: Symbol) -> DelegationScope {
    DelegationScope::Topic(sym)
}

// ---------------------------------------------------------------------------
// 1. Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_sets_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
fn test_double_initialize_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let res = client.try_initialize(&admin);
    assert_eq!(res, Err(Ok(DelegationError::AlreadyInitialized)));
}

// ---------------------------------------------------------------------------
// 2. Voting power management
// ---------------------------------------------------------------------------

#[test]
fn test_set_and_get_voting_power() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);

    client.set_voting_power(&admin, &user, &1_000);
    assert_eq!(client.get_voting_power(&user), 1_000);
}

#[test]
fn test_set_voting_power_non_admin_fails() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let attacker = Address::generate(&env);
    let user = Address::generate(&env);

    let res = client.try_set_voting_power(&attacker, &user, &500);
    assert_eq!(res, Err(Ok(DelegationError::Unauthorized)));
}

#[test]
fn test_get_voting_power_default_zero() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let nobody = Address::generate(&env);
    assert_eq!(client.get_voting_power(&nobody), 0);
}

// ---------------------------------------------------------------------------
// 3. Full delegation (basis_points = 10_000)
// ---------------------------------------------------------------------------

#[test]
fn test_full_delegation_and_effective_power() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let topic_sym = symbol_short!("gov");

    client.set_voting_power(&admin, &alice, &1_000);

    // Alice fully delegates to Bob (global scope)
    client.delegate(&alice, &bob, &10_000, &global());

    // Alice retained = 1000 - 1000*10000/10000 = 0
    assert_eq!(client.effective_power(&alice, &topic_sym), 0);
    // Bob received = 1000*10000/10000 = 1000
    assert_eq!(client.effective_power(&bob, &topic_sym), 1_000);
}

// ---------------------------------------------------------------------------
// 4. Partial delegation
// ---------------------------------------------------------------------------

#[test]
fn test_partial_delegation_splits_power() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let topic_sym = symbol_short!("param");

    client.set_voting_power(&admin, &alice, &10_000);

    // Alice delegates 30 % to Bob and 20 % to Carol
    client.delegate(&alice, &bob, &3_000, &global());
    client.delegate(&alice, &carol, &2_000, &global());

    // Alice retains 50 %
    assert_eq!(client.effective_power(&alice, &topic_sym), 5_000);
    // Bob receives 30 %
    assert_eq!(client.effective_power(&bob, &topic_sym), 3_000);
    // Carol receives 20 %
    assert_eq!(client.effective_power(&carol, &topic_sym), 2_000);
}

// ---------------------------------------------------------------------------
// 5. Topic-specific delegation
// ---------------------------------------------------------------------------

#[test]
fn test_topic_delegation_isolated() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let treasury = symbol_short!("treasury");
    let upgrade = symbol_short!("upgrade");

    client.set_voting_power(&admin, &alice, &1_000);

    // Alice delegates treasury topic only to Bob
    client.delegate(&alice, &bob, &5_000, &topic(treasury.clone()));

    // On treasury: Alice retains 500, Bob receives 500
    assert_eq!(client.effective_power(&alice, &treasury), 500);
    assert_eq!(client.effective_power(&bob, &treasury), 500);

    // On upgrade: Alice retains all 1000, Bob has 0
    assert_eq!(client.effective_power(&alice, &upgrade), 1_000);
    assert_eq!(client.effective_power(&bob, &upgrade), 0);
}

// ---------------------------------------------------------------------------
// 6. Delegation registry queries
// ---------------------------------------------------------------------------

#[test]
fn test_delegation_registry_outgoing_and_incoming() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.set_voting_power(&admin, &alice, &1_000);
    client.delegate(&alice, &bob, &5_000, &global());

    let outgoing = client.get_outgoing(&alice);
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing.get(0).unwrap().delegate, bob);

    let incoming = client.get_incoming(&bob);
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming.get(0).unwrap().delegator, alice);
}

#[test]
fn test_get_delegation_record() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.set_voting_power(&admin, &alice, &1_000);
    client.delegate(&alice, &bob, &4_000, &global());

    let rec = client.get_delegation(&alice, &bob, &global());
    assert_eq!(rec.basis_points, 4_000);
    assert!(rec.active);
}

// ---------------------------------------------------------------------------
// 7. Revocation
// ---------------------------------------------------------------------------

#[test]
fn test_revoke_restores_power() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let topic_sym = symbol_short!("gov");

    client.set_voting_power(&admin, &alice, &1_000);
    client.delegate(&alice, &bob, &10_000, &global());

    assert_eq!(client.effective_power(&alice, &topic_sym), 0);
    assert_eq!(client.effective_power(&bob, &topic_sym), 1_000);

    client.revoke(&alice, &bob, &global());

    // After revocation: Alice has all power back, Bob has 0
    assert_eq!(client.effective_power(&alice, &topic_sym), 1_000);
    assert_eq!(client.effective_power(&bob, &topic_sym), 0);

    // Outgoing / incoming indexes cleared
    assert_eq!(client.get_outgoing(&alice).len(), 0);
    assert_eq!(client.get_incoming(&bob).len(), 0);

    // Record is marked inactive
    let rec = client.get_delegation(&alice, &bob, &global());
    assert!(!rec.active);
}

#[test]
fn test_revoke_topic_delegation() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let treasury = symbol_short!("treasury");

    client.set_voting_power(&admin, &alice, &1_000);
    client.delegate(&alice, &bob, &5_000, &topic(treasury.clone()));
    client.revoke(&alice, &bob, &topic(treasury.clone()));

    assert_eq!(client.effective_power(&alice, &treasury), 1_000);
    assert_eq!(client.effective_power(&bob, &treasury), 0);
}

#[test]
fn test_revoke_nonexistent_fails() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let res = client.try_revoke(&alice, &bob, &global());
    assert_eq!(res, Err(Ok(DelegationError::DelegationNotFound)));
}

// ---------------------------------------------------------------------------
// 8. Guard: self-delegation
// ---------------------------------------------------------------------------

#[test]
fn test_self_delegation_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    client.set_voting_power(&admin, &alice, &1_000);

    let res = client.try_delegate(&alice, &alice, &5_000, &global());
    assert_eq!(res, Err(Ok(DelegationError::SelfDelegation)));
}

// ---------------------------------------------------------------------------
// 9. Guard: invalid basis points
// ---------------------------------------------------------------------------

#[test]
fn test_zero_basis_points_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.set_voting_power(&admin, &alice, &1_000);

    let res = client.try_delegate(&alice, &bob, &0, &global());
    assert_eq!(res, Err(Ok(DelegationError::InvalidBasisPoints)));
}

#[test]
fn test_over_10000_basis_points_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.set_voting_power(&admin, &alice, &1_000);

    let res = client.try_delegate(&alice, &bob, &10_001, &global());
    assert_eq!(res, Err(Ok(DelegationError::InvalidBasisPoints)));
}

// ---------------------------------------------------------------------------
// 10. Guard: no chaining
// ---------------------------------------------------------------------------

#[test]
fn test_delegation_chain_prevented() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.set_voting_power(&admin, &alice, &1_000);
    client.set_voting_power(&admin, &bob, &500);

    // Alice delegates to Bob
    client.delegate(&alice, &bob, &5_000, &global());

    // Bob tries to delegate Alice's received power to Carol — not allowed
    let res = client.try_delegate(&bob, &carol, &5_000, &global());
    assert_eq!(res, Err(Ok(DelegationError::DelegateeHasOutgoing)));
}

// ---------------------------------------------------------------------------
// 11. Guard: exceeds total power
// ---------------------------------------------------------------------------

#[test]
fn test_exceeds_total_power_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.set_voting_power(&admin, &alice, &1_000);

    client.delegate(&alice, &bob, &6_000, &global());

    // Alice already delegated 60 %; adding another 50 % would exceed 100 %
    let res = client.try_delegate(&alice, &carol, &5_000, &global());
    assert_eq!(res, Err(Ok(DelegationError::ExceedsTotalPower)));
}

// ---------------------------------------------------------------------------
// 12. Guard: duplicate delegation
// ---------------------------------------------------------------------------

#[test]
fn test_duplicate_delegation_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.set_voting_power(&admin, &alice, &1_000);

    client.delegate(&alice, &bob, &3_000, &global());

    let res = client.try_delegate(&alice, &bob, &2_000, &global());
    assert_eq!(res, Err(Ok(DelegationError::AlreadyDelegated)));
}

// ---------------------------------------------------------------------------
// 13. Mixed global + topic delegations
// ---------------------------------------------------------------------------

#[test]
fn test_mixed_global_and_topic_power() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let treasury = symbol_short!("treasury");
    let upgrade = symbol_short!("upgrade");

    client.set_voting_power(&admin, &alice, &10_000);

    // 30 % global to Bob
    client.delegate(&alice, &bob, &3_000, &global());
    // 20 % treasury-only to Carol
    client.delegate(&alice, &carol, &2_000, &topic(treasury.clone()));

    // On treasury: Alice retains 10000 - 3000 - 2000 = 5000
    assert_eq!(client.effective_power(&alice, &treasury), 5_000);

    // On upgrade: Alice retains 10000 - 3000 = 7000 (treasury delegation doesn't apply)
    assert_eq!(client.effective_power(&alice, &upgrade), 7_000);

    // Bob receives 30 % on any topic
    assert_eq!(client.effective_power(&bob, &treasury), 3_000);
    assert_eq!(client.effective_power(&bob, &upgrade), 3_000);

    // Carol receives 20 % only on treasury
    assert_eq!(client.effective_power(&carol, &treasury), 2_000);
    assert_eq!(client.effective_power(&carol, &upgrade), 0);
}

// ---------------------------------------------------------------------------
// 14. Re-delegate after revocation
// ---------------------------------------------------------------------------

#[test]
fn test_redelegate_after_revoke() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let topic_sym = symbol_short!("gov");

    client.set_voting_power(&admin, &alice, &1_000);

    client.delegate(&alice, &bob, &10_000, &global());
    client.revoke(&alice, &bob, &global());

    // After revoke Alice can delegate again to Carol
    client.delegate(&alice, &carol, &10_000, &global());

    assert_eq!(client.effective_power(&alice, &topic_sym), 0);
    assert_eq!(client.effective_power(&carol, &topic_sym), 1_000);
    assert_eq!(client.effective_power(&bob, &topic_sym), 0);
}

// ---------------------------------------------------------------------------
// 15. Empty registry returns empty vecs
// ---------------------------------------------------------------------------

#[test]
fn test_empty_registry() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let nobody = Address::generate(&env);
    assert_eq!(client.get_outgoing(&nobody).len(), 0);
    assert_eq!(client.get_incoming(&nobody).len(), 0);
}

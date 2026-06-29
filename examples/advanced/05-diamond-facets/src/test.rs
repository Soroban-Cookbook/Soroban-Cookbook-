#![cfg(test)]

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String};

use crate::{
    AccessFacet, AccessFacetClient, DiamondRouter, DiamondRouterClient, RegistryFacet,
    RegistryFacetClient, TokenFacet, TokenFacetClient, ROLE_ADMIN, ROLE_MINTER, ROLE_USER,
};

// ---------------------------------------------------------------------------
// TokenFacet tests
// ---------------------------------------------------------------------------

#[test]
fn test_token_mint_and_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(TokenFacet, ());
    let client = TokenFacetClient::new(&env, &id);

    let minter = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.mint(&minter, &recipient, &1000i128);

    assert_eq!(client.balance_of(&recipient), 1000);
    assert_eq!(client.total_supply(), 1000);
}

#[test]
fn test_token_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(TokenFacet, ());
    let client = TokenFacetClient::new(&env, &id);

    let minter = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.mint(&minter, &alice, &500i128);
    client.transfer(&alice, &bob, &200i128);

    assert_eq!(client.balance_of(&alice), 300);
    assert_eq!(client.balance_of(&bob), 200);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn test_token_transfer_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(TokenFacet, ());
    let client = TokenFacetClient::new(&env, &id);

    let minter = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.mint(&minter, &alice, &100i128);
    client.transfer(&alice, &bob, &200i128); // should panic
}

#[test]
fn test_token_allowance_and_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(TokenFacet, ());
    let client = TokenFacetClient::new(&env, &id);

    let minter = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.mint(&minter, &owner, &1000i128);
    client.approve(&owner, &spender, &300i128);
    client.transfer_from(&spender, &owner, &recipient, &200i128);

    assert_eq!(client.balance_of(&owner), 800);
    assert_eq!(client.balance_of(&recipient), 200);
}

#[test]
fn test_token_multi_mint_total_supply() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(TokenFacet, ());
    let client = TokenFacetClient::new(&env, &id);

    let minter = Address::generate(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);

    client.mint(&minter, &a, &400i128);
    client.mint(&minter, &b, &600i128);

    assert_eq!(client.total_supply(), 1000);
}

// ---------------------------------------------------------------------------
// AccessFacet tests
// ---------------------------------------------------------------------------

#[test]
fn test_access_initialize_and_get_role() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(AccessFacet, ());
    let client = AccessFacetClient::new(&env, &id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    assert_eq!(client.get_role(&admin), ROLE_ADMIN);
}

#[test]
fn test_access_grant_and_revoke_role() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(AccessFacet, ());
    let client = AccessFacetClient::new(&env, &id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);
    client.grant_role(&admin, &user, &ROLE_MINTER);

    assert_eq!(client.get_role(&user), ROLE_MINTER);
    assert!(client.has_role(&user, &ROLE_MINTER));

    client.revoke_role(&admin, &user);
    assert_eq!(client.get_role(&user), ROLE_USER);
}

#[test]
fn test_access_storage_isolation_from_token() {
    // Verify that AccessFacet and TokenFacet use distinct storage keys.
    let env = Env::default();
    env.mock_all_auths();

    let token_id = env.register(TokenFacet, ());
    let access_id = env.register(AccessFacet, ());

    let token_client = TokenFacetClient::new(&env, &token_id);
    let access_client = AccessFacetClient::new(&env, &access_id);

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);

    access_client.initialize(&admin);
    access_client.grant_role(&admin, &minter, &ROLE_MINTER);

    // Minting does NOT require AccessFacet state in this simple facet design
    // (each facet is standalone). The test verifies that access state doesn't
    // bleed into token state.
    token_client.mint(&minter, &admin, &500i128);

    // Token balance is independent of role storage.
    assert_eq!(token_client.balance_of(&admin), 500);
    assert_eq!(access_client.get_role(&minter), ROLE_MINTER);
}

// ---------------------------------------------------------------------------
// RegistryFacet tests
// ---------------------------------------------------------------------------

#[test]
fn test_registry_set_and_get() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(RegistryFacet, ());
    let client = RegistryFacetClient::new(&env, &id);

    let owner = Address::generate(&env);
    let key = symbol_short!("name");
    let value = String::from_str(&env, "Alice");

    client.set_entry(&owner, &key, &value);

    assert_eq!(client.get_entry(&key), Some(value));
    assert_eq!(client.get_owner(&key), Some(owner));
}

#[test]
fn test_registry_update_by_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(RegistryFacet, ());
    let client = RegistryFacetClient::new(&env, &id);

    let owner = Address::generate(&env);
    let key = symbol_short!("bio");

    client.set_entry(&owner, &key, &String::from_str(&env, "v1"));
    client.set_entry(&owner, &key, &String::from_str(&env, "v2"));

    assert_eq!(client.get_entry(&key), Some(String::from_str(&env, "v2")));
}

#[test]
fn test_registry_remove_entry() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(RegistryFacet, ());
    let client = RegistryFacetClient::new(&env, &id);

    let owner = Address::generate(&env);
    let key = symbol_short!("temp");

    client.set_entry(&owner, &key, &String::from_str(&env, "value"));
    client.remove_entry(&owner, &key);

    assert_eq!(client.get_entry(&key), None);
}

#[test]
fn test_registry_missing_entry_returns_none() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register(RegistryFacet, ());
    let client = RegistryFacetClient::new(&env, &id);

    assert_eq!(client.get_entry(&symbol_short!("nope")), None);
    assert_eq!(client.get_owner(&symbol_short!("nope")), None);
}

// ---------------------------------------------------------------------------
// DiamondRouter tests — inter-facet communication
// ---------------------------------------------------------------------------

#[test]
fn test_router_register_and_query_facets() {
    let env = Env::default();
    env.mock_all_auths();

    let router_id = env.register(DiamondRouter, ());
    let token_id = env.register(TokenFacet, ());
    let access_id = env.register(AccessFacet, ());
    let registry_id = env.register(RegistryFacet, ());

    let router = DiamondRouterClient::new(&env, &router_id);
    let admin = Address::generate(&env);

    router.register_facets(&admin, &token_id, &access_id, &registry_id);

    assert_eq!(router.get_facet(&symbol_short!("token")), Some(token_id));
    assert_eq!(router.get_facet(&symbol_short!("access")), Some(access_id));
    assert_eq!(
        router.get_facet(&symbol_short!("registry")),
        Some(registry_id)
    );
    assert_eq!(router.get_facet(&symbol_short!("unknown")), None);
}

#[test]
fn test_router_mint_and_register_inter_facet() {
    let env = Env::default();
    env.mock_all_auths();

    let router_id = env.register(DiamondRouter, ());
    let token_id = env.register(TokenFacet, ());
    let access_id = env.register(AccessFacet, ());
    let registry_id = env.register(RegistryFacet, ());

    let router = DiamondRouterClient::new(&env, &router_id);
    let token_client = TokenFacetClient::new(&env, &token_id);
    let registry_client = RegistryFacetClient::new(&env, &registry_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    router.register_facets(&admin, &token_id, &access_id, &registry_id);

    // Single router call that touches two facets atomically.
    router.mint_and_register(
        &admin,
        &recipient,
        &750i128,
        &symbol_short!("mint_meta"),
        &String::from_str(&env, "initial-mint"),
    );

    // TokenFacet state updated
    assert_eq!(token_client.balance_of(&recipient), 750);
    assert_eq!(token_client.total_supply(), 750);

    // RegistryFacet state updated in same transaction
    assert_eq!(
        registry_client.get_entry(&symbol_short!("mint_meta")),
        Some(String::from_str(&env, "initial-mint"))
    );
}

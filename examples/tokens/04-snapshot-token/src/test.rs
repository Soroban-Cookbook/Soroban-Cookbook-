#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Env, Event, String, Symbol,
};

struct Fixture {
    env: Env,
    admin: Address,
    token_id: Address,
    client: SnapshotTokenClient<'static>,
    alice: Address,
    bob: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register(SnapshotToken, ());
    let client = SnapshotTokenClient::new(&env, &token_id);

    let name = String::from_str(&env, "Governance Token");
    let symbol = Symbol::new(&env, "GOV");
    client.initialize(&admin, &name, &symbol, &18u32);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    Fixture {
        env,
        admin,
        token_id,
        client,
        alice,
        bob,
    }
}

#[test]
fn test_initialization() {
    let f = setup();
    assert_eq!(
        f.client.name(),
        String::from_str(&f.env, "Governance Token")
    );
    assert_eq!(f.client.symbol(), Symbol::new(&f.env, "GOV"));
    assert_eq!(f.client.decimals(), 18u32);
    assert_eq!(f.client.admin(), f.admin);
    assert_eq!(f.client.total_supply(), 0);
    assert_eq!(f.client.current_snapshot(), 0);
}

#[test]
fn test_mint_admin_only() {
    let f = setup();

    // Admin can mint
    let balance = f.client.mint(&f.admin, &f.alice, &1000);
    assert_eq!(balance, 1000);
    assert_eq!(f.client.balance(&f.alice), 1000);
    assert_eq!(f.client.total_supply(), 1000);

    // Non-admin minting fails with Unauthorized
    let non_admin = Address::generate(&f.env);
    let res = f.client.try_mint(&non_admin, &f.alice, &500);
    assert_eq!(res, Err(Ok(SnapshotTokenError::Unauthorized)));
}

#[test]
fn test_transfer_correctness() {
    let f = setup();
    f.client.mint(&f.admin, &f.alice, &1000);

    f.client.transfer(&f.alice, &f.bob, &400);
    assert_eq!(f.client.balance(&f.alice), 600);
    assert_eq!(f.client.balance(&f.bob), 400);
}

#[test]
fn test_transfer_insufficient_balance() {
    let f = setup();
    f.client.mint(&f.admin, &f.alice, &300);

    let res = f.client.try_transfer(&f.alice, &f.bob, &400);
    assert_eq!(res, Err(Ok(SnapshotTokenError::InsufficientBalance)));
}

#[test]
fn test_create_snapshot_unauthorized() {
    let f = setup();
    let non_admin = Address::generate(&f.env);
    let res = f.client.try_create_snapshot(&non_admin);
    assert_eq!(res, Err(Ok(SnapshotTokenError::Unauthorized)));
}

#[test]
fn test_create_snapshot_increments_id() {
    let f = setup();
    let id1 = f.client.create_snapshot(&f.admin);
    assert_eq!(id1, 1);
    assert_eq!(f.client.current_snapshot(), 1);
    assert_eq!(f.client.total_snapshots(), 1);

    let id2 = f.client.create_snapshot(&f.admin);
    assert_eq!(id2, 2);
    assert_eq!(f.client.current_snapshot(), 2);
    assert_eq!(f.client.total_snapshots(), 2);
}

#[test]
fn test_balance_at_snapshot_no_activity() {
    let f = setup();
    // Alice gets minted 100 tokens, then snapshot is taken.
    f.client.mint(&f.admin, &f.alice, &100);
    let id = f.client.create_snapshot(&f.admin);

    // No transactions happen after snapshot.
    // Her balance at that snapshot should be her current balance (100).
    assert_eq!(f.client.balance_at_snapshot(&f.alice, &id), 100);
}

#[test]
fn test_balance_at_snapshot_changed_before_snapshot() {
    let f = setup();
    // Alice gets minted 100, then transfers 30 to Bob.
    f.client.mint(&f.admin, &f.alice, &100);
    f.client.transfer(&f.alice, &f.bob, &30);

    // Snapshot taken.
    let id = f.client.create_snapshot(&f.admin);

    // Query balance at snapshot should reflect the transfer (70 for Alice, 30 for Bob)
    assert_eq!(f.client.balance_at_snapshot(&f.alice, &id), 70);
    assert_eq!(f.client.balance_at_snapshot(&f.bob, &id), 30);
}

#[test]
fn test_balance_at_snapshot_unaffected_by_later_transfer() {
    let f = setup();
    f.client.mint(&f.admin, &f.alice, &500);

    // Snapshot taken when Alice has 500, Bob has 0
    let id = f.client.create_snapshot(&f.admin);

    // After snapshot, Alice transfers 200 to Bob
    f.client.transfer(&f.alice, &f.bob, &200);

    // Check current balance has updated
    assert_eq!(f.client.balance(&f.alice), 300);
    assert_eq!(f.client.balance(&f.bob), 200);

    // Historical balance queries should still yield Alice: 500, Bob: 0
    assert_eq!(f.client.balance_at_snapshot(&f.alice, &id), 500);
    assert_eq!(f.client.balance_at_snapshot(&f.bob, &id), 0);
}

#[test]
fn test_multiple_snapshots_interleaved() {
    let f = setup();

    // Initial Mint
    f.client.mint(&f.admin, &f.alice, &1000); // Alice: 1000
    let id1 = f.client.create_snapshot(&f.admin); // Snapshot 1: Alice: 1000, Bob: 0

    // Transfer and Snapshot 2
    f.client.transfer(&f.alice, &f.bob, &300); // Alice: 700, Bob: 300
    let id2 = f.client.create_snapshot(&f.admin); // Snapshot 2: Alice: 700, Bob: 300

    // Mint and Snapshot 3
    f.client.mint(&f.admin, &f.bob, &100); // Bob: 400
    let id3 = f.client.create_snapshot(&f.admin); // Snapshot 3: Alice: 700, Bob: 400

    // Burn and Snapshot 4
    f.client.burn(&f.alice, &200); // Alice: 500
    let id4 = f.client.create_snapshot(&f.admin); // Snapshot 4: Alice: 500, Bob: 400

    // Final checks
    // Snapshot 1
    assert_eq!(f.client.balance_at_snapshot(&f.alice, &id1), 1000);
    assert_eq!(f.client.balance_at_snapshot(&f.bob, &id1), 0);
    assert_eq!(f.client.total_supply_at_snapshot(&id1), 1000);

    // Snapshot 2
    assert_eq!(f.client.balance_at_snapshot(&f.alice, &id2), 700);
    assert_eq!(f.client.balance_at_snapshot(&f.bob, &id2), 300);
    assert_eq!(f.client.total_supply_at_snapshot(&id2), 1000);

    // Snapshot 3
    assert_eq!(f.client.balance_at_snapshot(&f.alice, &id3), 700);
    assert_eq!(f.client.balance_at_snapshot(&f.bob, &id3), 400);
    assert_eq!(f.client.total_supply_at_snapshot(&id3), 1100);

    // Snapshot 4
    assert_eq!(f.client.balance_at_snapshot(&f.alice, &id4), 500);
    assert_eq!(f.client.balance_at_snapshot(&f.bob, &id4), 400);
    assert_eq!(f.client.total_supply_at_snapshot(&id4), 900);
}

#[test]
fn test_query_non_existent_snapshot_id() {
    let f = setup();
    f.client.create_snapshot(&f.admin);

    let res1 = f.client.try_balance_at_snapshot(&f.alice, &0);
    assert_eq!(res1, Err(Ok(SnapshotTokenError::SnapshotNotFound)));

    let res2 = f.client.try_balance_at_snapshot(&f.alice, &2);
    assert_eq!(res2, Err(Ok(SnapshotTokenError::SnapshotNotFound)));
}

#[test]
fn test_query_account_never_held_tokens() {
    let f = setup();
    let id = f.client.create_snapshot(&f.admin);

    let random_user = Address::generate(&f.env);
    assert_eq!(f.client.balance_at_snapshot(&random_user, &id), 0);
}

#[test]
fn test_events_emitted() {
    let f = setup();

    let mint_event = MintEvent {
        to: f.alice.clone(),
        amount: 1000,
    };
    f.client.mint(&f.admin, &f.alice, &1000);
    let events = f.env.events().all().filter_by_contract(&f.token_id);
    assert_eq!(events, [mint_event.to_xdr(&f.env, &f.token_id)]);

    let snapshot_event = SnapshotCreatedEvent { snapshot_id: 1 };
    f.client.create_snapshot(&f.admin);
    let events = f.env.events().all().filter_by_contract(&f.token_id);
    assert_eq!(events, [snapshot_event.to_xdr(&f.env, &f.token_id)]);

    let transfer_event = TransferEvent {
        from: f.alice.clone(),
        to: f.bob.clone(),
        amount: 200,
    };
    f.client.transfer(&f.alice, &f.bob, &200);
    let events = f.env.events().all().filter_by_contract(&f.token_id);
    assert_eq!(events, [transfer_event.to_xdr(&f.env, &f.token_id)]);

    let burn_event = BurnEvent {
        from: f.alice.clone(),
        amount: 100,
    };
    f.client.burn(&f.alice, &100);
    let events = f.env.events().all().filter_by_contract(&f.token_id);
    assert_eq!(events, [burn_event.to_xdr(&f.env, &f.token_id)]);
}

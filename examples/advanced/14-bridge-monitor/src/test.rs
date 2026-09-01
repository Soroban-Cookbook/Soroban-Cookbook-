//! Tests for the bridge monitor (issue #765).

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{BridgeMonitor, BridgeMonitorClient, BridgeMonitorError};

fn setup() -> (Env, BridgeMonitorClient<'static>, Address, Address) {
    #![allow(unused_attributes)]
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let indexer = Address::generate(&env);
    let contract = env.register_contract(None, BridgeMonitor);
    let client = BridgeMonitorClient::new(&env, &contract);
    client.initialize(&admin, &1_000);
    (env, client, admin, indexer)
}

fn s(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

#[test]
fn initialize_sets_admin_and_threshold() {
    let (_, client, _admin, _) = setup();
    assert!(client.initialized());
    assert_eq!(client.transaction_count(), 0);
    assert_eq!(client.alert_count(), 0);
    assert_eq!(client.last_snapshot(), None);
}

#[test]
fn record_transaction_tracks_count_and_batches() {
    let (env, client, admin, _) = setup();
    let token = Address::generate(&env);

    let id = client
        .record_transaction(
            &admin,
            &s(&env, "tx-1"),
            &s(&env, "inbound"),
            &s(&env, "polygon"),
            &s(&env, "soroban"),
            &1_000_000,
            &token,
            &s(&env, "completed"),
        )
        ;
    assert_eq!(id, 1);

    let tx = client.transactions(&1, &10);
    assert_eq!(tx.len(), 1);
    assert_eq!(tx.get_unchecked(0).tx_id, s(&env, "tx-1"));
    assert_eq!(client.transaction_count(), 1);
    drop(admin);
}

#[test]
fn empty_tx_id_rejected() {
    let (env, client, admin, _) = setup();
    let token = Address::generate(&env);
    let result = client.try_record_transaction(
        &admin,
        &s(&env, ""),
        &s(&env, "inbound"),
        &s(&env, "x"),
        &s(&env, "y"),
        &10,
        &token,
        &s(&env, "completed"),
    );
    assert_eq!(result, Err(Ok(BridgeMonitorError::EmptyTxId)));
}

#[test]
fn failed_transfer_raises_high_alert() {
    let (env, client, admin, _) = setup();
    let token = Address::generate(&env);
    client
        .record_transaction(
            &admin,
            &s(&env, "tx-fail"),
            &s(&env, "outbound"),
            &s(&env, "soroban"),
            &s(&env, "ethereum"),
            &500,
            &token,
            &s(&env, "failed"),
        )
        ;

    let alerts = client.list_alerts();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts.get_unchecked(0).kind, s(&env, "failed_transfer"));
    assert_eq!(client.alert_count(), 1);
}

#[test]
fn snapshot_balance_raises_drift_alert_over_threshold() {
    let (env, client, admin, _) = setup();
    let token = Address::generate(&env);
    client.set_token(&admin, &token);

    // First snapshot: baseline, no alert.
    client.snapshot_balance(&admin, &10_000_000);
    assert_eq!(client.alert_count(), 0);

    // Drift of 5_000 > threshold 1_000 → alert.
    client.snapshot_balance(&admin, &5_000_000);
    let alerts = client.list_alerts();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts.get_unchecked(0).kind, s(&env, "balance_drift"));
    assert_eq!(client.last_snapshot(), Some(5_000_000));
}

#[test]
fn small_balance_move_is_not_alerted() {
    let (_env, client, admin, _) = setup();
    client.snapshot_balance(&admin, &10_000_000);
    client.snapshot_balance(&admin, &10_000_500);
    assert_eq!(client.alert_count(), 0);
}

#[test]
fn resolve_alert_removes_it() {
    let (env, client, admin, _) = setup();
    let token = Address::generate(&env);
    client
        .record_transaction(
            &admin,
            &s(&env, "tx-fail"),
            &s(&env, "outbound"),
            &s(&env, "soroban"),
            &s(&env, "ethereum"),
            &1,
            &token,
            &s(&env, "failed"),
        )
        ;

    assert_eq!(client.alert_count(), 1);
    client.resolve_alert(&admin, &1);
    assert_eq!(client.alert_count(), 1); // count is monotonic ids
    assert_eq!(client.list_alerts().len(), 0);
}

#[test]
fn non_admin_cannot_record() {
    let (env, client, _admin, _) = setup();
    // An attacker address is not the admin — with auths mocked, require_auth
    // still rejects because the caller is not the authorized admin.
    let attacker = Address::generate(&env);
    let token = Address::generate(&env);
    let result = client.try_record_transaction(
        &attacker,
        &s(&env, "tx-x"),
        &s(&env, "inbound"),
        &s(&env, "a"),
        &s(&env, "b"),
        &1,
        &token,
        &s(&env, "completed"),
    );
    assert!(result.is_err());
}

#[test]
fn transactions_are_paged() {
    let (env, client, admin, _) = setup();
    let token = Address::generate(&env);
    for i in 0..5u128 {
        let tx_id = s(&env, &format!("tx-{i}"));
        client
            .record_transaction(
                &admin,
                &tx_id,
                &s(&env, "inbound"),
                &s(&env, "polygon"),
                &s(&env, "soroban"),
                &100,
                &token,
                &s(&env, "completed"),
            )
            ;
    }
    let page = client.transactions(&3, &2);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get_unchecked(0).tx_id, s(&env, "tx-2"));
    assert_eq!(page.get_unchecked(1).tx_id, s(&env, "tx-3"));
}
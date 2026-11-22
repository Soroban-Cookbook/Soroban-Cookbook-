#![allow(deprecated)]
extern crate std;
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup() -> (Env, Address, SnapshotTriggerClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &10);
    (env, admin, client)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &10);

    let (freq, enabled, last, total) = client.get_config();
    assert_eq!(freq, 10);
    assert!(enabled);
    assert_eq!(last, 0);
    assert_eq!(total, 0);
}

#[test]
fn test_initialize_cannot_be_called_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &10);
    let result = client.try_initialize(&admin, &20);
    assert!(result.is_err());
}

#[test]
fn test_record_value_event_based() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    assert!(client.record_value(&user, &1000));
    assert_eq!(client.snapshot_count(&user), 1);

    env.ledger().set_sequence_number(10);
    assert!(client.record_value(&user, &2000));
    assert_eq!(client.snapshot_count(&user), 2);

    let snapshots = client.get_all_snapshots(&user);
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots.get_unchecked(0).value, 1000);
    assert_eq!(snapshots.get_unchecked(1).value, 2000);
}

#[test]
fn test_auto_snapshot_time_based_when_due() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.record_value(&user, &500);
    assert_eq!(client.snapshot_count(&user), 1);

    env.ledger().set_sequence_number(10);
    let result = client.try_auto_snapshot(&user);
    assert!(result.is_ok());
    assert_eq!(client.snapshot_count(&user), 2);
}

#[test]
fn test_auto_snapshot_skipped_when_not_due() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.record_value(&user, &500);
    env.ledger().set_sequence_number(5);
    let result = client.try_auto_snapshot(&user);
    assert!(result.is_err());
    assert_eq!(client.snapshot_count(&user), 1);
}

#[test]
fn test_auto_snapshot_disabled() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    client.set_enabled(&admin, &false);
    env.ledger().set_sequence_number(20);
    let result = client.try_auto_snapshot(&user);
    assert!(result.is_err());
}

#[test]
fn test_get_snapshot_by_index() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.record_value(&user, &100);
    env.ledger().set_sequence_number(10);
    client.record_value(&user, &200);
    env.ledger().set_sequence_number(20);
    client.record_value(&user, &300);

    let first = client.get_snapshot(&user, &0);
    assert_eq!(first.value, 100);

    let second = client.get_snapshot(&user, &1);
    assert_eq!(second.value, 200);

    let third = client.get_snapshot(&user, &2);
    assert_eq!(third.value, 300);
}

#[test]
fn test_get_snapshot_out_of_bounds() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    client.record_value(&user, &100);
    let result = client.try_get_snapshot(&user, &5);
    assert!(result.is_err());
}

#[test]
fn test_get_latest_snapshot() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    client.record_value(&user, &42);
    env.ledger().set_sequence_number(10);
    client.record_value(&user, &99);
    let latest = client.get_latest(&user);
    assert_eq!(latest.value, 99);
}

#[test]
fn test_get_latest_empty_history() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    let result = client.try_get_latest(&user);
    assert!(result.is_err());
}

#[test]
fn test_prune_old_snapshots() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    env.ledger().set_sequence_number(10);
    client.record_value(&user, &10);
    env.ledger().set_sequence_number(20);
    client.record_value(&user, &20);
    env.ledger().set_sequence_number(30);
    client.record_value(&user, &30);
    assert_eq!(client.snapshot_count(&user), 3);

    let pruned = client.prune(&admin, &user, &20);
    assert_eq!(pruned, 1);

    let remaining = client.get_all_snapshots(&user);
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining.get_unchecked(0).value, 20);
}

#[test]
fn test_prune_nothing_to_remove() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    env.ledger().set_sequence_number(10);
    client.record_value(&user, &10);
    env.ledger().set_sequence_number(20);
    client.record_value(&user, &20);

    let result = client.try_prune(&admin, &user, &5);
    assert!(result.is_err());
}

#[test]
fn test_set_frequency_updates_config() {
    let (_env, admin, client) = setup();
    client.set_frequency(&admin, &25);
    let (freq, ..) = client.get_config();
    assert_eq!(freq, 25);
}

#[test]
fn test_unauthorized_admin_actions_fail() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    client.initialize(&admin, &10);

    let result = client.try_set_frequency(&attacker, &20);
    assert!(result.is_err());
}

#[test]
fn test_snapshot_timestamps_and_ledgers_recorded() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    env.ledger().set_sequence_number(100);
    env.ledger().set_timestamp(1000);
    client.record_value(&user, &777);

    let snap = client.get_snapshot(&user, &0);
    assert_eq!(snap.ledger, 100);
    assert_eq!(snap.timestamp, 1000);
    assert_eq!(snap.value, 777);
}

#[test]
fn test_get_config_returns_expected_values() {
    let (env, admin, client) = setup();
    let (freq, enabled, last, total) = client.get_config();
    assert_eq!(freq, 10);
    assert!(enabled);
    assert_eq!(last, 0);
    assert_eq!(total, 0);

    env.ledger().set_sequence_number(15);
    client.record_value(&admin, &500);
    let (.., total) = client.get_config();
    assert_eq!(total, 1);
}

// ── interval gating ─────────────────────────────────────────────────────────

#[test]
fn test_initialize_rejects_zero_interval() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    assert_eq!(
        client.try_initialize(&admin, &0),
        Err(Ok(SnapshotError::InvalidFrequency))
    );
    assert_eq!(
        client.try_initialize(&admin, &(MAX_INTERVAL_LEDGERS + 1)),
        Err(Ok(SnapshotError::InvalidFrequency))
    );
}

#[test]
fn test_set_frequency_rejects_zero() {
    let (_env, admin, client) = setup();
    assert_eq!(
        client.try_set_frequency(&admin, &0),
        Err(Ok(SnapshotError::InvalidFrequency))
    );
}

#[test]
fn test_record_within_interval_updates_current_only() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    assert!(client.record_value(&user, &100));
    env.ledger().set_sequence_number(3);
    assert!(!client.record_value(&user, &150));
    env.ledger().set_sequence_number(9);
    assert!(!client.record_value(&user, &175));

    assert_eq!(client.snapshot_count(&user), 1);
    assert_eq!(client.get_latest(&user).value, 100);
    assert_eq!(client.get_current(&user), 175);
    assert_eq!(client.next_snapshot_ledger(&user), 10);

    env.ledger().set_sequence_number(10);
    assert!(client.record_value(&user, &180));
    assert_eq!(client.snapshot_count(&user), 2);
}

#[test]
fn test_interval_is_per_owner() {
    let (env, _admin, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    assert!(client.record_value(&alice, &1));
    // Alice's recent snapshot must not block Bob's first snapshot.
    assert!(client.record_value(&bob, &2));
    assert_eq!(client.snapshot_count(&bob), 1);
}

#[test]
fn test_auto_snapshot_uses_current_value() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    client.record_value(&user, &10);
    env.ledger().set_sequence_number(4);
    client.record_value(&user, &40); // coalesced, not snapshotted
    env.ledger().set_sequence_number(10);
    client.auto_snapshot(&user);
    assert_eq!(client.get_latest(&user).value, 40);
}

#[test]
fn test_disabled_records_no_snapshots() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);
    client.set_enabled(&admin, &false);
    assert!(!client.record_value(&user, &10));
    assert_eq!(client.snapshot_count(&user), 0);
    assert_eq!(client.get_current(&user), 10);
}

#[test]
fn test_high_participation_interval_bounded() {
    let (env, _admin, client) = setup();
    let voter = Address::generate(&env);

    // 500 updates, one per ledger, over ledgers 0..500 with interval 10.
    for i in 0..500u32 {
        env.ledger().set_sequence_number(i);
        client.record_value(&voter, &(i as i128));
    }
    // At most one snapshot per 10 ledgers instead of 500.
    assert_eq!(client.snapshot_count(&voter), 50);
    assert_eq!(client.get_current(&voter), 499);
    let (.., total) = client.get_config();
    assert_eq!(total, 50);
}

// ── proposals and voting ────────────────────────────────────────────────────

#[test]
fn test_vote_uses_power_before_proposal_interval_mode() {
    let (env, _admin, client) = setup();
    let voter = Address::generate(&env);
    let proposer = Address::generate(&env);

    env.ledger().set_sequence_number(5);
    client.record_value(&voter, &300);
    env.ledger().set_sequence_number(20);
    let pid = client.create_proposal(&proposer);

    // Power increase after proposal creation does not count.
    env.ledger().set_sequence_number(40);
    client.record_value(&voter, &10_000);

    assert_eq!(client.vote(&voter, &pid, &true), 300);
    let p = client.get_proposal(&pid);
    assert_eq!(p.votes_for, 300);
    assert_eq!(p.voter_count, 1);
}

#[test]
fn test_double_vote_rejected() {
    let (env, _admin, client) = setup();
    let voter = Address::generate(&env);
    client.record_value(&voter, &5);
    env.ledger().set_sequence_number(1);
    let pid = client.create_proposal(&voter);
    client.vote(&voter, &pid, &false);
    assert_eq!(
        client.try_vote(&voter, &pid, &true),
        Err(Ok(SnapshotError::AlreadyVoted))
    );
    assert_eq!(client.get_proposal(&pid).votes_against, 5);
}

#[test]
fn test_vote_errors() {
    let (env, _admin, client) = setup();
    let voter = Address::generate(&env);
    assert_eq!(
        client.try_vote(&voter, &1, &true),
        Err(Ok(SnapshotError::ProposalNotFound))
    );
    let pid = client.create_proposal(&voter);
    assert_eq!(
        client.try_vote(&voter, &pid, &true),
        Err(Ok(SnapshotError::NoVotingPower))
    );
}

#[test]
fn test_create_proposal_requires_init() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SnapshotTrigger);
    let client = SnapshotTriggerClient::new(&env, &contract_id);
    let p = Address::generate(&env);
    assert_eq!(
        client.try_create_proposal(&p),
        Err(Ok(SnapshotError::NotInitialized))
    );
}

// ── epoch mode ──────────────────────────────────────────────────────────────

#[test]
fn test_epoch_mode_checkpoints_once_per_epoch() {
    let (env, admin, client) = setup();
    client.set_mode(&admin, &SnapshotMode::Epoch);
    assert_eq!(client.get_mode(), SnapshotMode::Epoch);
    let user = Address::generate(&env);

    // No proposal yet: nothing to checkpoint.
    assert!(!client.record_value(&user, &100));
    assert_eq!(client.epoch_snapshot_count(&user), 0);

    let p1 = client.create_proposal(&admin);
    assert!(client.record_value(&user, &200)); // checkpoints 100 @ epoch 1
    assert!(!client.record_value(&user, &300)); // same epoch: no write
    assert!(!client.record_value(&user, &400));
    assert_eq!(client.epoch_snapshot_count(&user), 1);

    let p2 = client.create_proposal(&admin);
    let p3 = client.create_proposal(&admin);
    assert!(client.record_value(&user, &500)); // checkpoints 400 @ epoch 3

    assert_eq!(client.voting_power_at(&user, &p1), 100);
    assert_eq!(client.voting_power_at(&user, &p2), 400);
    assert_eq!(client.voting_power_at(&user, &p3), 400);

    let p4 = client.create_proposal(&admin);
    assert_eq!(client.voting_power_at(&user, &p4), 500);

    let recs = client.get_epoch_snapshots(&user);
    assert_eq!(
        recs.get_unchecked(0),
        EpochRecord {
            epoch: 1,
            value: 100
        }
    );
    assert_eq!(
        recs.get_unchecked(1),
        EpochRecord {
            epoch: 3,
            value: 400
        }
    );
}

#[test]
fn test_high_participation_epoch_mode_no_vote_snapshots() {
    let (env, admin, client) = setup();
    client.set_mode(&admin, &SnapshotMode::Epoch);

    let mut voters = std::vec::Vec::new();
    for _ in 0..200 {
        let v = Address::generate(&env);
        client.record_value(&v, &10);
        voters.push(v);
    }
    let pid = client.create_proposal(&admin);
    for (i, v) in voters.iter().enumerate() {
        client.vote(v, &pid, &(i % 2 == 0));
    }

    let p = client.get_proposal(&pid);
    assert_eq!(p.voter_count, 200);
    assert_eq!(p.votes_for, 1000);
    assert_eq!(p.votes_against, 1000);
    // 200 votes created zero snapshot entries.
    let (.., total) = client.get_config();
    assert_eq!(total, 0);
}

extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(supply: i128) -> (Env, Sep41ExtensionsClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Sep41Extensions);
    let client = Sep41ExtensionsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &supply);
    (env, client, admin)
}

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_sets_balance() {
    let (env, client, admin) = setup(1_000);
    assert_eq!(client.balance(&admin), 1_000);
    let _ = env;
}

#[test]
fn test_initialize_double_call_fails() {
    let (env, client, admin) = setup(1_000);
    let result = client.try_initialize(&admin, &500);
    assert_eq!(result, Err(Ok(ExtError::AlreadyInitialized)));
    let _ = env;
}

#[test]
fn test_initialize_zero_supply_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Sep41Extensions);
    let client = Sep41ExtensionsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    assert_eq!(
        client.try_initialize(&admin, &0),
        Err(Ok(ExtError::InvalidAmount))
    );
}

// ---------------------------------------------------------------------------
// Permit tests
// ---------------------------------------------------------------------------

#[test]
fn test_permit_sets_allowance() {
    let (env, client, owner) = setup(1_000);
    let spender = Address::generate(&env);

    client.permit(&owner, &spender, &500, &100_000);

    assert_eq!(client.allowance(&owner, &spender), 500);
}

#[test]
fn test_permit_expired_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Sep41Extensions);
    let client = Sep41ExtensionsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &1_000);

    let spender = Address::generate(&env);
    // ledger sequence is 0 by default; expiration_ledger = 0 with amount > 0 is expired
    // We set a past ledger by advancing the ledger past the expiry.
    // Default sequence is 0, and 0 < 0 is false, so let's advance the ledger.
    // Actually: expiry 0 with amount > 0 means expiry < current sequence only if current > 0.
    // Let's use expiry = 1 with ledger advanced to 2.
    use soroban_sdk::testutils::Ledger as _;
    env.ledger().set_sequence_number(2);

    let result = client.try_permit(&admin, &spender, &100, &1);
    assert_eq!(result, Err(Ok(ExtError::ExpiredPermit)));
}

#[test]
fn test_permit_zero_amount_revokes() {
    let (env, client, owner) = setup(1_000);
    let spender = Address::generate(&env);

    client.permit(&owner, &spender, &500, &100_000);
    assert_eq!(client.allowance(&owner, &spender), 500);

    // Revoke by setting amount to 0
    client.permit(&owner, &spender, &0, &0);
    assert_eq!(client.allowance(&owner, &spender), 0);
}

// ---------------------------------------------------------------------------
// Batch transfer tests
// ---------------------------------------------------------------------------

#[test]
fn test_batch_transfer_single_recipient() {
    let (env, client, admin) = setup(1_000);
    let r = Address::generate(&env);

    let transfers = Vec::from_array(
        &env,
        [Transfer {
            to: r.clone(),
            amount: 300,
        }],
    );
    let count = client.batch_transfer(&admin, &transfers);

    assert_eq!(count, 1);
    assert_eq!(client.balance(&admin), 700);
    assert_eq!(client.balance(&r), 300);
}

#[test]
fn test_batch_transfer_multiple_recipients() {
    let (env, client, admin) = setup(1_000);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let r3 = Address::generate(&env);

    let transfers = Vec::from_array(
        &env,
        [
            Transfer {
                to: r1.clone(),
                amount: 100,
            },
            Transfer {
                to: r2.clone(),
                amount: 200,
            },
            Transfer {
                to: r3.clone(),
                amount: 300,
            },
        ],
    );
    let count = client.batch_transfer(&admin, &transfers);

    assert_eq!(count, 3);
    assert_eq!(client.balance(&admin), 400);
    assert_eq!(client.balance(&r1), 100);
    assert_eq!(client.balance(&r2), 200);
    assert_eq!(client.balance(&r3), 300);
}

#[test]
fn test_batch_transfer_empty_fails() {
    let (env, client, admin) = setup(1_000);
    let empty: Vec<Transfer> = Vec::new(&env);
    assert_eq!(
        client.try_batch_transfer(&admin, &empty),
        Err(Ok(ExtError::EmptyBatch))
    );
}

#[test]
fn test_batch_transfer_insufficient_balance_fails() {
    let (env, client, admin) = setup(100);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);

    let transfers = Vec::from_array(
        &env,
        [
            Transfer {
                to: r1.clone(),
                amount: 70,
            },
            Transfer {
                to: r2.clone(),
                amount: 50,
            }, // 70+50 = 120 > 100
        ],
    );
    assert_eq!(
        client.try_batch_transfer(&admin, &transfers),
        Err(Ok(ExtError::InsufficientBalance))
    );
    // No state mutation
    assert_eq!(client.balance(&admin), 100);
    assert_eq!(client.balance(&r1), 0);
}

#[test]
fn test_batch_transfer_zero_amount_fails() {
    let (env, client, admin) = setup(1_000);
    let r = Address::generate(&env);

    let transfers = Vec::from_array(
        &env,
        [
            Transfer {
                to: r.clone(),
                amount: 100,
            },
            Transfer {
                to: r.clone(),
                amount: 0,
            },
        ],
    );
    assert_eq!(
        client.try_batch_transfer(&admin, &transfers),
        Err(Ok(ExtError::InvalidAmount))
    );
    assert_eq!(client.balance(&admin), 1_000);
}

// ---------------------------------------------------------------------------
// Batch approve tests
// ---------------------------------------------------------------------------

#[test]
fn test_batch_approve_sets_allowances() {
    let (env, client, owner) = setup(1_000);
    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);

    let approvals = Vec::from_array(
        &env,
        [
            Approval {
                spender: s1.clone(),
                amount: 200,
                expiration_ledger: 100_000,
            },
            Approval {
                spender: s2.clone(),
                amount: 500,
                expiration_ledger: 100_000,
            },
        ],
    );
    let count = client.batch_approve(&owner, &approvals);

    assert_eq!(count, 2);
    assert_eq!(client.allowance(&owner, &s1), 200);
    assert_eq!(client.allowance(&owner, &s2), 500);
}

#[test]
fn test_batch_approve_empty_fails() {
    let (env, client, owner) = setup(1_000);
    let empty: Vec<Approval> = Vec::new(&env);
    assert_eq!(
        client.try_batch_approve(&owner, &empty),
        Err(Ok(ExtError::EmptyBatch))
    );
}

#[test]
fn test_batch_approve_negative_amount_fails() {
    let (env, client, owner) = setup(1_000);
    let s = Address::generate(&env);

    let approvals = Vec::from_array(
        &env,
        [Approval {
            spender: s.clone(),
            amount: -1,
            expiration_ledger: 100_000,
        }],
    );
    assert_eq!(
        client.try_batch_approve(&owner, &approvals),
        Err(Ok(ExtError::InvalidAmount))
    );
}

#[test]
fn test_batch_approve_revoke_with_zero() {
    let (env, client, owner) = setup(1_000);
    let s = Address::generate(&env);

    let set = Vec::from_array(
        &env,
        [Approval {
            spender: s.clone(),
            amount: 300,
            expiration_ledger: 100_000,
        }],
    );
    client.batch_approve(&owner, &set);
    assert_eq!(client.allowance(&owner, &s), 300);

    let revoke = Vec::from_array(
        &env,
        [Approval {
            spender: s.clone(),
            amount: 0,
            expiration_ledger: 0,
        }],
    );
    client.batch_approve(&owner, &revoke);
    assert_eq!(client.allowance(&owner, &s), 0);
}

// ---------------------------------------------------------------------------
// Standard transfer tests
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_basic() {
    let (env, client, admin) = setup(1_000);
    let bob = Address::generate(&env);

    client.transfer(&admin, &bob, &400);

    assert_eq!(client.balance(&admin), 600);
    assert_eq!(client.balance(&bob), 400);
}

#[test]
fn test_transfer_insufficient_balance_fails() {
    let (env, client, admin) = setup(100);
    let bob = Address::generate(&env);

    assert_eq!(
        client.try_transfer(&admin, &bob, &200),
        Err(Ok(ExtError::InsufficientBalance))
    );
}

// ---------------------------------------------------------------------------
// transfer_from / allowance tests
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_from_with_permit_allowance() {
    let (env, client, owner) = setup(1_000);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Grant allowance via permit
    client.permit(&owner, &spender, &300, &100_000);

    // Spender draws from the allowance
    client.transfer_from(&spender, &owner, &recipient, &200);

    assert_eq!(client.balance(&owner), 800);
    assert_eq!(client.balance(&recipient), 200);
    assert_eq!(client.allowance(&owner, &spender), 100);
}

#[test]
fn test_transfer_from_exceeds_allowance_fails() {
    let (env, client, owner) = setup(1_000);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.permit(&owner, &spender, &100, &100_000);

    assert_eq!(
        client.try_transfer_from(&spender, &owner, &recipient, &200),
        Err(Ok(ExtError::InsufficientAllowance))
    );
}

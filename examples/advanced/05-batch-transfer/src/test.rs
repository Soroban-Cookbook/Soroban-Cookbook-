extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(initial_supply: i128) -> (Env, BatchTransferContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BatchTransferContract);
    let client = BatchTransferContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    // Soroban SDK client unwraps Result<(), E> to () — no .unwrap() needed
    client.initialize(&admin, &initial_supply);
    (env, client, admin)
}

fn make_transfer(to: Address, amount: i128) -> Transfer {
    Transfer { to, amount }
}

// ---------------------------------------------------------------------------
// Initialisation tests
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
    assert_eq!(result, Err(Ok(BatchError::AlreadyInitialized)));
    let _ = env;
}

#[test]
fn test_initialize_zero_supply_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BatchTransferContract);
    let client = BatchTransferContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let result = client.try_initialize(&admin, &0);
    assert_eq!(result, Err(Ok(BatchError::InvalidAmount)));
}

#[test]
fn test_initialize_negative_supply_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BatchTransferContract);
    let client = BatchTransferContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let result = client.try_initialize(&admin, &-100);
    assert_eq!(result, Err(Ok(BatchError::InvalidAmount)));
}

// ---------------------------------------------------------------------------
// Batch transfer — happy paths
// ---------------------------------------------------------------------------

#[test]
fn test_single_recipient_batch() {
    let (env, client, admin) = setup(1_000);
    let recipient = Address::generate(&env);

    let transfers = Vec::from_array(&env, [make_transfer(recipient.clone(), 300)]);
    // Soroban client returns u32 directly (panics on error)
    let count = client.batch_transfer(&admin, &transfers);

    assert_eq!(count, 1);
    assert_eq!(client.balance(&admin), 700);
    assert_eq!(client.balance(&recipient), 300);
}

#[test]
fn test_multiple_recipients() {
    let (env, client, admin) = setup(1_000);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let r3 = Address::generate(&env);

    let transfers = Vec::from_array(
        &env,
        [
            make_transfer(r1.clone(), 100),
            make_transfer(r2.clone(), 200),
            make_transfer(r3.clone(), 300),
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
fn test_batch_to_same_recipient_twice_accumulates() {
    let (env, client, admin) = setup(1_000);
    let r = Address::generate(&env);

    // Two entries with the same recipient — both should be credited
    let transfers = Vec::from_array(
        &env,
        [make_transfer(r.clone(), 150), make_transfer(r.clone(), 250)],
    );
    client.batch_transfer(&admin, &transfers);

    assert_eq!(client.balance(&admin), 600);
    assert_eq!(client.balance(&r), 400);
}

#[test]
fn test_exact_balance_drains_to_zero() {
    let (env, client, admin) = setup(500);
    let r = Address::generate(&env);

    let transfers = Vec::from_array(&env, [make_transfer(r.clone(), 500)]);
    client.batch_transfer(&admin, &transfers);

    assert_eq!(client.balance(&admin), 0);
    assert_eq!(client.balance(&r), 500);
}

// ---------------------------------------------------------------------------
// Batch transfer — error paths
// ---------------------------------------------------------------------------

#[test]
fn test_batch_not_initialized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BatchTransferContract);
    let client = BatchTransferContractClient::new(&env, &contract_id);
    let from = Address::generate(&env);
    let r = Address::generate(&env);

    let transfers = Vec::from_array(&env, [make_transfer(r, 100)]);
    let result = client.try_batch_transfer(&from, &transfers);
    assert_eq!(result, Err(Ok(BatchError::NotInitialized)));
}

#[test]
fn test_empty_batch_fails() {
    let (env, client, admin) = setup(1_000);
    let empty: Vec<Transfer> = Vec::new(&env);
    let result = client.try_batch_transfer(&admin, &empty);
    assert_eq!(result, Err(Ok(BatchError::EmptyBatch)));
}

#[test]
fn test_zero_amount_in_batch_fails() {
    let (env, client, admin) = setup(1_000);
    let r = Address::generate(&env);

    let transfers = Vec::from_array(
        &env,
        [
            make_transfer(r.clone(), 100),
            make_transfer(r.clone(), 0), // invalid
        ],
    );
    let result = client.try_batch_transfer(&admin, &transfers);
    assert_eq!(result, Err(Ok(BatchError::InvalidAmount)));
    // State must be unchanged
    assert_eq!(client.balance(&admin), 1_000);
    assert_eq!(client.balance(&r), 0);
}

#[test]
fn test_negative_amount_in_batch_fails() {
    let (env, client, admin) = setup(1_000);
    let r = Address::generate(&env);

    let transfers = Vec::from_array(&env, [make_transfer(r.clone(), -50)]);
    let result = client.try_batch_transfer(&admin, &transfers);
    assert_eq!(result, Err(Ok(BatchError::InvalidAmount)));
    assert_eq!(client.balance(&admin), 1_000);
}

#[test]
fn test_insufficient_balance_fails() {
    let (env, client, admin) = setup(100);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);

    let transfers = Vec::from_array(
        &env,
        [
            make_transfer(r1.clone(), 70),
            make_transfer(r2.clone(), 50), // 70+50=120 > 100
        ],
    );
    let result = client.try_batch_transfer(&admin, &transfers);
    assert_eq!(result, Err(Ok(BatchError::InsufficientBalance)));

    // No state mutation on failure
    assert_eq!(client.balance(&admin), 100);
    assert_eq!(client.balance(&r1), 0);
    assert_eq!(client.balance(&r2), 0);
}

#[test]
fn test_batch_too_large_fails() {
    let supply: i128 = (MAX_BATCH_SIZE as i128 + 1) * 10;
    let (env, client, admin) = setup(supply);

    let mut transfers: Vec<Transfer> = Vec::new(&env);
    for _ in 0..=MAX_BATCH_SIZE {
        let r = Address::generate(&env);
        transfers.push_back(make_transfer(r, 10));
    }

    let result = client.try_batch_transfer(&admin, &transfers);
    assert_eq!(result, Err(Ok(BatchError::BatchTooLarge)));
}

// ---------------------------------------------------------------------------
// Authorization tests
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_batch_transfer_unauthorized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BatchTransferContract);
    let client = BatchTransferContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin, &1_000);

    // Strip auths so the batch call fails auth check
    env.set_auths(&[]);

    let r = Address::generate(&env);
    let transfers = Vec::from_array(&env, [make_transfer(r, 100)]);
    client.batch_transfer(&admin, &transfers);
}

// ---------------------------------------------------------------------------
// Gas optimisation: verify single read/write on sender balance
// (Behavioural proxy: correctness after N recipients stays consistent)
// ---------------------------------------------------------------------------

#[test]
fn test_large_valid_batch_correctness() {
    // 20 recipients keeps us well within Soroban's ledger-entry limits
    // (max 50 write entries; 1 sender + 20 recipients = 21 writes)
    let n: u32 = 20;
    let amount_each: i128 = 10;
    let supply = n as i128 * amount_each;

    let (env, client, admin) = setup(supply);

    let mut transfers: Vec<Transfer> = Vec::new(&env);
    let mut recipients: std::vec::Vec<Address> = std::vec::Vec::new();
    for _ in 0..n {
        let r = Address::generate(&env);
        recipients.push(r.clone());
        transfers.push_back(make_transfer(r, amount_each));
    }

    let count = client.batch_transfer(&admin, &transfers);
    assert_eq!(count, n);
    assert_eq!(client.balance(&admin), 0);
    for r in &recipients {
        assert_eq!(client.balance(r), amount_each);
    }
}

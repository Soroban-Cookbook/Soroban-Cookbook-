//! Fuzz / boundary tests for basic example contracts.
//!
//! Each test targets a specific edge case or boundary condition rather than
//! the happy path already covered in integration_tests.rs.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, IntoVal, Symbol, Vec};

// ---------------------------------------------------------------------------
// Storage — boundary values
// ---------------------------------------------------------------------------

#[test]
fn fuzz_storage_max_u64_persistent() {
    let env = Env::default();
    let id = env.register_contract(None, storage_patterns::StorageContract);
    let key = symbol_short!("maxkey");

    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [key.into_val(&env), u64::MAX.into_val(&env)]),
    );

    let val: Option<u64> = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(val, Some(u64::MAX));
}

#[test]
fn fuzz_storage_zero_u64_persistent() {
    let env = Env::default();
    let id = env.register_contract(None, storage_patterns::StorageContract);
    let key = symbol_short!("zerokey");

    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "set_persistent"),
        Vec::from_array(&env, [key.into_val(&env), 0u64.into_val(&env)]),
    );

    let val: Option<u64> = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(val, Some(0));
}

#[test]
fn fuzz_storage_overwrite_three_times() {
    let env = Env::default();
    let id = env.register_contract(None, storage_patterns::StorageContract);
    let key = symbol_short!("ovkey");

    for value in [100u64, 200u64, 300u64] {
        env.invoke_contract::<()>(
            &id,
            &Symbol::new(&env, "set_persistent"),
            Vec::from_array(&env, [key.into_val(&env), value.into_val(&env)]),
        );
    }

    let val: Option<u64> = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(val, Some(300));
}

#[test]
fn fuzz_storage_max_u64_temporary() {
    let env = Env::default();
    let id = env.register_contract(None, storage_patterns::StorageContract);
    let key = symbol_short!("tmaxkey");

    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "set_temporary"),
        Vec::from_array(&env, [key.into_val(&env), u64::MAX.into_val(&env)]),
    );

    let val: Option<u64> = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_temporary"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(val, Some(u64::MAX));
}

#[test]
fn fuzz_storage_missing_key_returns_none() {
    let env = Env::default();
    let id = env.register_contract(None, storage_patterns::StorageContract);
    let key = symbol_short!("absent");

    let val: Option<u64> = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert_eq!(val, None);

    let has: bool = env.invoke_contract(
        &id,
        &Symbol::new(&env, "has_persistent"),
        Vec::from_array(&env, [key.into_val(&env)]),
    );
    assert!(!has);
}

// ---------------------------------------------------------------------------
// Authentication — transfer boundary cases
// ---------------------------------------------------------------------------

#[test]
fn fuzz_auth_transfer_entire_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, authentication::AuthContract);
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);

    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                sender.clone().into_val(&env),
                100i128.into_val(&env),
            ],
        ),
    );

    env.invoke_contract::<()>(
        &id,
        &symbol_short!("transfer"),
        Vec::from_array(
            &env,
            [
                sender.clone().into_val(&env),
                receiver.clone().into_val(&env),
                100i128.into_val(&env),
            ],
        ),
    );

    let sender_bal: i128 = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [sender.into_val(&env)]),
    );
    let receiver_bal: i128 = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [receiver.into_val(&env)]),
    );

    assert_eq!(sender_bal, 0);
    assert_eq!(receiver_bal, 100);
}

#[test]
fn fuzz_auth_many_small_transfers() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, authentication::AuthContract);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "initialize"),
        Vec::from_array(&env, [admin.clone().into_val(&env)]),
    );
    env.invoke_contract::<()>(
        &id,
        &Symbol::new(&env, "set_balance"),
        Vec::from_array(
            &env,
            [
                admin.clone().into_val(&env),
                alice.clone().into_val(&env),
                50i128.into_val(&env),
            ],
        ),
    );

    for _ in 0..5 {
        env.invoke_contract::<()>(
            &id,
            &symbol_short!("transfer"),
            Vec::from_array(
                &env,
                [
                    alice.clone().into_val(&env),
                    bob.clone().into_val(&env),
                    1i128.into_val(&env),
                ],
            ),
        );
    }

    let alice_bal: i128 = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [alice.into_val(&env)]),
    );
    let bob_bal: i128 = env.invoke_contract(
        &id,
        &Symbol::new(&env, "get_balance"),
        Vec::from_array(&env, [bob.into_val(&env)]),
    );

    assert_eq!(alice_bal, 45);
    assert_eq!(bob_bal, 5);
}

// ---------------------------------------------------------------------------
// Error handling — deposit/withdraw boundary cases
// ---------------------------------------------------------------------------

#[test]
fn fuzz_error_handling_minimum_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, error_handling::ErrorDemoContract);
    let client = error_handling::ErrorDemoContractClient::new(&env, &id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin);

    let bal = client.deposit(&user, &1i128);
    assert_eq!(bal, 1);
    assert_eq!(client.balance(&user), 1);
}

#[test]
fn fuzz_error_handling_exact_withdraw_boundary() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, error_handling::ErrorDemoContract);
    let client = error_handling::ErrorDemoContractClient::new(&env, &id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin);

    client.deposit(&user, &500i128);
    client.withdraw(&user, &499i128);
    assert_eq!(client.balance(&user), 1);

    let final_bal = client.withdraw(&user, &1i128);
    assert_eq!(final_bal, 0);
    assert_eq!(client.balance(&user), 0);
}

#[test]
fn fuzz_error_handling_consecutive_deposits_accumulate() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, error_handling::ErrorDemoContract);
    let client = error_handling::ErrorDemoContractClient::new(&env, &id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin);

    client.deposit(&user, &100i128);
    client.deposit(&user, &200i128);
    client.deposit(&user, &300i128);

    assert_eq!(client.balance(&user), 600);
}

#[test]
fn fuzz_error_handling_overdraft_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, error_handling::ErrorDemoContract);
    let client = error_handling::ErrorDemoContractClient::new(&env, &id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin);

    client.deposit(&user, &100i128);

    let err = client.try_withdraw(&user, &101i128);
    assert_eq!(
        err,
        Err(Ok(error_handling::ContractError::InsufficientBalance))
    );

    assert_eq!(client.balance(&user), 100);
}

// ---------------------------------------------------------------------------
// Custom errors — input validation boundary cases
// ---------------------------------------------------------------------------

#[test]
fn fuzz_custom_errors_zero_input_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, custom_errors::CustomErrorsContract);
    let client = custom_errors::CustomErrorsContractClient::new(&env, &id);

    let err = client.try_validate_input(&0i64);
    assert_eq!(err, Err(Ok(custom_errors::ContractError::InvalidInput)));
}

#[test]
fn fuzz_custom_errors_one_accepted() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, custom_errors::CustomErrorsContract);
    let client = custom_errors::CustomErrorsContractClient::new(&env, &id);

    let result = client.try_validate_input(&1i64);
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Events counter — high increment count
// ---------------------------------------------------------------------------

#[test]
fn fuzz_events_high_increment_count() {
    let env = Env::default();

    let id = env.register_contract(None, events_structured::EventsContract);

    for _ in 0..20 {
        env.invoke_contract::<()>(&id, &symbol_short!("increment"), Vec::new(&env));
    }

    let count: u32 = env.invoke_contract(&id, &Symbol::new(&env, "get_number"), Vec::new(&env));
    assert_eq!(count, 20);
}

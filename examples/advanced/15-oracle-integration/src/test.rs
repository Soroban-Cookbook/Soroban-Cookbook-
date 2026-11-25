#![cfg(test)]
#![allow(deprecated)]

extern crate std;

use super::*;
<<<<<<< HEAD:examples/advanced/15-oracle-integration/src/test.rs
use soroban_sdk::testutils::{Address as _, Events as _, Ledger};
use soroban_sdk::{symbol_short, Address, Env, Event};
=======
use soroban_sdk::testutils::{Address as _, Ledger, Events};
use soroban_sdk::{symbol_short, Address, Env, Symbol, TryFromVal, Val};
>>>>>>> 8c99210 (Create Pattern Selection ADRs):examples/advanced/04-oracle-integration/src/test.rs

fn setup() -> (
    Env,
    Address,
    Address,
    OracleContractClient<'static>,
    ConsumerContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let oracle_id = env.register(OracleContract, ());
    let oracle_client = OracleContractClient::new(&env, &oracle_id);

    let consumer_id = env.register(ConsumerContract, ());
    let consumer_client = ConsumerContractClient::new(&env, &consumer_id);

    let admin = Address::generate(&env);
    let updater = Address::generate(&env);
    let max_age = 300u64; // 5 minutes
    let timeout = 3600u64; // 1 hour

    oracle_client.initialize(&admin, &updater, &max_age, &timeout);
    consumer_client.initialize(&admin, &oracle_id);

    (env, admin, updater, oracle_client, consumer_client)
}

/// Helper to safely search for an event with matching contract address and first topic symbol.
fn has_event(env: &Env, contract: &Address, topic: &str) -> bool {
    let binding = env.events().all();
    let all_events = binding.events();
    for event in all_events.iter() {
        if let Some(hash) = &event.contract_id {
            if let Ok(addr) = Address::try_from_val(env, &soroban_sdk::xdr::ScAddress::Contract(hash.clone())) {
                std::println!("has_event: comparing addr={:?} with contract={:?}", addr, contract);
                if addr == *contract {
                    let soroban_sdk::xdr::ContractEventBody::V0(v0) = &event.body;
                    if let Some(first_topic) = v0.topics.get(0) {
                        if let Ok(val) = Val::try_from_val(env, first_topic) {
                            std::println!("has_event: val converted ok");
                            match Symbol::try_from_val(env, &val) {
                                Ok(sym) => {
                                    std::println!("has_event: sym converted ok: {:?}", sym);
                                    if sym == Symbol::new(env, topic) {
                                        return true;
                                    } else {
                                        std::println!("has_event: sym mismatch: expected {}, got {:?}", topic, sym);
                                    }
                                }
                                Err(e) => {
                                    std::println!("has_event: sym conversion failed: {:?}", e);
                                }
                            }
                        } else {
                            std::println!("has_event: val conversion failed");
                        }
                    }
                }
            }
        }
    }
    false
}

// ── 1. Initialization ────────────────────────────────────────────────────────

#[test]
fn test_initialize_success() {
    let (_env, _admin, _updater, _oracle, _consumer) = setup();
}

#[test]
fn test_initialize_twice_fails() {
    let (env, admin, updater, oracle, _consumer) = setup();
    let result = oracle.try_initialize(&admin, &updater, &300, &3600);
    assert_eq!(result, Err(Ok(OracleError::AlreadyInitialized)));
    let _ = env;
}

#[test]
#[should_panic]
fn test_initialize_rejects_unauthorized_caller() {
    let env = Env::default();
    // No auths mocked: the admin's `require_auth` must fail, so an attacker
    // cannot front-run deployment and point the consumer at their own oracle
    // (issue #1087).
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let consumer_id = env.register(ConsumerContract, ());
    let consumer_client = ConsumerContractClient::new(&env, &consumer_id);
    consumer_client.initialize(&admin, &oracle);
}

// ── 2. Request Data ──────────────────────────────────────────────────────────

#[test]
fn test_request_data_success() {
    let (env, _admin, _updater, oracle, consumer) = setup();
    let query = symbol_short!("BTC_USD");

    // Request price via consumer
    let req_id1 = consumer.request_price(&query);
    assert_eq!(req_id1, 1);

    // Request second time, ID should increment
    let req_id2 = consumer.request_price(&query);
    assert_eq!(req_id2, 2);

<<<<<<< HEAD:examples/advanced/15-oracle-integration/src/test.rs
    // Print all events in the log
    let binding = env.events().all();
    let all_events = binding.events();
    for (i, event) in all_events.iter().enumerate() {
        std::println!(
            "DEBUG EVENT {}: contract={:?}, body={:?}",
            i,
            event.contract_id,
            event.body
        );
    }
    std::println!("DEBUG oracle.address = {:?}", oracle.address);

=======
>>>>>>> 8c99210 (Create Pattern Selection ADRs):examples/advanced/04-oracle-integration/src/test.rs
    // Check storage state on Oracle
    let req1 = oracle.get_request(&1).unwrap();
    assert_eq!(req1.id, 1);
    assert_eq!(req1.query, query);
    assert_eq!(req1.status, RequestStatus::Pending);

    // Verify event emission from Oracle request using has_event
    assert!(has_event(&env, &oracle.address, "oracle_request_event"));
}

#[test]
fn test_request_data_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let oracle_id = env.register(OracleContract, ());
    let oracle = OracleContractClient::new(&env, &oracle_id);
    let consumer = Address::generate(&env);

    let result =
        oracle.try_request_data(&consumer, &symbol_short!("callback"), &symbol_short!("BTC"));
    assert_eq!(result, Err(Ok(OracleError::NotInitialized)));
}

// ── 3. Fulfillment ───────────────────────────────────────────────────────────

#[test]
fn test_fulfill_request_success() {
    let (env, _admin, updater, oracle, consumer) = setup();
    let query = symbol_short!("ETH_USD");

    // Create request
    let req_id = consumer.request_price(&query);
    let timestamp = env.ledger().timestamp();
    let value = 1800_0000000i128; // $1800

    // Fulfill request
    let res = oracle.try_fulfill_request(&updater, &req_id, &value, &timestamp);
    assert!(res.is_ok());

    // Verify consumer received and updated the price feed state
    assert_eq!(consumer.get_price(&query), Some(value));
    assert_eq!(consumer.get_timestamp(&query), Some(timestamp));

    // Verify request is marked as Fulfilled
    let req = oracle.get_request(&req_id).unwrap();
    assert_eq!(req.status, RequestStatus::Fulfilled);

    // Verify event emissions using has_event
    assert!(has_event(&env, &oracle.address, "oracle_fulfill_event"));
    assert!(has_event(&env, &consumer.address, "consumer_update_event"));
}

#[test]
fn test_fulfill_request_unauthorized() {
    let (env, _admin, _updater, oracle, consumer) = setup();
    let query = symbol_short!("ETH_USD");

    let req_id = consumer.request_price(&query);
    let stranger = Address::generate(&env);
    let timestamp = env.ledger().timestamp();

    // Call from unauthorized address
    let result = oracle.try_fulfill_request(&stranger, &req_id, &100i128, &timestamp);
    assert_eq!(result, Err(Ok(OracleError::NotAuthorized)));
}

#[test]
fn test_fulfill_request_not_found() {
    let (env, _admin, updater, oracle, _consumer) = setup();
    let timestamp = env.ledger().timestamp();

    // ID 999 does not exist
    let result = oracle.try_fulfill_request(&updater, &999u32, &100i128, &timestamp);
    assert_eq!(result, Err(Ok(OracleError::RequestNotFound)));
}

#[test]
fn test_fulfill_request_expired() {
    let (env, _admin, updater, oracle, consumer) = setup();
    let query = symbol_short!("XLM_USD");

    let req_id = consumer.request_price(&query);

    // Advance time past timeout (3600s + 1s)
    env.ledger().with_mut(|l| l.timestamp += 3601);

    let timestamp = env.ledger().timestamp();
    let result = oracle.try_fulfill_request(&updater, &req_id, &100i128, &timestamp);
    assert_eq!(result, Err(Ok(OracleError::RequestExpired)));

    // Note: Request status remains Pending in contract persistent storage because 
    // the transaction failed and reverted state changes back to before the call.
    let req = oracle.get_request(&req_id).unwrap();
    assert_eq!(req.status, RequestStatus::Pending);
}

#[test]
fn test_fulfill_request_invalid_value() {
    let (env, _admin, updater, oracle, consumer) = setup();
    let query = symbol_short!("XLM_USD");

    let req_id = consumer.request_price(&query);
    let timestamp = env.ledger().timestamp();

    // Value 0 is invalid
    let result = oracle.try_fulfill_request(&updater, &req_id, &0i128, &timestamp);
    assert_eq!(result, Err(Ok(OracleError::InvalidValue)));

    // Negative value is invalid
    let result_neg = oracle.try_fulfill_request(&updater, &req_id, &-100i128, &timestamp);
    assert_eq!(result_neg, Err(Ok(OracleError::InvalidValue)));
}

#[test]
fn test_fulfill_request_stale_timestamp() {
    let (env, _admin, updater, oracle, consumer) = setup();
    let query = symbol_short!("XLM_USD");

    let req_id = consumer.request_price(&query);

    // Advance time by 301 seconds
    env.ledger().with_mut(|l| l.timestamp += 301);

    // Submit timestamp from before the delay (301 seconds old, which is > max_age = 300)
    let stale_timestamp = env.ledger().timestamp() - 301;
    let result = oracle.try_fulfill_request(&updater, &req_id, &100i128, &stale_timestamp);
    assert_eq!(result, Err(Ok(OracleError::InvalidTimestamp)));
}

#[test]
fn test_fulfill_request_future_timestamp() {
    let (env, _admin, updater, oracle, consumer) = setup();
    let query = symbol_short!("XLM_USD");

    let req_id = consumer.request_price(&query);
    // Submitting a future timestamp
    let future_timestamp = env.ledger().timestamp() + 10;
    let result = oracle.try_fulfill_request(&updater, &req_id, &100i128, &future_timestamp);
    assert_eq!(result, Err(Ok(OracleError::InvalidTimestamp)));
}

#[test]
fn test_fulfill_request_older_than_request_timestamp() {
    let (env, _admin, updater, oracle, consumer) = setup();
    let query = symbol_short!("XLM_USD");

    // Advance time to 1000
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let req_id = consumer.request_price(&query);

    // Response timestamp (999) is older than request timestamp (1000)
    let old_timestamp = 999u64;
    let result = oracle.try_fulfill_request(&updater, &req_id, &100i128, &old_timestamp);
    assert_eq!(result, Err(Ok(OracleError::InvalidTimestamp)));
}

#[test]
fn test_double_fulfillment_fails() {
    let (env, _admin, updater, oracle, consumer) = setup();
    let query = symbol_short!("ETH_USD");

    let req_id = consumer.request_price(&query);
    let timestamp = env.ledger().timestamp();
    let value = 1800_0000000i128;

    // First fulfillment succeeds
    let res = oracle.try_fulfill_request(&updater, &req_id, &value, &timestamp);
    assert!(res.is_ok());

    // Second fulfillment of the same request ID fails with RequestNotFound
    let result = oracle.try_fulfill_request(&updater, &req_id, &value, &timestamp);
    assert_eq!(result, Err(Ok(OracleError::RequestNotFound)));
}

// ── 4. Consumer / Security checks ───────────────────────────────────────────

#[test]
#[should_panic]
fn test_consumer_rejects_unauthorized_callback() {
    let env = Env::default();
    // `initialize` now requires the admin's auth, so mock it for setup only...
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let consumer_id = env.register(ConsumerContract, ());
    let consumer_client = ConsumerContractClient::new(&env, &consumer_id);
    consumer_client.initialize(&admin, &oracle);
    // ...then clear all auths so the callback's oracle.require_auth() fails.
    env.mock_auths(&[]);

    // Call callback directly - this should panic due to lack of authorization from the oracle address
    consumer_client.callback(&1, &100, &100);
}

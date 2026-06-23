extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

fn bytes(env: &Env, s: &[u8]) -> Bytes {
    Bytes::from_slice(env, s)
}

// ---------- Bounded queue tests ----------

#[test]
fn bounded_fifo_push_pop() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BoundedQueueContract);
    let client = BoundedQueueContractClient::new(&env, &contract_id);
    client.initialize(&3_i128, &DropPolicy::DropNewest);

    let a = bytes(&env, b"a");
    let b = bytes(&env, b"b");
    let c = bytes(&env, b"c");

    client.push(&a);
    client.push(&b);
    client.push(&c);

    assert_eq!(client.len(), 3_i128);
    assert_eq!(client.pop(), a);
    assert_eq!(client.pop(), b);
    assert_eq!(client.pop(), c);
}

#[test]
fn bounded_drop_oldest_on_full() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BoundedQueueContract);
    let client = BoundedQueueContractClient::new(&env, &contract_id);
    client.initialize(&2_i128, &DropPolicy::DropOldest);

    let a = bytes(&env, b"a");
    let b = bytes(&env, b"b");
    let c = bytes(&env, b"c");

    client.push(&a);
    client.push(&b);
    // now full; pushing c should drop oldest (a)
    client.push(&c);
    assert_eq!(client.len(), 2_i128);
    assert_eq!(client.pop(), b);
    assert_eq!(client.pop(), c);
}

#[test]
#[should_panic(expected = "Queue full")]
fn bounded_drop_newest_rejects() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BoundedQueueContract);
    let client = BoundedQueueContractClient::new(&env, &contract_id);
    client.initialize(&2_i128, &DropPolicy::DropNewest);

    let a = bytes(&env, b"a");
    let b = bytes(&env, b"b");
    let c = bytes(&env, b"c");

    client.push(&a);
    client.push(&b);
    // should panic
    client.push(&c);
}

#[test]
#[should_panic(expected = "Empty")]
fn bounded_pop_empty_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BoundedQueueContract);
    let client = BoundedQueueContractClient::new(&env, &contract_id);
    client.initialize(&1_i128, &DropPolicy::DropNewest);
    client.pop();
}

// ---------- Circular buffer tests ----------

#[test]
fn circular_overwrite_behavior() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CircularBufferContract);
    let client = CircularBufferContractClient::new(&env, &contract_id);
    client.initialize(&2_i128);

    let a = bytes(&env, b"a");
    let b = bytes(&env, b"b");
    let c = bytes(&env, b"c");

    client.push(&a);
    client.push(&b);
    // overwrite oldest (a)
    client.push(&c);

    assert_eq!(client.len(), 2_i128);
    assert_eq!(client.pop(), b);
    assert_eq!(client.pop(), c);
}

#[test]
#[should_panic(expected = "Empty")]
fn circular_pop_empty_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CircularBufferContract);
    let client = CircularBufferContractClient::new(&env, &contract_id);
    client.initialize(&3_i128);
    client.pop();
}

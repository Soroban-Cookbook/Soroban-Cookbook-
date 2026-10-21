#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::{symbol_short, Env};

fn setup() -> (Env, QueueContractClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(QueueContract, ());
    let client = QueueContractClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_empty_queue() {
    let (_env, client) = setup();

    assert!(client.is_empty());
    assert_eq!(client.size(), 0);
}

#[test]
fn test_single_item() {
    let (_env, client) = setup();

    client.enqueue(&symbol_short!("A"));
    assert!(!client.is_empty());
    assert_eq!(client.size(), 1);
    assert_eq!(client.peek(), symbol_short!("A"));
    assert_eq!(client.dequeue(), symbol_short!("A"));
    assert!(client.is_empty());
}

#[test]
fn test_fifo_order() {
    let (_env, client) = setup();

    client.enqueue(&symbol_short!("A"));
    client.enqueue(&symbol_short!("B"));
    client.enqueue(&symbol_short!("C"));

    assert_eq!(client.size(), 3);
    assert_eq!(client.dequeue(), symbol_short!("A"));
    assert_eq!(client.dequeue(), symbol_short!("B"));
    assert_eq!(client.dequeue(), symbol_short!("C"));
}

#[test]
fn test_peek_preserves_queue() {
    let (_env, client) = setup();

    client.enqueue(&symbol_short!("X"));
    assert_eq!(client.peek(), symbol_short!("X"));
    assert_eq!(client.size(), 1);
    assert_eq!(client.peek(), symbol_short!("X"));
    assert_eq!(client.dequeue(), symbol_short!("X"));
}

#[test]
#[should_panic(expected = "Queue is empty")]
fn test_dequeue_empty_panics() {
    let (_env, client) = setup();
    client.dequeue();
}

#[test]
#[should_panic(expected = "Queue is empty")]
fn test_peek_empty_panics() {
    let (_env, client) = setup();
    client.peek();
}

#[test]
fn test_large_queue() {
    let (_env, client) = setup();

    let items = [
        symbol_short!("q0"),
        symbol_short!("q1"),
        symbol_short!("q2"),
        symbol_short!("q3"),
        symbol_short!("q4"),
        symbol_short!("q5"),
        symbol_short!("q6"),
        symbol_short!("q7"),
        symbol_short!("q8"),
        symbol_short!("q9"),
    ];

    for item in &items {
        client.enqueue(item);
    }

    assert_eq!(client.size(), 10);

    for item in &items {
        assert_eq!(client.dequeue(), *item);
    }

    assert!(client.is_empty());
}

#[test]
#[should_panic(expected = "Queue is full")]
fn test_overflow() {
    let (_env, client) = setup();

    for _ in 0..=MAX_QUEUE_SIZE {
        client.enqueue(&symbol_short!("item"));
    }
}

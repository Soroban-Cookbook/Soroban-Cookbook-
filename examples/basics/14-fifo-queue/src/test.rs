#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, Env};

#[test]
fn test_empty_queue() {
    let env = Env::default();
    let contract = QueueContract {};

    assert!(QueueContract::is_empty(&env));
    assert_eq!(QueueContract::size(&env), 0);
}

#[test]
fn test_single_item() {
    let env = Env::default();
    let contract = QueueContract {};

    QueueContract::enqueue(&env, symbol_short!("A"));
    assert!(!QueueContract::is_empty(&env));
    assert_eq!(QueueContract::size(&env), 1);
    assert_eq!(QueueContract::peek(&env), symbol_short!("A"));
    assert_eq!(QueueContract::dequeue(&env), symbol_short!("A"));
    assert!(QueueContract::is_empty(&env));
}

#[test]
fn test_fifo_order() {
    let env = Env::default();
    let contract = QueueContract {};

    QueueContract::enqueue(&env, symbol_short!("A"));
    QueueContract::enqueue(&env, symbol_short!("B"));
    QueueContract::enqueue(&env, symbol_short!("C"));

    assert_eq!(QueueContract::size(&env), 3);
    assert_eq!(QueueContract::dequeue(&env), symbol_short!("A"));
    assert_eq!(QueueContract::dequeue(&env), symbol_short!("B"));
    assert_eq!(QueueContract::dequeue(&env), symbol_short!("C"));
}

#[test]
fn test_peek_preserves_queue() {
    let env = Env::default();
    let contract = QueueContract {};

    QueueContract::enqueue(&env, symbol_short!("X"));
    assert_eq!(QueueContract::peek(&env), symbol_short!("X"));
    assert_eq!(QueueContract::size(&env), 1);
    assert_eq!(QueueContract::peek(&env), symbol_short!("X"));
    assert_eq!(QueueContract::dequeue(&env), symbol_short!("X"));
}

#[test]
#[should_panic(expected = "Queue is empty")]
fn test_dequeue_empty_panics() {
    let env = Env::default();
    let contract = QueueContract {};

    QueueContract::dequeue(&env);
}

#[test]
#[should_panic(expected = "Queue is empty")]
fn test_peek_empty_panics() {
    let env = Env::default();
    let contract = QueueContract {};

    QueueContract::peek(&env);
}

#[test]
fn test_large_queue() {
    let env = Env::default();
    let contract = QueueContract {};

    // Enqueue many items
    for i in 0..100 {
        let symbol = Symbol::short(format!("item{}", i).as_str());
        QueueContract::enqueue(&env, symbol);
    }

    assert_eq!(QueueContract::size(&env), 100);

    // Dequeue and verify FIFO order
    for i in 0..100 {
        let symbol = Symbol::short(format!("item{}", i).as_str());
        assert_eq!(QueueContract::dequeue(&env), symbol);
    }

    assert!(QueueContract::is_empty(&env));
}

#[test]
#[should_panic(expected = "Queue is full")]
fn test_overflow() {
    let env = Env::default();
    let contract = QueueContract {};

    // Try to exceed max queue size
    for i in 0..=super::MAX_QUEUE_SIZE {
        let symbol = Symbol::short(format!("item{}", i).as_str());
        QueueContract::enqueue(&env, symbol);
    }
}

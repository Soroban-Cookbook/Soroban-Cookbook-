//! # FIFO Queue Contract
//!
//! This contract demonstrates a First-In-First-Out (FIFO) queue implementation
//! for ordering tasks or events on-chain. It shows how to efficiently manage
//! a queue using head/tail indices.

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const MAX_QUEUE_SIZE: u32 = 10000;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
enum DataKey {
    Head = 0,
    Tail = 1,
    Item = 2,
}

#[contract]
pub struct QueueContract;

#[contractimpl]
impl QueueContract {
    /// Enqueue an item to the back of the queue.
    ///
    /// # Arguments
    /// * `env` - the execution environment
    /// * `item` - the Symbol item to enqueue
    ///
    /// # Panics
    /// Panics if the queue is full (exceeds MAX_QUEUE_SIZE).
    pub fn enqueue(env: Env, item: Symbol) {
        let tail: u32 = env.storage().persistent().get(&DataKey::Tail).unwrap_or(0);

        let head: u32 = env.storage().persistent().get(&DataKey::Head).unwrap_or(0);

        // Check for overflow
        if tail - head >= MAX_QUEUE_SIZE {
            panic!("Queue is full");
        }

        // Store the item with tail index as part of the key
        env.storage()
            .persistent()
            .set(&(DataKey::Item, tail), &item);

        // Update tail
        env.storage().persistent().set(&DataKey::Tail, &(tail + 1));
    }

    /// Dequeue and return the item at the front of the queue.
    ///
    /// # Returns
    /// The item at the front of the queue
    ///
    /// # Panics
    /// Panics if the queue is empty.
    pub fn dequeue(env: Env) -> Symbol {
        let head: u32 = env.storage().persistent().get(&DataKey::Head).unwrap_or(0);

        let tail: u32 = env.storage().persistent().get(&DataKey::Tail).unwrap_or(0);

        // Check for empty queue
        if head >= tail {
            panic!("Queue is empty");
        }

        // Get the item at head
        let item: Symbol = env
            .storage()
            .persistent()
            .get(&(DataKey::Item, head))
            .unwrap_or_else(|| panic!("Item not found at head"));

        // Update head
        env.storage().persistent().set(&DataKey::Head, &(head + 1));

        item
    }

    /// Peek at the item at the front of the queue without removing it.
    ///
    /// # Returns
    /// The item at the front of the queue
    ///
    /// # Panics
    /// Panics if the queue is empty.
    pub fn peek(env: Env) -> Symbol {
        let head: u32 = env.storage().persistent().get(&DataKey::Head).unwrap_or(0);

        let tail: u32 = env.storage().persistent().get(&DataKey::Tail).unwrap_or(0);

        if head >= tail {
            panic!("Queue is empty");
        }

        env.storage()
            .persistent()
            .get(&(DataKey::Item, head))
            .unwrap_or_else(|| panic!("Item not found at head"))
    }

    /// Get the number of items currently in the queue.
    ///
    /// # Returns
    /// The size of the queue
    pub fn size(env: Env) -> u32 {
        let head: u32 = env.storage().persistent().get(&DataKey::Head).unwrap_or(0);

        let tail: u32 = env.storage().persistent().get(&DataKey::Tail).unwrap_or(0);

        tail.saturating_sub(head)
    }

    /// Check if the queue is empty.
    ///
    /// # Returns
    /// true if the queue is empty, false otherwise
    pub fn is_empty(env: Env) -> bool {
        Self::size(env) == 0
    }
}

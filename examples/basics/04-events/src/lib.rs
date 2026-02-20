//! # Events Contract
//!
//! Demonstrates Soroban event emission:
//! - Event structure: topics (up to 4) + data payload
//! - Deterministic event emission for testing
//! - Multiple event types with distinct topics
//!
//! Events are published via `env.events().publish()` and can be
//! queried off-chain for indexing and monitoring.

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

/// Event-emitting contract with deterministic topics and data.
#[contract]
pub struct EventsContract;

#[contractimpl]
impl EventsContract {
    /// Emits a simple event with topic ("simple") and data value.
    pub fn emit_simple(env: Env, value: u64) {
        env.events().publish((symbol_short!("simple"),), value);
    }

    /// Emits a tagged event with topic ("tagged", tag) and data value.
    pub fn emit_tagged(env: Env, tag: Symbol, value: u64) {
        env.events().publish((symbol_short!("tagged"), tag), value);
    }

    /// Emits multiple events (count) with topic ("multi", i) and data i.
    pub fn emit_multiple(env: Env, count: u32) {
        for i in 0..count {
            env.events().publish((symbol_short!("multi"), i), i as u64);
        }
    }
}

mod test;

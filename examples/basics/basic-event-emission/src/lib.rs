//! # Basic Event Emission
//!
//! Demonstrates how to emit events from a Soroban smart contract using
//! `env.events().publish()`.
//!
//! ## Event Anatomy
//!
//! ```text
//! env.events().publish(
//!     (topic_1, topic_2),  // tuple of topics — up to 4, used for off-chain filtering
//!     data_payload,        // arbitrary value attached as the event body
//! );
//! ```
//!
//! - **Topics** are short, filterable identifiers (e.g. `Symbol`, `Address`).
//! - **Data** is the richer payload read after matching on topics.
//!
//! ## What This Example Shows
//!
//! | Acceptance criterion          | Where                          |
//! |-------------------------------|--------------------------------|
//! | `env.events().publish()` usage | every function below           |
//! | Simple event topics           | single-symbol and two-symbol tuples |
//! | Event data payload            | `u32` value attached to each event |

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct EventContract;

#[contractimpl]
impl EventContract {
    /// Emit a `("set", value)` event whenever the stored value changes.
    ///
    /// Topic:  `symbol_short!("set")`
    /// Data:   the new `u32` value
    pub fn set(env: Env, value: u32) {
        env.storage().instance().set(&symbol_short!("val"), &value);

        // Single-topic event — simplest possible publish call.
        env.events().publish((symbol_short!("set"),), value);
    }

    /// Emit a `("counter", "inc")` event after incrementing the stored counter.
    ///
    /// Topics: `symbol_short!("counter")`, `symbol_short!("inc")`
    /// Data:   the updated `u32` counter value
    pub fn increment(env: Env) {
        let mut val: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("val"))
            .unwrap_or(0);

        val += 1;
        env.storage().instance().set(&symbol_short!("val"), &val);

        // Two-topic event — namespace + action pattern.
        env.events()
            .publish((symbol_short!("counter"), symbol_short!("inc")), val);
    }

    /// Return the current stored value.
    pub fn get(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&symbol_short!("val"))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;

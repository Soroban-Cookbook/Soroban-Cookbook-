//! # Structured Event Patterns
//!
//! Demonstrates how to emit well-structured events in Soroban contracts using:
//!
//! - **Custom event types** – `#[contracttype]` enums/structs as event data
//! - **Multiple topics** – up to 4 topic slots (contract address consumes none)
//! - **Indexed parameters** – placing searchable fields in topics, payload in data
//! - **Naming conventions** – `(contract_name, action)` as the first two topics
//!
//! ## Soroban Event Anatomy
//!
//! ```text
//! env.events().publish(
//!     (topic_1, topic_2, topic_3, topic_4),  // up to 4 topics; indexed for off-chain search
//!     data_payload,                           // arbitrary SCVal; not indexed
//! );
//! ```
//!
//! **Topics** should contain discrete, filterable identifiers (contract name,
//! action type, primary key, secondary key).  **Data** holds the rich payload
//! that off-chain consumers decode after matching on topics.
//!
//! ## Event Naming Convention
//!
//! Adopt a consistent `(contract, action, [key...])` topic layout so that
//! indexers and monitoring tools can build efficient filters:
//!
//! | Topic slot | Purpose            | Example              |
//! |------------|--------------------|----------------------|
//! | 0          | Contract namespace | `"events"`           |
//! | 1          | Action name        | `"transfer"`         |
//! | 2          | Primary index      | `sender: Address`    |
//! | 3          | Secondary index    | `recipient: Address` |

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Custom event payload types
// ---------------------------------------------------------------------------

/// Payload for a token-transfer event.
///
/// This struct is annotated with `#[contracttype]` so it can be serialised
/// as an `SCVal` and attached to the event's data slot.
#[contracttype]
pub struct TransferEventData {
    /// Number of units moved.
    pub amount: i128,
    /// Optional memo / reference identifier (0 = none).
    pub memo: u64,
}

/// Payload for a contract-configuration event.
///
/// Records an old and new value so off-chain consumers can compute diffs.
#[contracttype]
pub struct ConfigUpdateEventData {
    /// Previous configuration value.
    pub old_value: u64,
    /// Newly applied configuration value.
    pub new_value: u64,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// Namespace symbol used as the first topic of every event this contract emits.
///
/// Using a shared namespace lets indexers filter all events from this contract
/// with a single topic prefix.
const CONTRACT_NS: Symbol = symbol_short!("events");

/// Contract demonstrating structured, multi-topic event patterns.
#[contract]
pub struct EventsContract;

#[contractimpl]
impl EventsContract {
    // -----------------------------------------------------------------------
    // Example 1 – Transfer event (4 topics + structured data)
    // -----------------------------------------------------------------------

    /// Emit a token-transfer event.
    ///
    /// **Topic layout (4 topics):**
    ///
    /// | Index | Value                | Role               |
    /// |-------|----------------------|--------------------|
    /// | 0     | `"events"`           | Contract namespace |
    /// | 1     | `"transfer"`         | Action name        |
    /// | 2     | `sender: Address`    | Indexed sender     |
    /// | 3     | `recipient: Address` | Indexed recipient  |
    ///
    /// **Data:** [`TransferEventData`] `{ amount, memo }`
    ///
    /// Placing both addresses in topics means an off-chain indexer can
    /// efficiently retrieve all transfers _to_ or _from_ a given address.
    pub fn transfer(env: Env, sender: Address, recipient: Address, amount: i128, memo: u64) {
        // All four topic slots used: namespace · action · sender · recipient
        env.events().publish(
            (CONTRACT_NS, symbol_short!("transfer"), sender, recipient),
            TransferEventData { amount, memo },
        );
    }

    // -----------------------------------------------------------------------
    // Example 2 – Configuration-update event (3 topics + structured data)
    // -----------------------------------------------------------------------

    /// Emit a configuration-update event.
    ///
    /// **Topic layout (3 topics):**
    ///
    /// | Index | Value          | Role               |
    /// |-------|----------------|--------------------|
    /// | 0     | `"events"`     | Contract namespace |
    /// | 1     | `"cfg_upd"`    | Action name        |
    /// | 2     | `key: Symbol`  | Indexed config key |
    ///
    /// **Data:** [`ConfigUpdateEventData`] `{ old_value, new_value }`
    ///
    /// The config `key` is in the topics so consumers can subscribe to changes
    /// for a specific parameter (e.g. only `"max_supply"` updates).
    pub fn update_config(env: Env, key: Symbol, old_value: u64, new_value: u64) {
        env.events().publish(
            (CONTRACT_NS, symbol_short!("cfg_upd"), key),
            ConfigUpdateEventData { old_value, new_value },
        );
    }

    // -----------------------------------------------------------------------
    // Preserved simple helpers (kept for backward-compatibility)
    // -----------------------------------------------------------------------

    /// Emit a simple one-topic event – demonstrates the minimal event form.
    pub fn emit_simple(env: Env, value: u64) {
        env.events().publish((symbol_short!("simple"),), value);
    }

    /// Emit a tagged two-topic event.
    pub fn emit_tagged(env: Env, tag: Symbol, value: u64) {
        env.events().publish((symbol_short!("tagged"), tag), value);
    }

    /// Emit `count` indexed events – demonstrates a loop emission pattern.
    pub fn emit_multiple(env: Env, count: u32) {
        for i in 0..count {
            env.events()
                .publish((symbol_short!("multi"), i), i as u64);
        }
    }
}

mod test;

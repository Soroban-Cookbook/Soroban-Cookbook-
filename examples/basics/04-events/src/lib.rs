//! # Structured Event Patterns
//!
//! Demonstrates Soroban event emission and query-friendly topic design:
//!
//! ## Basics
//! - Event structure: topics (up to 4) + data payload
//! - Deterministic event emission for testing
//! - Multiple event types with distinct topics
//!
//! ## Query-Friendly Design Patterns
//! Off-chain indexers (e.g., Stellar Horizon, custom listeners) filter events
//! by topic position. Designing topics intentionally lets callers narrow results
//! without scanning every event.
//!
//! ### Topic Layout Convention
//! ```text
//! topic[0]  — event category / action name  (always present, used as primary filter)
//! topic[1]  — primary entity (from-address, contract-id, pool-id …)
//! topic[2]  — secondary entity (to-address, token-id …)        [optional]
//! topic[3]  — sub-type or status                               [optional]
//! data      — non-indexed payload (amounts, metadata, structs)
//! ```
//!
//! ### Best Practices
//! - Put the most-commonly filtered field in the earliest topic position.
//! - Keep topics to `Symbol` / `Address` / small integers — they must be
//!   `Val`-serialisable and live inside the 4-topic limit.
//! - Reserve the data payload for values that are *read* after filtering but
//!   not used to filter (amounts, timestamps, raw bytes).
//! - Use a consistent first-topic naming scheme across all events in a contract
//!   so indexers can discover every event type from a single contract.
//!
//! Events are published via `env.events().publish()` and can be
//! queried off-chain for indexing and monitoring.

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

/// Event-emitting contract demonstrating both basic emission and
/// query-friendly topic design.
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
    // ==================== BASIC EMISSION ====================

    /// Emits a simple event with topic ("simple",) and data value.
    ///
    /// Off-chain query: filter topic[0] == "simple"
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
            ConfigUpdateEventData {
                old_value,
                new_value,
            },
        );
    }

    // -----------------------------------------------------------------------
    // Preserved simple helpers (kept for backward-compatibility)
    // -----------------------------------------------------------------------

    /// Emit a simple one-topic event – demonstrates the minimal event form.
    pub fn emit_simple(env: Env, value: u64) {
        env.events().publish((symbol_short!("simple"),), value);
    }

    /// Emits a tagged event with topics ("tagged", tag) and data value.
    ///
    /// Off-chain query: filter topic[0] == "tagged" AND topic[1] == <tag>
    /// Emit a tagged two-topic event.
    pub fn emit_tagged(env: Env, tag: Symbol, value: u64) {
        env.events().publish((symbol_short!("tagged"), tag), value);
    }

    /// Emits `count` events each with topics ("multi", index) and data index.
    ///
    /// Demonstrates sequential event emission within a single invocation.
    /// Emit `count` indexed events – demonstrates a loop emission pattern.
    pub fn emit_multiple(env: Env, count: u32) {
        for i in 0..count {
            env.events().publish((symbol_short!("multi"), i), i as u64);
        }
    }

    // ==================== QUERY-FRIENDLY PATTERNS ====================

    /// Emits a transfer event following the 3-topic pattern:
    ///   topic[0] = "transfer"   — filters all transfer events
    ///   topic[1] = from         — filters transfers *from* a specific address
    ///   topic[2] = to           — filters transfers *to* a specific address
    ///   data     = amount       — read after filtering; not used to filter
    ///
    /// Off-chain query examples:
    ///   • All transfers:                topic[0] == "transfer"
    ///   • All sends by Alice:           topic[0] == "transfer" AND topic[1] == Alice
    ///   • All receives by Bob:          topic[0] == "transfer" AND topic[2] == Bob
    ///   • Alice → Bob transfers only:   topic[0] == "transfer" AND topic[1] == Alice AND topic[2] == Bob
    pub fn emit_transfer(env: Env, from: Address, to: Address, amount: u64) {
        // Put action name first so every transfer is discoverable with one filter.
        // Put from/to next so indexers can build per-address history efficiently.
        env.events()
            .publish((symbol_short!("transfer"), from, to), amount);
    }

    /// Emits a namespaced event using a 3-topic hierarchy:
    ///   topic[0] = category (e.g. "defi")
    ///   topic[1] = action   (e.g. "swap")
    ///   topic[2] = pool_id  (any Symbol identifier)
    ///   data     = amount
    ///
    /// This pattern is useful when a single contract owns multiple logical
    /// sub-systems. Indexers can either:
    ///   • Catch all "defi" events   → filter topic[0] == "defi"
    ///   • Catch all swaps           → filter topic[0] == "defi" AND topic[1] == "swap"
    ///   • Catch swaps on one pool   → all three topics fixed
    ///
    /// Keep category and action as short Symbols (≤ 9 chars, symbol_short!).
    pub fn emit_namespaced(env: Env, category: Symbol, action: Symbol, pool_id: Symbol, amount: u64) {
        env.events()
            .publish((category, action, pool_id), amount);
    }

    /// Emits a status-change event with a 4-topic layout:
    ///   topic[0] = "status"
    ///   topic[1] = entity_id  (which entity changed)
    ///   topic[2] = old_status
    ///   topic[3] = new_status
    ///   data     = ledger sequence (for ordering / deduplication off-chain)
    ///
    /// Using all 4 topics lets off-chain systems query:
    ///   • Any status change for entity X
    ///   • Any transition *from* a specific state (e.g. "pending" → anything)
    ///   • Specific old → new transitions for audit trails
    pub fn emit_status_change(env: Env, entity_id: Symbol, old_status: Symbol, new_status: Symbol) {
        let ledger = env.ledger().sequence();
        env.events()
            .publish((symbol_short!("status"), entity_id, old_status, new_status), ledger);
    }
}

mod test;

//! # Event Filtering
//!
//! Demonstrates how to design Soroban events for efficient off-chain filtering.
//!
//! ## Key Concepts
//!
//! Soroban events have up to **4 topic slots** plus a **data payload**:
//!
//! ```text
//! env.events().publish(
//!     (topic_0, topic_1, topic_2, topic_3),  // indexed — used for filtering
//!     data,                                   // not indexed — read after match
//! );
//! ```
//!
//! Topics are indexed by off-chain systems (Horizon, custom listeners).
//! Data is only read once a matching event is found.
//!
//! ## Recommended Topic Layout
//!
//! | Slot    | Purpose                        | Example              |
//! |---------|--------------------------------|----------------------|
//! | topic_0 | Contract namespace / category  | `"marketplace"`      |
//! | topic_1 | Action name                    | `"sale"`             |
//! | topic_2 | Primary entity (most filtered) | `seller: Address`    |
//! | topic_3 | Secondary entity               | `buyer: Address`     |
//! | data    | Non-indexed payload            | `{ price, token_id }`|
//!
//! ## Off-Chain Query Examples
//!
//! ```text
//! All marketplace events:          topic_0 == "marketplace"
//! All sales:                       topic_0 == "marketplace" AND topic_1 == "sale"
//! All sales by Alice:              topic_0 == "marketplace" AND topic_1 == "sale" AND topic_2 == Alice
//! Alice → Bob sales only:          all four topics fixed
//! ```

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Event data payloads
// ---------------------------------------------------------------------------

/// Data payload for a token transfer event.
/// Placed in `data` (not topics) because amount is read after filtering.
#[contracttype]
pub struct TransferData {
    pub amount: i128,
}

/// Data payload for a marketplace sale event.
#[contracttype]
pub struct SaleData {
    pub price: i128,
    pub token_id: u64,
}

/// Data payload for a status-change event.
#[contracttype]
pub struct StatusData {
    pub old_status: Symbol,
    pub new_status: Symbol,
}

// ---------------------------------------------------------------------------
// Topic constants — define the event schema explicitly
// ---------------------------------------------------------------------------

/// Shared namespace for all events from this contract.
/// Placing it in topic_0 lets indexers discover every event type at once.
const NS: Symbol = symbol_short!("filter");

const ACT_TRANSFER: Symbol = symbol_short!("transfer");
const ACT_SALE: Symbol = symbol_short!("sale");
const ACT_STATUS: Symbol = symbol_short!("status");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct EventFilteringContract;

#[contractimpl]
impl EventFilteringContract {
    /// **2-topic event** — minimal filterable form.
    ///
    /// Topic layout:
    /// - topic_0: `"filter"` (namespace)
    /// - topic_1: `"transfer"` (action)
    /// - data: `TransferData { amount }`
    ///
    /// Off-chain: filter `topic_0 == "filter" AND topic_1 == "transfer"`
    /// to get all transfers from this contract.
    pub fn transfer_simple(env: Env, amount: i128) {
        env.events()
            .publish((NS, ACT_TRANSFER), TransferData { amount });
    }

    /// **3-topic event** — adds a primary indexed entity.
    ///
    /// Topic layout:
    /// - topic_0: `"filter"`
    /// - topic_1: `"transfer"`
    /// - topic_2: `from` (primary filter — most commonly queried)
    /// - data: `TransferData { amount }`
    ///
    /// Off-chain: filter by `topic_2 == <address>` to get all sends by a user.
    pub fn transfer_from(env: Env, from: Address, amount: i128) {
        env.events()
            .publish((NS, ACT_TRANSFER, from), TransferData { amount });
    }

    /// **4-topic event** — maximum filtering granularity.
    ///
    /// Topic layout:
    /// - topic_0: `"filter"`
    /// - topic_1: `"transfer"`
    /// - topic_2: `from` (sender — filter all sends by this address)
    /// - topic_3: `to`   (recipient — filter all receives by this address)
    /// - data: `TransferData { amount }`
    ///
    /// Off-chain query patterns:
    /// - All transfers:              `topic_0 == "filter"`
    /// - All transfers (action):     `topic_1 == "transfer"`
    /// - Sends by Alice:             `topic_2 == Alice`
    /// - Receives by Bob:            `topic_3 == Bob`
    /// - Alice → Bob only:           `topic_2 == Alice AND topic_3 == Bob`
    pub fn transfer_full(env: Env, from: Address, to: Address, amount: i128) {
        env.events()
            .publish((NS, ACT_TRANSFER, from, to), TransferData { amount });
    }

    /// **Namespaced category event** — groups related actions under one category.
    ///
    /// Topic layout:
    /// - topic_0: `"filter"`
    /// - topic_1: `"sale"`
    /// - topic_2: `seller`
    /// - topic_3: `buyer`
    /// - data: `SaleData { price, token_id }`
    ///
    /// Off-chain: filter `topic_1 == "sale"` to get all marketplace sales,
    /// or narrow to a specific seller/buyer via topic_2/topic_3.
    pub fn record_sale(env: Env, seller: Address, buyer: Address, price: i128, token_id: u64) {
        env.events()
            .publish((NS, ACT_SALE, seller, buyer), SaleData { price, token_id });
    }

    /// **Status-change event** — encodes transition in data, entity in topics.
    ///
    /// Topic layout:
    /// - topic_0: `"filter"`
    /// - topic_1: `"status"`
    /// - topic_2: `entity` (which entity changed state)
    /// - data: `StatusData { old_status, new_status }`
    ///
    /// The old/new status values go in `data` (not topics) because queries
    /// typically filter by *entity*, then read the transition details.
    /// If you need to filter by status value, move it to topic_3.
    pub fn update_status(env: Env, entity: Address, old_status: Symbol, new_status: Symbol) {
        env.events().publish(
            (NS, ACT_STATUS, entity),
            StatusData {
                old_status,
                new_status,
            },
        );
    }
}

#[cfg(test)]
mod test;

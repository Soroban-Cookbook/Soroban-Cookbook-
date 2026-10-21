#![allow(deprecated)]
//! # Event Aggregation Example
//!
//! Demonstrates how to **batch multiple actions into a single emitted event**
//! rather than emitting one event per action.  This is useful when a contract
//! processes a list of items in a single invocation and the individual events
//! would add up to significant emission overhead.
//!
//! ## Problem
//!
//! Emitting one Soroban event carries a fixed overhead (serialisation, XDR
//! encoding, ledger write).  A loop that processes 50 items and emits 50
//! events pays that overhead 50 times.  For high-throughput contracts this
//! overhead is non-trivial.
//!
//! ## Solution — Batch / Aggregate
//!
//! 1. Accumulate individual [`ActionEntry`] values into an in-memory `Vec`
//!    during the invocation.
//! 2. Emit exactly **one** [`BatchEvent`] at the end of the invocation
//!    containing the entire array.
//!
//! Downstream indexers decode the inner `actions` array to reconstruct
//! individual entries (see "Indexing Guide" in README.md).
//!
//! ## Topic Layout
//!
//! All cookbook events follow `(namespace, action, [key…])`:
//!
//! | Slot | Value          | Type     | Purpose                            |
//! |------|----------------|----------|------------------------------------|
//! | [0]  | `"evt_agg"`    | Symbol   | Contract namespace — catch-all     |
//! | [1]  | `"batch"`      | Symbol   | Action tag — filter batch events   |
//! | [2]  | `batch_id: u32`| u32      | Correlate batch across services    |
//!
//! The rich payload (action list, timestamp, count) is in the **data** slot
//! where it is decoded *after* a topic match — not used as a filter.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

// ── Topic symbols ────────────────────────────────────────────────────────────

/// Contract-level namespace — slot [0] in every event emitted by this contract.
const NS: Symbol = symbol_short!("evt_agg");
/// Action tag for the aggregated batch event — slot [1].
const ACTION_BATCH: Symbol = symbol_short!("batch");

// ── Storage keys ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Auto-incrementing batch counter (persists across invocations).
    BatchCounter,
    /// Pending action queue for the *current* invocation (instance storage,
    /// cleared on every flush).
    PendingActions,
}

// ── Domain types ─────────────────────────────────────────────────────────────

/// A single logical action captured before the batch is flushed.
///
/// Mirrors a simplified SEP-41-style transfer record: who acted, on behalf of
/// whom, for how much, and with what memo tag.  In a real contract this would
/// map to your domain's action vocabulary.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionEntry {
    /// Symbolic action type, e.g. `symbol_short!("transfer")`, `"mint"`, `"burn"`.
    pub action_type: Symbol,
    /// Address that initiated the action (auth principal).
    pub actor: Address,
    /// Numeric payload — token amount, vote weight, etc.
    pub amount: i128,
    /// Opaque memo for application-level correlation (0 = none).
    pub memo: u64,
}

/// Emitted once per `flush()` call.  Contains every [`ActionEntry`] accumulated
/// since the last flush, together with metadata needed by indexers.
///
/// # Indexing a BatchEvent
///
/// Off-chain consumers should:
/// 1. Filter by `topic[0] == "evt_agg"` and `topic[1] == "batch"` to identify
///    batch events from this contract.
/// 2. Decode the XDR `data` field as `BatchEvent`.
/// 3. Iterate `batch.actions` in order — index within the array is the
///    intra-batch sequence number (preserved, deterministic).
/// 4. To filter by inner action type query `entry.action_type` after decoding.
/// 5. Use `batch.batch_id` to correlate related ledger queries and
///    `batch.ledger_timestamp` for time-range filters.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchEvent {
    /// Monotonically increasing batch identifier (starts at 0).
    pub batch_id: u32,
    /// Ledger timestamp at the moment of flush (`env.ledger().timestamp()`).
    pub ledger_timestamp: u64,
    /// Total number of actions in this batch (== `actions.len()`).
    pub action_count: u32,
    /// Ordered list of actions included in this batch.
    pub actions: Vec<ActionEntry>,
}

// ── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AggError {
    /// `flush()` was called with no queued actions.  Callers may treat this as
    /// a no-op or as an error depending on their requirements; we surface it
    /// explicitly so the caller has the choice.
    EmptyBatch = 1,
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct EventAggregator;

#[contractimpl]
impl EventAggregator {
    // ── Mutation helpers ──────────────────────────────────────────────────

    /// Queue one [`ActionEntry`] for the next batch flush.
    ///
    /// Typically called multiple times within the same invocation (or
    /// cross-invocation accumulation is possible since instance storage
    /// persists across ledgers).  No event is emitted here — this is
    /// deliberate: the emission cost is deferred to `flush()`.
    pub fn queue_action(env: Env, actor: Address, action_type: Symbol, amount: i128, memo: u64) {
        actor.require_auth();

        let entry = ActionEntry {
            action_type,
            actor,
            amount,
            memo,
        };

        let mut pending: Vec<ActionEntry> = env
            .storage()
            .instance()
            .get(&DataKey::PendingActions)
            .unwrap_or_else(|| Vec::new(&env));

        pending.push_back(entry);
        env.storage()
            .instance()
            .set(&DataKey::PendingActions, &pending);
    }

    /// Flush all queued actions as a single [`BatchEvent`].
    ///
    /// Returns the `batch_id` of the emitted event.
    ///
    /// # Errors
    /// - [`AggError::EmptyBatch`] if there are no queued actions.
    ///
    /// # Design note — Empty-batch policy
    ///
    /// We return an explicit error rather than silently emitting an empty
    /// batch.  An empty flush almost always indicates a programming mistake
    /// in the caller (e.g. forgot to queue anything).  Callers that want a
    /// no-op can check the queue length first, or simply ignore this error.
    pub fn flush(env: Env) -> Result<u32, AggError> {
        let pending: Vec<ActionEntry> = env
            .storage()
            .instance()
            .get(&DataKey::PendingActions)
            .unwrap_or_else(|| Vec::new(&env));

        if pending.is_empty() {
            return Err(AggError::EmptyBatch);
        }

        // Assign a monotonically increasing batch id.
        let batch_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::BatchCounter)
            .unwrap_or(0u32);

        let action_count = pending.len();
        let ledger_timestamp = env.ledger().timestamp();

        let event_data = BatchEvent {
            batch_id,
            ledger_timestamp,
            action_count,
            actions: pending,
        };

        // Emit ONE event for N actions — this is the whole point.
        //
        // Topics: (namespace, "batch", batch_id)
        //   [0] "evt_agg" — namespace; filter all events from this contract
        //   [1] "batch"   — action tag; filter batch events specifically
        //   [2] batch_id  — correlation id; join across services
        env.events()
            .publish((NS, ACTION_BATCH, batch_id), event_data);

        // Advance the counter and clear the pending queue.
        env.storage()
            .instance()
            .set(&DataKey::BatchCounter, &(batch_id + 1));
        env.storage().instance().remove(&DataKey::PendingActions);

        Ok(batch_id)
    }

    // ── Read helpers ──────────────────────────────────────────────────────

    /// Returns the number of actions currently queued (not yet flushed).
    pub fn pending_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<DataKey, Vec<ActionEntry>>(&DataKey::PendingActions)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Returns the next batch id that will be assigned on `flush()`.
    pub fn next_batch_id(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::BatchCounter)
            .unwrap_or(0u32)
    }
}

#[cfg(test)]
mod test;
